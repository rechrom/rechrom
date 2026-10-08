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
    window_surface::WindowTarget,
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

pub(crate) type BeginMainFrameClient =
    Arc<dyn Fn(crate::begin_frame_source::NativeBeginFrame) + Send + Sync>;

pub(crate) fn Spawn(
    config: Config,
    target: WindowTarget,
    output: Output,
    input_frame_source: Arc<crate::begin_frame_source::LogicalBeginFrameSource>,
    display_frame_source: Arc<crate::begin_frame_source::LogicalBeginFrameSource>,
    begin_main_frame: BeginMainFrameClient,
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
            begin_main_frame,
        )?,
        DisplayExecutor::CompositorThread => SpawnShared(
            target,
            output,
            frame_source,
            compositor_receiver,
            raster_sender,
            config.capture_frames,
            begin_main_frame,
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
    target: WindowTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    compositor_sender: mpsc::Sender<CompositorCommand>,
    compositor_receiver: mpsc::Receiver<CompositorCommand>,
    raster_sender: mpsc::Sender<RasterJob>,
    capture_frames: bool,
    begin_main_frame: BeginMainFrameClient,
) -> io::Result<()> {
    let (display_sender, display_receiver) = mpsc::channel::<display::Message>();
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
                let _ = display_reply.send(display::Message::SwapAck(sequence));
            });
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            while let Ok(message) = display_receiver.recv() {
                if matches!(message, display::Message::Stop) {
                    break;
                }
                let effects = match display.Handle(message, &post_swap_ack) {
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
                begin_main_frame,
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
    display_sender: mpsc::Sender<display::Message>,
    frame_source: Arc<dyn BeginFrameSource>,
    begin_main_frame: BeginMainFrameClient,
) -> io::Result<()> {
    let mut compositor = CompositorEngine::default();
    let mut scheduler = viz::DisplayScheduler::default();
    loop {
        let timeout = PendingTimeout(&scheduler);
        match receiver.recv_timeout(timeout) {
            Ok(CompositorCommand::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = display_sender.send(display::Message::Stop);
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                AdvanceCycle(
                    &mut compositor,
                    &mut scheduler,
                    &raster_sender,
                    &display_sender,
                    &frame_source,
                    &begin_main_frame,
                )?;
            }
            Ok(CompositorCommand::DisplayBeginFrame(frame)) => {
                if let Some(previous) = scheduler.BeginFrame(frame, FrameTiming(frame)) {
                    if !previous.IsPrepared() {
                        RouteDedicated(
                            compositor.Prepare(previous.Payload())?,
                            &raster_sender,
                            &display_sender,
                            &frame_source,
                            &begin_main_frame,
                        )?;
                    }
                    RouteDedicated(
                        compositor.Draw(previous.Payload()),
                        &raster_sender,
                        &display_sender,
                        &frame_source,
                        &begin_main_frame,
                    )?;
                }
            }
            Ok(CompositorCommand::BeginFrame(frame)) => RouteDedicated(
                compositor.Handle(CompositorCommand::BeginFrame(frame))?,
                &raster_sender,
                &display_sender,
                &frame_source,
                &begin_main_frame,
            )?,
            Ok(message) => RouteDedicated(
                compositor.Handle(message)?,
                &raster_sender,
                &display_sender,
                &frame_source,
                &begin_main_frame,
            )?,
        }
        AdvanceCycle(
            &mut compositor,
            &mut scheduler,
            &raster_sender,
            &display_sender,
            &frame_source,
            &begin_main_frame,
        )?;
    }
}

fn AdvanceCycle(
    compositor: &mut CompositorEngine,
    scheduler: &mut viz::DisplayScheduler<crate::begin_frame_source::NativeBeginFrame>,
    raster_sender: &mpsc::Sender<RasterJob>,
    display_sender: &mpsc::Sender<display::Message>,
    frame_source: &Arc<dyn BeginFrameSource>,
    begin_main_frame: &BeginMainFrameClient,
) -> io::Result<()> {
    let now = Instant::now();
    if scheduler.NeedsPrepare(now) {
        let frame = scheduler
            .PendingPayload()
            .expect("Viz scheduler reported pending prepare without a frame");
        RouteDedicated(
            compositor.Prepare(frame)?,
            raster_sender,
            display_sender,
            frame_source,
            begin_main_frame,
        )?;
        scheduler.MarkPrepared();
    }
    if let Some(frame) = scheduler.TakeIfDrawDue(now) {
        RouteDedicated(
            compositor.Draw(frame),
            raster_sender,
            display_sender,
            frame_source,
            begin_main_frame,
        )?;
    }
    Ok(())
}

fn RouteDedicated(
    effects: Vec<CompositorEffect>,
    raster_sender: &mpsc::Sender<RasterJob>,
    display_sender: &mpsc::Sender<display::Message>,
    frame_source: &Arc<dyn BeginFrameSource>,
    begin_main_frame: &BeginMainFrameClient,
) -> io::Result<()> {
    for effect in effects {
        match effect {
            compositor::Effect::Raster(job) => raster_sender
                .send(job)
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "raster owner stopped"))?,
            compositor::Effect::InstallRasterBundle(bundle) => display_sender
                .send(display::Message::Install(bundle))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "display stopped"))?,
            compositor::Effect::FrameProduced {
                frame,
                submitted,
                scroll_active,
            } => display_sender
                .send(display::Message::Prepare {
                    frame,
                    submitted,
                    scroll_active,
                })
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "display stopped"))?,
            compositor::Effect::Submit {
                frame,
                scroll_active,
            } => display_sender
                .send(display::Message::Submit {
                    frame,
                    scroll_active,
                })
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "display stopped"))?,
            compositor::Effect::BeginMainFrame(frame) => begin_main_frame(frame),
            compositor::Effect::RequestBeginFrame => frame_source.request_begin_frame(),
        }
    }
    Ok(())
}

