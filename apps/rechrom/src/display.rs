//! Final display compositor state with one Viz/Renderer service shared by all Pages.
//!
//! This module owns the native surface and display resources, but deliberately
//! knows nothing about threads, channels, or executors. `PresentationRuntime`
//! delivers messages and routes the returned effects.
use crate::page_presentation::{Document, FrameProvenance, PageFrame as SubmittedFrame, Scene};
use crate::{
    engine::{CopyOutputResult, CopyOutputTarget, Output, UserEvent, Viewport},
    window_surface::{DisplayTarget, PreparedDisplayTarget},
};
use browser::page::PageReturnedResource;
use renderer_skia::RendererSkiaBackend;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
    sync::{mpsc, Arc},
    time::Instant,
};

fn display_root_frame(
    root: &crate::page_presentation::BrowserRootFrame,
) -> viz::adapter::DisplayRootFrame {
    let color = |[red, green, blue, alpha]: [u8; 4]| viz::adapter::Color4f {
        red: f32::from(red) / 255.0,
        green: f32::from(green) / 255.0,
        blue: f32::from(blue) / 255.0,
        alpha: f32::from(alpha) / 255.0,
    };
    viz::adapter::DisplayRootFrame {
        background_color: color(root.background_color),
        solid_color_quads: root
            .solid_color_quads
            .iter()
            .map(|quad| viz::adapter::DisplayRootSolidColorQuad {
                destination: viz::adapter::DeviceRect::new(
                    quad.destination.x,
                    quad.destination.y,
                    quad.destination.width as i32,
                    quad.destination.height as i32,
                ),
                color: color(quad.color),
            })
            .collect(),
    }
}

pub(crate) enum Message {
    Scene(Scene),
    Prepare {
        submitted: SubmittedFrame,
    },
    Submit {
        frame: crate::begin_frame_source::BeginFrame,
    },
    SwapAck(u64),
    CopyOutput {
        expected_viewport: Viewport,
        target: CopyOutputTarget,
        /// Page producers that must publish a newer main-frame transaction
        /// than the one currently resident in Display before this copy can
        /// read back the active root.
        fresh_documents: Vec<Document>,
        reply: mpsc::Sender<Result<CopyOutputResult, String>>,
    },
    Stop,
}
pub(crate) enum Effect {
    RequestBeginFrame,
}
#[derive(Clone)]
struct PresentationParticipant {
    frame: viz::adapter::PresentedSurfaceFrame,
    notify: Arc<dyn Fn(browser::page::PageFramePresentationFeedback) + Send + Sync>,
}
struct PageSurface {
    surface: viz::adapter::SurfaceId,
    /// Public child identities embedded by this Page's active parent frame.
    /// They remain Viz-owned resources, not top-level Scene placements.
    child_surfaces: BTreeSet<viz::adapter::SurfaceId>,
    local_surface_id: compositor::runtime::types::LocalSurfaceId,
    viewport: Viewport,
    provenance: FrameProvenance,
    is_handling_interaction: bool,
    reclaim: Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
    child_reclaim: Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
    presentation_feedback: Arc<dyn Fn(browser::page::PageFramePresentationFeedback) + Send + Sync>,
}
struct PreparedDisplayFrame {
    presentation: Vec<PresentationParticipant>,
    output: PreparedDisplayTarget,
    provenance: FrameProvenance,
    sequence: u64,
    viewport: Viewport,
    drag_regions: Vec<crate::chrome::DragRegion>,
    is_handling_interaction: bool,
}

struct PendingCopyOutput {
    expected_viewport: Viewport,
    target: CopyOutputTarget,
    minimum_main_frame_epochs: Vec<(Document, u64)>,
    reply: mpsc::Sender<Result<CopyOutputResult, String>>,
}

pub(crate) struct Display {
    target: DisplayTarget,
    viz: viz::adapter::VizEngine,
    renderer: renderer::Renderer<RendererSkiaBackend>,
    pages: BTreeMap<Document, PageSurface>,
    scene: Option<Scene>,
    pending: Option<()>,
    prepared: Option<PreparedDisplayFrame>,
    sequence: u64,
    next_surface: u64,
    notify: Arc<dyn Fn(UserEvent) + Send + Sync>,
    diagnostic_output: Option<Output>,
    pending_swap: Option<u64>,
    last_rendered_placements: Vec<viz::adapter::SurfacePlacement>,
    last_presented_frame: Option<(Viewport, FrameProvenance)>,
    offscreen_frame_ready_notified: bool,
    pending_copy_outputs: VecDeque<PendingCopyOutput>,
}

impl Display {
    pub(crate) fn New(target: DisplayTarget, output: Output, capture_frames: bool) -> Self {
        Self {
            target,
            viz: viz::adapter::VizEngine::New(),
            renderer: renderer::Renderer::new(RendererSkiaBackend::default()),
            pages: BTreeMap::new(),
            scene: None,
            pending: None,
            prepared: None,
            sequence: 0,
            // Page producers allocate delegated child SurfaceIds in their own
            // low public range. Display roots are private embedding surfaces;
            // keep them in a disjoint range so a child identity can be
            // submitted verbatim as required by its SurfaceDrawQuad.
            next_surface: 1_u64 << 63,
            notify: output.notify.clone(),
            diagnostic_output: capture_frames.then_some(output),
            pending_swap: None,
            last_rendered_placements: Vec::new(),
            last_presented_frame: None,
            offscreen_frame_ready_notified: false,
            pending_copy_outputs: VecDeque::new(),
        }
    }

