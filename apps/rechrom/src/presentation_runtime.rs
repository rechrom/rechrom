//! Physical execution and message routing for compositor, raster and display.
//!
//! Core modules never create a thread or inspect their executor. This runtime
//! can place Display on its own presentation thread or co-locate it with the
//! Compositor while retaining the same mailbox boundary.
use crate::{
    compositor::{self, Compositor},
    display::{self, Display},
    engine::{Output, UserEvent},
    window_surface::WindowTarget,
};
use foundation::begin_frame::BeginFrameSource;
use std::{
    collections::VecDeque,
    io,
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplayExecutor {
    #[default]
    DedicatedThread,
    CompositorThread,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Config {
    pub display_executor: DisplayExecutor,
}

#[derive(Clone, Copy)]
struct PendingCycle {
    frame: crate::begin_frame_source::NativeBeginFrame,
    prepared: bool,
}

enum LocalMessage {
    Compositor(compositor::Command),
    Display(display::Message),
}

pub(crate) fn Spawn(
    config: Config,
    target: WindowTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
) -> io::Result<mpsc::Sender<compositor::Command>> {
    let (compositor_sender, compositor_receiver) = mpsc::channel();
    let raster_sender = SpawnRasterOwner(compositor_sender.clone())?;
    match config.display_executor {
        DisplayExecutor::DedicatedThread => SpawnDedicated(
            target,
            output,
            frame_source,
            compositor_sender.clone(),
            compositor_receiver,
            raster_sender,
        )?,
        DisplayExecutor::CompositorThread => SpawnShared(
            target,
            output,
            frame_source,
            compositor_receiver,
            raster_sender,
        )?,
    }
    Ok(compositor_sender)
}

fn SpawnRasterOwner(
    compositor_sender: mpsc::Sender<compositor::Command>,
) -> io::Result<mpsc::Sender<compositor::RasterJob>> {
    let (raster_sender, raster_receiver) = mpsc::channel::<compositor::RasterJob>();
    std::thread::Builder::new()
        .name("browser-raster-owner".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            while let Ok(mut job) = raster_receiver.recv() {
                let result = compositor::PlanAndRaster(
                    &mut job.bundle,
                    job.snapshot,
                    job.activation_target,
                    job.prepaint_target,
                );
                if compositor_sender
                    .send(compositor::Command::RasterReady(compositor::RasterReady {
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
    compositor_sender: mpsc::Sender<compositor::Command>,
    compositor_receiver: mpsc::Receiver<compositor::Command>,
    raster_sender: mpsc::Sender<compositor::RasterJob>,
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
            let mut display = Display::New(target, display_output.notify.clone());
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
                        (display_output.notify)(UserEvent::Fatal(error.to_string()));
                        break;
                    }
                };
                for effect in effects {
                    match effect {
                        display::Effect::ReturnBundle(bundle) => {
                            if display_compositor
                                .send(compositor::Command::SpareBundle(bundle))
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
            ) {
                (output.notify)(UserEvent::Fatal(error.to_string()));
            }
        })?;
    Ok(())
}

fn RunDedicatedCompositor(
    receiver: mpsc::Receiver<compositor::Command>,
    raster_sender: mpsc::Sender<compositor::RasterJob>,
    display_sender: mpsc::Sender<display::Message>,
    frame_source: Arc<dyn BeginFrameSource>,
) -> io::Result<()> {
    let mut compositor = Compositor::default();
    let mut pending_cycle: Option<PendingCycle> = None;
    loop {
        let timeout = PendingTimeout(pending_cycle);
        match receiver.recv_timeout(timeout) {
            Ok(compositor::Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = display_sender.send(display::Message::Stop);
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                AdvanceCycle(
                    &mut compositor,
                    &mut pending_cycle,
                    &raster_sender,
                    &display_sender,
                    &frame_source,
                )?;
            }
            Ok(compositor::Command::BeginFrame(frame)) => {
                if let Some(previous) = pending_cycle.replace(PendingCycle {
                    frame,
                    prepared: false,
                }) {
                    if !previous.prepared {
                        RouteDedicated(
                            compositor.Prepare(previous.frame),
                            &raster_sender,
                            &display_sender,
                            &frame_source,
                        )?;
                    }
                    RouteDedicated(
                        compositor.Draw(previous.frame),
                        &raster_sender,
                        &display_sender,
                        &frame_source,
                    )?;
                }
            }
            Ok(message) => RouteDedicated(
                compositor.Handle(message)?,
                &raster_sender,
                &display_sender,
                &frame_source,
            )?,
        }
        AdvanceCycle(
            &mut compositor,
            &mut pending_cycle,
            &raster_sender,
            &display_sender,
            &frame_source,
        )?;
    }
}

fn AdvanceCycle(
    compositor: &mut Compositor,
    pending_cycle: &mut Option<PendingCycle>,
    raster_sender: &mpsc::Sender<compositor::RasterJob>,
    display_sender: &mpsc::Sender<display::Message>,
    frame_source: &Arc<dyn BeginFrameSource>,
) -> io::Result<()> {
    if let Some(cycle) = pending_cycle {
        if !cycle.prepared && Instant::now() >= CompositorPrepareDeadline(cycle.frame) {
            RouteDedicated(
                compositor.Prepare(cycle.frame),
                raster_sender,
                display_sender,
                frame_source,
            )?;
            cycle.prepared = true;
        }
    }
    if pending_cycle.is_some_and(|cycle| Instant::now() >= DisplayDrawDeadline(cycle.frame)) {
        RouteDedicated(
            compositor.Draw(pending_cycle.take().unwrap().frame),
            raster_sender,
            display_sender,
            frame_source,
        )?;
    }
    Ok(())
}

fn RouteDedicated(
    effects: Vec<compositor::Effect>,
    raster_sender: &mpsc::Sender<compositor::RasterJob>,
    display_sender: &mpsc::Sender<display::Message>,
    frame_source: &Arc<dyn BeginFrameSource>,
) -> io::Result<()> {
    for effect in effects {
        match effect {
            compositor::Effect::Raster(job) => raster_sender
                .send(job)
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "raster owner stopped"))?,
            compositor::Effect::Display(message) => display_sender
                .send(message)
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "display stopped"))?,
            compositor::Effect::RequestBeginFrame => frame_source.request_begin_frame(),
        }
    }
    Ok(())
}

fn SpawnShared(
    target: WindowTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    receiver: mpsc::Receiver<compositor::Command>,
    raster_sender: mpsc::Sender<compositor::RasterJob>,
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
            ) {
                (output.notify)(UserEvent::Fatal(error.to_string()));
            }
        })?;
    Ok(())
}

