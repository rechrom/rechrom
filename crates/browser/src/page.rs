#![allow(non_snake_case)]
//! Persistent Page lifecycle. All frames and measurements consume the original
//! parser arena. Module loading and the remaining DOM host continuations are
//! tracked separately; this service is not evidence of complete browser parity.
//! Page connects DOM mutation notifications and the style/layout/paint lifecycle;
//! each engine owns its state and invalidation, like LocalFrameView lifecycle wiring.
use crate::interaction_services::PageInteraction;
use document::{
    DocumentEffects, DocumentEngine, LayoutMutation as DocumentLayoutMutation, ResourceRequest,
};
use document_loader::{DocumentLoadBudget, ResourceFetcher, ResourceFetcherClient};
use dom::{dom_mutation::DOMMutationType, UserInteractionState, DOM};
use image_decoder::image_decoder::ImageDecoder;
use image_resource::DocumentImageDecoder;
pub use interaction::cursor::Cursor;
use interaction::event::{EventListenerInvocation, EventType, MakeSyntheticEvent};
pub use interaction::frame_aligned_input_queue::DispatchedFrameInput;
use javascript::javascript_runtime::{
    JavaScriptException, JavaScriptHostRuntime, JavaScriptRealm, JavaScriptResult,
    JavaScriptRuntime,
};
use javascript::script_engine::ScriptEngine;
use layoutng_assembly::{
    fragment_tree::{FragmentNode, PaintResources},
    internal::{
        layout_input::{ConstraintSpace, Offset},
        layout_input_types::IntSize,
    },
};
pub use open::OpenStatus;
use open::{script_scheduler::ScriptLoadClient, OpenEngine};
use page_mutation::{PageMutation, ResourceMutation};
pub use paint::paint_engine::{CaretGeometry, PaintRect as CaretRect};
use resource::ResourceEngine;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io,
    rc::{Rc, Weak},
    sync::Arc,
};
pub use style::PreferredColorScheme;
use url_loader::{URLLoader, URLRequest, URLWakeCallback};
use webapi::{
    dom_bindings::{DOMBindingsHost, DOMJavaScriptBindings},
    web_api_engine::{WebApiEffect, WebApiEngine, WebApiMutation},
    window_bindings::WindowJavaScriptBindings,
};

#[cfg(feature = "pure_source_png")]
use layer_tile::{FrameConfig, LayerTileEngine};
#[cfg(feature = "pure_source_png")]
use raster::{RasterEngine, RasterStats};
#[cfg(feature = "pure_source_png")]
use renderer::{PixelFormat, RenderTarget, RenderUpdate, Renderer, SingleSurfaceResources};

#[cfg(feature = "pure_source_png")]
#[derive(Default)]
struct PageRenderPipeline {
    tiles: LayerTileEngine,
    resources: RasterEngine,
    compositor: compositor::FrameBuilder,
    viz: viz::VizEngine,
    renderer: Renderer,
    // Identity-only marker. A strong Arc here would force PaintEngine's next
    // property-only scroll update to deep-copy the complete PaintArtifact.
    prepared_artifact: Option<std::sync::Weak<paint::PaintArtifact>>,
    prepared_config: Option<FrameConfig>,
}

#[cfg(feature = "pure_source_png")]
impl PageRenderPipeline {
    fn Render(
        &mut self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
    ) -> io::Result<RenderUpdate> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "raster scale must be positive",
            ));
        }

        // Page is the composition root. Every stage keeps its own retained
        // state, while immutable protocol values cross each boundary directly.
        let config = FrameConfig {
            viewport: paint::PaintRect {
                x: 0.0,
                y: 0.0,
                width: width as f64 / scale,
                height: height as f64 / scale,
            },
            raster_scale: scale,
            activation_scroll: None,
            frame_time: None,
        };
        let already_prepared = self.prepared_config == Some(config)
            && self
                .prepared_artifact
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .is_some_and(|prepared| Arc::ptr_eq(&prepared, artifact));
        if !already_prepared {
            let pending = self
                .tiles
                .UpdatePending(artifact, config)
                .map_err(|error| io::Error::new(io::ErrorKind::Unsupported, error.reason()))?;
            let raster = self.resources.prepare(
                &pending.frame_plan,
                &pending.raster_batch,
                width,
                height,
            )?;
            self.tiles.ApplyRasterResults(&raster.completions);
            let release = self.tiles.ActivatePending().ok_or_else(|| {
                io::Error::other("pending Page layer tree is not ready for activation")
            })?;
            let released = self.resources.release_resources(&release)?;
            self.tiles.AcknowledgeResourceRelease(&released);
            self.prepared_artifact = Some(Arc::downgrade(artifact));
            self.prepared_config = Some(config);
        }

        let compositor_frame = self
            .compositor
            .BuildFrame(
                self.tiles
                    .GetActiveFramePlan()
                    .expect("prepared Page has FramePlan"),
            )
            .map_err(io::Error::other)?;
        let surface = viz::SurfaceId(1);
        self.viz
            .SubmitFrame(surface, compositor_frame)
            .map_err(io::Error::other)?;
        let output = compositor::DeviceRect::new(0, 0, width, height);
        let aggregated_frame = self
            .viz
            .Aggregate(
                output,
                &[viz::SurfacePlacement {
                    surface_id: surface,
                    destination: output,
                }],
            )
            .map_err(io::Error::other)?;
        let resources = SingleSurfaceResources {
            surface_id: surface,
            resources: &self.resources,
        };
        self.renderer.Render(
            &aggregated_frame,
            &resources,
            RenderTarget {
                width,
                height,
                pixels: target,
                format,
                row_stride: stride,
            },
        )
    }

    fn InvalidatePreparedArtifact(&mut self) {
        // FramePlan owns the artifact used by its raster tasks. Tile identity
        // and resident pixels remain retained by their respective engines.
        self.tiles.ReleaseFramePlans();
        self.prepared_artifact = None;
        self.prepared_config = None;
    }
}

// cpp: browser/browser.h:47-51
pub struct PageFrame {
    pub sequence: u64,
    pub fragments: Rc<FragmentNode>,
    pub display_items: Arc<paint::paint_engine::PaintArtifact>,
}

/// Local document navigation request; tab/window policy belongs to the host.
#[derive(Clone, Debug)]
pub struct NavigationRequest {
    pub request: URLRequest,
    pub target: String,
    /// Replace the current session-history entry (Location.replace).
    pub replace_history: bool,
}
// cpp: browser/browser.h:53-67
pub trait PageClient {
    /// The embedding host schedules navigation after the active Page call returns.
    fn DidRequestNavigation(&mut self, _request: &NavigationRequest) {}

    fn DidCommit(&mut self, _url: &str) {}
    /// Host notification independent of frame painting; map this to the window cursor.
    fn DidChangeCursor(&mut self, _cursor: Cursor) {}
    fn DidPresentFrame(&mut self, _frame: &PageFrame) {}
    fn DidFinishLoad(&mut self) {}
    fn DidFail(&mut self, _message: &str) {}
    fn DidFailResource(&mut self, _url: &str, _message: &str) {}
    fn DidReportScriptError(&mut self, _error: &JavaScriptException) {}
}
struct ScriptClient(Rc<RefCell<dyn PageClient>>, Weak<PageState>);
impl ScriptLoadClient for ScriptClient {
    fn DidPumpParser(&mut self) {
        if let Some(state) = self.1.upgrade() {
            state.InvalidateMeasurement();
            state.InvalidateStyle();
        }
    }
    fn DidApplyStyleSheet(&mut self, _: &str) {
        if let Some(state) = self.1.upgrade() {
            // A stylesheet first invalidates computed style. Whether it also
            // invalidates geometry is decided by the computed-style diff in
            // ResolveStyles, matching Blink's style/layout lifecycle split.
            state.InvalidateStylePreservingGeometry();
        }
    }
    fn DidReportScriptError(&mut self, e: &JavaScriptException) {
        self.0.borrow_mut().DidReportScriptError(e);
    }
    fn DidFailResource(&mut self, url: &str, message: &str) {
        self.0.borrow_mut().DidFailResource(url, message);
    }
}

mod measurement;
mod paint_commit;
use measurement::MeasurementSnapshot;
use paint_commit::CommitPaint;

struct PendingImageEvent {
    id: u64,
    source: String,
    loaded: bool,
    active: Rc<Cell<bool>>,
}
struct PageState {
    document: Rc<DocumentEngine>,
    style_engine: RefCell<style::StyleEngine>,
    layout_engine: RefCell<layoutng_assembly::layout_engine::LayoutEngine>,
    paint_engine: RefCell<paint::PaintEngine>,
    #[cfg(feature = "pure_source_png")]
    rendering: RefCell<PageRenderPipeline>,
    constraints: Rc<RefCell<ConstraintSpace>>,
    interaction: Rc<RefCell<UserInteractionState>>,
    layout_editing: Rc<layoutng_assembly::editing_state::LayoutEditingState>,
    bindings: RefCell<Weak<RefCell<DOMJavaScriptBindings>>>,
    resources: RefCell<Option<Rc<ResourceFetcher>>>,
    connected_resources: open::ConnectedResourceDiscovery,
    image_events: RefCell<Vec<PendingImageEvent>>,
    pending_image_events: RefCell<HashMap<u64, Rc<Cell<bool>>>>,
    scroll_event_targets: RefCell<Vec<u64>>,
    client: Rc<RefCell<dyn PageClient>>,
    dirty: Cell<bool>,
    // Only offsets have changed since the last completed presentation.
    scroll_only: Cell<bool>,
    styles_resolved: Cell<bool>,
    resource_references_dirty: Cell<bool>,
    // A stable image resource id published a new immutable content snapshot.
    // This never means DOM, layout geometry or display-item recording changed.
    paint_resources_dirty: Cell<bool>,
    pending_paint_mutations: RefCell<Vec<paint::paint_engine::PaintMutation>>,
    measurement: RefCell<Option<MeasurementSnapshot>>,
    measurement_candidate: RefCell<Option<MeasurementSnapshot>>,
    #[cfg(test)]
    full_layout_lifecycles: Cell<usize>,
    scroll_updates: RefCell<Vec<(u64, Offset)>>,
    animation_engine: RefCell<animation::AnimationEngine>,
    animation_begin_frame_requested: Cell<bool>,
    preferred_color_scheme: Cell<PreferredColorScheme>,
}
impl PageState {
    fn WithEngines(
        document: Rc<DocumentEngine>,
        layout_engine: layoutng_assembly::layout_engine::LayoutEngine,
        paint_engine: paint::PaintEngine,
        #[cfg(feature = "pure_source_png")] rendering: PageRenderPipeline,
        constraints: ConstraintSpace,
        client: Rc<RefCell<dyn PageClient>>,
    ) -> Rc<Self> {
        let style_engine = style::StyleEngine::new(&document.borrow());
        let connected_resources = open::ConnectedResourceDiscovery::new(document.Handle());
        Rc::new(Self {
            document,
            style_engine: RefCell::new(style_engine),
            layout_engine: RefCell::new(layout_engine),
            paint_engine: RefCell::new(paint_engine),
            #[cfg(feature = "pure_source_png")]
            rendering: RefCell::new(rendering),
            constraints: Rc::new(RefCell::new(constraints)),
            interaction: Rc::new(RefCell::new(UserInteractionState::default())),
            layout_editing: Rc::new(Default::default()),
            bindings: RefCell::new(Weak::new()),
            resources: RefCell::new(None),
            connected_resources,
            image_events: RefCell::new(Vec::new()),
            pending_image_events: RefCell::new(HashMap::new()),
            scroll_event_targets: RefCell::new(Vec::new()),
            client: client.clone(),
            dirty: Cell::new(true),
            scroll_only: Cell::new(false),
            styles_resolved: Cell::new(false),
            resource_references_dirty: Cell::new(true),
            paint_resources_dirty: Cell::new(false),
            pending_paint_mutations: RefCell::new(Vec::new()),
            measurement: RefCell::new(None),
            measurement_candidate: RefCell::new(None),
            #[cfg(test)]
            full_layout_lifecycles: Cell::new(0),
            scroll_updates: RefCell::new(Vec::new()),
            animation_engine: RefCell::new(animation::AnimationEngine::new()),
            animation_begin_frame_requested: Cell::new(false),
            preferred_color_scheme: Cell::new(PreferredColorScheme::kLight),
        })
    }