    pub(crate) fn Handle(
        &mut self,
        message: Message,
        post_swap_started: &Arc<dyn Fn() + Send + Sync>,
        post_swap_ack: &Arc<dyn Fn(u64) + Send + Sync>,
    ) -> io::Result<Vec<Effect>> {
        let effects = match message {
            Message::Stop => {
                for request in self.pending_copy_outputs.drain(..) {
                    let _ = request.reply.send(Err("Display stopped".into()));
                }
                return Ok(Vec::new());
            }
            Message::Scene(scene) => {
                if self.scene.as_ref() == Some(&scene) {
                    return Ok(Vec::new());
                }
                // A prepared target is an aggregate of the exact root scene
                // that existed when it was rendered.  It may be waiting
                // behind the preceding native swap when a tab switch changes
                // the embedded Surface.  Never submit that old aggregate
                // after accepting the new scene: if the new toolbar frame is
                // not ready yet, DrawPending must leave the last *presented*
                // frame on screen rather than publish an already-rendered
                // frame for the previous tab.
                self.target.invalidate_pending_presentations();
                self.prepared = None;
                self.UpdateCopyOutputsForScene(&scene);
                self.scene = Some(scene);
                self.PrunePages();
                self.pending = Some(());
                Ok(vec![Effect::RequestBeginFrame])
            }
            Message::Prepare { submitted } => self.Prepare(submitted),
            Message::Submit { frame } => self.Submit(frame, post_swap_started, post_swap_ack),
            Message::SwapAck(sequence) => {
                let accepted = self.pending_swap == Some(sequence);
                if accepted {
                    self.pending_swap = None;
                }
                browser_tracing::instant(
                    "frame",
                    "DisplaySwapAcknowledged",
                    &[
                        ("present_sequence", sequence as f64),
                        ("accepted", accepted as u8 as f64),
                    ],
                );
                Ok(
                    if accepted && (self.pending.is_some() || self.prepared.is_some()) {
                        vec![Effect::RequestBeginFrame]
                    } else {
                        Vec::new()
                    },
                )
            }
            Message::CopyOutput {
                expected_viewport,
                target,
                fresh_documents,
                reply,
            } => {
                let mut minimum_main_frame_epochs = fresh_documents
                    .into_iter()
                    .map(|document| {
                        let next = self
                            .pages
                            .get(&document)
                            .and_then(|page| page.provenance.content)
                            .map_or(1, |content| content.main_frame_epoch.saturating_add(1));
                        (document, next)
                    })
                    .collect::<Vec<_>>();
                minimum_main_frame_epochs.sort_unstable_by_key(|(document, _)| *document);
                minimum_main_frame_epochs.dedup_by_key(|(document, _)| *document);
                // Page.captureScreenshot reads the active display root.  An
                // offscreen target retains the pixels from its last completed
                // swap, so a copy request issued after that swap can complete
                // immediately only when no Page freshness fence was requested;
                // waiting for unrelated future damage can leave an otherwise
                // idle page blocked until the protocol timeout.
                //
                // DevTools input, evaluation and CopyOutput share BrowserHost's
                // ordered ingress, but Page production is asynchronous. A
                // same-size previous frame is therefore not proof that the
                // preceding mutation has reached Viz. BrowserHost posts a
                // normal main-frame transaction to each document below and
                // this request waits for its provenance instead of redrawing
                // or reading Page state from the display sequence.
                // If a resize/scene transition is still pending, retain the
                // request for the first presentation of that viewport below.
                if minimum_main_frame_epochs.is_empty() {
                    if let Some((viewport, provenance)) = self.last_presented_frame {
                        if viewport == expected_viewport {
                            let result = self
                                .target
                                .copy_output_rgba(viewport, target)
                                .map(|mut output| {
                                    output.provenance = Some(provenance);
                                    output
                                })
                                .map_err(|error| error.to_string());
                            let _ = reply.send(result);
                            return Ok(Vec::new());
                        }
                    }
                }
                self.pending_copy_outputs.push_back(PendingCopyOutput {
                    expected_viewport,
                    target,
                    minimum_main_frame_epochs,
                    reply,
                });
                // CopyOutput participates in the ordinary DisplayScheduler
                // damage path. If the matching Scene has already arrived,
                // mark it dirty and request a normal BeginFrame. If not, the
                // later Scene message supplies that damage; never draw the old
                // viewport merely to satisfy a new-size capture.
                if self
                    .scene
                    .as_ref()
                    .is_some_and(|scene| scene.viewport == expected_viewport)
                {
                    self.pending = Some(());
                    Ok(vec![Effect::RequestBeginFrame])
                } else {
                    Ok(Vec::new())
                }
            }
        }?;
        Ok(effects)
    }

    fn Selected(&self, requested: Document) -> Option<Document> {
        if self
            .pages
            .get(&requested)
            .is_some_and(|page| page.provenance.content.is_some())
        {
            return Some(requested);
        }
        // DelegatedFrameHost's previous Surface remains visible during a new
        // document's first commit. The fallback is local to that embedding view.
        self.pages.iter().rev().find_map(|(&document, page)| {
            (document.0 == requested.0 && page.provenance.content.is_some()).then_some(document)
        })
    }

    fn Reclaim(
        reclaim: &Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
        resources: Vec<PageReturnedResource>,
    ) {
        if !resources.is_empty() {
            reclaim(resources);
        }
    }

    fn DestroySurface(
        &mut self,
        surface: viz::adapter::SurfaceId,
        reclaim: &Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
    ) {
        if let Some(resources) = self.viz.DestroySurface(surface) {
            Self::Reclaim(reclaim, resources);
        }
    }

