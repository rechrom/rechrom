//! Final display compositor state.
//!
//! This module owns the native surface and display resources, but deliberately
//! knows nothing about threads, channels, or executors. `PresentationRuntime`
//! delivers messages and routes the returned effects.
use crate::{
    compositor::{PlannedFrame, RasterBundle},
    engine::{UserEvent, Viewport},
    window_surface::{PreparedWindowFrame, WindowTarget},
};
use std::{io, sync::Arc, time::Instant};

pub(crate) enum Message {
    Install(RasterBundle),
    Prepare {
        frame: crate::begin_frame_source::NativeBeginFrame,
        planned: PlannedFrame,
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

struct PreparedCompositorFrame {
    output: PreparedWindowFrame,
    sequence: u64,
    viewport: Viewport,
    drag_regions: Vec<crate::chrome::DragRegion>,
}

pub(crate) struct Display {
    target: WindowTarget,
    active_bundle: Option<RasterBundle>,
    prepared: Option<PreparedCompositorFrame>,
    pending_swap: Option<u64>,
    sequence: u64,
    notify: Arc<dyn Fn(UserEvent) + Send + Sync>,
}

impl Display {
    pub(crate) fn New(target: WindowTarget, notify: Arc<dyn Fn(UserEvent) + Send + Sync>) -> Self {
        Self {
            target,
            active_bundle: None,
            prepared: None,
            pending_swap: None,
            sequence: 0,
            notify,
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
                planned,
                scroll_active,
            } => {
                self.Prepare(frame, planned, scroll_active)?;
                Ok(Vec::new())
            }
            Message::Submit {
                frame,
                scroll_active,
            } => self.Submit(frame, scroll_active, post_swap_ack),
            Message::SwapAck(sequence) => {
                if self.pending_swap == Some(sequence) {
                    self.pending_swap = None;
                }
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
        planned: PlannedFrame,
        scroll_active: bool,
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
        let overlay_scrollbar = scroll_active
            .then(|| planned.RootOverlayScrollbar())
            .flatten();
        let output = self.target.compose_prepared_plans(
            &planned.toolbar,
            &mut bundle.toolbar_renderer,
            planned.content.as_ref(),
            &mut bundle.content_renderer,
            planned.snapshot.viewport,
            sequence,
            overlay_scrollbar,
        )?;
        self.prepared = Some(PreparedCompositorFrame {
            output,
            sequence,
            viewport: planned.snapshot.viewport,
            drag_regions: planned.snapshot.drag_regions,
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
            crate::presentation_runtime::DisplayDrawDeadline(frame)
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
            return Ok(vec![Effect::RequestBeginFrame]);
        }
        let Some(prepared) = self.prepared.take() else {
            frame_trace.set("submitted", 0.0);
            return Ok(Vec::new());
        };
        let PreparedCompositorFrame {
            output,
            sequence,
            viewport,
            drag_regions,
        } = prepared;
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
        self.pending_swap = Some(sequence);
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