fn SpawnShared(
    target: WindowTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    receiver: mpsc::Receiver<CompositorCommand>,
    raster_sender: mpsc::Sender<RasterJob>,
    capture_frames: bool,
    begin_main_frame: BeginMainFrameClient,
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
                begin_main_frame,
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
    target: WindowTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    receiver: mpsc::Receiver<CompositorCommand>,
    raster_sender: mpsc::Sender<RasterJob>,
    capture_frames: bool,
    begin_main_frame: BeginMainFrameClient,
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
        while let Ok(sequence) = ack_receiver.try_recv() {
            local.push_back(LocalMessage::Display(display::Message::SwapAck(sequence)));
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
            match message {
                LocalMessage::Compositor(CompositorCommand::DisplayBeginFrame(frame)) => {
                    if let Some(previous) = scheduler.BeginFrame(frame, FrameTiming(frame)) {
                        if !previous.IsPrepared() {
                            RouteSharedCompositor(
                                compositor.Prepare(previous.Payload())?,
                                &mut local,
                                &raster_sender,
                                &frame_source,
                                &begin_main_frame,
                            )?;
                        }
                        RouteSharedCompositor(
                            compositor.Draw(previous.Payload()),
                            &mut local,
                            &raster_sender,
                            &frame_source,
                            &begin_main_frame,
                        )?;
                    }
                }
                LocalMessage::Compositor(CompositorCommand::BeginFrame(frame)) => {
                    RouteSharedCompositor(
                        compositor.Handle(CompositorCommand::BeginFrame(frame))?,
                        &mut local,
                        &raster_sender,
                        &frame_source,
                        &begin_main_frame,
                    )?;
                }
                LocalMessage::Compositor(CompositorCommand::Stop) => {
                    let _ = display.Handle(display::Message::Stop, &post_swap_ack)?;
                    return Ok(());
                }
                LocalMessage::Compositor(message) => RouteSharedCompositor(
                    compositor.Handle(message)?,
                    &mut local,
                    &raster_sender,
                    &frame_source,
                    &begin_main_frame,
                )?,
                LocalMessage::Display(message) => {
                    for effect in display.Handle(message, &post_swap_ack)? {
                        match effect {
                            display::Effect::ReturnBundle(bundle) => local.push_back(
                                LocalMessage::Compositor(CompositorCommand::SpareBundle(bundle)),
                            ),
                            display::Effect::RequestBeginFrame => {
                                frame_source.request_begin_frame()
                            }
                        }
                    }
                }
            }
        }
        let now = Instant::now();
        if scheduler.NeedsPrepare(now) {
            let frame = scheduler
                .PendingPayload()
                .expect("Viz scheduler reported pending prepare without a frame");
            RouteSharedCompositor(
                compositor.Prepare(frame)?,
                &mut local,
                &raster_sender,
                &frame_source,
                &begin_main_frame,
            )?;
            scheduler.MarkPrepared();
        }
        if let Some(frame) = scheduler.TakeIfDrawDue(now) {
            RouteSharedCompositor(
                compositor.Draw(frame),
                &mut local,
                &raster_sender,
                &frame_source,
                &begin_main_frame,
            )?;
        }
    }
}

fn RouteSharedCompositor(
    effects: Vec<CompositorEffect>,
    local: &mut VecDeque<LocalMessage>,
    raster_sender: &mpsc::Sender<RasterJob>,
    frame_source: &Arc<dyn BeginFrameSource>,
    begin_main_frame: &BeginMainFrameClient,
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
            compositor::Effect::BeginMainFrame(frame) => begin_main_frame(frame),
            compositor::Effect::RequestBeginFrame => frame_source.request_begin_frame(),
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