    fn DestroyPageSurface(&mut self, page: PageSurface) {
        // Remove the parent graph edge before destroying its delegated
        // children. Surface destruction returns every still-owned resource to
        // the Page compositor through the same FrameSink reclaim route.
        self.DestroySurface(page.surface, &page.reclaim);
        for child in page.child_surfaces {
            self.DestroySurface(child, &page.child_reclaim);
        }
    }

    fn ReclaimUnsubmitted(submitted: &SubmittedFrame) {
        submitted.ReclaimAll();
    }

    fn ParentReferencesChild(
        frame: &compositor::CompositorFrame,
        child: viz::adapter::SurfaceId,
    ) -> bool {
        let runtime_id = viz::adapter::VizEngine::RuntimeSurfaceId(child);
        frame
            .metadata
            .referenced_surfaces
            .iter()
            .any(|range| range.start == Some(runtime_id) || range.end == runtime_id)
            || frame.render_pass_list.iter().any(|pass| {
                pass.quad_list.iter().any(|quad| {
                    matches!(
                        &quad.material,
                        compositor::runtime::frame::QuadMaterial::Surface {
                            surface_range,
                            ..
                        } if surface_range.start == Some(runtime_id)
                            || surface_range.end == runtime_id
                    )
                })
            })
    }

    fn PrunePages(&mut self) {
        let Some(scene) = &self.scene else {
            return;
        };
        let retained: Vec<_> = scene
            .placements
            .iter()
            .filter_map(|p| self.Selected(p.document))
            .collect();
        let removed: Vec<_> = self
            .pages
            .iter()
            .filter_map(|(&document, _page)| {
                (!scene.live_documents.contains(&document) && !retained.contains(&document))
                    .then_some(document)
            })
            .collect();
        for document in removed {
            if let Some(page) = self.pages.remove(&document) {
                self.DestroyPageSurface(page);
            }
        }
    }

    fn Prepare(&mut self, mut submitted: SubmittedFrame) -> io::Result<Vec<Effect>> {
        let document = submitted.provenance.document;
        let resize = submitted
            .provenance
            .content
            .and_then(|content| content.resize);
        let _resize_scope = resize.map(|resize| {
            browser_tracing::scope(browser_tracing::Context {
                task_id: resize.id,
                ..Default::default()
            })
        });
        browser_tracing::instant(
            "frame",
            "DisplayPageFrameReceived",
            &[
                ("tab_id", document.0 as f64),
                ("document_generation", document.1 as f64),
                (
                    "content_sequence",
                    submitted.provenance.content.map_or(0, |c| c.sequence) as f64,
                ),
                (
                    "embedder_revision",
                    submitted
                        .provenance
                        .content
                        .map_or(0, |content| content.embedder_revision) as f64,
                ),
                ("child_frames", submitted.child_frames.len() as f64),
            ],
        );
        let local_surface_id = submitted.frame.local_surface_id;
        let Some(parameters) = submitted
            .frame
            .frame
            .render_pass_list
            .last()
            .map(|pass| (pass.output_rect.width, pass.output_rect.height))
        else {
            Self::ReclaimUnsubmitted(&submitted);
            return Err(io::Error::other("compositor frame has no root render pass"));
        };
        if let Some(current) = self.pages.get(&document) {
            let same_embed = current.local_surface_id.embed_token == local_surface_id.embed_token;
            if same_embed && !local_surface_id.IsSameOrNewerThan(&current.local_surface_id) {
                Self::ReclaimUnsubmitted(&submitted);
                return Ok(Vec::new());
            }
            if current.local_surface_id == local_surface_id
                && (
                    current.viewport.width as i32,
                    current.viewport.height as i32,
                ) != parameters
            {
                Self::ReclaimUnsubmitted(&submitted);
                return Err(io::Error::other(
                    "CompositorFrame dimensions changed without a new LocalSurfaceId",
                ));
            }
        }
        let generation_changed = self
            .pages
            .get(&document)
            .is_some_and(|current| current.local_surface_id != local_surface_id);
        let carried_resize = self
            .pages
            .get(&document)
            .and_then(|current| current.provenance.content)
            .and_then(|content| content.resize);
        let (surface, mut child_surfaces) = if generation_changed {
            let mut previous = self.pages.remove(&document).expect("Page surface exists");
            let children = std::mem::take(&mut previous.child_surfaces);
            self.DestroySurface(previous.surface, &previous.reclaim);
            let surface = viz::adapter::SurfaceId(self.next_surface);
            self.next_surface = self.next_surface.wrapping_add(1).max(1);
            (surface, children)
        } else if let Some(current) = self.pages.get(&document) {
            (current.surface, current.child_surfaces.clone())
        } else {
            let surface = viz::adapter::SurfaceId(self.next_surface);
            self.next_surface = self.next_surface.wrapping_add(1).max(1);
            (surface, BTreeSet::new())
        };
        if let (Some(resize), Some(content)) =
            (carried_resize, submitted.provenance.content.as_mut())
        {
            if let Some(current) = &mut content.resize {
                current.merge(resize);
            } else {
                content.resize = Some(resize);
            }
        }
        let reclaim = submitted.reclaim.clone();
        let child_reclaim = submitted.child_reclaim.clone();
        let incoming_children = std::mem::take(&mut submitted.child_frames);
        for child in incoming_children {
            if !Self::ParentReferencesChild(&submitted.frame.frame, child.surface_id) {
                Self::Reclaim(
                    &child_reclaim,
                    child
                        .frame
                        .resource_list
                        .iter()
                        .map(|resource| resource.ToReturnedResource())
                        .collect(),
                );
                continue;
            }
            let child_surface = child.surface_id;
            let submission = self
                .viz
                .SubmitFrame(child_surface, child.frame)
                .map_err(|error| io::Error::other(error.to_string()))?;
            let returned_resources = submission.returned_resources.len();
            Self::Reclaim(&child_reclaim, submission.returned_resources);
            child_surfaces.insert(child_surface);
            browser_tracing::instant(
                "frame",
                "DisplayChildFrameSubmitted",
                &[
                    ("surface_id", child_surface.0 as f64),
                    ("returned_resources", returned_resources as f64),
                ],
            );
        }
        let retained_children = child_surfaces
            .iter()
            .copied()
            .filter(|child| Self::ParentReferencesChild(&submitted.frame.frame, *child))
            .collect::<BTreeSet<_>>();
        let submission = self
            .viz
            .SubmitFrame(surface, submitted.frame.frame)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Self::Reclaim(&reclaim, submission.returned_resources);
        for child in child_surfaces.difference(&retained_children).copied() {
            self.DestroySurface(child, &child_reclaim);
        }
        self.pages.insert(
            document,
            PageSurface {
                surface,
                child_surfaces: retained_children,
                local_surface_id,
                viewport: submitted.viewport,
                provenance: submitted.provenance,
                is_handling_interaction: submitted.is_handling_interaction,
                reclaim,
                child_reclaim,
                presentation_feedback: submitted.presentation_feedback,
            },
        );
        // Source: RenderWidgetHostImpl::OnLocalSurfaceIdChanged. Any accepted
        // renderer frame clears the one-in-flight VisualProperties gate; the
        // ACK is deliberately not matched to a resize id or exact size.
        (submitted.visual_properties_ack)();
        let affects_current_scene = self.scene.as_ref().is_some_and(|scene| {
            scene
                .placements
                .iter()
                .any(|placement| self.Selected(placement.document) == Some(document))
        });
        if affects_current_scene {
            // Native presentation is handed to AppKit asynchronously. A
            // newer Surface can arrive after an older root was queued but
            // before that IOSurface entered its Core Animation transaction.
            // Invalidate the queued root here so it cannot land after the
            // content update that replaced it.
            self.target.invalidate_pending_presentations();

            // Only damage reachable from the current root can replace a
            // prepared aggregate or request another display draw. Background
            // tabs still submit into Viz so their latest Surface is ready for
            // activation, but their frames are not part of this root scene.
            self.prepared = None;
            self.pending = Some(());
        }
        self.PrunePages();
        Ok(vec![])
    }