    #[cfg(test)]
    fn new(constraints: ConstraintSpace, client: Rc<RefCell<dyn PageClient>>) -> Rc<Self> {
        Self::WithEngines(
            Rc::new(DocumentEngine::new()),
            layoutng_assembly::layout_engine::LayoutEngine::new(&crate::CreateLayoutAssembly()),
            paint::PaintEngine::new(),
            #[cfg(feature = "pure_source_png")]
            PageRenderPipeline::default(),
            constraints,
            client,
        )
    }

    #[track_caller]
    fn InvalidateMeasurement(&self) {
        let had_snapshot = self.measurement.borrow().is_some();
        let had_candidate = self.measurement_candidate.borrow().is_some();
        if had_snapshot || had_candidate {
            browser_tracing::instant(
                "layout",
                "MeasurementInvalidated",
                &[
                    ("caller_line", std::panic::Location::caller().line() as f64),
                    ("had_snapshot", had_snapshot as u8 as f64),
                    ("had_candidate", had_candidate as u8 as f64),
                ],
            );
        }
        self.measurement.borrow_mut().take();
        self.measurement_candidate.borrow_mut().take();
    }
    fn PreserveMeasurementGeometry(&self) {
        if let Some(mut snapshot) = self.measurement.borrow_mut().take() {
            snapshot.MarkPaintStale();
            *self.measurement_candidate.borrow_mut() = Some(snapshot);
        }
    }
    fn InstallMeasurement(&self, fragments: Rc<FragmentNode>) {
        let revision = self.document.borrow().GetMeasurementGeometryRevision();
        self.measurement_candidate.borrow_mut().take();
        *self.measurement.borrow_mut() =
            Some(MeasurementSnapshot::with_revision(fragments, revision));
    }

    fn PublishPaintResources(&self, frame: &mut PageFrame) {
        let pending = std::mem::take(&mut *self.pending_paint_mutations.borrow_mut());
        let update = if pending.is_empty() {
            let constraints = self.constraints.borrow();
            paint::paint_engine::PaintResourceUpdate::Snapshot(Arc::new(PaintResources {
                fonts: constraints.fonts.clone(),
                images: constraints.images.clone(),
                device_pixel_ratio: constraints.device_pixel_ratio,
                viewport: constraints.viewport,
            }))
        } else {
            paint::paint_engine::PaintResourceUpdate::Mutations(pending)
        };
        let mut paint = self.paint_engine.borrow_mut();
        frame.display_items = paint
            .PublishResourceUpdate(std::mem::take(&mut frame.display_items), update)
            .unwrap_or_else(|| dom::error::logic_error("paint resource update rejected"));
        self.paint_resources_dirty.set(false);
    }
    fn InvalidateStyle(&self) {
        self.InvalidateMeasurement();
        self.InvalidateStylePreservingGeometry();
    }
    fn InvalidateStylePreservingGeometry(&self) {
        self.PreserveMeasurementGeometry();
        self.styles_resolved.set(false);
        self.resource_references_dirty.set(true);
        self.scroll_only.set(false);
        self.dirty.set(true);
    }
    // cpp: browser/browser.cc:947-981
    fn ApplyDOMMutation(
        &self,
        m: &dom::dom_mutation::DOMMutation,
        notify: &mut dyn FnMut(),
        dispatch_image: &mut dyn FnMut(u64, bool),
    ) {
        let effect = self
            .document
            .ApplyDOMMutation(m)
            .unwrap_or_else(|error| std::panic::panic_any(error));
        self.ApplyDocumentLifecycleEffects(&effect);
        // A changed image request cancels its queued event, even if src later
        // changes back to the same URL before the event task runs.
        for request in &effect.resource {
            if let ResourceRequest::ResetImage { node_id } = *request {
                self.CancelImageEvent(node_id);
                if let Some(resources) = self.resources.borrow().as_ref() {
                    resources.ResetImageEventSource(node_id);
                }
            }
        }
        notify();
        let Some(connected) = effect
            .resource
            .into_iter()
            .find_map(|request| match request {
                ResourceRequest::DiscoverConnectedSubtree(request) => Some(request),
                ResourceRequest::ResetImage { .. } => None,
            })
        else {
            return;
        };
        let node = self
            .document
            .borrow()
            .GetDocument()
            .FindNodeById(connected.root_node_id);
        if let Some(i) = node {
            // Attribute parsing belongs to this element. Descendant style
            // resource discovery still runs through the invalidated StyleState;
            // only insertion/content replacement prepares a connected subtree.
            let result = if m.mutation_type == DOMMutationType::kSetAttribute {
                // HTMLImageElement::ParseAttribute only updates the image
                // request for source/request attributes. Styling an already
                // loaded image must not post another load event.
                self.connected_resources.PrepareNode(
                    i,
                    connected.prepare_scripts,
                    connected.image_request,
                    dispatch_image,
                )
            } else {
                self.connected_resources.PrepareSubtree(
                    i,
                    connected.prepare_scripts,
                    dispatch_image,
                )
            };
            let resource_effects = result.unwrap_or_else(|e| std::panic::panic_any(e));
            if resource_effects.style_sheet_changed {
                self.InvalidateStylePreservingGeometry();
            }
        }
        // Preparation invalidates newly parsed sheets itself. Image completion
        // registers a later DOM task; its listener writes use this same path.
    }

    fn ApplyDocumentLifecycleEffects(&self, effects: &DocumentEffects) {
        // Blink leaves detailed dirty nodes on Document. The effect only admits
        // the relevant lifecycle engines. Attribute changes keep committed
        // geometry until computed-style diffing proves layout changed.
        if effects.layout == Some(DocumentLayoutMutation::InvalidateGeometry) {
            self.InvalidateMeasurement();
        } else if effects.style.is_some() {
            self.PreserveMeasurementGeometry();
        }
        if effects.style.is_some() {
            self.InvalidateStylePreservingGeometry();
        }
    }
    fn ApplyAnimationOutput(&self, output: animation::AnimationOutput) {
        for mutation in output.document {
            let effects = self.document.ApplyAnimationMutation(mutation);
            self.ApplyDocumentLifecycleEffects(&effects);
        }
        for mutation in output.paint {
            self.paint_engine
                .borrow_mut()
                .ApplyAnimationMutation(mutation);
        }
        for effect in output.effects {
            match effect {
                animation::AnimationEffect::RequestBeginFrame => {
                    self.animation_begin_frame_requested.set(true)
                }
            }
        }
    }
    fn ApplyAnimationMutation(&self, mutation: animation::AnimationMutation) {
        let output = self
            .animation_engine
            .borrow_mut()
            .ApplyMutation(mutation)
            .unwrap_or_else(|_| {
                dom::error::invalid_argument("animation time must be finite and frame ordered")
            });
        self.ApplyAnimationOutput(output);
    }
    fn ApplyInteractionOutput(&self, output: interaction::InteractionOutput) {
        match output {
            interaction::InteractionOutput::DocumentMutation(mutation) => {
                self.ApplyMutation(PageMutation::DOMMutation(mutation));
            }
            interaction::InteractionOutput::StateMutation(mutation) => {
                self.ApplyMutation(PageMutation::InteractionStateMutation(mutation));
            }
            interaction::InteractionOutput::Effect(
                interaction::InteractionEffect::SubmitForm { form, submitter },
            ) => {
                let base = self
                    .resources
                    .borrow()
                    .as_ref()
                    .map(|resources| resources.BaseURL())
                    .unwrap_or_default();
                let request = {
                    let owner = self.document.borrow();
                    crate::form_submission::BuildRequest(
                        owner.GetDocument(),
                        &base,
                        form,
                        submitter,
                    )
                };
                match request {
                    Ok(Some(request)) => self.client.borrow_mut().DidRequestNavigation(&request),
                    Err(error) => self.client.borrow_mut().DidFail(&error.to_string()),
                    Ok(None) => {}
                }
            }
        }
    }
    fn QueueImageEvent(&self, id: u64, loaded: bool) {
        let source = {
            let owner = self.document.borrow();
            let tree = owner.GetDocument();
            let Some(i) = tree.FindNodeById(id) else {
                return;
            };
            let node = tree.Node(i);
            if !node.IsHTMLElement("img") {
                return;
            }
            let Some(src) = node.FindAttribute("src").filter(|a| !a.value.is_empty()) else {
                return;
            };
            src.value.clone()
        };
        self.CancelImageEvent(id);
        let active = Rc::new(Cell::new(true));
        self.pending_image_events
            .borrow_mut()
            .insert(id, active.clone());
        self.image_events.borrow_mut().push(PendingImageEvent {
            id,
            source,
            loaded,
            active,
        });
    }
    fn CancelImageEvent(&self, id: u64) {
        if let Some(active) = self.pending_image_events.borrow_mut().remove(&id) {
            active.set(false);
        }
    }
    fn CancelImageEvents(&self) {
        for (_, active) in self.pending_image_events.borrow_mut().drain() {
            active.set(false);
        }
        self.image_events.borrow_mut().clear();
    }
    fn ConnectResources(self: &Rc<Self>, resources: Rc<ResourceFetcher>) {
        // Commit into the parser's supplied arena. That arena owns its CSSOM
        // collection, just as Blink's Document owns its StyleEngine collection.
        let state = Rc::downgrade(self);
        resources.SetStyleSheetReceiver(Rc::new(move |document, sheet| {
            let effects = DocumentEngine::AppendStyleSheetTo(document, sheet);
            if let Some(state) = state.upgrade() {
                // A newly connected sheet can change both geometry and the set
                // of referenced @font-face/image resources. CSSOM owns rule
                // invalidation; Page owns lifecycle and resource discovery.
                state.ApplyDocumentLifecycleEffects(&effects);
            }
        }));
        self.connected_resources.SetResources(resources.clone());
        *self.resources.borrow_mut() = Some(resources);
    }
    fn CreateWebApiDOMHost(self: &Rc<Self>) -> DOMBindingsHost {
        let state = self;
        let style = state.clone();
        let metric = state.clone();
        let geometry = state.clone();
        let scroll = state.clone();
        let stylesheet = state.clone();
        let animation = state.clone();
        let host = DOMBindingsHost {
            write_scroll: Some(Box::new(move |target_node_id, offset| {
                scroll.ApplyMutation(PageMutation::ScrollMutation(
                    page_mutation::ScrollMutation {
                        target_node_id,
                        offset,
                    },
                ));
            })),
            // cpp: browser/browser.cc:634-637
            sample_animation: Some(Box::new(move |node_id, effect_id, time, declarations| {
                animation.ApplyAnimationMutation(animation::AnimationMutation::Tick(
                    animation::AnimationTick {
                        monotonic_time: time / 1000.0,
                        samples: vec![animation::AnimationStyleSample {
                            node_id,
                            effect_id,
                            declarations,
                        }],
                        ..Default::default()
                    },
                ));
            })),
            update_style: Some(Box::new(move || style.ResolveStyles())),
            emit_style_sheet: Some(Box::new(move |sheet| {
                let base = stylesheet
                    .resources
                    .borrow()
                    .as_ref()
                    .expect("CSSOM before navigation")
                    .BaseURL();
                stylesheet.ApplyMutation(PageMutation::CSSOMMutation(
                    page_mutation::CSSOMMutation {
                        style_sheet: sheet,
                        base_url: base,
                    },
                ));
            })),
            read_metric: Some(Box::new(move |id, name| {
                metric.EnsureMeasurement();
                crate::style_services::FragmentMetric(
                    metric.measurement.borrow().as_ref().unwrap(),
                    id,
                    name,
                )
            })),
            dispatch_synthetic: None,
            submit_form: None,
            read_geometry: Some(Box::new(move |id| {
                geometry.EnsureMeasurement();
                geometry
                    .measurement
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .ClientRects(id)
            })),
        };
        host
    }

