//! Physical execution and message routing for compositor, raster and display.
//!
//! Core modules never create a thread or inspect their executor. This runtime
//! can place Display on its own presentation thread or co-locate it with the
//! Compositor while retaining the same mailbox boundary.
use crate::{
    begin_frame_source::NativeBeginFrame,
    chrome::DragRegion,
    display::{self, Display},
    engine::{Output, UserEvent, Viewport},
    window_surface::DisplayTarget,
};
use foundation::begin_frame::BeginFrameSource;
use std::{
    collections::VecDeque,
    io,
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

pub(crate) type CompositorCommand =
    compositor::Command<Viewport, NativeBeginFrame, Vec<DragRegion>>;
type CompositorEffect = compositor::Effect<Viewport, NativeBeginFrame, Vec<DragRegion>>;
type CompositorEngine = compositor::CompositorEngine<Viewport, NativeBeginFrame, Vec<DragRegion>>;
type RasterJob = compositor::RasterJob<Viewport, Vec<DragRegion>>;
type RasterReady = compositor::RasterReady<Viewport, Vec<DragRegion>>;
pub(crate) type ArtifactSnapshot = compositor::ArtifactSnapshot<Viewport, Vec<DragRegion>>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplayExecutor {
    #[default]
    DedicatedThread,
    CompositorThread,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Config {
    pub display_executor: DisplayExecutor,
    pub capture_frames: bool,
}

enum LocalMessage {
    Compositor(CompositorCommand),
    Display(display::Message),
}

enum DisplayMessage {
    Command(display::Message),
    SwapAckReady,
}

// Surface submission and DrawAndSwap are distinct lifecycle operations.
// Deliver every Prepare to Viz (including its resources), while multiple queued
// draw deadlines represent one outstanding draw demand, as in DisplayScheduler.
// Bundle ownership changes and control messages remain ordered barriers.
fn ReceiveDisplayMessage(
    receiver: &mpsc::Receiver<DisplayMessage>,
    pending: &mut VecDeque<DisplayMessage>,
) -> Result<DisplayMessage, mpsc::RecvError> {
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
            DisplayMessage::Command(command @ display::Message::Prepare { .. }) => {
                submissions.push_back(DisplayMessage::Command(command));
                draw = None;
            }
            DisplayMessage::Command(command @ display::Message::Submit { .. }) => {
                draws += 1;
                draw = Some(DisplayMessage::Command(command));
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

/// Completion is control input, not more draw work. Keep actual completion
/// tokens outside the draw FIFO so already-queued Prepare/Submit commands do
/// not run against an obsolete pending-swap count. Only the Display owner
/// consumes these tokens, after its current Handle call has returned.
pub(crate) fn ReceiveSwapAcks(
    receiver: &mpsc::Receiver<u64>,
    display: &mut Display,
    post_swap_ack: &Arc<dyn Fn(u64) + Send + Sync>,
) -> io::Result<Vec<display::Effect>> {
    let mut effects = Vec::new();
    while let Ok(sequence) = receiver.try_recv() {
        effects.extend(display.Handle(display::Message::SwapAck(sequence), post_swap_ack)?);
    }
    Ok(effects)
}

pub(crate) type MainThreadClient = Arc<
    dyn Fn(compositor::MainThreadEvent<crate::begin_frame_source::NativeBeginFrame>) + Send + Sync,
>;

pub(crate) fn Spawn(
    config: Config,
    target: DisplayTarget,
    output: Output,
    input_frame_source: Arc<crate::begin_frame_source::LogicalBeginFrameSource>,
    display_frame_source: Arc<crate::begin_frame_source::LogicalBeginFrameSource>,
    main_thread: MainThreadClient,
) -> io::Result<mpsc::Sender<CompositorCommand>> {
    let (compositor_sender, compositor_receiver) = mpsc::channel();
    let input_sender = compositor_sender.clone();
    input_frame_source.SetClient(Arc::new(move |frame| {
        let _ = input_sender.send(CompositorCommand::InputBeginFrame(frame));
    }))?;
    let display_sender = compositor_sender.clone();
    display_frame_source.SetClient(Arc::new(move |frame| {
        let _ = display_sender.send(CompositorCommand::DisplayBeginFrame(frame));
    }))?;
    let frame_source: Arc<dyn BeginFrameSource> = display_frame_source;
    let raster_sender = SpawnRasterOwner(compositor_sender.clone())?;
    match config.display_executor {
        DisplayExecutor::DedicatedThread => SpawnDedicated(
            target,
            output,
            frame_source,
            compositor_sender.clone(),
            compositor_receiver,
            raster_sender,
            config.capture_frames,
            main_thread,
        )?,
        DisplayExecutor::CompositorThread => SpawnShared(
            target,
            output,
            frame_source,
            compositor_receiver,
            raster_sender,
            config.capture_frames,
            main_thread,
        )?,
    }
    Ok(compositor_sender)
}

fn SpawnRasterOwner(
    compositor_sender: mpsc::Sender<CompositorCommand>,
) -> io::Result<mpsc::Sender<RasterJob>> {
    let (raster_sender, raster_receiver) = mpsc::channel::<RasterJob>();
    std::thread::Builder::new()
        .name("browser-raster-owner".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            while let Ok(mut job) = raster_receiver.recv() {
                let result =
                    compositor::PlanAndRaster(&mut job.bundle, job.snapshot, job.activation_target);
                if compositor_sender
                    .send(CompositorCommand::RasterReady(RasterReady {
                        result,
                        bundle: job.bundle,
                    }))
                    .is_err()
                {
                    break;
                }
            }
        })?;
    Ok(raster_sender)
}

fn SpawnDedicated(
    target: DisplayTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    compositor_sender: mpsc::Sender<CompositorCommand>,
    compositor_receiver: mpsc::Receiver<CompositorCommand>,
    raster_sender: mpsc::Sender<RasterJob>,
    capture_frames: bool,
    main_thread: MainThreadClient,
) -> io::Result<()> {
    let (display_sender, display_receiver) = mpsc::channel::<DisplayMessage>();
    let (ack_sender, ack_receiver) = mpsc::channel();
    let display_reply = display_sender.clone();
    let display_compositor = compositor_sender.clone();
    let display_frames = frame_source.clone();
    let display_output = output.clone();
    std::thread::Builder::new()
        .name("browser-display".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            SetPresentationThreadPriority();
            let mut display = Display::New(target, display_output.clone(), capture_frames);
            let post_swap_ack: Arc<dyn Fn(u64) + Send + Sync> = Arc::new(move |sequence| {
                if ack_sender.send(sequence).is_ok() {
                    let _ = display_reply.send(DisplayMessage::SwapAckReady);
                }
            });
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            let mut pending = VecDeque::new();
            while let Ok(message) = ReceiveDisplayMessage(&display_receiver, &mut pending) {
                if matches!(message, DisplayMessage::Command(display::Message::Stop)) {
                    break;
                }
                let result = ReceiveSwapAcks(&ack_receiver, &mut display, &post_swap_ack).and_then(
                    |mut effects| {
                        if let DisplayMessage::Command(message) = message {
                            effects.extend(display.Handle(message, &post_swap_ack)?);
                        }
                        Ok(effects)
                    },
                );
                let effects = match result {
                    Ok(effects) => effects,
                    Err(error) => {
                        (display_output.notify)(UserEvent::Fatal(crate::engine::FatalError::new(
                            "display", error,
                        )));
                        break;
                    }
                };
                for effect in effects {
                    match effect {
                        display::Effect::ReturnBundle(bundle) => {
                            if display_compositor
                                .send(CompositorCommand::SpareBundle(bundle))
                                .is_err()
                            {
                                return;
                            }
                        }
                        display::Effect::RasterReady(ready) => {
                            if display_compositor
                                .send(CompositorCommand::ActiveRasterReady(ready))
                                .is_err()
                            {
                                return;
                            }
                        }
                        display::Effect::RequestBeginFrame => display_frames.request_begin_frame(),
                    }
                }
            }
        })?;
    std::thread::Builder::new()
        .name("browser-compositor".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            SetPresentationThreadPriority();
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            if let Err(error) = RunDedicatedCompositor(
                compositor_receiver,
                raster_sender,
                display_sender,
                frame_source,
                main_thread,
            ) {
                (output.notify)(UserEvent::Fatal(crate::engine::FatalError::new(
                    "compositor",
                    error,
                )));
            }
        })?;
    Ok(())
}

fn RunDedicatedCompositor(
    receiver: mpsc::Receiver<CompositorCommand>,
    raster_sender: mpsc::Sender<RasterJob>,
    display_sender: mpsc::Sender<DisplayMessage>,
    frame_source: Arc<dyn BeginFrameSource>,
    main_thread: MainThreadClient,
) -> io::Result<()> {
    let mut compositor = CompositorEngine::default();
    let mut scheduler = viz::DisplayScheduler::default();
    loop {
        let timeout = PendingTimeout(&scheduler);
        match receiver.recv_timeout(timeout) {
            Ok(CompositorCommand::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = display_sender.send(DisplayMessage::Command(display::Message::Stop));
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                AdvanceCycle(
                    &mut compositor,
                    &mut scheduler,
                    &raster_sender,
                    &display_sender,
                    &frame_source,
                    &main_thread,
                )?;
            }
            Ok(CompositorCommand::DisplayBeginFrame(frame)) => {
                if let Some(previous) = scheduler.BeginFrame(frame, FrameTiming(frame)) {
                    if !previous.IsPrepared() {
                        RouteDedicated(
                            &mut scheduler,
                            compositor.Prepare(previous.Payload())?,
                            &raster_sender,
                            &display_sender,
                            &frame_source,
                            &main_thread,
                        )?;
                    }
                    RouteDedicated(
                        &mut scheduler,
                        compositor.Draw(previous.Payload()),
                        &raster_sender,
                        &display_sender,
                        &frame_source,
                        &main_thread,
                    )?;
                }
            }
            Ok(CompositorCommand::BeginFrame(frame)) => RouteDedicated(
                &mut scheduler,
                compositor.Handle(CompositorCommand::BeginFrame(frame))?,
                &raster_sender,
                &display_sender,
                &frame_source,
                &main_thread,
            )?,
            Ok(message) => RouteDedicated(
                &mut scheduler,
                compositor.Handle(message)?,
                &raster_sender,
                &display_sender,
                &frame_source,
                &main_thread,
            )?,
        }
        AdvanceCycle(
            &mut compositor,
            &mut scheduler,
            &raster_sender,
            &display_sender,
            &frame_source,
            &main_thread,
        )?;
    }
}

fn AdvanceCycle(
    compositor: &mut CompositorEngine,
    scheduler: &mut viz::DisplayScheduler<crate::begin_frame_source::NativeBeginFrame>,
    raster_sender: &mpsc::Sender<RasterJob>,
    display_sender: &mpsc::Sender<DisplayMessage>,
    frame_source: &Arc<dyn BeginFrameSource>,
    main_thread: &MainThreadClient,
) -> io::Result<()> {
    let now = Instant::now();
    if scheduler.NeedsPrepare(now) {
        let attempt = scheduler
            .PendingFrame()
            .expect("Viz scheduler reported pending prepare without a frame");
        let effects = compositor.Prepare(attempt.Payload())?;
        scheduler.MarkPrepared(
            attempt,
            effects
                .iter()
                .any(|effect| matches!(effect, compositor::Effect::FrameProduced { .. })),
        );
        RouteDedicated(
            scheduler,
            effects,
            raster_sender,
            display_sender,
            frame_source,
            main_thread,
        )?;
    }
    if let Some(frame) = scheduler.TakeIfDrawDue(now) {
        RouteDedicated(
            scheduler,
            compositor.Draw(frame),
            raster_sender,
            display_sender,
            frame_source,
            main_thread,
        )?;
    }
    Ok(())
}

fn RouteDedicated(
    scheduler: &mut viz::DisplayScheduler<crate::begin_frame_source::NativeBeginFrame>,
    effects: Vec<CompositorEffect>,
    raster_sender: &mpsc::Sender<RasterJob>,
    display_sender: &mpsc::Sender<DisplayMessage>,
    frame_source: &Arc<dyn BeginFrameSource>,
    main_thread: &MainThreadClient,
) -> io::Result<()> {
    for effect in effects {
        match effect {
            compositor::Effect::Raster(job) => raster_sender
                .send(job)
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "raster owner stopped"))?,
            compositor::Effect::InstallRasterBundle(bundle) => display_sender
                .send(DisplayMessage::Command(display::Message::Install(bundle)))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "display stopped"))?,
            compositor::Effect::FrameProduced {
                frame,
                submitted,
                scroll_active,
            } => display_sender
                .send(DisplayMessage::Command(display::Message::Prepare {
                    frame,
                    submitted,
                    scroll_active,
                }))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "display stopped"))?,
            compositor::Effect::Submit {
                frame,
                scroll_active,
            } => display_sender
                .send(DisplayMessage::Command(display::Message::Submit {
                    frame,
                    scroll_active,
                }))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "display stopped"))?,
            compositor::Effect::MainThread(event) => main_thread(event),
            compositor::Effect::RequestBeginFrame => frame_source.request_begin_frame(),
            compositor::Effect::VisualDamage => scheduler.NotifyVisualDamage(Instant::now()),
        }
    }
    Ok(())
}