    fn DrawPending(&mut self, frame: crate::begin_frame_source::BeginFrame) -> io::Result<()> {
        if self.pending.is_none() {
            return Ok(());
        }
        let Some(scene) = &self.scene else {
            return Ok(());
        };
        let viewport = scene.viewport;
        let mut placements = vec![];
        let mut presentation_routes = BTreeMap::new();
        let mut is_handling_interaction = false;
        let mut stale_surfaces = 0usize;
        let mut missing_surfaces = 0usize;
        let mut provenance = FrameProvenance {
            document: scene
                .placements
                .iter()
                .find(|p| p.document.0 != 0 && p.document.0 != u64::MAX)
                .map_or((0, 0), |p| p.document),
            content: None,
        };
        for placement in &scene.placements {
            let Some(document) = self.Selected(placement.document) else {
                // A newly selected WebContents can have no CompositorFrame yet.
                // Chromium still commits the browser root and exposes the
                // view's background; it does not keep pixels from the previous
                // tab in that child view. Omit only this unavailable child. A
                // root-scene change below makes the host target unretained, so
                // the uncovered destination is cleared before ready roots are
                // published. Same-tab navigation continues to use Selected's
                // previous-generation Surface fallback.
                missing_surfaces += 1;
                continue;
            };
            let page = &self.pages[&document];
            presentation_routes.insert(page.surface, page.presentation_feedback.clone());
            if placement.document.0 == 0
                && page
                    .provenance
                    .content
                    .map_or(0, |content| content.embedder_revision)
                    != scene.toolbar_revision
            {
                browser_tracing::instant(
                    "frame",
                    "DisplayToolbarRevisionPending",
                    &[
                        (
                            "available_revision",
                            page.provenance
                                .content
                                .map_or(0, |content| content.embedder_revision)
                                as f64,
                        ),
                        ("required_revision", scene.toolbar_revision as f64),
                    ],
                );
                // Browser UI and the selected child Surface form one root
                // commit. Exact matching rejects both directions of the race:
                // an old toolbar under a new tab and a new toolbar that reaches
                // Display before its matching Scene.
                return Ok(());
            }
            is_handling_interaction |= page.is_handling_interaction;
            let local = page.viewport;
            // Desktop Chromium gives top-level resizes a zero-frame surface
            // deadline. Viz can therefore aggregate the previous child
            // surface immediately, without stretching it, while the renderer
            // produces the new LocalSurfaceId. The uncovered part of the new
            // bounds is the fallback gutter. Requiring an exact size here
            // instead caused the fast toolbar and slower document producers
            // to continually overtake one another during a live resize, so
            // almost no window frames could be submitted until the drag ended.
            if local.width != placement.destination.width
                || local.height != placement.destination.height
            {
                stale_surfaces += 1;
            }
            placements.push(viz::adapter::SurfacePlacement {
                surface_id: page.surface,
                destination: viz::adapter::DeviceRect::new(
                    placement.destination.x,
                    placement.destination.y,
                    local.width as i32,
                    local.height as i32,
                ),
            });
            if placement.document.0 != 0 && placement.document.0 != u64::MAX {
                provenance = page.provenance;
            }
        }
        if placements.is_empty() {
            return Ok(());
        }
        // Consume the dirty bit only after a complete scene candidate exists.
        // A resize can deliver toolbar/content LocalSurfaceId generations on
        // different Page sequences; the last complete aggregate remains live
        // while either producer catches up.
        self.pending.take();
        self.sequence = self.sequence.wrapping_add(1);
        let sequence = self.sequence;
        let resize = provenance.content.and_then(|content| content.resize);
        let _resize_scope = resize.map(|resize| {
            browser_tracing::scope(browser_tracing::Context {
                task_id: resize.id,
                ..Default::default()
            })
        });
        let mut prepare_trace = browser_tracing::span("present", "Display.PrepareFrame");
        prepare_trace.set("width", viewport.width as f64);
        prepare_trace.set("height", viewport.height as f64);
        prepare_trace.set("device_scale", viewport.scale);
        prepare_trace.set("stale_surfaces", stale_surfaces as f64);
        prepare_trace.set("missing_surfaces", missing_surfaces as f64);
        let output_rect =
            viz::adapter::DeviceRect::new(0, 0, viewport.width as i32, viewport.height as i32);
        let display_root = display_root_frame(&scene.browser_root);
        let mut aggregate = self
            .viz
            .AggregateWithDisplayRoot(output_rect, &placements, &display_root)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let presentation = self
            .viz
            .PresentationFrames(&aggregate)
            .into_iter()
            .filter_map(|frame| {
                presentation_routes
                    .get(&frame.surface_id)
                    .cloned()
                    .map(|notify| PresentationParticipant { frame, notify })
            })
            .collect();
        let damage_pixels = aggregate.DamagePixels();
        // Compare the resolved Viz roots that will actually be drawn. A new
        // document can initially select the previous generation's Surface;
        // when its first frame arrives the Scene document is unchanged but
        // the selected SurfaceId changes. That root replacement must clear
        // the retained host target before a transparent Page root is drawn.
        let full_redraw = stale_surfaces != 0 || self.last_rendered_placements != placements;
        if std::env::var_os("RECHROM_DIAG_ROOTS").is_some() {
            let roots = aggregate
                .Frame()
                .RenderPasses()
                .iter()
                .filter_map(|pass| {
                    aggregate.RootDestination(pass.RenderPassId()).map(|destination| {
                        format!(
                            "pass={} destination={destination:?} output={:?} damage={:?} transparent={}",
                            pass.RenderPassId().0,
                            pass.OutputRect(),
                            pass.DamageRect(),
                            pass.has_transparent_background,
                        )
                    })
                })
                .collect::<Vec<_>>()
                .join(" | ");
            let passes = aggregate
                .Frame()
                .RenderPasses()
                .iter()
                .map(|pass| {
                    let quads = pass
                        .Quads()
                        .iter()
                        .map(|quad| {
                            let material = match &quad.material {
                                compositor::runtime::frame::QuadMaterial::Picture { .. } => {
                                    "Picture".to_string()
                                }
                                compositor::runtime::frame::QuadMaterial::SharedElement { .. } => {
                                    "SharedElement".to_string()
                                }
                                compositor::runtime::frame::QuadMaterial::SolidColor { .. } => {
                                    "SolidColor".to_string()
                                }
                                compositor::runtime::frame::QuadMaterial::DebugBorder { .. } => {
                                    "DebugBorder".to_string()
                                }
                                compositor::runtime::frame::QuadMaterial::Tile { .. } => {
                                    "Tile".to_string()
                                }
                                compositor::runtime::frame::QuadMaterial::Texture {
                                    is_video_frame,
                                    ..
                                } => format!("Texture(video={is_video_frame})"),
                                compositor::runtime::frame::QuadMaterial::Surface { .. } => {
                                    "Surface".to_string()
                                }
                                compositor::runtime::frame::QuadMaterial::CompositorRenderPass {
                                    render_pass_id,
                                    ..
                                } => format!("RenderPass({})", render_pass_id.0),
                            };
                            format!(
                                "rect={:?} visible={:?} material={material}",
                                quad.geometry.rect, quad.geometry.visible_rect,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "pass={} output={:?} damage={:?} quads=[{quads}]",
                        pass.RenderPassId().0,
                        pass.OutputRect(),
                        pass.DamageRect(),
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
            eprintln!(
                "display-roots sequence={sequence} placements={placements:?} previous={:?} full_redraw={full_redraw} stale={stale_surfaces} roots=[{roots}] passes=[{passes}]",
                self.last_rendered_placements,
            );
        }
        let output_damage = if full_redraw {
            vec![compositor::adapter::geometry::Rect::new(
                0,
                0,
                viewport.width as i32,
                viewport.height as i32,
            )]
        } else {
            aggregate.OutputDamageRects().to_vec()
        };
        let output = self
            .target
            .prepare_render_target(viewport, sequence, &output_damage)?;
        let row_stride = output.row_stride();
        let format = output.format();
        // The native target retains pixels outside Viz damage. A different
        // root scene cannot inherit those pixels: the incoming Page may be
        // smaller while its LocalSurfaceId catches up, and the uncovered
        // region must show the output background rather than the previous
        // tab. Chromium expresses this through full root damage when the
        // embedded surface changes. Preserve partial damage for ordinary
        // frames and scrolling.
        let retained = output.retained() && !full_redraw;
        let (update, target) = self.renderer.Render(
            &mut aggregate,
            renderer_skia::RenderTarget {
                width: viewport.width,
                height: viewport.height,
                pixels: renderer_skia::PixelStorage::host(output),
                format,
                row_stride,
                // Native buffers rotate behind WindowServer ownership. Until
                // their accumulated-damage owner is translated, only the
                // single retained offscreen allocation may use partial damage.
                retained,
            },
        )?;
        let mut output = *target
            .pixels
            .into_host::<PreparedDisplayTarget>()
            .map_err(|_| io::Error::other("Renderer returned a foreign host target"))?;
        output.finish_render(viewport, sequence);
        self.viz.DidDraw(aggregate);
        self.last_rendered_placements = placements;
        prepare_trace.set("direct_root_target", update.direct_root_target as u8 as f64);
        prepare_trace.set("damage_pixels", damage_pixels as f64);
        self.prepared = Some(PreparedDisplayFrame {
            presentation,
            output,
            provenance,
            sequence,
            viewport,
            drag_regions: scene.metadata.clone(),
            is_handling_interaction,
        });
        if resize.is_some() {
            if let Some(content) = self
                .pages
                .get_mut(&provenance.document)
                .and_then(|page| page.provenance.content.as_mut())
            {
                content.resize = None;
            }
        }
        browser_tracing::instant(
            "frame",
            "CompositorCandidateReady",
            &[
                ("present_sequence", sequence as f64),
                ("tab_id", provenance.document.0 as f64),
                ("document_generation", provenance.document.1 as f64),
                ("has_content", provenance.content.is_some() as u8 as f64),
                (
                    "content_sequence",
                    provenance.content.map_or(0, |c| c.sequence) as f64,
                ),
                (
                    "content_recording_revision",
                    provenance.content.map_or(0, |c| c.recording_revision) as f64,
                ),
                (
                    "compositor_scroll_sequence",
                    provenance
                        .content
                        .map_or(0, |c| c.compositor_scroll_sequence) as f64,
                ),
                (
                    "prepare_phase_ms",
                    Instant::now()
                        .saturating_duration_since(frame.frame_time)
                        .as_secs_f64()
                        * 1000.,
                ),
                ("stale_surfaces", stale_surfaces as f64),
            ],
        );
        Ok(())
    }

    fn Submit(
        &mut self,
        frame: crate::begin_frame_source::BeginFrame,
        post_swap_started: &Arc<dyn Fn() + Send + Sync>,
        post_swap_ack: &Arc<dyn Fn(u64) + Send + Sync>,
    ) -> io::Result<Vec<Effect>> {
        let _frame = browser_tracing::scope(browser_tracing::Context {
            target_id: 1,
            source_id: frame.source_id,
            frame_id: frame.sequence_number,
            ..Default::default()
        });
        let started = Instant::now();
        let mut frame_trace = browser_tracing::span("frame", "FrameCycle");
        frame_trace.set("interval_ms", frame.interval.as_secs_f64() * 1000.0);
        frame_trace.set(
            "source_deadline_ms",
            frame
                .deadline
                .saturating_duration_since(frame.frame_time)
                .as_secs_f64()
                * 1000.0,
        );
        frame_trace.set(
            "deadline_ms",
            viz::FrameTiming {
                frame_id: viz::BeginFrameId {
                    source_id: frame.source_id,
                    sequence_number: frame.sequence_number,
                },
                frame_time: frame.frame_time,
                interval: frame.interval,
                source_deadline: frame.deadline,
            }
            .Deadlines()
            .draw_and_swap
            .saturating_duration_since(frame.frame_time)
            .as_secs_f64()
                * 1000.0,
        );
        frame_trace.set(
            "start_age_ms",
            started
                .saturating_duration_since(frame.frame_time)
                .as_secs_f64()
                * 1000.0,
        );
        frame_trace.set(
            "draw_start_age_ms",
            started
                .saturating_duration_since(frame.frame_time)
                .as_secs_f64()
                * 1000.0,
        );
        if self.pending_swap.is_some() {
            frame_trace.set("submitted", 0.0);
            frame_trace.set("swap_throttled", 1.0);
            return Ok(vec![Effect::RequestBeginFrame]);
        }
        self.DrawPending(frame)?;
        if self.pending.is_some() {
            // DrawPending deliberately keeps the dirty state when the exact
            // browser-root candidate is incomplete (for example, the toolbar
            // has not reached the Scene's revision yet).  In that state an
            // older prepared target is not a legal fallback: it was rendered
            // from a different root scene.  Keep the last frame that actually
            // reached the native presenter and try again on the next pulse.
            frame_trace.set("submitted", 0.0);
            frame_trace.set("root_candidate_incomplete", 1.0);
            return Ok(vec![Effect::RequestBeginFrame]);
        }
        let Some(prepared) = self.prepared.take() else {
            frame_trace.set("submitted", 0.0);
            return Ok(Vec::new());
        };
        let PreparedDisplayFrame {
            presentation: participants,
            mut output,
            provenance,
            sequence,
            viewport,
            drag_regions,
            is_handling_interaction,
        } = prepared;
        let resize = provenance.content.and_then(|content| content.resize);
        let _resize_scope = resize.map(|resize| {
            browser_tracing::scope(browser_tracing::Context {
                task_id: resize.id,
                ..Default::default()
            })
        });
        frame_trace.set(
            "is_handling_interaction",
            is_handling_interaction as u8 as f64,
        );
        // Source: SoftwareOutputDeviceMac::EndPaint ->
        // DisplayCALayerTree::GotIOSurfaceFrame. This output submits the ready
        // IOSurface immediately; the GPU overlay presenter's separate
        // VSyncAlignedPresentationForScrolling queue does not apply here.
        // Headless capture has no visible native presentation boundary.
        // Submission activates this complete compositor frame; pending or
        // failed candidates never replace the last published frame.
        if self.target.is_window() {
            if let Some(diagnostic) = &self.diagnostic_output {
                let mut readback = output.take_readback_rgba(viewport)?;
                readback.provenance = Some(provenance);
                diagnostic.publish(readback);
            }
        }
        let notify = self.notify.clone();
        let presentation = self.target.is_window();
        let post_swap_ack = post_swap_ack.clone();
        // Chromium Display::DrawAndSwap registers DidSwapBuffers before
        // invoking SwapBuffers: software/native completion can be synchronous.
        // The callback only posts its token; the Display owner consumes it.
        self.pending_swap = Some(sequence);
        post_swap_started();
        let presented = self.target.present_prepared(
            output,
            sequence,
            Box::new(move |_, finished| {
                if let Some(resize) = resize {
                    browser_tracing::interval(
                        "resize",
                        "ResizeToPresentationComplete",
                        resize.queued_at,
                        finished,
                        &[
                            ("samples", resize.samples as f64),
                            ("width", viewport.width as f64),
                            ("height", viewport.height as f64),
                            ("device_scale", viewport.scale),
                            ("present_sequence", sequence as f64),
                            ("native", presentation as u8 as f64),
                        ],
                    );
                }
                if presentation {
                    (notify)(UserEvent::FrameSwapCompleted(viewport));
                }
                (post_swap_ack)(sequence);
            }),
            Box::new(move|feedback|{
                use browser::page::{PagePresentationOutcome as Outcome,PagePresentationUnavailableReason as Reason};
                let outcome=match feedback{
                    softbuffer::PresentationFeedback::Presented{timestamp,interval,flags}=>Outcome::Presented{timestamp,interval,flags},
                    softbuffer::PresentationFeedback::Unavailable(reason)=>Outcome::Unavailable(match reason{
                        softbuffer::PresentationUnavailableReason::BackendHasNoTimestamp=>Reason::BackendHasNoTimestamp,
                        softbuffer::PresentationUnavailableReason::DiscardedBeforePresentation=>Reason::DiscardedBeforePresentation,
                        softbuffer::PresentationUnavailableReason::OffscreenTarget=>Reason::OffscreenTarget,
                    }),
                };
                for participant in participants{
                    (participant.notify)(browser::page::PageFramePresentationFeedback{surface_id:participant.frame.surface_id.0,viz_submission:participant.frame.submission,frame_token:participant.frame.frame_token,frame_id:participant.frame.frame_id,outcome});
                }
            }),
        );
        if let Err(error) = presented {
            self.pending_swap = None;
            return Err(error);
        }
        self.last_presented_frame = Some((viewport, provenance));
        self.FulfillCopyOutputs(viewport, provenance);
        if !presentation && !self.offscreen_frame_ready_notified {
            self.offscreen_frame_ready_notified = true;
            (self.notify)(UserEvent::FrameReady);
        }
        frame_trace.set("submitted", 1.0);
        frame_trace.set(
            "deadline_missed",
            (Instant::now() > frame.deadline) as u8 as f64,
        );
        (self.notify)(UserEvent::ChromeDragRegions {
            viewport,
            frame_sequence: sequence,
            regions: drag_regions,
        });
        browser_tracing::instant(
            "frame",
            "CompositorFrameSubmitted",
            &[
                ("present_sequence", sequence as f64),
                ("source_id", frame.source_id as f64),
                ("frame_id", frame.sequence_number as f64),
                ("tab_id", provenance.document.0 as f64),
                ("document_generation", provenance.document.1 as f64),
                ("has_content", provenance.content.is_some() as u8 as f64),
                (
                    "content_sequence",
                    provenance.content.map_or(0, |c| c.sequence) as f64,
                ),
                (
                    "content_recording_revision",
                    provenance.content.map_or(0, |c| c.recording_revision) as f64,
                ),
                (
                    "compositor_scroll_sequence",
                    provenance
                        .content
                        .map_or(0, |c| c.compositor_scroll_sequence) as f64,
                ),
            ],
        );
        if !presentation && self.diagnostic_output.is_some() {
            if crate::engine::memory_diagnostics_enabled() {
                eprintln!("pipeline-offscreen-published time_ns={} present={} tab={} generation={} sequence={} revision={}", crate::engine::diagnostic_time_ns(), sequence,
                    provenance.document.0, provenance.document.1,
                    provenance.content.map_or(0, |c| c.sequence), provenance.content.map_or(0, |c| c.recording_revision));
            }
            // This exact prepared candidate is now the completed offscreen
            // target. RGBA conversion is deferred until CopyOutput is
            // requested, like Chromium's DirectRenderer readback path. This
            // is neither an on-screen presentation timestamp nor FCP/LCP.
            browser_tracing::instant(
                "frame",
                "OffscreenFramePublished",
                &[
                    ("present_sequence", sequence as f64),
                    ("tab_id", provenance.document.0 as f64),
                    ("document_generation", provenance.document.1 as f64),
                    ("has_content", provenance.content.is_some() as u8 as f64),
                    (
                        "content_sequence",
                        provenance.content.map_or(0, |c| c.sequence) as f64,
                    ),
                    (
                        "content_recording_revision",
                        provenance.content.map_or(0, |c| c.recording_revision) as f64,
                    ),
                ],
            );
        }
        if presentation {
            browser_tracing::instant(
                "frame",
                "FrameSubmittedToCompositor",
                &[("present_sequence", sequence as f64), ("compositor", 1.0)],
            );
        }
        Ok(Vec::new())
    }

    fn FulfillCopyOutputs(&mut self, viewport: Viewport, provenance: FrameProvenance) {
        let mut waiting = VecDeque::new();
        for request in self.pending_copy_outputs.drain(..) {
            let fresh = request
                .minimum_main_frame_epochs
                .iter()
                .all(|(document, minimum)| {
                    self.pages
                        .get(document)
                        .and_then(|page| page.provenance.content)
                        .is_some_and(|content| content.main_frame_epoch >= *minimum)
                });
            if request.expected_viewport != viewport || !fresh {
                waiting.push_back(request);
                continue;
            }
            let result = self
                .target
                .copy_output_rgba(viewport, request.target)
                .map(|mut output| {
                    output.provenance = Some(provenance);
                    output
                })
                .map_err(|error| error.to_string());
            let _ = request.reply.send(result);
        }
        self.pending_copy_outputs = waiting;
    }

    fn UpdateCopyOutputsForScene(&mut self, scene: &Scene) {
        let mut waiting = VecDeque::new();
        for mut request in self.pending_copy_outputs.drain(..) {
            // A CopyOutputRequest belongs to the active display root, not to a
            // discarded resize candidate. If a newer BrowserView Scene wins
            // before the draw, follow that Scene and fulfill from its first
            // completed presentation. This matches Chromium carrying the copy
            // request onto the active render pass and avoids returning an old
            // size or failing rapid resize sampling.
            request.expected_viewport = scene.viewport;
            request.minimum_main_frame_epochs.retain(|(document, _)| {
                scene
                    .placements
                    .iter()
                    .any(|placement| placement.document == *document)
            });
            waiting.push_back(request);
        }
        self.pending_copy_outputs = waiting;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn viewport(width: u32) -> Viewport {
        Viewport {
            width,
            height: crate::chrome::HEIGHT as u32 + 1,
            scale: 1.0,
        }
    }

    fn scene(viewport: Viewport) -> Scene {
        Scene {
            viewport,
            browser_root: crate::page_presentation::BrowserRootFrame {
                background_color: [255; 4],
                solid_color_quads: Vec::new(),
            },
            placements: Vec::new(),
            live_documents: Vec::new(),
            metadata: Vec::new(),
            toolbar_revision: 0,
        }
    }

    fn output() -> Output {
        Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: Arc::new(|_| {}),
        }
    }

    fn provenance() -> FrameProvenance {
        FrameProvenance {
            document: (1, 1),
            content: None,
        }
    }

    fn post_swap_ack() -> Arc<dyn Fn(u64) + Send + Sync> {
        Arc::new(|_| {})
    }

    fn post_swap_started() -> Arc<dyn Fn() + Send + Sync> {
        Arc::new(|| {})
    }

    #[test]
    fn resize_then_copy_output_waits_for_matching_presentation() {
        let old = viewport(2);
        let new = viewport(3);
        let mut display = Display::New(
            DisplayTarget::Offscreen {
                pixels: vec![0xff11_1111; old.width as usize * old.height as usize],
            },
            output(),
            true,
        );
        display.scene = Some(scene(old));
        display.last_presented_frame = Some((old, provenance()));
        let (reply, result) = mpsc::channel();
        let post_swap_started = post_swap_started();
        let post_swap_ack = post_swap_ack();

        let effects = display
            .Handle(
                Message::CopyOutput {
                    expected_viewport: new,
                    target: CopyOutputTarget::Page,
                    fresh_documents: Vec::new(),
                    reply,
                },
                &post_swap_started,
                &post_swap_ack,
            )
            .unwrap();

        assert!(effects.is_empty());
        assert!(result.try_recv().is_err());
        assert_eq!(display.pending_copy_outputs.len(), 1);

        let effects = display
            .Handle(
                Message::Scene(scene(new)),
                &post_swap_started,
                &post_swap_ack,
            )
            .unwrap();
        assert!(matches!(effects.as_slice(), [Effect::RequestBeginFrame]));
        assert!(result.try_recv().is_err());

        display.target = DisplayTarget::Offscreen {
            pixels: vec![0xff60_5040; new.width as usize * new.height as usize],
        };
        display.FulfillCopyOutputs(new, provenance());
        let captured = result.recv().unwrap().unwrap();
        assert_eq!((captured.width, captured.height), (3, 1));
        assert_eq!(captured.pixels, [0x60, 0x50, 0x40, 255].repeat(3));
        assert!(display.pending_copy_outputs.is_empty());
    }

    #[test]
    fn superseded_copy_output_follows_latest_scene() {
        let expected = viewport(3);
        let replacement = viewport(4);
        let mut display = Display::New(
            DisplayTarget::Offscreen { pixels: Vec::new() },
            output(),
            true,
        );
        display.scene = Some(scene(expected));
        let (reply, result) = mpsc::channel();
        let post_swap_started = post_swap_started();
        let post_swap_ack = post_swap_ack();
        display
            .Handle(
                Message::CopyOutput {
                    expected_viewport: expected,
                    target: CopyOutputTarget::Toolbar,
                    fresh_documents: Vec::new(),
                    reply,
                },
                &post_swap_started,
                &post_swap_ack,
            )
            .unwrap();

        display
            .Handle(
                Message::Scene(scene(replacement)),
                &post_swap_started,
                &post_swap_ack,
            )
            .unwrap();

        assert!(result.try_recv().is_err());
        assert_eq!(display.pending_copy_outputs.len(), 1);
        display.target = DisplayTarget::Offscreen {
            pixels: vec![0xff30_2010; replacement.width as usize * replacement.height as usize],
        };
        display.FulfillCopyOutputs(replacement, provenance());
        let captured = result.recv().unwrap().unwrap();
        assert_eq!(captured.width, replacement.width);
        assert!(display.pending_copy_outputs.is_empty());
    }
}