    fn ConnectDOMBindings(&self, bindings: &Rc<RefCell<DOMJavaScriptBindings>>) {
        *self.bindings.borrow_mut() = Rc::downgrade(bindings);
    }
    fn ApplyPendingScrollUpdates(&self, frame: Option<&mut PageFrame>) -> bool {
        let mut trace = browser_tracing::span("lifecycle", "Page.ApplyPendingScrollUpdates");
        let mut updates = self.scroll_updates.borrow_mut();
        trace.set("pending_scrolls", updates.len() as f64);
        trace.set("records_reused", 0.0);
        if updates.is_empty() {
            return false;
        }
        let mut reused_recordings = false;
        let reused = frame.is_some_and(|frame| {
            // Reject mixed lifecycles before copy-on-write can duplicate an old
            // snapshot that the ordinary layout export will replace anyway.
            if !self.document.borrow().GetStyleImpact().IsEmpty() {
                trace.set("impact_rejected", 1.0);
                return false;
            }
            // A DOM/style write initially disqualifies the input batch, but
            // resolving it can prove that no layout or paint input changed.
            // Style resolution and geometry queries never consume paint
            // impact; only a committed Paint does. Re-admit that resolved noop
            // here, without skipping the resource/style flush above or the
            // engine's native NeedsLayout/result-identity checks below.
            let offset_only = self.scroll_only.get()
                || (self.styles_resolved.get() && {
                    let owner = self.document.borrow();
                    let style = owner.GetDocument().StyleState();
                    !style.all_dirty && style.changes.is_empty()
                });
            trace.set("offset_only", offset_only as u8 as f64);
            trace.set(
                "resolved_noop",
                (offset_only && !self.scroll_only.get()) as u8 as f64,
            );
            // The engine and committed frame share the current snapshot between
            // updates. Release the engine's reference before mutating the frame;
            // only genuinely retained external snapshots should trigger COW.
            self.layout_engine.borrow_mut().ReleaseLayoutResult();
            let scroll_layout = {
                let mut engine = self.layout_engine.borrow_mut();
                crate::persistent_layout::LayoutPersistentScrollDocumentWithMode(
                    &mut engine,
                    &mut self.document.borrow_mut(),
                    &self.constraints.borrow(),
                    &updates,
                    Rc::make_mut(&mut frame.fragments),
                    offset_only,
                )
            };
            let Some(scroll_layout) = scroll_layout else {
                self.layout_engine
                    .borrow_mut()
                    .PublishLayoutResult(frame.fragments.clone());
                return false;
            };
            trace.set(
                "retained_geometry",
                (scroll_layout == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry)
                    as u8 as f64,
            );
            if std::env::var_os("BROWSER_PROFILE_INPUT").is_some()
                && (!self.scroll_only.get()
                    || scroll_layout
                        != crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry)
            {
                eprintln!(
                    "scroll-layout-admission input_offset_only={} resolved_noop={} mode={}",
                    self.scroll_only.get(),
                    offset_only && !self.scroll_only.get(),
                    if scroll_layout
                        == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry
                    {
                        "retained-geometry"
                    } else if offset_only {
                        // These are the two remaining native fast-path gates;
                        // do not attribute this fallback to the input flag.
                        "native-needs-layout-or-missing-result"
                    } else {
                        "unresolved-mixed-input"
                    }
                );
            }
            let mut paint = self.paint_engine.borrow_mut();
            #[cfg(feature = "pure_source_png")]
            {
                // The old FramePlan retains the artifact needed to replay its
                // raster tasks. This scroll is about to replace that plan;
                // release only the publication so Arc::make_mut below does not
                // clone the complete PaintArtifact. Tile identities and pixels
                // remain resident in their owning engines.
                self.rendering.borrow_mut().InvalidatePreparedArtifact();
            }
            // The frame and Engine reference the same committed artifact.
            // Move the frame's reference back before a property-only update.
            paint.AdoptPaintResult(std::mem::take(&mut frame.display_items));
            let property_update = (scroll_layout
                == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry)
                .then(|| paint.TryUpdateScrollProperties(&frame.fragments));
            reused_recordings = matches!(&property_update, Some(Ok(_)));
            if !reused_recordings {
                trace.set("paint_fallback", 1.0);
                if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                    if let Some(Err(reason)) = property_update {
                        eprintln!("scroll-paint-reuse rejected={reason:?}");
                    }
                }
                paint.Paint(
                    &frame.fragments,
                    self.layout_editing.caret.borrow().paint_state(),
                );
                paint.CommitPaintResult(|artifact| {
                    CommitPaint(
                        &mut self.layout_engine.borrow_mut(),
                        &self.layout_editing.caret.borrow(),
                        &mut frame.fragments,
                        artifact,
                    )
                });
            }
            frame.display_items = paint
                .GetPaintResult()
                .expect("paint result available")
                .clone();
            drop(paint);
            // Share the immutable post-scroll snapshot. Every mutation clears
            // measurement first, so the next scroll mutates the unique frame;
            // callers retaining an old snapshot get copy-on-write isolation.
            self.InstallMeasurement(frame.fragments.clone());
            self.layout_engine
                .borrow_mut()
                .PublishLayoutResult(frame.fragments.clone());
            true
        });
        if !reused {
            self.document.borrow_mut().InvalidateLayout();
        }
        updates.clear();
        trace.set("records_reused", reused_recordings as u8 as f64);
        reused_recordings
    }

    // cpp: browser/browser.cc:1748-1758
    fn ResolveStyles(&self) {
        let mut trace = browser_tracing::span("lifecycle", "Page.ResolveStyles");
        trace.set("cache_hit", self.styles_resolved.get() as u8 as f64);
        if self.styles_resolved.get() {
            return;
        }
        crate::style_services::ResolveLayoutStylesWithEngine(
            &mut self.style_engine.borrow_mut(),
            &mut self.document.borrow_mut(),
            &self.constraints.borrow(),
            self.preferred_color_scheme.get(),
        );
        self.styles_resolved.set(true);
    }

    // cpp: style_engine.cc:3865-3950; container_query_evaluator.cc:369-385.
    // Chromium interleaves descendant style recalc with the sized container's
    // layout. This assembly exports complete native layout batches, so advance
    // style/layout until the published content sizes are unchanged before any
    // measurement or paint consumes the batch.
    fn LayoutWithContainerQueries(&self) {
        let mut observed = std::collections::HashSet::new();
        let limit = self.document.borrow().GetDocument().NodeCount() + 1;
        for _ in 0..limit {
            #[cfg(test)]
            self.full_layout_lifecycles
                .set(self.full_layout_lifecycles.get() + 1);
            let sizes = {
                let mut layout = self.layout_engine.borrow_mut();
                crate::persistent_layout::LayoutDocumentWithEngine(
                    &mut layout,
                    &mut self.document.borrow_mut(),
                    &self.interaction.borrow(),
                    &self.constraints.borrow(),
                );
                if !self.style_engine.borrow().HasContainerQueries() {
                    return;
                }
                layout.ContainerQuerySizes()
            };
            let changed = crate::style_services::PublishContainerSizes(
                &mut self.style_engine.borrow_mut(),
                &mut self.document.borrow_mut(),
                &sizes,
            );
            if !changed {
                return;
            }
            let signature = sizes
                .iter()
                .map(|s| (s.node_id, s.width.to_bits(), s.height.to_bits()))
                .collect::<Vec<_>>();
            if !observed.insert(signature) {
                break;
            }
            self.styles_resolved.set(false);
            self.ResolveStyles();
        }
        std::panic::panic_any(foundation::UnsupportedLayout::new(
            "container query style/layout did not converge (style_engine.cc:3865)",
        ));
    }

    // cpp: browser/browser.cc:932-939
    fn EnsureMeasurement(&self) {
        let mut trace = browser_tracing::span("layout", "Page.EnsureMeasurement");
        trace.set(
            "candidate",
            self.measurement_candidate.borrow().is_some() as u8 as f64,
        );
        trace.set(
            "cache_hit",
            self.measurement.borrow().is_some() as u8 as f64,
        );
        if self.measurement.borrow().is_some() {
            return;
        }
        self.ResolveStyles();
        let revision = self.document.borrow().GetMeasurementGeometryRevision();
        trace.set("geometry_revision", revision as f64);
        if let Some(snapshot) = self.measurement_candidate.borrow().as_ref() {
            trace.set("candidate_revision", snapshot.GeometryRevision() as f64);
        }
        if let Some(snapshot) = self
            .measurement_candidate
            .borrow_mut()
            .take()
            .filter(|snapshot| snapshot.GeometryRevision() == revision)
        {
            // Style has been updated. This old snapshot proves query geometry
            // only: its paint properties must never stand in for a new Paint.
            *self.measurement.borrow_mut() = Some(snapshot);
            trace.set("geometry_reused", 1.0);
            return;
        }
        let fragments = {
            trace.set("layout", 1.0);
            self.LayoutWithContainerQueries();
            self.layout_engine
                .borrow()
                .GetLayoutResult()
                .expect("successful layout has fragments")
                .clone()
        };
        self.InstallMeasurement(fragments);
    }

    // cpp: browser/browser.cc:947-1044
    fn ApplyMutation(&self, mutation: PageMutation) {
        // One BeginFrame can deliver several Element.animate() samples and a
        // document-image frame. Neither is an unrelated outer-document edit:
        // keep accumulating the targeted paint batch until this lifecycle is
        // committed. Geometry/style mutations still revoke the proof.
        if !matches!(
            &mutation,
            PageMutation::AnimationTick(_)
                | PageMutation::ResourceMutation(ResourceMutation::DocumentImageFrameChanged(_))
        ) {
            self.ApplyAnimationMutation(animation::AnimationMutation::InvalidatePaintBatch);
        }
        match mutation {
            PageMutation::DOMMutation(m) => {
                // Runtime-independent callers are no-JS Page mutations and
                // interaction control-value/checked actions. JavaScript DOM
                // mutations and public Page::Apply use the runtime scope below.
                self.ApplyDOMMutation(&m, &mut || {}, &mut |_, _| {
                    assert!(
                        self.bindings.borrow().upgrade().is_none(),
                        "cached image event requires the Page runtime scope"
                    );
                });
            }
            PageMutation::ResourceMutation(m) => self.ApplyResourceMutation(m),
            PageMutation::InteractionStateMutation(m) => {
                let previous = *self.interaction.borrow();
                if previous != m.state {
                    let needs_paint = {
                        let owner = self.document.borrow();
                        let document = owner.GetDocument();
                        if !interaction::ValidateInteractionState(document, m.state) {
                            dom::error::invalid_argument(
                                "interaction state references a missing DOM node",
                            );
                        }
                        interaction::InteractionStateNeedsPaint(document, previous, m.state)
                    };
                    *self.interaction.borrow_mut() = m.state;
                    if let Some(bindings) = self.bindings.borrow().upgrade() {
                        bindings
                            .borrow_mut()
                            .SetFocusedNode(m.state.focused_node_id);
                    }
                    if needs_paint {
                        self.document.borrow_mut().InvalidatePaint();
                        // Focus/hover/pressed state changes native appearance
                        // and focus/caret paint, but not box geometry. Preserve
                        // the last geometry as a stale-paint candidate so a
                        // synchronous getBoundingClientRect() in the input
                        // listener does not force a duplicate full layout.
                        // UpdateFrameIfNeeded still rejects it for PaintCurrent
                        // and exports the new interaction paint state.
                        self.PreserveMeasurementGeometry();
                        self.scroll_only.set(false);
                        self.dirty.set(true);
                    }
                }
            }
            PageMutation::CSSOMMutation(m) => {
                self.resources
                    .borrow()
                    .as_ref()
                    .expect("stylesheet before navigation")
                    .AddParsedStyleSheet(m.style_sheet, &m.base_url)
                    .unwrap_or_else(|e| std::panic::panic_any(e));
                self.InvalidateStylePreservingGeometry();
            }
            PageMutation::ViewportMutation(m) => {
                if !m.size.width.is_finite()
                    || !m.size.height.is_finite()
                    || m.size.width <= 0.0
                    || m.size.height <= 0.0
                {
                    dom::error::invalid_argument("viewport size must be positive");
                }
                let mut constraints = self.constraints.borrow_mut();
                constraints.available_size = m.size;
                if let Some(viewport) = &mut constraints.viewport {
                    viewport.size = IntSize {
                        width: m.size.width.round() as i32,
                        height: m.size.height.round() as i32,
                    };
                }
                self.InvalidateStyle();
            }
            PageMutation::ScrollMutation(m) => {
                if let Some(effect) = self.document.ApplyScrollOffset(m.target_node_id, m.offset) {
                    self.InvalidateMeasurement();
                    browser_tracing::instant(
                        "input",
                        "ScrollApplied",
                        &[
                            ("scroll_node", effect.target_node_id as f64),
                            ("before_x", effect.before.x),
                            ("before_y", effect.before.y),
                            ("offset_x", effect.offset.x),
                            ("offset_y", effect.offset.y),
                            ("delta_x", effect.offset.x - effect.before.x),
                            ("delta_y", effect.offset.y - effect.before.y),
                        ],
                    );
                    let mut updates = self.scroll_updates.borrow_mut();
                    if let Some((_, offset)) = updates
                        .iter_mut()
                        .find(|(id, _)| *id == effect.target_node_id)
                    {
                        *offset = effect.offset;
                    } else {
                        updates.push((effect.target_node_id, effect.offset));
                    }
                    let mut targets = self.scroll_event_targets.borrow_mut();
                    if !targets.contains(&effect.event_target_node_id) {
                        targets.push(effect.event_target_node_id);
                    }
                    // Keep the proof across coalesced offset writes, but never
                    // admit a scroll that follows another pending mutation.
                    if !self.dirty.get() {
                        self.scroll_only.set(true);
                    }
                    self.dirty.set(true);
                }
            }
            PageMutation::AnimationTick(m) => {
                self.ApplyAnimationMutation(animation::AnimationMutation::Tick(m));
            }
        }
    }

    // cpp: browser/browser.cc:1046-1067
    fn ApplyResourceMutation(&self, m: ResourceMutation) {
        match m {
            ResourceMutation::ImageResourceReady(m) => {
                dom::image_resource::AddImageResource(
                    &mut self.document.borrow_mut(),
                    &mut self.constraints.borrow_mut(),
                    m.source,
                    m.image,
                );
                // Completion updates intrinsic metadata and the resource ids of
                // existing style consumers, without changing their URL/font
                // references. Preserve any pending DOM/sheet discovery; load
                // listeners invalidate references through the normal mutation path.
                self.InvalidateMeasurement();
                self.styles_resolved.set(false);
                self.scroll_only.set(false);
                self.dirty.set(true);
            }
            ResourceMutation::DocumentImageFrameChanged(m) => {
                let mut constraints = self.constraints.borrow_mut();
                let image = constraints
                    .images
                    .iter_mut()
                    .find(|image| image.id == m.frame.resource_id)
                    .unwrap_or_else(|| {
                        dom::error::invalid_argument("animated image resource missing")
                    });
                if image.width != m.frame.intrinsic_size.width
                    || image.height != m.frame.intrinsic_size.height
                {
                    dom::error::invalid_argument("animated image changed intrinsic dimensions");
                }
                if m.frame.revision <= image.revision {
                    dom::error::invalid_argument("animated image revision must increase");
                }
                image.revision = m.frame.revision;
                image.content = image_resource::PaintImageContent::Document(m.frame.record.clone());
                drop(constraints);
                self.pending_paint_mutations.borrow_mut().push(
                    paint::paint_engine::PaintMutation::ImageChanged {
                        image_id: m.frame.resource_id,
                        revision: m.frame.revision,
                        frame: m.frame,
                    },
                );
                self.paint_resources_dirty.set(true);
                self.dirty.set(true);
            }
            ResourceMutation::DocumentImageIntrinsicSizeChanged(m) => {
                let mut constraints = self.constraints.borrow_mut();
                let image = constraints
                    .images
                    .iter_mut()
                    .find(|image| image.id == m.resource_id)
                    .unwrap_or_else(|| {
                        dom::error::invalid_argument("document image resource missing")
                    });
                if m.revision <= image.revision || m.size.width == 0 || m.size.height == 0 {
                    dom::error::invalid_argument("invalid document image intrinsic size revision");
                }
                image.width = m.size.width;
                image.height = m.size.height;
                image.revision = m.revision;
                let resolution_scale = image.resolution_scale;
                drop(constraints);
                if !self
                    .document
                    .borrow_mut()
                    .GetDocumentMut()
                    .UpdateImageResourceIntrinsicSize(
                        &m.source,
                        dom::ImageResourceMetadata {
                            id: m.resource_id,
                            natural_width: m.size.width as f64 / resolution_scale,
                            natural_height: m.size.height as f64 / resolution_scale,
                            resolution_scale,
                        },
                    )
                {
                    dom::error::invalid_argument("document image source missing");
                }
                self.InvalidateMeasurement();
                self.document.borrow_mut().InvalidateLayout();
                self.scroll_only.set(false);
                self.dirty.set(true);
            }
            ResourceMutation::FontResourceReady(m) => {
                self.InvalidateMeasurement();
                self.constraints.borrow_mut().fonts.push(m.font);
                self.document.borrow_mut().InvalidateLayout();
                self.scroll_only.set(false);
                self.dirty.set(true);
            }
            ResourceMutation::ResourceLoadFailed(m) => {
                self.client.borrow_mut().DidFailResource(&m.url, &m.error)
            }
        }
    }
}

