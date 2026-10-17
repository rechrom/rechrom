//! Test-only synchronous coordinator fixtures.
use super::*;
enum LocalMessage {
    Compositor(CompositorCommand),
    Display(display::Message),
}

use crate::window_runtime::{
    DisplayWork as DisplayMessage, ReceiveDisplayWork as ReceiveDisplayMessage,
};

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
    let compositor_reply = compositor_sender.clone();
    match config.display_executor {
        DisplayExecutor::DedicatedThread => SpawnDedicated(
            target,
            output,
            frame_source,
            compositor_receiver,
            compositor_reply,
            config.capture_frames,
            main_thread,
        )?,
        DisplayExecutor::CompositorThread => SpawnShared(
            target,
            output,
            frame_source,
            compositor_receiver,
            compositor_reply,
            config.capture_frames,
            main_thread,
        )?,
    }
    Ok(compositor_sender)
}

fn SpawnDedicated(
    target: DisplayTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    compositor_receiver: mpsc::Receiver<CompositorCommand>,
    compositor_reply: mpsc::Sender<CompositorCommand>,
    capture_frames: bool,
    main_thread: MainThreadClient,
) -> io::Result<()> {
    let (display_sender, display_receiver) = mpsc::channel::<DisplayMessage>();
    let (ack_sender, ack_receiver) = mpsc::channel();
    let display_reply = display_sender.clone();
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
                compositor_reply,
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
    compositor_reply: mpsc::Sender<CompositorCommand>,
    display_sender: mpsc::Sender<DisplayMessage>,
    frame_source: Arc<dyn BeginFrameSource>,
    main_thread: MainThreadClient,
) -> io::Result<()> {
    let mut compositor = CompositorEngine::new(compositor_reply.clone());
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
                            &display_sender,
                            &frame_source,
                            &main_thread,
                        )?;
                    }
                    RouteDedicated(
                        &mut scheduler,
                        compositor.Draw(previous.Payload()),
                        &display_sender,
                        &frame_source,
                        &main_thread,
                    )?;
                }
            }
            Ok(CompositorCommand::BeginFrame(frame)) => RouteDedicated(
                &mut scheduler,
                compositor.Handle(CompositorCommand::BeginFrame(frame))?,
                &display_sender,
                &frame_source,
                &main_thread,
            )?,
            Ok(message) => RouteDedicated(
                &mut scheduler,
                compositor.Handle(message)?,
                &display_sender,
                &frame_source,
                &main_thread,
            )?,
        }
        AdvanceCycle(
            &mut compositor,
            &mut scheduler,
            &display_sender,
            &frame_source,
            &main_thread,
        )?;
    }
}

fn AdvanceCycle(
    compositor: &mut CompositorEngine,
    scheduler: &mut viz::DisplayScheduler<crate::begin_frame_source::NativeBeginFrame>,
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
                .any(|effect| matches!(effect, CompositorEffect::FrameProduced { .. })),
        );
        RouteDedicated(
            scheduler,
            effects,
            display_sender,
            frame_source,
            main_thread,
        )?;
    }
    if let Some(frame) = scheduler.TakeIfDrawDue(now) {
        RouteDedicated(
            scheduler,
            compositor.Draw(frame),
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
    display_sender: &mpsc::Sender<DisplayMessage>,
    frame_source: &Arc<dyn BeginFrameSource>,
    main_thread: &MainThreadClient,
) -> io::Result<()> {
    for effect in effects {
        match effect {
            CompositorEffect::Raster { sender, job } => sender
                .send(crate::page_presentation::RasterWork::Raster {
                    job,
                    completion_owner: sender.clone(),
                })
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "raster owner stopped"))?,
            CompositorEffect::FrameProduced {
                frame,
                submitted,
                resources,
                scroll_active,
            } => display_sender
                .send(DisplayMessage::Command(display::Message::Prepare {
                    frame,
                    submitted,
                    resources,
                    scroll_active,
                }))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "display stopped"))?,
            CompositorEffect::Submit {
                frame,
                scroll_active,
            } => display_sender
                .send(DisplayMessage::Command(display::Message::Submit {
                    frame,
                    scroll_active,
                }))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "display stopped"))?,
            CompositorEffect::Scene(scene) => display_sender
                .send(DisplayMessage::Command(display::Message::Scene(scene)))
                .map_err(|_| io::Error::other("display stopped"))?,
            CompositorEffect::MainThread(event) => main_thread(event),
            CompositorEffect::RequestBeginFrame => frame_source.request_begin_frame(),
            CompositorEffect::VisualDamage => scheduler.NotifyVisualDamage(Instant::now()),
        }
    }
    Ok(())
}

