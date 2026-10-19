//! Final display compositor state.
//!
//! This module owns the native surface and display resources, but deliberately
//! knows nothing about threads, channels, or executors. `PresentationRuntime`
//! delivers messages and routes the returned effects.
use crate::page_presentation::{Document, FrameProvenance, PageFrame as SubmittedFrame, Scene};
use crate::{
    engine::{Output, UserEvent, Viewport},
    window_surface::{DisplayTarget, PreparedDisplayTarget},
};
use std::{collections::BTreeMap, io, sync::Arc, time::Instant};

pub(crate) enum Message {
    Scene(Scene),
    Prepare {
        submitted: SubmittedFrame,
    },
    Submit {
        frame: crate::begin_frame_source::NativeBeginFrame,
        scroll_active: bool,
    },
    SwapAck(u64),
    Stop,
}

pub(crate) enum Effect {
    RequestBeginFrame,
}

struct PageSurface {
    frame: SubmittedFrame,
}
struct PreparedDisplayFrame {
    output: PreparedDisplayTarget,
    provenance: FrameProvenance,
    sequence: u64,
    viewport: Viewport,
    drag_regions: Vec<crate::chrome::DragRegion>,
}

pub(crate) struct Display {
    target: DisplayTarget,
    pages: BTreeMap<Document, PageSurface>,
    scene: Option<Scene>,
    pending: Option<()>,
    prepared: Option<PreparedDisplayFrame>,
    sequence: u64,
    notify: Arc<dyn Fn(UserEvent) + Send + Sync>,
    diagnostic_output: Option<Output>,
    pending_swap: Option<u64>,
}

impl Display {
    pub(crate) fn New(target: DisplayTarget, output: Output, capture_frames: bool) -> Self {
        Self {
            target,
            pages: BTreeMap::new(),
            scene: None,
            pending: None,
            prepared: None,
            sequence: 0,
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
            Message::Submit {
                frame,
                scroll_active,
            } => self.Submit(frame, scroll_active, post_swap_ack),
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
            .is_some_and(|page| page.frame.provenance.content.is_some())
        {
            return Some(requested);
        }
        // DelegatedFrameHost's previous Surface remains visible during a new
        // document's first commit. The fallback is local to that embedding view.
        self.pages.iter().rev().find_map(|(&document, page)| {
            (document.0 == requested.0 && page.frame.provenance.content.is_some())
                .then_some(document)
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
            .filter_map(|(&document, page)| {
                (!scene.live_documents.contains(&document) && !retained.contains(&document))
                    .then_some(document)
            })
            .collect();
        for document in removed {
            self.pages.remove(&document);
        }
    }

    fn Prepare(&mut self, submitted: SubmittedFrame) -> io::Result<Vec<Effect>> {
        let document = submitted.provenance.document;
        self.pages
            .insert(document, PageSurface { frame: submitted });
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
            let local = page.frame.viewport;
            // A resize waits for a matching producer frame; Viz never stretches
            // an old Page into a different local viewport.
            if local.width != placement.destination.width
                || local.height != placement.destination.height
            {
                return Ok(());
            }
            placements.push((&page.frame, placement.destination));
            if placement.document.0 != 0 && placement.document.0 != u64::MAX {
                provenance = page.frame.provenance;
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
        let output = self
            .target
            .render_page_surfaces(&placements, viewport, sequence)?;
        self.prepared = Some(PreparedDisplayFrame {
            output,
            provenance,
            sequence,
            viewport,
            drag_regions: scene.metadata.clone(),
        });
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
                    "prepare_phase_ms",
                    Instant::now()
                        .saturating_duration_since(frame.frame_time)
                        .as_secs_f64()
                        * 1000.,
                ),
            ],
        );
        Ok(())
    }

    fn Submit(
        &mut self,
        frame: crate::begin_frame_source::NativeBeginFrame,
        scroll_active: bool,
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
        frame_trace.set("scroll_active", scroll_active as u8 as f64);
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
        } = prepared;
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
            Box::new(move |_, _| {
                if native_presentation {
                    (notify)(UserEvent::FramePresented(viewport));
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
                "FramePresentedToNative",
                &[("present_sequence", sequence as f64), ("compositor", 1.0)],
            );
        }
        Ok(if scroll_active {
            vec![Effect::RequestBeginFrame]
        } else {
            Vec::new()
        })
    }
}