mod begin_frame;
mod opening;
mod scripting;
mod scroll;
use opening::PageOpenAdapter;
use scripting::PageScripts;

/// Optional execution services. None disables scripting without constructing
/// a runtime, realm, Window bindings or DOM bindings.
pub struct ScriptEnvironment {
    pub runtime: Box<dyn JavaScriptRuntime>,
    pub xhr: Box<dyn xhr_transport::XMLHttpRequestTransport>,
    pub user_agent: String,
}

// cpp: browser/browser.h:69-95
/// One persistent Page lifecycle, independent of whether scripts are enabled.
pub struct Page {
    begin_frame: begin_frame::PageBeginFrame,
    state: Rc<PageState>,
    interaction: interaction::Interaction<'static>,
    open_engine: OpenEngine,
    resources: Rc<ResourceFetcher>,
    scripts: Option<Box<PageScripts>>,
    script_client: Rc<RefCell<dyn ScriptLoadClient>>,
    frame: Option<PageFrame>,
    cursor_position: Option<Offset>,
    // Blink's MouseEventManager marks hover dirty after layout and resolves it
    // from the last native mouse position at the next BeginMainFrame.
    hover_state_dirty: bool,
    // ScrollableArea::OnScrollFinished marks hover dirty once per completed
    // gesture, after compositor scrolling has stopped moving content beneath
    // a stationary pointer.
    wheel_scroll_active: bool,
    cursor: Cursor,
    // HitTestCache's point/DOM version proof, restricted to one clean committed
    // frame. No fragment Rc is retained; scroll/layout changes advance sequence.
    cursor_hit_test: Option<(
        u64,
        Option<Offset>,
        dom::persistent_document::DOMOwnerHandle,
        u64,
    )>,
    // Chromium retains wheel event regions in the committed layer tree. Keep
    // the flattened root-scroll equivalent until either listener identity or
    // committed fragment geometry changes.
    wheel_event_regions: RefCell<interaction::WheelEventRegionResolver>,
    #[cfg(test)]
    cursor_hit_test_queries: usize,
    active: bool,
    selection_revision: u64,
    editing_paint_dirty: bool,
}
impl Page {
    pub fn HasBlockingWheelListener(&self) -> bool {
        self.scripts
            .as_ref()
            .is_some_and(|scripts| scripts.HasBlockingWheelListener())
    }

