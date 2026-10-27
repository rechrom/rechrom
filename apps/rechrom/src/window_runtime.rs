//! Window presentation receives placement and already rendered Page surfaces.
//! No document, paint artifact, Page compositor, or raster worker lives here.
use crate::{
    begin_frame_source::{LogicalBeginFrameSource, NativeBeginFrame},
    display::{self, Display},
    engine::{Output, UserEvent},
    presentation_runtime::{Config, DisplayExecutor},
    window_surface::DisplayTarget,
};
use foundation::begin_frame::BeginFrameSource;
use std::{
    collections::{BTreeMap, VecDeque},
    io,
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

/// Window-owned readiness for the Page surfaces embedded by the current Scene.
///
/// Page producers finish independently. During a resize, treating the first
/// arriving surface as an all-surfaces-ready signal lets a fast toolbar frame
/// force an immediate draw while the content surface still has the old size.
/// Keep this coordination beside DisplayScheduler; Page and Viz algorithms only
/// continue to publish their normal immutable outputs.
#[derive(Default)]
struct SurfaceReadiness {
    scene: Option<crate::page_presentation::Scene>,
    viewports: BTreeMap<crate::page_presentation::Document, crate::engine::Viewport>,
    ready_damage: bool,
    interaction_damage: bool,
}

impl SurfaceReadiness {
    fn observe(&mut self, message: &display::Message) -> Option<bool> {
        match message {
            display::Message::Scene(scene) => {
                self.scene = Some(scene.clone());
                self.ready_damage = false;
                self.interaction_damage = false;
                None
            }
            display::Message::Prepare { submitted } => {
                self.viewports
                    .insert(submitted.provenance.document, submitted.viewport);
                self.interaction_damage |= submitted.is_handling_interaction;
                let all_ready = self.all_placed_surfaces_match();
                self.ready_damage |= all_ready;
                Some(all_ready)
            }
            display::Message::Submit { .. }
            | display::Message::SwapAck(_)
            | display::Message::Stop => None,
        }
    }

    fn all_placed_surfaces_match(&self) -> bool {
        self.scene.as_ref().is_some_and(|scene| {
            !scene.placements.is_empty()
                && scene.placements.iter().all(|placement| {
                    self.viewports
                        .get(&placement.document)
                        .is_some_and(|viewport| {
                            viewport.width == placement.destination.width
                                && viewport.height == placement.destination.height
                        })
                })
        })
    }

    fn has_ready_damage(&self) -> bool {
        self.ready_damage
    }

    fn has_interaction_damage(&self) -> bool {
        self.interaction_damage
    }

    fn did_schedule_draw(&mut self) {
        self.ready_damage = false;
        self.interaction_damage = false;
    }
}

pub(crate) enum Command {
    Display(display::Message),
    Tick(NativeBeginFrame),
    Ack(u64),
}
pub(crate) fn spawn(
    config: Config,
    target: DisplayTarget,
    output: Output,
    source: Arc<LogicalBeginFrameSource>,
) -> io::Result<mpsc::Sender<Command>> {
    let (sender, receiver) = mpsc::channel();
    let ticks = sender.clone();
    source.SetClient(Arc::new(move |frame| {
        let _ = ticks.send(Command::Tick(frame));
    }))?;
    let reply = sender.clone();
    std::thread::Builder::new()
        .name("window-presentation".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            crate::engine::set_page_thread_presentation_priority();
            let _scope = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            let result = run(config, target, output.clone(), source, receiver, reply);
            if let Err(error) = result {
                (output.notify)(UserEvent::Fatal(crate::engine::FatalError::new(
                    "window-presentation",
                    error,
                )));
            }
        })?;
    Ok(sender)
}
pub(crate) enum DisplayWork {
    Command(display::Message),
    SwapAckReady,
}
pub(crate) fn ReceiveDisplayWork(
    receiver: &mpsc::Receiver<DisplayWork>,
    pending: &mut VecDeque<DisplayWork>,
) -> Result<DisplayWork, mpsc::RecvError> {
    // Finish the selected batch before accepting new work; incoming frames
    // cannot indefinitely postpone its selected draw deadline.
    if let Some(message) = pending.pop_front() {
        return Ok(message);
    }
    let mut message = receiver.recv()?;
    let mut submissions = VecDeque::new();
    let mut draw = None;
    let mut draws = 0;
    let mut barrier = None;
    loop {
        match message {
            DisplayWork::Command(command @ display::Message::Prepare { .. }) => {
                submissions.push_back(DisplayWork::Command(command));
                draw = None;
            }
            DisplayWork::Command(command @ display::Message::Submit { .. }) => {
                draws += 1;
                draw = Some(DisplayWork::Command(command));
            }
            control => {
                barrier = Some(control);
                break;
            }
        }
        match receiver.try_recv() {
            Ok(next) => message = next,
            Err(_) => break,
        }
    }
    if draws > usize::from(draw.is_some()) {
        browser_tracing::instant(
            "frame",
            "DisplayDrawRequestsCoalesced",
            &[("requests", (draws - usize::from(draw.is_some())) as f64)],
        );
    }
    pending.append(&mut submissions);
    pending.extend(draw);
    pending.extend(barrier);
    Ok(pending.pop_front().expect("received display work"))
}