fn RunShared(
    target: WindowTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    receiver: mpsc::Receiver<compositor::Command>,
    raster_sender: mpsc::Sender<compositor::RasterJob>,
) -> io::Result<()> {
    let mut compositor = Compositor::default();
    let mut display = Display::New(target, output.notify.clone());
    let mut pending_cycle: Option<PendingCycle> = None;
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
            match receiver.recv_timeout(PendingTimeout(pending_cycle)) {
                Ok(compositor::Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    let _ = display.Handle(display::Message::Stop, &post_swap_ack)?;
                    return Ok(());
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Ok(message) => local.push_back(LocalMessage::Compositor(message)),
            }
        }
        while let Some(message) = local.pop_front() {
            match message {
                LocalMessage::Compositor(compositor::Command::BeginFrame(frame)) => {
                    if let Some(previous) = pending_cycle.replace(PendingCycle {
                        frame,
                        prepared: false,
                    }) {
                        if !previous.prepared {
                            RouteSharedCompositor(
                                compositor.Prepare(previous.frame),
                                &mut local,
                                &raster_sender,
                                &frame_source,
                            )?;
                        }
                        RouteSharedCompositor(
                            compositor.Draw(previous.frame),
                            &mut local,
                            &raster_sender,
                            &frame_source,
                        )?;
                    }
                }
                LocalMessage::Compositor(compositor::Command::Stop) => {
                    let _ = display.Handle(display::Message::Stop, &post_swap_ack)?;
                    return Ok(());
                }
                LocalMessage::Compositor(message) => RouteSharedCompositor(
                    compositor.Handle(message)?,
                    &mut local,
                    &raster_sender,
                    &frame_source,
                )?,
                LocalMessage::Display(message) => {
                    for effect in display.Handle(message, &post_swap_ack)? {
                        match effect {
                            display::Effect::ReturnBundle(bundle) => local.push_back(
                                LocalMessage::Compositor(compositor::Command::SpareBundle(bundle)),
                            ),
                            display::Effect::RequestBeginFrame => {
                                frame_source.request_begin_frame()
                            }
                        }
                    }
                }
            }
        }
        if let Some(cycle) = &mut pending_cycle {
            if !cycle.prepared && Instant::now() >= CompositorPrepareDeadline(cycle.frame) {
                RouteSharedCompositor(
                    compositor.Prepare(cycle.frame),
                    &mut local,
                    &raster_sender,
                    &frame_source,
                )?;
                cycle.prepared = true;
            }
        }
        if pending_cycle.is_some_and(|cycle| Instant::now() >= DisplayDrawDeadline(cycle.frame)) {
            RouteSharedCompositor(
                compositor.Draw(pending_cycle.take().unwrap().frame),
                &mut local,
                &raster_sender,
                &frame_source,
            )?;
        }
    }
}

fn RouteSharedCompositor(
    effects: Vec<compositor::Effect>,
    local: &mut VecDeque<LocalMessage>,
    raster_sender: &mpsc::Sender<compositor::RasterJob>,
    frame_source: &Arc<dyn BeginFrameSource>,
) -> io::Result<()> {
    for effect in effects {
        match effect {
            compositor::Effect::Raster(job) => raster_sender
                .send(job)
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "raster owner stopped"))?,
            compositor::Effect::Display(message) => local.push_back(LocalMessage::Display(message)),
            compositor::Effect::RequestBeginFrame => frame_source.request_begin_frame(),
        }
    }
    Ok(())
}

fn PendingTimeout(cycle: Option<PendingCycle>) -> Duration {
    cycle.map_or(Duration::from_millis(50), |cycle| {
        let deadline = if cycle.prepared {
            DisplayDrawDeadline(cycle.frame)
        } else {
            CompositorPrepareDeadline(cycle.frame)
        };
        deadline.saturating_duration_since(Instant::now())
    })
}

pub(crate) fn CompositorPrepareDeadline(
    frame: crate::begin_frame_source::NativeBeginFrame,
) -> Instant {
    frame.frame_time + frame.interval / 3
}

pub(crate) fn DisplayDrawDeadline(frame: crate::begin_frame_source::NativeBeginFrame) -> Instant {
    frame
        .deadline
        .checked_sub(frame.interval / 3)
        .unwrap_or(frame.frame_time)
        .max(frame.frame_time)
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