    /// Resolve cancelable wheel listener targets against the current immutable
    /// Fragment tree. A document/window target covers the viewport; element
    /// targets include painted descendants because wheel events bubble through
    /// their DOM ancestors.
    pub fn BlockingWheelEventRegions(&self) -> interaction::BlockingWheelEventRegions {
        let Some(scripts) = &self.scripts else {
            return interaction::BlockingWheelEventRegions::default();
        };
        let frame_sequence = self.frame.as_ref().map_or(0, |frame| frame.sequence);
        let owner = self.state.document.borrow();
        self.wheel_event_regions.borrow_mut().Resolve(
            frame_sequence,
            scripts.BlockingWheelListenerTargets(),
            owner.GetDocument(),
            self.frame.as_ref().map(|frame| frame.fragments.as_ref()),
        )
    }

    // cpp: browser/browser.cc:610-737,1834-1856
    pub fn Create(
        loader: Rc<RefCell<dyn URLLoader>>,
        images: Rc<RefCell<dyn ImageDecoder>>,
        document_images: Rc<RefCell<dyn DocumentImageDecoder>>,
        constraints: ConstraintSpace,
        scripting: Option<ScriptEnvironment>,
        client: Option<Rc<RefCell<dyn PageClient>>>,
    ) -> Self {
        // Keep the complete Page dependency graph visible in this one
        // composition root. The individual services remain independent and
        // communicate through their existing typed inputs and immutable
        // snapshots; Page only owns their lifetime and routing.
        let client = client.unwrap_or_else(|| Rc::new(RefCell::new(NullPageClient)));
        let decode_engine = Rc::new(decode::DecodeEngine::WithDecoders(images, document_images));
        let resource_engine = Rc::new(ResourceEngine::WithDecodeEngine(
            loader.clone(),
            decode_engine,
            String::new(),
        ));
        let document = Rc::new(DocumentEngine::new());
        let layout_engine =
            layoutng_assembly::layout_engine::LayoutEngine::new(&crate::CreateLayoutAssembly());
        let paint_engine = paint::PaintEngine::new();
        #[cfg(feature = "pure_source_png")]
        let rendering = PageRenderPipeline::default();
        let state = PageState::WithEngines(
            document,
            layout_engine,
            paint_engine,
            #[cfg(feature = "pure_source_png")]
            rendering,
            constraints,
            client.clone(),
        );
        let resources = Rc::new(ResourceFetcher::WithEngine(
            resource_engine.clone(),
            state.document.Handle(),
            state.constraints.clone(),
        ));
        state.ConnectResources(resources.clone());
        let script_client: Rc<RefCell<dyn ScriptLoadClient>> =
            Rc::new(RefCell::new(ScriptClient(client, Rc::downgrade(&state))));
        let scripts = scripting.map(|environment| {
            Box::new(PageScripts::new(
                state.clone(),
                environment,
                script_client.clone(),
            ))
        });
        let open_engine = OpenEngine::new(
            resource_engine.clone(),
            scripts.as_ref().map(|scripts| scripts.Executor()),
        );
        let interaction = if let Some(scripts) = &scripts {
            scripts.Engine()
        } else {
            let mutate = state.clone();
            interaction::Interaction::WithSelectionState(
                Rc::new(move |output| mutate.ApplyInteractionOutput(output)),
                None,
                state.layout_editing.selections.clone(),
            )
        };
        Self {
            begin_frame: begin_frame::PageBeginFrame::default(),
            state,
            interaction,
            open_engine,
            resources,
            scripts,
            script_client,
            frame: None,
            cursor_position: None,
            hover_state_dirty: false,
            wheel_scroll_active: false,
            cursor: Cursor::kDefault,
            cursor_hit_test: None,
            wheel_event_regions: RefCell::new(Default::default()),
            #[cfg(test)]
            cursor_hit_test_queries: 0,
            active: true,
            selection_revision: 0,
            editing_paint_dirty: false,
        }
    }
    pub fn ResourceEngine(&self) -> Rc<ResourceEngine> {
        self.open_engine.Resources()
    }
    pub fn SetPreferredColorScheme(&mut self, preference: PreferredColorScheme) {
        if let Some(scripts) = &self.scripts {
            scripts.SetPreferredColorScheme(preference);
        }
        if self.state.preferred_color_scheme.replace(preference) != preference {
            self.state.InvalidateMeasurement();
            self.state.InvalidateStyle();
        }
    }
    // cpp: browser/browser.cc:740-799
    /// Start navigation without waiting for headers, body or parser completion.
    /// The host event loop drives `RunTask`; later failures reach DidFail and
    /// that task's result. IsLoading includes pending document resources/scripts.
    pub fn Open(&mut self, url: &str, chunk_size: usize, token_budget: usize) -> io::Result<()> {
        self.OpenRequest(
            &URLRequest {
                url: url.into(),
                ..Default::default()
            },
            chunk_size,
            token_budget,
        )
    }
    pub fn OpenRequest(
        &mut self,
        request: &URLRequest,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<()> {
        let mut trace = browser_tracing::span("navigation", "OpenRequest");
        trace.set("chunk_size", chunk_size as f64);
        trace.set("token_budget", token_budget as f64);
        let result = self.OpenDocument(request, chunk_size, token_budget);
        trace.set("failed", result.is_err() as u8 as f64);
        if let Err(error) = &result {
            let mut adapter = PageOpenAdapter::new(
                self.state.clone(),
                self.resources.clone(),
                &mut self.scripts,
                self.script_client.clone(),
            );
            self.open_engine.Fail(&mut adapter, error);
        }
        result
    }
    fn OpenDocument(
        &mut self,
        request: &URLRequest,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<()> {
        self.open_engine
            .Open(
                request.clone(),
                DocumentLoadBudget {
                    body_bytes: chunk_size,
                    parser_tokens: token_budget,
                },
            )
            .map(|_| ())
    }
    /// Drive the ordinary OpenEngine path to completion for serialized fixtures.
    /// This method supplies only the host loop; URL loading, parsing, resource
    /// readiness and load completion remain owned by OpenEngine.
    #[cfg(test)]
    pub(crate) fn OpenSynchronously(
        &mut self,
        url: &str,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<()> {
        self.Open(url, chunk_size, token_budget)?;
        while self.open_engine.IsLoading() {
            self.RunTask()?;
        }
        Ok(())
    }
    pub fn IsLoading(&self) -> bool {
        self.open_engine.IsLoading()
    }
    /// A body-stream failure after parser initialization keeps this document.
    /// The host can report the failure without replacing its received content.
    pub fn HasCommittedDocument(&self) -> bool {
        self.open_engine.HasCommittedDocument()
    }
    pub fn LoadingFailure(&self) -> Option<&str> {
        self.open_engine.LoadingFailure()
    }
    pub fn OpenStatus(&self) -> OpenStatus {
        self.open_engine.Status()
    }
    pub fn SetLoadingWakeCallback(&mut self, callback: URLWakeCallback) {
        self.open_engine.SetWakeCallback(callback);
    }
    pub fn StopLoading(&mut self) {
        self.open_engine.Stop();
    }
    /// OpenEngine owns navigation admission and completion state. Page adapts
    /// the selected parser/script implementation and routes lifecycle effects.
    fn PumpLoading(&mut self) -> io::Result<()> {
        let mut trace = browser_tracing::span("lifecycle", "Page.PumpLoading");
        trace.set("loading", self.open_engine.IsLoading() as u8 as f64);
        if !self.open_engine.IsLoading() {
            return Ok(());
        }
        let mut adapter = PageOpenAdapter::new(
            self.state.clone(),
            self.resources.clone(),
            &mut self.scripts,
            self.script_client.clone(),
        );
        self.open_engine.Advance(&mut adapter)?;
        Ok(())
    }
    pub fn IsRenderingReady(&self) -> bool {
        // The Page lifecycle owns the first-content barrier. Parser/resource
        // scheduling reports Blink-style render-blocking sheets discovered
        // before body insertion; renderer and host presentation never infer
        // loading policy. Once the first frame exists, later sheets restyle
        // normally without freezing input, matching rendering_has_begun_.
        if self.frame.is_some() {
            return true;
        }
        if !self.open_engine.HasCommittedDocument() {
            return false;
        }
        if self.open_engine.HasPendingRenderBlockingStyleSheets() {
            return false;
        }
        self.state.document.HasRenderableRoot()
    }
    // cpp: browser/browser.cc:1646-1680
    fn ParseDocument(
        &mut self,
        source: &str,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<()> {
        let mut adapter = PageOpenAdapter::new(
            self.state.clone(),
            self.resources.clone(),
            &mut self.scripts,
            self.script_client.clone(),
        );
        self.open_engine
            .ParseSuppliedDocument(&mut adapter, source, chunk_size, token_budget)
            .map(|_| ())
    }
    // cpp: browser/browser.cc:922-929
    pub fn Apply(&mut self, mutation: PageMutation) -> io::Result<()> {
        if let PageMutation::DOMMutation(m) = &mutation {
            if m.mutation_type == DOMMutationType::kParseDocument {
                self.ParseDocument(&m.value, 16 * 1024, 4096)?;
                if let Some(scripts) = &self.scripts {
                    scripts.FlushTasks();
                }
                self.FinishSuppliedDocumentResources()?;
                return self.UpdateFrameIfNeeded();
            }
        }
        if let Some(scripts) = &mut self.scripts {
            scripts.ApplyMutation(mutation)?;
        } else {
            self.state.ApplyMutation(mutation);
        }
        self.UpdateFrameIfNeeded()
    }
    pub fn Evaluate(&mut self, source: &str, source_name: &str) -> io::Result<JavaScriptResult> {
        let Some(scripts) = &mut self.scripts else {
            return Ok(JavaScriptResult::Failure(
                javascript::javascript_runtime::JavaScriptExceptionKind::kRuntimeError,
                "No JavaScript runtime",
                "",
                0,
                0,
            ));
        };
        let result = scripts.Evaluate(source, source_name)?;
        self.UpdateFrameIfNeeded()?;
        Ok(result)
    }
    /// Execute one complete Page task turn, including its microtask checkpoint
    /// and any lifecycle update caused by that task. Task selection and wake
    /// delivery remain the responsibility of the embedding event loop.
    pub fn RunTask(&mut self) -> io::Result<()> {
        self.RunTaskTurn(0.0)
    }

    /// Drive Page tasks for a bounded interval. This is an explicit embedding
    /// helper for synchronous document bootstrap and tests; native operation
    /// uses `RunTask`, one event-loop turn at a time.
    pub fn RunFor(&mut self, budget: std::time::Duration) -> io::Result<()> {
        self.RunTaskTurn(budget.as_secs_f64() * 1000.0)
    }

    fn RunTaskTurn(&mut self, milliseconds: f64) -> io::Result<()> {
        let mut trace = browser_tracing::span("lifecycle", "Page.RunTask");
        trace.set("budget_ms", milliseconds);
        trace.set("loading", self.open_engine.IsLoading() as u8 as f64);
        if self.begin_frame.source.is_none() {
            self.FlushFrameInputs()?;
        }
        self.open_engine.BeginTaskTurn(milliseconds);
        let result = (|| {
            let started = std::env::var_os("BROWSER_PROFILE_INPUT")
                .is_some()
                .then(std::time::Instant::now);
            let was_loading = self.open_engine.IsLoading();
            self.PumpLoading()?;
            if !self.open_engine.IsLoading() {
                self.resources.StartDeferredImages(4);
            }
            // A clean frame does not imply that late image/font requests have no
            // work. Continue their transport/completion lifecycle after window load.
            if !was_loading
                && (self.resources.HasPendingImages() || self.resources.HasPendingFonts())
            {
                self.ResolveStylesAndLoadResources()?;
            }
            let loading_done = started.map(|start| start.elapsed());
            let run_host_tasks = if let Some(scripts) = &mut self.scripts {
                self.open_engine
                    .RunScriptTaskTurn(milliseconds, |scheduler, executor, limit| {
                        scripts.PumpModuleTasks(scheduler, executor, limit)
                    })?
            } else {
                false
            };
            if run_host_tasks {
                self.scripts
                    .as_deref_mut()
                    .expect("host tasks require a script environment")
                    .RunHostTaskTurn(milliseconds)?;
            }
            // Hosts without a BeginFrameSource treat this task turn as their
            // rendering opportunity. Native hosts dispatch the same queue in
            // Page::UpdateRendering before rAF.
            if self.begin_frame.source.is_none() {
                if !self.state.scroll_event_targets.borrow().is_empty() {
                    self.state.ApplyPendingScrollUpdates(self.frame.as_mut());
                }
                if let Some(scripts) = &mut self.scripts {
                    scripts.DispatchPendingScrollEvents();
                }
            }
            let tasks_done = started.map(|start| start.elapsed());
            self.UpdateFrameIfNeeded()?;
            if let Some(start) = started {
                if start.elapsed() >= std::time::Duration::from_millis(16) {
                    eprintln!(
                        "page-task-profile loading_ms={:.3} script_tasks_ms={:.3} lifecycle_ms={:.3} total_ms={:.3}",
                        loading_done.unwrap().as_secs_f64() * 1000.0,
                        (tasks_done.unwrap() - loading_done.unwrap()).as_secs_f64() * 1000.0,
                        (start.elapsed() - tasks_done.unwrap()).as_secs_f64() * 1000.0,
                        start.elapsed().as_secs_f64() * 1000.0
                    );
                }
            }
            if std::env::var_os("BROWSER_TRACE_LOADING_STATE").is_some() {
                eprintln!(
                    "page-loading-state loading={} document={:?} images={} fonts={}",
                    self.open_engine.IsLoading(),
                    self.open_engine.LoaderStatus(),
                    self.resources.HasPendingImages(),
                    self.resources.HasPendingFonts()
                );
                self.open_engine.TraceLoadingState();
            }
            Ok(())
        })();
        // IO failures must not leave a one-turn allowance attached to later
        // Evaluate/Apply/Dispatch or the explicit synchronous parser adapter.
        self.open_engine.EndTaskTurn();
        result
    }
    /// Earliest ordinary Window task deadline. Document/resource transports
    /// wake the owner through `SetLoadingWakeCallback`; this value covers
    /// timers and the remaining poll-based Window task sources.
    pub fn NextTaskDeadline(&self, now: std::time::Instant) -> Option<std::time::Instant> {
        self.scripts
            .as_ref()
            .and_then(|scripts| scripts.NextTaskDeadline(now))
    }
    pub fn ResizeViewport(&mut self, width: f64, height: f64, device_scale: f64) -> io::Result<()> {
        if !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
            || !device_scale.is_finite()
            || device_scale <= 0.0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid host viewport",
            ));
        }
        self.state.constraints.borrow_mut().device_pixel_ratio = device_scale;
        if let Some(scripts) = &self.scripts {
            scripts.ResizeViewport(width, height, device_scale);
        }
        self.Apply(PageMutation::ViewportMutation(
            page_mutation::ViewportMutation {
                size: layoutng_assembly::internal::layout_input::Size { width, height },
            },
        ))
    }
    // cpp: browser/browser.cc:894-903
    pub fn Dispatch(
        &mut self,
        input: &interaction::input_event::InputEvent,
    ) -> io::Result<interaction::interaction_engine::InteractionResult> {
        self.FlushFrameInputs()?;
        let started = std::env::var_os("BROWSER_PROFILE_INPUT").map(|_| std::time::Instant::now());
        let result = self.DispatchInputWithoutLifecycle(input)?;
        let dispatched = started.map(|start| start.elapsed());
        self.UpdateFrameIfNeeded()?;
        self.UpdateCursor();
        if let Some(start) = started {
            eprintln!(
                "page-input-profile dispatch_ms={:.3} lifecycle_cursor_ms={:.3}",
                dispatched.unwrap().as_secs_f64() * 1000.0,
                (start.elapsed() - dispatched.unwrap()).as_secs_f64() * 1000.0
            );
        }
        Ok(result)
    }
    fn DispatchInputWithoutLifecycle(
        &mut self,
        input: &interaction::input_event::InputEvent,
    ) -> io::Result<interaction::interaction_engine::InteractionResult> {
        if let interaction::input_event::InputEvent::Mouse(event) = input {
            self.cursor_position =
                if event.r#type == interaction::input_event::MouseEventType::kLeave {
                    None
                } else {
                    Some(event.position)
                };
            // Blink's mouse-move path selects the cursor from the hit result
            // before dispatching script. Native feedback must not await paint.
            self.UpdateCursorFeedback(true);
        }
        let frame = self
            .frame
            .as_ref()
            .ok_or_else(|| io::Error::other("cannot dispatch input before the first frame"))?;
        let result = if let Some(scripts) = &mut self.scripts {
            scripts.Dispatch(input, &frame.fragments)?
        } else {
            self.interaction
                .Dispatch(input, &self.state.document.Handle(), &frame.fragments)
        };
        let restart = matches!(
            input,
            interaction::input_event::InputEvent::Key(_)
                | interaction::input_event::InputEvent::TextInput(_)
                | interaction::input_event::InputEvent::Composition(_)
                | interaction::input_event::InputEvent::Focus(_)
        ) || matches!(input, interaction::input_event::InputEvent::Mouse(event)
                if event.r#type == interaction::input_event::MouseEventType::kDown);
        self.UpdateCaret(restart);
        Ok(result)
    }
    /// Host view activation is independent of the DOM's focused element.
    pub fn SetActive(&mut self, active: bool) -> io::Result<()> {
        if self.active != active {
            self.active = active;
            self.UpdateCaret(true);
            self.UpdateFrameIfNeeded()?;
        }
        Ok(())
    }
    pub fn Caret(&self) -> Option<CaretGeometry> {
        self.frame
            .as_ref()
            .and_then(|frame| frame.display_items.caret)
    }
    fn UpdateCaret(&mut self, restart: bool) {
        let position = self
            .active
            .then(|| self.interaction.State().focused_node_id)
            .flatten()
            .and_then(|id| {
                self.interaction
                    .Editor()
                    .CaretFor(&self.state.document.Handle(), id)
            });
        let mut caret = self.state.layout_editing.caret.borrow_mut();
        let changed = caret.update(position, std::time::Instant::now(), restart);
        drop(caret);
        let revision = self.state.layout_editing.selections.Revision();
        let selection_changed = self.selection_revision != revision;
        self.selection_revision = revision;
        if changed || selection_changed {
            self.editing_paint_dirty = true;
            // FrameCaret and CaretDisplayItemClient schedule paint
            // invalidation; moving/hiding the caret or changing a selection
            // does not make the text control need layout. Control-value DOM
            // mutations still invalidate geometry through ApplyDOMMutation.
            // Keep the completed fragment geometry for the paint-only pass.
            self.state.PreserveMeasurementGeometry();
            self.state.document.borrow_mut().InvalidatePaint();
            self.state.scroll_only.set(false);
            self.state.dirty.set(true);
        }
    }
    pub fn Cursor(&self) -> Cursor {
        self.cursor
    }
    /// Preserve the widget's native mouse position across document replacement.
    /// The new document resolves its hover target after it has first produced
    /// hit-test geometry, matching Blink's persistent MouseEventManager state.
    pub fn SetLastMousePosition(&mut self, position: Option<Offset>) {
        if self.cursor_position != position {
            self.cursor_position = position;
            self.hover_state_dirty = position.is_some();
            self.cursor_hit_test = None;
            self.RequestBeginFrameIfNeeded();
        }
    }
    pub fn FocusedNodeId(&self) -> Option<u64> {
        self.state.interaction.borrow().focused_node_id
    }
    fn UpdateCursor(&mut self) {
        self.UpdateCursorFeedback(false);
    }
    fn UpdateCursorFeedback(&mut self, force: bool) {
        // Blink LayoutView::HitTestNoLifecycleUpdate consults HitTestCache for
        // the same point and DOM version. Coalesce repeated cursor queries
        // from lifecycle and BeginFrame/Flush tails with the same proof here.
        // Pending mutations/uncertain styles cannot use an old hit result.
        let query = if !self.state.dirty.get() && self.state.styles_resolved.get() {
            let owner = self.state.document.borrow();
            let document = owner.GetDocument();
            let style = document.StyleState();
            if style.impact.IsEmpty()
                && !style.all_dirty
                && style.changes.is_empty()
                && style.dirty_style_elements.is_empty()
            {
                document
                    .CollectionMembershipKey()
                    .zip(self.frame.as_ref())
                    .map(|((owner, revision), frame)| {
                        (frame.sequence, self.cursor_position, owner, revision)
                    })
            } else {
                None
            }
        } else {
            None
        };
        if !force && query.is_some() && query == self.cursor_hit_test {
            return;
        }
        self.cursor_hit_test = query;
        #[cfg(test)]
        {
            self.cursor_hit_test_queries += 1;
        }
        let cursor = match (self.cursor_position, self.frame.as_ref()) {
            (Some(point), Some(frame)) => {
                self.interaction
                    .CursorAt(&self.state.document.Handle(), &frame.fragments, point)
            }
            _ => Cursor::kDefault,
        };
        // A host may have switched from another Page (browser toolbar) even
        // when this Page's cached cursor has not changed.
        if force || cursor != self.cursor {
            self.cursor = cursor;
            self.state.client.borrow_mut().DidChangeCursor(cursor);
        }
    }
    pub fn URL(&self) -> &str {
        self.open_engine.CurrentURL()
    }
    pub fn Document(&self) -> std::cell::Ref<'_, DOM> {
        self.state.document.borrow()
    }
    /// Read-only access to resident native state for diagnostics.
    #[doc(hidden)]
    pub fn GetLayoutEngine(
        &self,
    ) -> std::cell::Ref<'_, layoutng_assembly::layout_engine::LayoutEngine> {
        self.state.layout_engine.borrow()
    }
    pub fn CurrentFrame(&self) -> Option<&PageFrame> {
        self.frame.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn FullLayoutLifecycleCount(&self) -> usize {
        self.state.full_layout_lifecycles.get()
    }

    // Explicit complete-source mutation/fixture callers retain their synchronous
    // resource-completion contract. Network navigation never enters this adapter.
    fn FinishSuppliedDocumentResources(&mut self) -> io::Result<()> {
        self.state.ResolveStyles();
        self.resources.QueueReferencedImages()?;
        self.resources.SelectUsedFontFaces();
        self.resources.StartPendingFonts();
        let mut client = ResourceClient {
            state: self.state.clone(),
            scripts: self.scripts.as_deref_mut(),
        };
        self.resources.LoadPendingImages(&mut client);
        self.resources.LoadPendingFonts(&mut client);
        self.state.ResolveStyles();
        Ok(())
    }

    // cpp: browser/browser.cc:1761-1771
    fn ResolveStylesAndLoadResources(&mut self) -> io::Result<()> {
        opening::ResolvePageStylesAndLoadResources(
            &self.state,
            &self.resources,
            &mut self.scripts,
            usize::MAX,
        )
        .map(|_| ())
    }

    // cpp: browser/browser.cc:1731-1745
    fn UpdateFrameIfNeeded(&mut self) -> io::Result<()> {
        let mut trace = browser_tracing::span("lifecycle", "Page.UpdateFrameIfNeeded");
        trace.set("dirty", self.state.dirty.get() as u8 as f64);
        trace.set("presented", 0.0);
        self.UpdateCaret(false);
        if self.DeferLifecycleForBeginFrame() {
            return Ok(());
        }
        if !self.state.dirty.get() {
            return Ok(());
        }
        if !self.IsRenderingReady() {
            return Ok(());
        }
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT").map(|_| std::time::Instant::now());
        // Resource completions are separate Page tasks. An offset-only input
        // must not synchronously decode images or dispatch unrelated load
        // listeners before presenting its already committed paint artifact.
        // Subsequent task turns continue polling those completions normally.
        let retained_input_only = (self.state.scroll_only.get()
            || self.state.paint_resources_dirty.get())
            && self.frame.is_some()
            && !self.editing_paint_dirty
            && self.state.styles_resolved.get()
            && self.state.document.borrow().GetStyleImpact().IsEmpty();
        trace.set("retained_input_only", retained_input_only as u8 as f64);
        if !retained_input_only {
            self.ResolveStylesAndLoadResources()?;
        }
        let resources_done = profile.map(|start| start.elapsed());
        let reused_recordings = self.state.ApplyPendingScrollUpdates(self.frame.as_mut());
        trace.set("scroll_records_reused", reused_recordings as u8 as f64);
        let scroll_done = profile.map(|start| start.elapsed());
        let paint_only_style = self.frame.is_some() && {
            let impact = self.state.document.borrow().GetStyleImpact();
            impact.paint && !impact.layout && !impact.reattach
        };
        trace.set("paint_only_style", paint_only_style as u8 as f64);
        if self.frame.is_some()
            && !self.editing_paint_dirty
            && self.state.document.borrow().GetStyleImpact().IsEmpty()
        {
            trace.set("frame_retained", 1.0);
            if self.state.paint_resources_dirty.get() {
                self.state
                    .PublishPaintResources(self.frame.as_mut().unwrap());
                trace.set("paint_resources_updated", 1.0);
            }
            // Keep the host's presentation/lifecycle contract while reusing
            // the existing fragments and display list for an ineffective edit.
            if let Some(start) = profile {
                eprintln!(
                    "page-lifecycle-profile resources_ms={:.3} scroll_layout_paint_ms={:.3} layout_ms=0 paint_ms=0 total_ms={:.3} reused=true scroll_records_reused={}",
                    resources_done.unwrap().as_secs_f64() * 1000.0,
                    (scroll_done.unwrap() - resources_done.unwrap()).as_secs_f64() * 1000.0,
                    start.elapsed().as_secs_f64() * 1000.0,
                    reused_recordings
                );
            }
            let frame = self.frame.as_mut().unwrap();
            if self
                .state
                .measurement
                .borrow()
                .as_ref()
                .is_none_or(|snapshot| !snapshot.PaintCurrent())
            {
                self.state.InstallMeasurement(frame.fragments.clone());
            }
            frame.sequence += 1;
            self.state.scroll_only.set(false);
            self.state.dirty.set(false);
            if let Some(scripts) = &mut self.scripts {
                scripts.DidPaint();
            }
            self.state.client.borrow_mut().DidPresentFrame(frame);
            trace.set("presented", 1.0);
            self.UpdateCursor();
            return Ok(());
        }
        // A full geometry flush may already own the current exported paint
        // inputs. A geometry-only reuse after a proven color/opacity change
        // does not: its old paint properties must be exported again below.
        let measured = {
            let mut measurement = self.state.measurement.borrow_mut();
            if measurement
                .as_ref()
                .is_some_and(MeasurementSnapshot::PaintCurrent)
            {
                measurement.take()
            } else {
                None
            }
        };
        let measurement_reused = measured.is_some();
        trace.set("measurement_reused", measurement_reused as u8 as f64);
        let (measured, measurement_cache) = match measured {
            Some(snapshot) => {
                let (fragments, cache) = snapshot.IntoPaintParts();
                (Some(fragments), Some(cache))
            }
            None => (None, None),
        };
        let animation_paint_nodes = if paint_only_style {
            self.state
                .paint_engine
                .borrow()
                .AnimationTargetedNodes()
                .to_vec()
        } else {
            Vec::new()
        };
        // A targeted paint mutation updates the retained FragmentData-style
        // snapshot before taking it for Paint. Generic export replaces the
        // snapshot, so release its old reference first. Either path leaves no
        // extra engine reference while CommitPaint mutates client lifecycle.
        if animation_paint_nodes.is_empty() {
            self.state.layout_engine.borrow_mut().ReleaseLayoutResult();
        } else if let Some(fragments) = &measured {
            self.state
                .layout_engine
                .borrow_mut()
                .PublishLayoutResult(fragments.clone());
        }
        trace.set(
            "targeted_animation_paint",
            (!animation_paint_nodes.is_empty()) as u8 as f64,
        );
        let paint_only_fragments = paint_only_style
            .then(|| {
                let mut layout = self.state.layout_engine.borrow_mut();
                if animation_paint_nodes.is_empty() {
                    crate::persistent_layout::ExportPaintOnlyDocumentWithEngine(
                        &mut layout,
                        &mut self.state.document.borrow_mut(),
                        &self.state.interaction.borrow(),
                    )
                } else {
                    crate::persistent_layout::ExportTargetedPaintOnlyDocumentWithEngine(
                        &mut layout,
                        &self.state.document.borrow(),
                        &animation_paint_nodes,
                    )
                }
            })
            .flatten();
        trace.set(
            "paint_only_geometry_reused",
            paint_only_fragments.is_some() as u8 as f64,
        );
        let mut fragments = paint_only_fragments.or(measured).unwrap_or_else(|| {
            trace.set("full_layout", 1.0);
            self.state.LayoutWithContainerQueries();
            self.state
                .layout_engine
                .borrow_mut()
                .TakeLayoutResult()
                .expect("successful layout has fragments")
        });
        let layout_done = profile.map(|start| start.elapsed());
        self.state.document.borrow().EmitEditingState(
            self.state.layout_editing.clone(),
            |mutation| {
                self.state
                    .layout_engine
                    .borrow_mut()
                    .ApplyMutation(mutation);
            },
        );
        let display_items = {
            trace.set("full_paint", 1.0);
            let mut paint = self.state.paint_engine.borrow_mut();
            paint.Paint(
                &fragments,
                self.state.layout_editing.caret.borrow().paint_state(),
            );
            paint.CommitPaintResult(|artifact| {
                CommitPaint(
                    &mut self.state.layout_engine.borrow_mut(),
                    &self.state.layout_editing.caret.borrow(),
                    &mut fragments,
                    artifact,
                )
            });
            let artifact = paint
                .GetPaintResult()
                .expect("successful paint has artifact");
            artifact.clone()
        };
        if let Some(start) = profile {
            let elapsed = start.elapsed();
            eprintln!(
                "page-lifecycle-profile resources_ms={:.3} scroll_layout_paint_ms={:.3} layout_ms={:.3} paint_ms={:.3} total_ms={:.3} reused=false measurement_reused={}",
                resources_done.unwrap().as_secs_f64() * 1000.0,
                (scroll_done.unwrap() - resources_done.unwrap()).as_secs_f64() * 1000.0,
                (layout_done.unwrap() - scroll_done.unwrap()).as_secs_f64() * 1000.0,
                (elapsed - layout_done.unwrap()).as_secs_f64() * 1000.0,
                elapsed.as_secs_f64() * 1000.0,
                measurement_reused
            );
        }
        let frame = PageFrame {
            sequence: self.frame.as_ref().map_or(1, |f| f.sequence + 1),
            fragments,
            display_items,
        };
        self.state
            .layout_engine
            .borrow_mut()
            .PublishLayoutResult(frame.fragments.clone());
        // Geometry queries after presentation share this completed lifecycle;
        // later inline/animation edits retain the index only if actual style
        // resolution proves the complete query geometry unchanged.
        if let Some(cache) = measurement_cache {
            self.state.measurement_candidate.borrow_mut().take();
            *self.state.measurement.borrow_mut() = Some(MeasurementSnapshot::FromPaintParts(
                frame.fragments.clone(),
                cache,
            ));
        } else {
            let measurement_changed = self
                .state
                .measurement
                .borrow()
                .as_ref()
                .is_none_or(|snapshot| !Rc::ptr_eq(&snapshot.Fragments(), &frame.fragments));
            if measurement_changed {
                self.state.InstallMeasurement(frame.fragments.clone());
            }
        }
        self.frame = Some(frame);
        self.editing_paint_dirty = false;
        // LocalFrameView::PerformPostLayoutTasks marks hover dirty. The
        // synthetic move is deliberately deferred to BeginFrame so script
        // cannot re-enter the layout that just completed.
        self.hover_state_dirty = self.cursor_position.is_some();
        self.state.document.borrow_mut().DidCommitPaint();
        self.state.paint_resources_dirty.set(false);
        self.state.pending_paint_mutations.borrow_mut().clear();
        self.state
            .ApplyAnimationMutation(animation::AnimationMutation::CommitPaint);
        self.state.scroll_only.set(false);
        self.state.dirty.set(false);
        if let Some(scripts) = &mut self.scripts {
            scripts.DidPaint();
        }
        self.state
            .client
            .borrow_mut()
            .DidPresentFrame(self.frame.as_ref().unwrap());
        trace.set("presented", 1.0);
        self.UpdateCursor();
        Ok(())
    }
}
#[cfg(test)]
#[test]
fn cursor_hit_test_same_point_reuses_clean_frame_and_scroll_invalidates() {
    crate::native_test_thread::run(|| {
        struct NoNetwork;
        impl URLLoader for NoNetwork {
            fn Load(
                &mut self,
                _: &URLRequest,
            ) -> io::Result<Box<dyn url_loader::URLLoadOperation>> {
                Err(io::Error::other("unexpected fixture network request"))
            }
        }
        let assembly = crate::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(NoNetwork)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(document_image::SVGImageDecoder::new(
                &assembly,
            ))),
            crate::CreateBrowserConstraints(320, 200),
            None,
            None,
        );
        *page.state.document.borrow_mut() = html::html_parser::ParseHTML(
            "<!doctype html><body style='margin:0'><div id=s style='width:100px;height:40px;overflow:hidden;cursor:crosshair'><div style='height:200px'>scrolling</div></div></body>",
        );
        page.open_engine.AdoptDocument().unwrap();
        page.cursor_position = Some(Offset { x: 10.0, y: 10.0 });
        page.UpdateFrameIfNeeded().unwrap();
        assert!(page.cursor_hit_test.is_some());
        let queries = page.cursor_hit_test_queries;
        page.UpdateCursor();
        assert_eq!(
            page.cursor_hit_test_queries, queries,
            "the lifecycle tail must reuse its clean hit"
        );
        let scroller = {
            let owner = page.state.document.borrow();
            let document = owner.GetDocument();
            (0..document.NodeCount())
                .find_map(|i| {
                    document
                        .Node(i)
                        .FindAttribute("id")
                        .filter(|attribute| attribute.value == "s")
                        .map(|_| document.Node(i).Id())
                })
                .unwrap()
        };
        let before = page.CurrentFrame().unwrap().sequence;
        page.state.ApplyMutation(PageMutation::ScrollMutation(
            page_mutation::ScrollMutation {
                target_node_id: scroller,
                offset: Offset { x: 0.0, y: 20.0 },
            },
        ));
        page.UpdateFrameIfNeeded().unwrap();
        assert!(page.CurrentFrame().unwrap().sequence > before);
        fn scroll_offset(fragment: &FragmentNode, id: u64) -> Option<Offset> {
            if fragment.node_id == id {
                return Some(fragment.paint.scroll_offset);
            }
            fragment
                .children
                .iter()
                .find_map(|child| scroll_offset(child, id))
        }
        assert_eq!(
            scroll_offset(&page.CurrentFrame().unwrap().fragments, scroller),
            Some(Offset { x: 0.0, y: 20.0 })
        );
        assert_eq!(
            page.cursor_hit_test_queries,
            queries + 1,
            "same point after real scroll needs a fresh hit"
        );
        page.UpdateCursor();
        assert_eq!(page.cursor_hit_test_queries, queries + 1);
        page.UpdateCursorFeedback(true);
        assert_eq!(
            page.cursor_hit_test_queries,
            queries + 2,
            "immediate mouse feedback remains forced"
        );
    });
}
struct ResourceClient<'a> {
    state: Rc<PageState>,
    scripts: Option<&'a mut PageScripts>,
}
impl ResourceFetcherClient for ResourceClient<'_> {
    fn HasJavaScript(&self) -> bool {
        self.scripts.is_some()
    }
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation) {
        self.state.ApplyResourceMutation(mutation);
    }
    fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        self.scripts
            .as_deref_mut()
            .expect("image listeners require scripting")
            .DispatchImageEvent(id, loaded);
    }
}
impl Drop for Page {
    fn drop(&mut self) {
        self.state.CancelImageEvents();
        self.state.connected_resources.SetScriptPreparer(None);
    }
}

