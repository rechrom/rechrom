//! Final display compositor state.
//!
//! This module owns the native surface and display resources, but deliberately
//! knows nothing about threads, channels, or executors. `PresentationRuntime`
//! delivers messages and routes the returned effects.
use crate::page_presentation::{Document, PageFrame as SubmittedFrame, Scene};
use crate::{
    engine::{Output, UserEvent, Viewport},
    window_surface::{DisplayTarget, PreparedDisplayTarget},
};
use renderer::Renderer;
use std::{collections::BTreeMap, io, sync::Arc, time::Instant};
use viz::{SurfaceId, SwapId, VizEngine};

pub(crate) enum Message {
    Scene(Scene),
    Prepare {
        frame: crate::begin_frame_source::NativeBeginFrame,
        submitted: SubmittedFrame,
        resources: raster::RasterResourceSnapshot,
        scroll_active: bool,
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
    id: SurfaceId,
    resources: raster::RasterResourceSnapshot,
    provenance: Option<compositor::FrameProvenance>,
    viewport: Option<Viewport>,
}
struct PreparedDisplayFrame {
    output: PreparedDisplayTarget,
    provenance: compositor::FrameProvenance,
    sequence: u64,
    viewport: Viewport,
    drag_regions: Vec<crate::chrome::DragRegion>,
}

pub(crate) struct Display {
    target: DisplayTarget,
    pages: BTreeMap<Document, PageSurface>,
    next_surface_id: u64,
    scene: Option<Scene>,
    pending: Option<()>,
    prepared: Option<PreparedDisplayFrame>,
    sequence: u64,
    notify: Arc<dyn Fn(UserEvent) + Send + Sync>,
    diagnostic_output: Option<Output>,
    viz: VizEngine,
    renderer: Renderer,
}

impl Display {
    pub(crate) fn New(target: DisplayTarget, output: Output, capture_frames: bool) -> Self {
        Self {
            target,
            pages: BTreeMap::new(),
            next_surface_id: 0,
            scene: None,
            pending: None,
            prepared: None,
            sequence: 0,
            notify: output.notify.clone(),
            diagnostic_output: capture_frames.then_some(output),
            viz: VizEngine::New(),
            renderer: Renderer::default(),
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
            Message::Prepare {
                frame,
                submitted,
                resources,
                scroll_active,
            } => self.Prepare(frame, submitted, resources, scroll_active),
            Message::Submit {
                frame,
                scroll_active,
            } => self.Submit(frame, scroll_active, post_swap_ack),
            Message::SwapAck(sequence) => {
                let accepted = self.viz.DidReceiveSwapAck(SwapId(sequence));
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
            .is_some_and(|page| page.provenance.is_some())
        {
            return Some(requested);
        }
        // DelegatedFrameHost's previous Surface remains visible during a new
        // document's first commit. The fallback is local to that embedding view.
        self.pages.iter().rev().find_map(|(&document, page)| {
            (document.0 == requested.0 && page.provenance.is_some()).then_some(document)
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
                    .then_some((document, page.id))
            })
            .collect();
        for (document, surface) in removed {
            self.pages.remove(&document);
            self.viz.DestroySurface(surface);
            self.renderer.InvalidateSurface(surface);
        }
    }

    fn Prepare(
        &mut self,
        frame: crate::begin_frame_source::NativeBeginFrame,
        submitted: SubmittedFrame,
        resources: raster::RasterResourceSnapshot,
        _scroll_active: bool,
    ) -> io::Result<Vec<Effect>> {
        let _frame = browser_tracing::scope(browser_tracing::Context {
            target_id: 1,
            source_id: frame.source_id,
            frame_id: frame.sequence_number,
            ..Default::default()
        });
        // The exact frame and its immutable resource leases are accepted as
        // one message. No raster engine or tile scheduler crosses this boundary.
        let document = submitted.provenance.document;
        let page = self.pages.entry(document).or_insert_with(|| {
            self.next_surface_id += 1;
            PageSurface {
                id: SurfaceId(self.next_surface_id),
                resources: resources.clone(),
                provenance: None,
                viewport: None,
            }
        });
        page.resources = resources;
        if let Some(frame) = submitted.content {
            self.viz
                .SubmitFrame(page.id, frame)
                .map_err(io::Error::other)?;
            page.provenance = Some(submitted.provenance);
            page.viewport = Some(submitted.viewport);
        } else {
            page.provenance = None;
            self.viz.DestroySurface(page.id);
            self.renderer.InvalidateSurface(page.id);
        }
        self.PrunePages();
        self.pending = Some(());
        Ok(vec![])
    }

    fn DrawPending(
        &mut self,
        frame: crate::begin_frame_source::NativeBeginFrame,
    ) -> io::Result<()> {
        if self.pending.take().is_none() {
            return Ok(());
        }
        let Some(scene) = &self.scene else {
            return Ok(());
        };
        let viewport = scene.viewport;
        let mut placements = vec![];
        let mut provenance = compositor::FrameProvenance {
            document: scene
                .placements
                .iter()
                .find(|p| p.document.0 != 0 && p.document.0 != u64::MAX)
                .map_or((0, 0), |p| p.document),
            content: None,
        };
        for placement in &scene.placements {
            let Some(document) = self.Selected(placement.document) else {
                continue;
            };
            let page = &self.pages[&document];
            let Some(local) = page.viewport else {
                continue;
            };
            // A resize waits for a matching producer frame; Viz never stretches
            // an old Page into a different local viewport.
            if local.width != placement.destination.width
                || local.height != placement.destination.height
            {
                continue;
            }
            placements.push(viz::SurfacePlacement {
                surface_id: page.id,
                destination: placement.destination,
            });
            if placement.document.0 != 0 && placement.document.0 != u64::MAX {
                provenance = page.provenance.unwrap();
            }
        }
        if placements.is_empty() {
            return Ok(());
        }
        self.sequence = self.sequence.wrapping_add(1);
        let sequence = self.sequence;
        let aggregated = self
            .viz
            .Aggregate(
                compositor::DeviceRect::new(0, 0, viewport.width, viewport.height),
                &placements,
            )
            .map_err(io::Error::other)?;
        let resources: Vec<_> = self
            .pages
            .values()
            .map(|page| (page.id, &page.resources))
            .collect();
        let output = self.target.render_aggregated_frame(
            &aggregated,
            &mut self.renderer,
            &resources,
            viewport,
            sequence,
        )?;
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
        if !self.viz.CanDrawAndSwap() {
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
        self.viz
            .DidSubmitSwap(SwapId(sequence))
            .map_err(io::Error::other)?;
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
            self.viz.DidReceiveSwapAck(SwapId(sequence));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::Duration;

    fn prepared(sequence: u64, viewport: Viewport) -> PreparedDisplayFrame {
        PreparedDisplayFrame {
            output: PreparedDisplayTarget::Offscreen {
                pixels: vec![0x0011_2233; viewport.width as usize * viewport.height as usize],
                row_stride: viewport.width as usize,
            },
            provenance: compositor::FrameProvenance {
                document: (7, 4),
                content: Some(compositor::ContentFrameProvenance {
                    sequence,
                    recording_revision: 19,
                }),
            },
            sequence,
            viewport,
            drag_regions: Vec::new(),
        }
    }

    #[test]
    fn completed_swap_is_received_before_queued_prepare_and_submit() {
        use crate::presentation_runtime::ReceiveSwapAcks;
        use std::{collections::VecDeque, sync::mpsc};
        let source_id = browser_tracing::next_id();
        let mailbox = Arc::new(Mutex::new(None));
        let mut display = Display::New(
            DisplayTarget::Offscreen,
            Output {
                mailbox: mailbox.clone(),
                notify: Arc::new(|_| {}),
            },
            true,
        );
        let viewport = Viewport {
            width: 2,
            height: crate::engine::TOOLBAR_HEIGHT as u32 + 2,
            scale: 1.0,
        };
        let now = Instant::now();
        let frame = crate::begin_frame_source::NativeBeginFrame::from(
            foundation::begin_frame::BeginFrameArgs {
                source_id,
                sequence_number: 1,
                frame_time: now,
                deadline: now + Duration::from_millis(16),
                interval: Duration::from_millis(16),
            },
        );
        let (ack_sender, ack_receiver) = mpsc::channel();
        let ack: Arc<dyn Fn(u64) + Send + Sync> = Arc::new(move |sequence| {
            ack_sender.send(sequence).unwrap();
        });
        // These candidates already exist when swap 1 completes. The old FIFO
        // would see Prepare(2), Submit(2), Prepare(3), Submit(3), then Ack(1).
        let mut queued = VecDeque::from([prepared(2, viewport), prepared(3, viewport)]);
        display.prepared = Some(prepared(1, viewport));
        display.Submit(frame, true, &ack).unwrap();
        assert!(!display.viz.CanDrawAndSwap());
        for expected in [2, 3] {
            ReceiveSwapAcks(&ack_receiver, &mut display, &ack).unwrap();
            assert!(display.viz.CanDrawAndSwap());
            display.prepared = queued.pop_front();
            ReceiveSwapAcks(&ack_receiver, &mut display, &ack).unwrap();
            display.Submit(frame, true, &ack).unwrap();
            assert!(display.prepared.is_none());
            assert_eq!(
                mailbox
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .provenance
                    .unwrap()
                    .content
                    .unwrap()
                    .sequence,
                expected
            );
        }
        ReceiveSwapAcks(&ack_receiver, &mut display, &ack).unwrap();
        assert!(display.viz.CanDrawAndSwap());
    }

    #[test]
    fn delayed_swap_completion_keeps_throttle_and_stale_tokens_are_noops() {
        use crate::presentation_runtime::ReceiveSwapAcks;
        use std::sync::mpsc;
        let mut display = Display::New(
            DisplayTarget::Offscreen,
            Output {
                mailbox: Arc::new(Mutex::new(None)),
                notify: Arc::new(|_| {}),
            },
            false,
        );
        let viewport = Viewport {
            width: 2,
            height: crate::engine::TOOLBAR_HEIGHT as u32 + 2,
            scale: 1.0,
        };
        display.prepared = Some(prepared(42, viewport));
        display.viz.DidSubmitSwap(SwapId(41)).unwrap();
        let (sender, receiver) = mpsc::channel();
        let ack: Arc<dyn Fn(u64) + Send + Sync> = Arc::new(|_| {});
        // Models a native callback which has not completed yet: owner turns
        // alone do not free its slot or consume the prepared candidate.
        assert!(ReceiveSwapAcks(&receiver, &mut display, &ack)
            .unwrap()
            .is_empty());
        assert!(!display.viz.CanDrawAndSwap());
        sender.send(40).unwrap();
        assert!(ReceiveSwapAcks(&receiver, &mut display, &ack)
            .unwrap()
            .is_empty());
        assert!(!display.viz.CanDrawAndSwap());
        sender.send(41).unwrap();
        assert!(matches!(
            ReceiveSwapAcks(&receiver, &mut display, &ack)
                .unwrap()
                .as_slice(),
            [Effect::RequestBeginFrame]
        ));
        assert!(display.viz.CanDrawAndSwap());
        display.viz.DidSubmitSwap(SwapId(42)).unwrap();
        sender.send(41).unwrap();
        assert!(ReceiveSwapAcks(&receiver, &mut display, &ack)
            .unwrap()
            .is_empty());
        assert!(!display.viz.CanDrawAndSwap());
        assert_eq!(display.prepared.as_ref().unwrap().sequence, 42);
    }

    #[test]
    fn offscreen_publication_traces_prepared_content_identity() {
        let source_id = browser_tracing::next_id();
        let mailbox = Arc::new(Mutex::new(None));
        let output = Output {
            mailbox: mailbox.clone(),
            notify: Arc::new(|_| {}),
        };
        let mut display = Display::New(DisplayTarget::Offscreen, output, true);
        let viewport = Viewport {
            width: 2,
            height: crate::engine::TOOLBAR_HEIGHT as u32 + 2,
            scale: 1.0,
        };
        let now = Instant::now();
        let frame = crate::begin_frame_source::NativeBeginFrame::from(
            foundation::begin_frame::BeginFrameArgs {
                source_id,
                sequence_number: 1,
                frame_time: now,
                deadline: now + Duration::from_millis(16),
                interval: Duration::from_millis(16),
            },
        );
        let ack: Arc<dyn Fn(u64) + Send + Sync> = Arc::new(|_| {});
        let recording = browser_tracing::start(Default::default()).unwrap();
        for (sequence, content) in [
            None,
            Some(compositor::ContentFrameProvenance {
                sequence: 1,
                recording_revision: 19,
            }),
        ]
        .into_iter()
        .enumerate()
        {
            let sequence = sequence as u64 + 1;
            display.prepared = Some(PreparedDisplayFrame {
                output: PreparedDisplayTarget::Offscreen {
                    pixels: vec![0x0011_2233; viewport.width as usize * viewport.height as usize],
                    row_stride: viewport.width as usize,
                },
                provenance: compositor::FrameProvenance {
                    document: (7, 4),
                    content,
                },
                sequence,
                viewport,
                drag_regions: Vec::new(),
            });
            display.Submit(frame, false, &ack).unwrap();
            assert_eq!(
                mailbox.lock().unwrap().as_ref().unwrap().provenance,
                Some(compositor::FrameProvenance {
                    document: (7, 4),
                    content
                })
            );
            display.viz.DidReceiveSwapAck(SwapId(sequence));
        }
        browser_tracing::stop().unwrap();
        let snapshot = browser_tracing::snapshot(Some(recording)).unwrap();
        let published: Vec<_> = snapshot
            .instants
            .iter()
            .filter(|event| {
                event.name == "OffscreenFramePublished" && event.context.source_id == source_id
            })
            .collect();
        assert_eq!(published.len(), 2);
        assert_eq!(published[0].fields["has_content"], 0.0);
        assert_eq!(published[0].fields["content_sequence"], 0.0);
        assert_eq!(published[1].fields["has_content"], 1.0);
        assert_eq!(published[1].fields["tab_id"], 7.0);
        assert_eq!(published[1].fields["document_generation"], 4.0);
        assert_eq!(published[1].fields["content_sequence"], 1.0);
        assert_eq!(published[1].fields["content_recording_revision"], 19.0);
        assert!(!snapshot
            .instants
            .iter()
            .any(|event| event.name == "FramePresentedToNative"));
    }
}
