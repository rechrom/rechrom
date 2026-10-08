//! Final display compositor state.
//!
//! This module owns the native surface and display resources, but deliberately
//! knows nothing about threads, channels, or executors. `PresentationRuntime`
//! delivers messages and routes the returned effects.
use crate::{
    compositor::{RasterBundle, SubmittedFrame},
    engine::{Output, UserEvent, Viewport},
    window_surface::{PreparedWindowFrame, WindowTarget},
};
use renderer::Renderer;
use std::{io, sync::Arc, time::Instant};
use viz::{SurfaceId, SwapId, VizEngine};

pub(crate) const TOOLBAR_SURFACE: SurfaceId = SurfaceId(1);
pub(crate) const CONTENT_SURFACE: SurfaceId = SurfaceId(2);

pub(crate) enum Message {
    Install(RasterBundle),
    Prepare {
        frame: crate::begin_frame_source::NativeBeginFrame,
        submitted: SubmittedFrame,
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
    ReturnBundle(RasterBundle),
    RequestBeginFrame,
}

struct PreparedDisplayFrame {
    output: PreparedWindowFrame,
    sequence: u64,
    viewport: Viewport,
    drag_regions: Vec<crate::chrome::DragRegion>,
}

pub(crate) struct Display {
    target: WindowTarget,
    active_bundle: Option<RasterBundle>,
    prepared: Option<PreparedDisplayFrame>,
    sequence: u64,
    notify: Arc<dyn Fn(UserEvent) + Send + Sync>,
    diagnostic_output: Option<Output>,
    viz: VizEngine,
    renderer: Renderer,
}

impl Display {
    pub(crate) fn New(target: WindowTarget, output: Output, capture_frames: bool) -> Self {
        Self {
            target,
            active_bundle: None,
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
        match message {
            Message::Stop => Ok(Vec::new()),
            Message::Install(bundle) => Ok(self
                .active_bundle
                .replace(bundle)
                .map(Effect::ReturnBundle)
                .into_iter()
                .collect()),
            Message::Prepare {
                frame,
                submitted,
                scroll_active,
            } => {
                self.Prepare(frame, submitted, scroll_active)?;
                Ok(Vec::new())
            }
            Message::Submit {
                frame,
                scroll_active,
            } => self.Submit(frame, scroll_active, post_swap_ack),
            Message::SwapAck(sequence) => {
                self.viz.DidReceiveSwapAck(SwapId(sequence));
                Ok(if self.prepared.is_some() {
                    vec![Effect::RequestBeginFrame]
                } else {
                    Vec::new()
                })
            }
        }
    }

    fn Prepare(
        &mut self,
        frame: crate::begin_frame_source::NativeBeginFrame,
        submitted: SubmittedFrame,
        _scroll_active: bool,
    ) -> io::Result<()> {
        let _frame = browser_tracing::scope(browser_tracing::Context {
            target_id: 1,
            source_id: frame.source_id,
            frame_id: frame.sequence_number,
            ..Default::default()
        });
        let Some(bundle) = &mut self.active_bundle else {
            return Ok(());
        };
        self.sequence = self.sequence.wrapping_add(1);
        let sequence = self.sequence;
        self.viz
            .SubmitFrame(TOOLBAR_SURFACE, submitted.toolbar)
            .map_err(io::Error::other)?;
        let has_content = if let Some(content) = submitted.content {
            self.viz
                .SubmitFrame(CONTENT_SURFACE, content)
                .map_err(io::Error::other)?;
            true
        } else {
            self.viz.DestroySurface(CONTENT_SURFACE);
            self.renderer.InvalidateSurface(CONTENT_SURFACE);
            false
        };
        let toolbar_height = submitted.viewport.toolbar_pixels();
        let output_rect = ::compositor::DeviceRect::new(
            0,
            0,
            submitted.viewport.width,
            submitted.viewport.height,
        );
        let mut placements = vec![viz::SurfacePlacement {
            surface_id: TOOLBAR_SURFACE,
            destination: ::compositor::DeviceRect::new(
                0,
                0,
                submitted.viewport.width,
                toolbar_height,
            ),
        }];
        if has_content {
            placements.push(viz::SurfacePlacement {
                surface_id: CONTENT_SURFACE,
                destination: ::compositor::DeviceRect::new(
                    0,
                    toolbar_height as i32,
                    submitted.viewport.width,
                    submitted.viewport.height.saturating_sub(toolbar_height),
                ),
            });
        }
        let aggregated = self
            .viz
            .Aggregate(output_rect, &placements)
            .map_err(io::Error::other)?;
        let output = self.target.render_aggregated_frame(
            &aggregated,
            &mut self.renderer,
            &bundle.toolbar_raster,
            has_content.then_some(&bundle.content_raster),
            submitted.viewport,
            sequence,
        )?;
        self.prepared = Some(PreparedDisplayFrame {
            output,
            sequence,
            viewport: submitted.viewport,
            drag_regions: submitted.drag_regions,
        });
        browser_tracing::instant(
            "frame",
            "CompositorCandidateReady",
            &[
                ("present_sequence", sequence as f64),
                (
                    "prepare_phase_ms",
                    Instant::now()
                        .saturating_duration_since(frame.frame_time)
                        .as_secs_f64()
                        * 1000.0,
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
            return Ok(vec![Effect::RequestBeginFrame]);
        }
        let Some(prepared) = self.prepared.take() else {
            frame_trace.set("submitted", 0.0);
            return Ok(Vec::new());
        };
        let PreparedDisplayFrame {
            output,
            sequence,
            viewport,
            drag_regions,
        } = prepared;
        if let Some(diagnostic) = &self.diagnostic_output {
            diagnostic.publish(output.readback_rgba(viewport)?);
        }
        let notify = self.notify.clone();
        let post_swap_ack = post_swap_ack.clone();
        self.target.present_prepared(
            output,
            sequence,
            Box::new(move |_, _| {
                (notify)(UserEvent::FramePresented(viewport));
                (post_swap_ack)(sequence);
            }),
        )?;
        self.viz
            .DidSubmitSwap(SwapId(sequence))
            .map_err(io::Error::other)?;
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
            ],
        );
        browser_tracing::instant(
            "frame",
            "FramePresentedToNative",
            &[("present_sequence", sequence as f64), ("compositor", 1.0)],
        );
        Ok(if scroll_active {
            vec![Effect::RequestBeginFrame]
        } else {
            Vec::new()
        })
    }
}