// Mutation notification adapter. Image completion only registers a DOM task;
// it must not call listeners recursively inside the emitting host operation.
struct ScopedResourceClient<'a> {
    state: Rc<PageState>,
    runtime: &'a mut dyn JavaScriptHostRuntime,
}
impl ResourceFetcherClient for ScopedResourceClient<'_> {
    fn HasJavaScript(&self) -> bool {
        true
    }
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation) {
        self.state.ApplyResourceMutation(mutation);
    }
    fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        self.state.QueueImageEvent(id, loaded);
    }
}
fn MakeImageEvent(id: u64, loaded: bool) -> interaction::event::Event {
    let mut event = MakeSyntheticEvent(
        if loaded {
            EventType::kLoad
        } else {
            EventType::kCustom
        },
        id,
    );
    if !loaded {
        event.custom_type = "error".into();
    }
    event.bubbles = false;
    event.cancelable = false;
    event.composed = false;
    event.trusted = true;
    event
}
struct JavaScriptResourceClient<'a> {
    state: Rc<PageState>,
    bindings: Rc<RefCell<DOMJavaScriptBindings>>,
    runtime: &'a mut dyn JavaScriptRuntime,
    realm: &'a JavaScriptRealm,
    engine: interaction::Interaction<'static>,
}
impl ResourceFetcherClient for JavaScriptResourceClient<'_> {
    fn HasJavaScript(&self) -> bool {
        true
    }
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation) {
        self.state.ApplyResourceMutation(mutation);
    }
    fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        self.state.QueueImageEvent(id, loaded);
    }
}
impl JavaScriptResourceClient<'_> {
    fn DispatchImageEventTask(&mut self, id: u64, loaded: bool) {
        let mut event = MakeImageEvent(id, loaded);
        let runtime = RefCell::new(&mut *self.runtime);
        let listeners = Rc::new(
            |invocation: &EventListenerInvocation<'_>,
             _: &interaction::ownership::InteractionDocument,
             _: &interaction::ownership::InteractionDOMMutationEmitter| {
                DispatchListeners(
                    &self.state,
                    &self.bindings,
                    invocation,
                    &mut **runtime.borrow_mut(),
                    self.realm,
                )
            },
        );
        self.engine.WithScopedListeners(listeners).DispatchDOMEvent(
            &mut event,
            id,
            &self.state.document.Handle(),
        );
        drop(runtime);
        // WindowJavaScriptBindings completes this DOM task with the normal
        // full microtask checkpoint and exception reporting, exactly once.
    }
}