fn SpawnShared(
    target: DisplayTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    receiver: mpsc::Receiver<CompositorCommand>,
    raster_sender: mpsc::Sender<RasterJob>,
    capture_frames: bool,
    main_thread: MainThreadClient,
) -> io::Result<()> {
    std::thread::Builder::new()
        .name("browser-presentation".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            SetPresentationThreadPriority();
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            if let Err(error) = RunShared(
                target,
                output.clone(),
                frame_source,
                receiver,
                raster_sender,
                capture_frames,
                main_thread,
            ) {
                (output.notify)(UserEvent::Fatal(crate::engine::FatalError::new(
                    "presentation",
                    error,
                )));
            }
        })?;
    Ok(())
}

fn RunShared(
    target: DisplayTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    receiver: mpsc::Receiver<CompositorCommand>,
    raster_sender: mpsc::Sender<RasterJob>,
    capture_frames: bool,
    main_thread: MainThreadClient,
) -> io::Result<()> {
    let mut compositor = CompositorEngine::default();
    let mut display = Display::New(target, output, capture_frames);
    let mut scheduler = viz::DisplayScheduler::default();
    let mut local = VecDeque::new();
    let (ack_sender, ack_receiver) = mpsc::channel();
    let wake_frames = frame_source.clone();
    let post_swap_ack: Arc<dyn Fn(u64) + Send + Sync> = Arc::new(move |sequence| {
        let _ = ack_sender.send(sequence);
        wake_frames.request_begin_frame();
    });
    loop {
        for effect in ReceiveSwapAcks(&ack_receiver, &mut display, &post_swap_ack)? {
            RouteSharedDisplay(effect, &mut local, &frame_source);
        }
        if local.is_empty() {
            match receiver.recv_timeout(PendingTimeout(&scheduler)) {
                Ok(CompositorCommand::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    let _ = display.Handle(display::Message::Stop, &post_swap_ack)?;
                    return Ok(());
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Ok(message) => local.push_back(LocalMessage::Compositor(message)),
            }
        }
        while let Some(message) = local.pop_front() {
            // A synchronous completion can arrive while this local queue is
            // non-empty. Apply it before the next command, just as in the
            // dedicated executor. Draining cannot produce another swap: only
            // Submit does that, and it is not executed by ReceiveSwapAcks.
            for effect in ReceiveSwapAcks(&ack_receiver, &mut display, &post_swap_ack)? {
                RouteSharedDisplay(effect, &mut local, &frame_source);
            }
            match message {
                LocalMessage::Compositor(CompositorCommand::DisplayBeginFrame(frame)) => {
                    if let Some(previous) = scheduler.BeginFrame(frame, FrameTiming(frame)) {
                        if !previous.IsPrepared() {
                            RouteSharedCompositor(
                                &mut scheduler,
                                compositor.Prepare(previous.Payload())?,
                                &mut local,
                                &raster_sender,
                                &frame_source,
                                &main_thread,
                            )?;
                        }
                        RouteSharedCompositor(
                            &mut scheduler,
                            compositor.Draw(previous.Payload()),
                            &mut local,
                            &raster_sender,
                            &frame_source,
                            &main_thread,
                        )?;
                    }
                }
                LocalMessage::Compositor(CompositorCommand::BeginFrame(frame)) => {
                    RouteSharedCompositor(
                        &mut scheduler,
                        compositor.Handle(CompositorCommand::BeginFrame(frame))?,
                        &mut local,
                        &raster_sender,
                        &frame_source,
                        &main_thread,
                    )?;
                }
                LocalMessage::Compositor(CompositorCommand::Stop) => {
                    let _ = display.Handle(display::Message::Stop, &post_swap_ack)?;
                    return Ok(());
                }
                LocalMessage::Compositor(message) => RouteSharedCompositor(
                    &mut scheduler,
                    compositor.Handle(message)?,
                    &mut local,
                    &raster_sender,
                    &frame_source,
                    &main_thread,
                )?,
                LocalMessage::Display(message) => {
                    for effect in display.Handle(message, &post_swap_ack)? {
                        RouteSharedDisplay(effect, &mut local, &frame_source);
                    }
                }
            }
        }
        let now = Instant::now();
        if scheduler.NeedsPrepare(now) {
            let attempt = scheduler
                .PendingFrame()
                .expect("Viz scheduler reported pending prepare without a frame");
            let effects = compositor.Prepare(attempt.Payload())?;
            scheduler.MarkPrepared(
                attempt,
                effects
                    .iter()
                    .any(|effect| matches!(effect, compositor::Effect::FrameProduced { .. })),
            );
            RouteSharedCompositor(
                &mut scheduler,
                effects,
                &mut local,
                &raster_sender,
                &frame_source,
                &main_thread,
            )?;
        }
        if let Some(frame) = scheduler.TakeIfDrawDue(now) {
            RouteSharedCompositor(
                &mut scheduler,
                compositor.Draw(frame),
                &mut local,
                &raster_sender,
                &frame_source,
                &main_thread,
            )?;
        }
    }
}

fn RouteSharedDisplay(
    effect: display::Effect,
    local: &mut VecDeque<LocalMessage>,
    frame_source: &Arc<dyn BeginFrameSource>,
) {
    match effect {
        display::Effect::ReturnBundle(bundle) => local.push_back(LocalMessage::Compositor(
            CompositorCommand::SpareBundle(bundle),
        )),
        display::Effect::RasterReady(ready) => local.push_back(LocalMessage::Compositor(
            CompositorCommand::ActiveRasterReady(ready),
        )),
        display::Effect::RequestBeginFrame => frame_source.request_begin_frame(),
    }
}

fn RouteSharedCompositor(
    scheduler: &mut viz::DisplayScheduler<crate::begin_frame_source::NativeBeginFrame>,
    effects: Vec<CompositorEffect>,
    local: &mut VecDeque<LocalMessage>,
    raster_sender: &mpsc::Sender<RasterJob>,
    frame_source: &Arc<dyn BeginFrameSource>,
    main_thread: &MainThreadClient,
) -> io::Result<()> {
    for effect in effects {
        match effect {
            compositor::Effect::Raster(job) => raster_sender
                .send(job)
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "raster owner stopped"))?,
            compositor::Effect::InstallRasterBundle(bundle) => {
                local.push_back(LocalMessage::Display(display::Message::Install(bundle)))
            }
            compositor::Effect::FrameProduced {
                frame,
                submitted,
                scroll_active,
            } => local.push_back(LocalMessage::Display(display::Message::Prepare {
                frame,
                submitted,
                scroll_active,
            })),
            compositor::Effect::Submit {
                frame,
                scroll_active,
            } => local.push_back(LocalMessage::Display(display::Message::Submit {
                frame,
                scroll_active,
            })),
            compositor::Effect::MainThread(event) => main_thread(event),
            compositor::Effect::RequestBeginFrame => frame_source.request_begin_frame(),
            compositor::Effect::VisualDamage => scheduler.NotifyVisualDamage(Instant::now()),
        }
    }
    Ok(())
}

