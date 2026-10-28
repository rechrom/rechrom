//! Final display compositor state with one Viz/Renderer service shared by all Pages.
//!
//! This module owns the native surface and display resources, but deliberately
//! knows nothing about threads, channels, or executors. `PresentationRuntime`
//! delivers messages and routes the returned effects.
use crate::page_presentation::{Document, FrameProvenance, PageFrame as SubmittedFrame, Scene};
use crate::{
    engine::{Output, UserEvent, Viewport},
    window_surface::{DisplayTarget, PreparedDisplayTarget},
};
use browser::page::PageReturnedResource;
use renderer_skia::{PixelFormat, RendererSkiaBackend};
use std::{collections::BTreeMap, io, sync::Arc, time::Instant};

pub(crate) enum Message {
    Scene(Scene),
    Prepare {
        submitted: SubmittedFrame,
    },
    Submit {
        frame: crate::begin_frame_source::NativeBeginFrame,
    },
    SwapAck(u64),
    Stop,
}

pub(crate) enum Effect {
    RequestBeginFrame,
}

struct PageSurface {
    surface: viz::native::SurfaceId,
    local_surface_id: compositor::runtime::types::LocalSurfaceId,
    viewport: Viewport,
    provenance: FrameProvenance,
    is_handling_interaction: bool,
    reclaim: Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
}
struct PreparedDisplayFrame {
    output: PreparedDisplayTarget,
    provenance: FrameProvenance,
    sequence: u64,
    viewport: Viewport,
    drag_regions: Vec<crate::chrome::DragRegion>,
    is_handling_interaction: bool,
}

pub(crate) struct Display {
    target: DisplayTarget,
    viz: viz::native::VizEngine,
    renderer: renderer::Renderer<RendererSkiaBackend>,
    pages: BTreeMap<Document, PageSurface>,
    scene: Option<Scene>,
    pending: Option<()>,
    prepared: Option<PreparedDisplayFrame>,
    sequence: u64,
    next_surface: u64,
    rendered_pixels: Vec<u32>,
    rendered_viewport: Option<Viewport>,
    notify: Arc<dyn Fn(UserEvent) + Send + Sync>,
    diagnostic_output: Option<Output>,
    pending_swap: Option<u64>,
}

impl Display {
    pub(crate) fn New(target: DisplayTarget, output: Output, capture_frames: bool) -> Self {
        Self {
            target,
            viz: viz::native::VizEngine::New(),
            renderer: renderer::Renderer::new(RendererSkiaBackend::default()),
            pages: BTreeMap::new(),
            scene: None,
            pending: None,
            prepared: None,
            sequence: 0,
            next_surface: 1,
            rendered_pixels: Vec::new(),
            rendered_viewport: None,
            notify: output.notify.clone(),
            diagnostic_output: capture_frames.then_some(output),
            pending_swap: None,
        }
    }