// ImageLoader::ImageNotifyFinished posts a cancellable DOM-manipulation task
// (blink/renderer/core/loader/image_loader.cc). Keep registration outside any
// borrowed Window host object: mutations can be emitted from a host callback.
fn EnqueueImageEventTasks(
    state: &Rc<PageState>,
    window: &Rc<RefCell<WindowJavaScriptBindings>>,
    bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
    engine: &interaction::Interaction<'static>,
) {
    let events = std::mem::take(&mut *state.image_events.borrow_mut());
    for event in events {
        if !event.active.get() {
            continue;
        }
        let weak_state = Rc::downgrade(state);
        let weak_bindings = Rc::downgrade(bindings);
        let engine = engine.clone();
        window.borrow_mut().EnqueueTaskWithRuntime(
            Box::new(move |runtime, realm| {
                if !event.active.replace(false) {
                    return;
                }
                let (Some(state), Some(bindings)) = (weak_state.upgrade(), weak_bindings.upgrade())
                else {
                    return;
                };
                let current_request = {
                    let owner = state.document.borrow();
                    let tree = owner.GetDocument();
                    tree.FindNodeById(event.id).is_some_and(|i| {
                        let node = tree.Node(i);
                        node.IsHTMLElement("img")
                            && node
                                .FindAttribute("src")
                                .is_some_and(|a| a.value == event.source)
                    })
                };
                if current_request {
                    JavaScriptResourceClient {
                        state: state.clone(),
                        bindings,
                        runtime,
                        realm,
                        engine,
                    }
                    .DispatchImageEventTask(event.id, event.loaded);
                }
                // A listener may have queued another request for the same element.
                let mut pending = state.pending_image_events.borrow_mut();
                if pending
                    .get(&event.id)
                    .is_some_and(|active| Rc::ptr_eq(active, &event.active))
                {
                    pending.remove(&event.id);
                }
            }),
            None,
        );
    }
}