fn SpawnShared(
    target: DisplayTarget,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    receiver: mpsc::Receiver<CompositorCommand>,
    compositor_reply: mpsc::Sender<CompositorCommand>,
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
                compositor_reply,
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
    compositor_reply: mpsc::Sender<CompositorCommand>,
    capture_frames: bool,
    main_thread: MainThreadClient,
) -> io::Result<()> {
    let mut compositor = CompositorEngine::new(compositor_reply.clone());
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
                                &frame_source,
                                &main_thread,
                            )?;
                        }
                        RouteSharedCompositor(
                            &mut scheduler,
                            compositor.Draw(previous.Payload()),
                            &mut local,
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
                    .any(|effect| matches!(effect, CompositorEffect::FrameProduced { .. })),
            );
            RouteSharedCompositor(
                &mut scheduler,
                effects,
                &mut local,
                &frame_source,
                &main_thread,
            )?;
        }
        if let Some(frame) = scheduler.TakeIfDrawDue(now) {
            RouteSharedCompositor(
                &mut scheduler,
                compositor.Draw(frame),
                &mut local,
                &frame_source,
                &main_thread,
            )?;
        }
    }
}

fn RouteSharedDisplay(
    effect: display::Effect,
    _local: &mut VecDeque<LocalMessage>,
    frame_source: &Arc<dyn BeginFrameSource>,
) {
    match effect {
        display::Effect::RequestBeginFrame => frame_source.request_begin_frame(),
    }
}