    pub(crate) fn Handle(
        &mut self,
        message: Message,
        post_swap_ack: &Arc<dyn Fn(u64) + Send + Sync>,
    ) -> io::Result<Vec<Effect>> {
        let effects = match message {
            Message::Stop => return Ok(Vec::new()),
            Message::Scene(scene) => {
                self.scene = Some(scene);
                self.PrunePages();
                self.pending = Some(());
                Ok(vec![Effect::RequestBeginFrame])
            }
            Message::Prepare { submitted } => self.Prepare(submitted),
            Message::Submit { frame } => self.Submit(frame, post_swap_ack),
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

    fn DestroyPageSurface(&mut self, page: PageSurface) {
        if let Some(resources) = self.viz.DestroySurface(page.surface) {
            Self::Reclaim(&page.reclaim, resources);
        }
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
            ],
        );
        let local_surface_id = submitted.frame.local_surface_id;
        let parameters = submitted
            .frame
            .frame
            .render_pass_list
            .last()
            .map(|pass| (pass.output_rect.width, pass.output_rect.height))
            .ok_or_else(|| io::Error::other("compositor frame has no root render pass"))?;
        if let Some(current) = self.pages.get(&document) {
            let same_embed = current.local_surface_id.embed_token == local_surface_id.embed_token;
            if same_embed && !local_surface_id.IsSameOrNewerThan(&current.local_surface_id) {
                submitted.Reclaim(
                    submitted
                        .frame
                        .frame
                        .resource_list
                        .iter()
                        .map(|resource| resource.ToReturnedResource())
                        .collect(),
                );
                return Ok(Vec::new());
            }
            if current.local_surface_id == local_surface_id
                && (
                    current.viewport.width as i32,
                    current.viewport.height as i32,
                ) != parameters
            {
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
        let surface = if generation_changed {
            let previous = self.pages.remove(&document).expect("Page surface exists");
            self.DestroyPageSurface(previous);
            let surface = viz::native::SurfaceId(self.next_surface);
            self.next_surface = self.next_surface.wrapping_add(1).max(1);
            surface
        } else if let Some(current) = self.pages.get(&document) {
            current.surface
        } else {
            let surface = viz::native::SurfaceId(self.next_surface);
            self.next_surface = self.next_surface.wrapping_add(1).max(1);
            surface
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
        let submission = self
            .viz
            .SubmitFrame(surface, submitted.frame.frame)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Self::Reclaim(&reclaim, submission.returned_resources);
        self.pages.insert(
            document,
            PageSurface {
                surface,
                local_surface_id,
                viewport: submitted.viewport,
                provenance: submitted.provenance,
                is_handling_interaction: submitted.is_handling_interaction,
                reclaim,
            },
        );
        self.PrunePages();
        self.pending = Some(());
        Ok(vec![])
    }

    fn DrawPending(
        &mut self,
        frame: crate::begin_frame_source::NativeBeginFrame,
    ) -> io::Result<()> {
        if self.pending.is_none() {
            return Ok(());
        }
        let Some(scene) = &self.scene else {
            return Ok(());
        };
        let viewport = scene.viewport;
        let mut placements = vec![];
        let mut is_handling_interaction = false;
        let mut stale_surfaces = 0usize;
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
                // SurfaceAggregator keeps the previous complete aggregate
                // until every embedded surface required by the new scene has
                // a frame.  Presenting only the ready children would clear
                // the missing Page's pixels in the newly acquired buffer.
                return Ok(());
            };
            let page = &self.pages[&document];
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
            placements.push(viz::native::SurfacePlacement {
                surface_id: page.surface,
                destination: viz::native::DeviceRect::new(
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
        let output_rect =
            viz::native::DeviceRect::new(0, 0, viewport.width as i32, viewport.height as i32);
        let mut aggregate = self
            .viz
            .Aggregate(output_rect, &placements)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let damage_pixels = aggregate.DamagePixels();
        let retained = self.rendered_viewport == Some(viewport);
        let pixel_count = viewport.width as usize * viewport.height as usize;
        self.rendered_pixels.resize(pixel_count, 0);
        let target = std::mem::take(&mut self.rendered_pixels);
        let (update, target) = self.renderer.Render(
            &mut aggregate,
            renderer_skia::RenderTarget {
                width: viewport.width,
                height: viewport.height,
                pixels: target,
                format: PixelFormat::Bgra8888,
                row_stride: viewport.width as usize,
                retained,
            },
        )?;
        self.rendered_pixels = target.pixels;
        self.rendered_viewport = Some(viewport);
        self.viz.DidDraw(aggregate);
        prepare_trace.set("direct_root_target", update.direct_root_target as u8 as f64);
        prepare_trace.set("damage_pixels", damage_pixels as f64);
        let output = self.target.prepare_rendered_pixels(
            &self.rendered_pixels,
            viewport,
            sequence,
            damage_pixels,
        )?;
        self.prepared = Some(PreparedDisplayFrame {
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
        frame: crate::begin_frame_source::NativeBeginFrame,
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
        let Some(prepared) = self.prepared.take() else {
            frame_trace.set("submitted", 0.0);
            return Ok(Vec::new());
        };
        let PreparedDisplayFrame {
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
        if let Some(diagnostic) = &self.diagnostic_output {
            let mut readback = output.take_readback_rgba(viewport)?;
            readback.provenance = Some(provenance);
            diagnostic.publish(readback);
        }
        let notify = self.notify.clone();
        let native_presentation = self.target.is_native();
        let post_swap_ack = post_swap_ack.clone();
        // Chromium Display::DrawAndSwap registers DidSwapBuffers before
        // invoking SwapBuffers: software/native completion can be synchronous.
        // The callback only posts its token; the Display owner consumes it.
        self.pending_swap = Some(sequence);
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
                            ("native", native_presentation as u8 as f64),
                        ],
                    );
                }
                if native_presentation {
                    (notify)(UserEvent::FrameSwapCompleted(viewport));
                }
                (post_swap_ack)(sequence);
            }),
        );
        if let Err(error) = presented {
            self.pending_swap = None;
            return Err(error);
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
        if !native_presentation && self.diagnostic_output.is_some() {
            if crate::engine::memory_diagnostics_enabled() {
                eprintln!("pipeline-offscreen-published time_ns={} present={} tab={} generation={} sequence={} revision={}", crate::engine::diagnostic_time_ns(), sequence,
                    provenance.document.0, provenance.document.1,
                    provenance.content.map_or(0, |c| c.sequence), provenance.content.map_or(0, |c| c.recording_revision));
            }
            // Pixels from this exact prepared candidate are now in the
            // screenshot mailbox and its offscreen swap completed. This is
            // neither an on-screen presentation timestamp nor FCP/LCP.
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
        if native_presentation {
            browser_tracing::instant(
                "frame",
                "FrameSubmittedToNative",
                &[("present_sequence", sequence as f64), ("compositor", 1.0)],
            );
        }
        Ok(Vec::new())
    }
}