// cpp: browser/browser.cc:702-734
fn DispatchListeners(
    state: &PageState,
    bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
    invocation: &EventListenerInvocation<'_>,
    runtime: &mut dyn JavaScriptRuntime,
    realm: &JavaScriptRealm,
) -> interaction::event::EventListenerResult {
    let at_document = {
        let owner = state.document.borrow();
        invocation.current_target_node_id
            == owner.GetDocument().Node(owner.GetDocument().Root()).Id()
    };
    let window = EventListenerInvocation {
        current_target_node_id: 0,
        ..*invocation
    };
    let connects_window = crate::interaction_services::NodeEventReachesWindow(invocation.event);
    let mut result = interaction::event::EventListenerResult::default();
    let mut dispatch = |invocation: &EventListenerInvocation<'_>| {
        DOMJavaScriptBindings::DispatchEventListeners(
            bindings,
            invocation,
            runtime,
            realm,
            &mut |error| state.client.borrow_mut().DidReportScriptError(error),
        )
    };
    if at_document && connects_window && invocation.capture_listeners {
        result = dispatch(&window);
        if result.stop_propagation || result.stop_immediate_propagation {
            return result;
        }
    }
    fn merge(
        result: &mut interaction::event::EventListenerResult,
        next: interaction::event::EventListenerResult,
    ) {
        result.prevent_default |= next.prevent_default;
        result.stop_propagation |= next.stop_propagation;
        result.stop_immediate_propagation |= next.stop_immediate_propagation;
    }
    merge(&mut result, dispatch(invocation));
    if at_document
        && connects_window
        && !invocation.capture_listeners
        && invocation.event.bubbles
        && !result.stop_propagation
        && !result.stop_immediate_propagation
    {
        merge(&mut result, dispatch(&window));
    }
    result
}

// cpp: browser/browser.cc:1859-1877
/// Real network navigation with the translated Rust runtime and Page services.
/// The returned Page owns its original DOM, runtime, resources and latest frame.
pub fn OpenUrl(
    url: &str,
    width: u32,
    height: u32,
    client: Rc<RefCell<dyn PageClient>>,
) -> io::Result<Page> {
    let options = url_loader::DefaultURLLoaderOptions::default();
    let loader = Rc::new(RefCell::new(url_loader::DefaultURLLoader::new(
        options.clone(),
    )?));
    let xhr = xhr_transport::CreateHTTPXMLHttpRequestTransport(&options)?;
    let assembly = crate::CreateLayoutAssembly();
    let mut page = Page::Create(
        loader,
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(document_image::SVGImageDecoder::new(
            &assembly,
        ))),
        crate::CreateBrowserConstraints(width, height),
        Some(ScriptEnvironment {
            runtime: Box::new(
                javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::new(),
            ),
            xhr: xhr,
            user_agent: options.user_agent,
        }),
        Some(client),
    );
    page.Open(url, 16384, 4096)?;
    while page.IsLoading() {
        page.RunFor(std::time::Duration::from_millis(1))?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    Ok(page)
}

#[derive(Default)]
pub(crate) struct NullPageClient;
impl PageClient for NullPageClient {}

#[cfg(feature = "pure_source_png")]
impl Page {
    /// Plan and raster the current Page artifact into the host's borrowed target.
    pub fn PaintInto(
        &self,
        width: u32,
        height: u32,
        scale: f64,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
    ) -> io::Result<RenderUpdate> {
        let artifact = &self
            .CurrentFrame()
            .ok_or_else(|| io::Error::other("Page has no frame"))?
            .display_items;
        self.PaintArtifactInto(artifact, width, height, scale, target, format, stride)
    }

    /// The host may append transient chrome content to this Page's recording.
    /// Both paths use the same Page-owned LayerTile, Raster, Compositor, Viz
    /// and Renderer instances, so retained identities stay coherent.
    pub fn PaintArtifactInto(
        &self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
    ) -> io::Result<RenderUpdate> {
        self.state
            .rendering
            .borrow_mut()
            .Render(artifact, width, height, scale, target, format, stride)
    }

    pub fn LayerTileStats(&self) -> RasterStats {
        self.state.rendering.borrow().resources.stats()
    }
}