fn RouteSharedCompositor(
    scheduler: &mut viz::DisplayScheduler<crate::begin_frame_source::NativeBeginFrame>,
    effects: Vec<CompositorEffect>,
    local: &mut VecDeque<LocalMessage>,
    frame_source: &Arc<dyn BeginFrameSource>,
    main_thread: &MainThreadClient,
) -> io::Result<()> {
    for effect in effects {
        match effect {
            CompositorEffect::Raster { sender, job } => sender
                .send(crate::page_presentation::RasterWork::Raster {
                    job,
                    completion_owner: sender.clone(),
                })
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "raster owner stopped"))?,
            CompositorEffect::FrameProduced {
                frame,
                submitted,
                resources,
                scroll_active,
            } => local.push_back(LocalMessage::Display(display::Message::Prepare {
                frame,
                submitted,
                resources,
                scroll_active,
            })),
            CompositorEffect::Submit {
                frame,
                scroll_active,
            } => local.push_back(LocalMessage::Display(display::Message::Submit {
                frame,
                scroll_active,
            })),
            CompositorEffect::Scene(scene) => {
                local.push_back(LocalMessage::Display(display::Message::Scene(scene)))
            }
            CompositorEffect::MainThread(event) => main_thread(event),
            CompositorEffect::RequestBeginFrame => frame_source.request_begin_frame(),
            CompositorEffect::VisualDamage => scheduler.NotifyVisualDamage(Instant::now()),
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
    use crate::{begin_frame_source::NativeBeginFrame, engine::Viewport};

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
                    has_transparent_background: false,
                    output_rect: rect,
                    damage_rect: rect,
                    damage_rects: vec![rect],
                    backing_shift: None,
                    shared_quad_state_list: vec![],
                    quad_list: vec![],
                }],
            };
            DisplayMessage::Command(display::Message::Prepare {
                frame: frame(sequence),
                scroll_active: true,
                resources: raster::RasterEngine::default().ExportResources([]),
                submitted: compositor::SubmittedFrame {
                    provenance: compositor::FrameProvenance {
                        document: (1, 1),
                        content: None,
                    },
                    viewport,
                    content: Some(toolbar),
                    metadata: (),
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
            .send(DisplayMessage::Command(display::Message::Scene(
                crate::page_presentation::Scene {
                    viewport: Viewport {
                        width: 2,
                        height: 2,
                        scale: 1.,
                    },
                    placements: vec![],
                    live_documents: vec![],
                    metadata: vec![],
                },
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
            DisplayMessage::Command(display::Message::Scene(_))
        ));
        assert!(matches!(
            ReceiveDisplayMessage(&receiver, &mut pending).unwrap(),
            DisplayMessage::Command(display::Message::Stop)
        ));
        // A window resize must route Page-local viewport sizes, and redraw
        // must retain a presentable scene before another Page commit exists.
        let (reply, _) = mpsc::channel();
        let mut coordinator = CompositorEngine::new(reply);
        let viewport = Viewport {
            width: 800,
            height: 600,
            scale: 1.,
        };
        let documents = [(0, 0), (1, 1)];
        coordinator
            .Handle(CompositorCommand::Snapshot(ArtifactSnapshot {
                scene: crate::page_presentation::Scene {
                    viewport,
                    placements: documents
                        .into_iter()
                        .map(|document| crate::page_presentation::Placement {
                            document,
                            destination: compositor::DeviceRect::new(0, 0, 1, 1),
                        })
                        .collect(),
                    live_documents: documents.to_vec(),
                    metadata: vec![],
                },
                pages: vec![],
            }))
            .unwrap();
        let resized = Viewport {
            width: 1280,
            height: 900,
            scale: 2.,
        };
        let effects = coordinator
            .Handle(CompositorCommand::Resize(resized))
            .unwrap();
        let scene = effects
            .into_iter()
            .find_map(|effect| match effect {
                CompositorEffect::Scene(scene) => Some(scene),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            scene.placements[0].destination,
            compositor::DeviceRect::new(0, 0, 1280, 172)
        );
        assert_eq!(
            scene.placements[1].destination,
            compositor::DeviceRect::new(0, 172, 1280, 728)
        );
        assert!(coordinator.Handle(CompositorCommand::Redraw).unwrap().iter()
            .any(|effect| matches!(effect, CompositorEffect::Scene(scene) if scene.viewport == resized)));
        // A completion queued during Page closure keeps the Page's raster
        // executor alive just long enough to release its own resources.
        let (reply, completed) = mpsc::channel();
        let mut owner = CompositorEngine::new(reply);
        let document = (9, 1);
        let viewport = Viewport {
            width: 16,
            height: 16,
            scale: 1.,
        };
        let effects = owner
            .Handle(CompositorCommand::Snapshot(ArtifactSnapshot {
                scene: crate::page_presentation::Scene {
                    viewport,
                    placements: vec![],
                    live_documents: vec![document],
                    metadata: vec![],
                },
                pages: vec![crate::page_presentation::PageSnapshot {
                    document,
                    viewport,
                    frame_time: Instant::now(),
                    sequence: 1,
                    compositor_scroll_sequence: 0,
                    transparent_background: false,
                    content: compositor::ContentUpdate::Replace(Arc::new(
                        paint::paint_engine::PaintArtifact::default(),
                    )),
                    metadata: (),
                    async_root_scroll: false,
                    blocking_wheel_regions: Default::default(),
                }],
            }))
            .unwrap();
        for effect in effects {
            if let CompositorEffect::Raster { sender, job } = effect {
                sender
                    .send(crate::page_presentation::RasterWork::Raster {
                        job,
                        completion_owner: sender.clone(),
                    })
                    .unwrap_or_else(|_| panic!("Page raster owner stopped early"));
            }
        }
        drop(owner);
        let ready = completed
            .recv_timeout(Duration::from_secs(5))
            .expect("queued Page completion");
        assert!(matches!(ready, CompositorCommand::RasterReady(id, _) if id == document));
        drop(ready);
        assert!(
            matches!(
                completed.recv_timeout(Duration::from_secs(5)),
                Err(mpsc::RecvTimeoutError::Disconnected)
            ),
            "closing a Page releases the executor after its last completion lease"
        );
    }
}