fn apply_display_work(
    display: &mut Display,
    acks: &mpsc::Receiver<u64>,
    work: DisplayWork,
    post_ack: &Arc<dyn Fn(u64) + Send + Sync>,
) -> io::Result<Vec<display::Effect>> {
    // Completion tokens have their own queue and are consumed by the Display
    // owner before any queued draw, exactly as in the previous executor.
    let mut effects = Vec::new();
    while let Ok(id) = acks.try_recv() {
        effects.extend(display.Handle(display::Message::SwapAck(id), post_ack)?);
    }
    if let DisplayWork::Command(message) = work {
        effects.extend(display.Handle(message, post_ack)?);
    }
    Ok(effects)
}
fn run(
    config: Config,
    target: DisplayTarget,
    output: Output,
    source: Arc<LogicalBeginFrameSource>,
    receiver: mpsc::Receiver<Command>,
    reply: mpsc::Sender<Command>,
) -> io::Result<()> {
    let (acks, ack_receiver) = mpsc::channel();
    let post_ack: Arc<dyn Fn(u64) + Send + Sync> = Arc::new(move |id| {
        if acks.send(id).is_ok() {
            let _ = reply.send(Command::Ack(id));
        }
    });
    let mut local = None;
    let mut remote = None;
    if config.display_executor == DisplayExecutor::DedicatedThread {
        let (sender, receiver) = mpsc::channel();
        remote = Some(sender);
        let source = source.clone();
        let post_ack = post_ack.clone();
        std::thread::Builder::new()
            .name("window-display".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                crate::engine::set_page_thread_presentation_priority();
                let mut display = Display::New(target, output.clone(), config.capture_frames);
                let mut pending = VecDeque::new();
                while let Ok(work) = ReceiveDisplayWork(&receiver, &mut pending) {
                    if matches!(work, DisplayWork::Command(display::Message::Stop)) {
                        break;
                    }
                    let result = apply_display_work(&mut display, &ack_receiver, work, &post_ack);
                    match result {
                        Ok(effects) => {
                            for _ in effects {
                                source.request_begin_frame();
                            }
                        }
                        Err(error) => {
                            (output.notify)(UserEvent::Fatal(crate::engine::FatalError::new(
                                "display", error,
                            )));
                            break;
                        }
                    }
                }
            })?;
    } else {
        local = Some((
            Display::New(target, output, config.capture_frames),
            ack_receiver,
        ));
    }
    let mut dispatch = |work| -> io::Result<()> {
        if let Some(sender) = &remote {
            sender
                .send(work)
                .map_err(|_| io::Error::other("Display stopped"))?;
        } else if let Some((display, acks)) = &mut local {
            for _ in apply_display_work(display, acks, work, &post_ack)? {
                source.request_begin_frame();
            }
        }
        Ok(())
    };
    let mut scheduler = viz::DisplayScheduler::default();
    let mut surfaces = SurfaceReadiness::default();
    loop {
        let timeout = scheduler
            .NextDeadline()
            .map_or(Duration::from_secs(1), |t| {
                t.saturating_duration_since(Instant::now())
            });
        match receiver.recv_timeout(timeout) {
            Ok(Command::Display(display::Message::Stop))
            | Err(mpsc::RecvTimeoutError::Disconnected) => {
                dispatch(DisplayWork::Command(display::Message::Stop))?;
                return Ok(());
            }
            Ok(Command::Display(message)) => {
                let all_surfaces_ready = surfaces.observe(&message);
                dispatch(DisplayWork::Command(message))?;
                if let Some(all_surfaces_ready) = all_surfaces_ready {
                    // Damage after a completed deadline belongs to the next
                    // scheduler opportunity. Chromium's optional previous-frame
                    // swap requires CanDrawForPreviousFrame; a deadline alone
                    // does not authorize another Submit for the same frame.
                    // Source: components/viz/service/display/
                    //         display_scheduler.cc OnDisplayDamaged.
                    //
                    // SurfaceDamageObserver has already folded the submitted
                    // CompositorFrameMetadata into DamageTracker at this
                    // point in Chromium. Preserve that ordering here: active
                    // scroll damage is ready now and therefore selects the
                    // immediate deadline instead of waiting until the regular
                    // display draw deadline and missing Core Animation's latch.
                    // is_handling_interaction is aggregate display-frame state.
                    // Do not expose it to DisplayScheduler while another placed
                    // Page surface still has the previous resize geometry: its
                    // immediate mode would otherwise bypass the pending-surface
                    // wait. Retain the bit until the complete candidate arrives.
                    if all_surfaces_ready && surfaces.has_interaction_damage() {
                        scheduler.SetDamageDueToInteraction(true);
                    }
                    scheduler.OnPendingSurfacesChanged(!all_surfaces_ready);
                    scheduler.OnDisplayDamaged(Instant::now());
                }
                source.request_begin_frame();
            }
            Ok(Command::Tick(frame)) => {
                if let Some(previous) = scheduler.BeginFrame(
                    frame,
                    viz::FrameTiming {
                        frame_time: frame.frame_time,
                        interval: frame.interval,
                        source_deadline: frame.deadline,
                    },
                ) {
                    dispatch(DisplayWork::Command(display::Message::Submit {
                        frame: previous.Payload(),
                    }))?;
                }
                // The physical source has already dispatched this BeginFrame
                // to Page compositors. Keep Viz in regular-deadline mode until
                // a Page surface arrives; if none does, the deadline still
                // aggregates the previously submitted surfaces.
                if surfaces.has_ready_damage() && surfaces.has_interaction_damage() {
                    scheduler.SetDamageDueToInteraction(true);
                }
                scheduler.OnPendingSurfacesChanged(!surfaces.has_ready_damage());
                let pending = scheduler.PendingFrame().unwrap();
                scheduler.MarkPrepared(pending, true);
            }
            Ok(Command::Ack(_id)) => {
                dispatch(DisplayWork::SwapAckReady)?;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if let Some(frame) = scheduler.TakeIfDrawDue(Instant::now()) {
            surfaces.did_schedule_draw();
            dispatch(DisplayWork::Command(display::Message::Submit { frame }))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::Viewport,
        page_presentation::{DeviceRect, FrameProvenance, PageFrame, Placement, Scene},
    };

    fn page(document: (u64, u64), width: u32, height: u32, interaction: bool) -> display::Message {
        display::Message::Prepare {
            submitted: PageFrame {
                provenance: FrameProvenance {
                    document,
                    content: None,
                },
                viewport: Viewport {
                    width,
                    height,
                    scale: 1.0,
                },
                is_handling_interaction: interaction,
                pixels: Arc::new(Vec::new()),
                recycler: None,
            },
        }
    }

    #[test]
    fn resize_damage_waits_for_every_placed_surface() {
        let toolbar = (0, 0);
        let content = (1, 1);
        let mut state = SurfaceReadiness::default();
        state.observe(&display::Message::Scene(Scene {
            viewport: Viewport {
                width: 1000,
                height: 700,
                scale: 1.0,
            },
            placements: vec![
                Placement {
                    document: toolbar,
                    destination: DeviceRect::new(0, 0, 1000, 86),
                },
                Placement {
                    document: content,
                    destination: DeviceRect::new(0, 86, 1000, 614),
                },
            ],
            live_documents: vec![toolbar, content],
            metadata: Vec::new(),
        }));

        assert_eq!(state.observe(&page(toolbar, 1000, 86, true)), Some(false));
        assert!(!state.has_ready_damage());
        assert!(state.has_interaction_damage());
        assert_eq!(state.observe(&page(content, 900, 614, false)), Some(false));
        assert!(!state.has_ready_damage());
        assert_eq!(state.observe(&page(content, 1000, 614, false)), Some(true));
        assert!(state.has_ready_damage());
        assert!(state.has_interaction_damage());

        state.did_schedule_draw();
        assert!(!state.has_ready_damage());
        assert!(!state.has_interaction_damage());
    }
}