fn PendingTimeout(
    scheduler: &viz::DisplayScheduler<crate::begin_frame_source::NativeBeginFrame>,
) -> Duration {
    scheduler
        .NextDeadline()
        .map_or(Duration::from_millis(50), |deadline| {
            deadline.saturating_duration_since(Instant::now())
        })
}

fn FrameTiming(frame: crate::begin_frame_source::NativeBeginFrame) -> viz::FrameTiming {
    viz::FrameTiming {
        frame_time: frame.frame_time,
        interval: frame.interval,
        source_deadline: frame.deadline,
    }
}

fn SetPresentationThreadPriority() {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
        }
        const QOS_CLASS_USER_INTERACTIVE: u32 = 0x21;
        let result = unsafe { pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0) };
        if result != 0 {
            eprintln!(
                "presentation-thread-priority: {}",
                io::Error::from_raw_os_error(result)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_accepts_all_surfaces_and_coalesces_draws_at_resource_barriers() {
        let frame = |sequence_number| {
            NativeBeginFrame::from(foundation::begin_frame::BeginFrameArgs {
                source_id: 1,
                sequence_number,
                frame_time: Instant::now(),
                deadline: Instant::now() + Duration::from_millis(16),
                interval: Duration::from_millis(16),
            })
        };
        let submit = |sequence| {
            DisplayMessage::Command(display::Message::Submit {
                frame: frame(sequence),
                scroll_active: true,
            })
        };
        let prepare = |sequence| {
            let viewport = Viewport {
                width: 2,
                height: 2,
                scale: 1.,
            };
            let rect = compositor::DeviceRect::new(0, 0, 2, 2);
            let toolbar = compositor::CompositorFrame {
                metadata: compositor::CompositorFrameMetadata {
                    frame_token: sequence,
                    source_frame_id: sequence,
                    device_scale_factor: 1.,
                    raster_scale_factor: 1.,
                    logical_viewport: paint::paint_engine::PaintRect {
                        x: 0.,
                        y: 0.,
                        width: 2.,
                        height: 2.,
                    },
                    viewport: rect,
                    root_damage_rect: rect,
                    raster_task_count: 0,
                    begin_frame_ack: None,
                },
                resource_list: vec![],
                render_pass_list: vec![compositor::CompositorRenderPass {
                    id: compositor::RenderPassId(1),
                    owner_effect: None,
                    output_rect: rect,
                    damage_rect: rect,
                    shared_quad_state_list: vec![],
                    quad_list: vec![],
                }],
            };
            DisplayMessage::Command(display::Message::Prepare {
                frame: frame(sequence),
                scroll_active: true,
                submitted: compositor::SubmittedFrame {
                    provenance: compositor::FrameProvenance {
                        document: (1, 1),
                        content: None,
                    },
                    viewport,
                    toolbar,
                    content: None,
                    metadata: vec![],
                },
            })
        };
        let (sender, receiver) = mpsc::channel();
        let mut pending = VecDeque::new();
        for sequence in 1..=3 {
            sender.send(prepare(sequence)).unwrap();
            sender.send(submit(sequence)).unwrap();
        }
        sender.send(DisplayMessage::SwapAckReady).unwrap();
        for expected in 1..=3 {
            assert!(
                matches!(ReceiveDisplayMessage(&receiver, &mut pending).unwrap(),
                DisplayMessage::Command(display::Message::Prepare { frame, .. }) if frame.sequence_number == expected)
            );
        }
        // New arrivals cannot postpone the selected frame's matching Submit.
        sender.send(prepare(4)).unwrap();
        sender.send(submit(4)).unwrap();
        assert!(
            matches!(ReceiveDisplayMessage(&receiver, &mut pending).unwrap(),
            DisplayMessage::Command(display::Message::Submit { frame, .. }) if frame.sequence_number == 3)
        );
        assert!(matches!(
            ReceiveDisplayMessage(&receiver, &mut pending).unwrap(),
            DisplayMessage::SwapAckReady
        ));
        assert!(
            matches!(ReceiveDisplayMessage(&receiver, &mut pending).unwrap(),
            DisplayMessage::Command(display::Message::Prepare { frame, .. }) if frame.sequence_number == 4)
        );
        assert!(
            matches!(ReceiveDisplayMessage(&receiver, &mut pending).unwrap(),
            DisplayMessage::Command(display::Message::Submit { frame, .. }) if frame.sequence_number == 4)
        );
        sender.send(prepare(5)).unwrap();
        sender.send(submit(5)).unwrap();
        sender.send(prepare(6)).unwrap();
        sender
            .send(DisplayMessage::Command(display::Message::Install(
                compositor::RasterBundle::default(),
            )))
            .unwrap();
        sender
            .send(DisplayMessage::Command(display::Message::Stop))
            .unwrap();
        for expected in [5, 6] {
            assert!(
                matches!(ReceiveDisplayMessage(&receiver, &mut pending).unwrap(),
                DisplayMessage::Command(display::Message::Prepare { frame, .. }) if frame.sequence_number == expected)
            );
        }
        assert!(matches!(
            ReceiveDisplayMessage(&receiver, &mut pending).unwrap(),
            DisplayMessage::Command(display::Message::Install(_))
        ));
        assert!(matches!(
            ReceiveDisplayMessage(&receiver, &mut pending).unwrap(),
            DisplayMessage::Command(display::Message::Stop)
        ));
        // Surface resources stay with the accepted frame across an Install
        // barrier. Returning them before its successor is accepted can erase
        // every undrawn candidate when main commits arrive each vsync.
        let mut display = Display::New(
            DisplayTarget::Offscreen,
            Output {
                mailbox: Arc::new(std::sync::Mutex::new(None)),
                notify: Arc::new(|_| {}),
            },
            false,
        );
        let ack: Arc<dyn Fn(u64) + Send + Sync> = Arc::new(|_| {});
        for sequence in 1..=2 {
            let installed = display
                .Handle(
                    display::Message::Install(compositor::RasterBundle::default()),
                    &ack,
                )
                .unwrap();
            assert!(!installed
                .iter()
                .any(|effect| matches!(effect, display::Effect::ReturnBundle(_))));
            let DisplayMessage::Command(message) = prepare(sequence) else {
                unreachable!()
            };
            let accepted = display.Handle(message, &ack).unwrap();
            assert_eq!(
                accepted
                    .iter()
                    .filter(|effect| matches!(effect, display::Effect::ReturnBundle(_)))
                    .count(),
                usize::from(sequence > 1)
            );
        }
    }
}
