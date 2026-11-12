//! Window presentation receives placement and already rendered Page surfaces.
//! No document, paint artifact, Page compositor, or raster worker lives here.
use crate::{
    begin_frame_source::{BeginFrame, LogicalBeginFrameSource},
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

/// Window-owned availability of the surfaces embedded by the current Scene.
/// Desktop DelegatedFrameHost::EmbedSurface uses a zero-frame resize deadline:
/// an existing fallback surface is drawable even at the preceding size. Page
/// producers remain independent; geometry mismatch must not make Display wait
/// for their next frames. Browser-owned tab state still uses its revision fence.
#[derive(Default)]
struct SurfaceReadiness {
    scene: Option<crate::page_presentation::Scene>,
    viewports: BTreeMap<crate::page_presentation::Document, crate::engine::Viewport>,
    local_surface_ids:
        BTreeMap<crate::page_presentation::Document, compositor::runtime::types::LocalSurfaceId>,
    embedder_revisions: BTreeMap<crate::page_presentation::Document, u64>,
    ready_damage: bool,
    interaction_damage: bool,
}

impl SurfaceReadiness {
    fn affects_current_scene(&self, document: crate::page_presentation::Document) -> bool {
        self.scene.as_ref().is_some_and(|scene| {
            scene
                .placements
                .iter()
                // The document generation may advance during a same-tab
                // navigation. It still replaces the Surface embedded by this
                // tab view. A different tab id is unreachable from this root.
                .any(|placement| placement.document.0 == document.0)
        })
    }

    fn observe_page(&mut self, interaction: bool) -> bool {
        let all_ready = self.all_placed_surfaces_available();
        if all_ready {
            self.ready_damage = true;
            self.interaction_damage |= interaction;
        }
        all_ready
    }

    fn observe(&mut self, message: &display::Message) -> Option<bool> {
        match message {
            display::Message::Scene(scene) => {
                self.scene = Some(scene.clone());
                // A tab switch normally selects an already submitted Surface.
                // Treat that retained surface as ready immediately, including
                // during resize. The new child surface replaces its fallback
                // independently when it arrives.
                let all_ready = self.all_placed_surfaces_available();
                self.ready_damage = all_ready;
                // Scene geometry is browser state, not user interaction.
                // Do not let a stale content fallback select the interaction
                // deadline before the exact toolbar/root surface arrives.
                self.interaction_damage = false;
                Some(all_ready)
            }
            display::Message::Prepare { submitted } => {
                let document = submitted.provenance.document;
                let local_surface_id = submitted.frame.local_surface_id;
                let Some(parameters) = submitted
                    .frame
                    .frame
                    .render_pass_list
                    .last()
                    .map(|pass| (pass.output_rect.width, pass.output_rect.height))
                else {
                    return None;
                };
                if let Some(current) = self.local_surface_ids.get(&document) {
                    let same_embed = current.embed_token == local_surface_id.embed_token;
                    if same_embed && !local_surface_id.IsSameOrNewerThan(current) {
                        return None;
                    }
                    if *current == local_surface_id
                        && self.viewports.get(&document).is_some_and(|viewport| {
                            parameters != (viewport.width as i32, viewport.height as i32)
                        })
                    {
                        return None;
                    }
                }
                self.local_surface_ids.insert(document, local_surface_id);
                self.viewports.insert(document, submitted.viewport);
                self.embedder_revisions.insert(
                    document,
                    submitted
                        .provenance
                        .content
                        .map_or(0, |content| content.embedder_revision),
                );
                self.affects_current_scene(document)
                    .then(|| self.observe_page(submitted.is_handling_interaction))
            }
            display::Message::CopyOutput {
                expected_viewport, ..
            } => {
                // DirectRenderer copy requests count as display damage. Keep
                // the request behind the exact root Scene so a resize followed
                // immediately by capture cannot schedule the preceding size.
                let all_ready = self
                    .scene
                    .as_ref()
                    .is_some_and(|scene| scene.viewport == *expected_viewport)
                    && self.all_placed_surfaces_available();
                if all_ready {
                    self.ready_damage = true;
                }
                Some(all_ready)
            }
            display::Message::Submit { .. }
            | display::Message::SwapAck(_)
            | display::Message::Stop => None,
        }
    }

    fn all_placed_surfaces_available(&self) -> bool {
        self.scene.as_ref().is_some_and(|scene| {
            !scene.placements.is_empty()
                && scene.placements.iter().all(|placement| {
                    let selected = self.viewports.get(&placement.document).or_else(|| {
                        // DelegatedFrameHost keeps the previous document's
                        // Surface in the same tab view until the new
                        // document submits its first frame.
                        self.viewports
                            .iter()
                            .rev()
                            .find_map(|(document, viewport)| {
                                (document.0 == placement.document.0).then_some(viewport)
                            })
                    });
                    let surface_available = selected.is_some()
                        || (selected.is_none()
                            // The browser chrome must itself have a frame before
                            // a root can be presented. A newly selected content
                            // view is different: before its first frame Chromium
                            // draws the root with that child absent, exposing the
                            // view background instead of retaining the old tab.
                            && placement.document.0 != 0
                            && placement.document.0 != u64::MAX);
                    let revision_ready = placement.document.0 != 0
                        || self
                            .embedder_revisions
                            .get(&placement.document)
                            .copied()
                            .unwrap_or(0)
                            == scene.toolbar_revision;
                    // BrowserView adopts new root bounds synchronously. Until
                    // our Page-backed toolbar submits that size, its existing
                    // Surface is the same zero-frame-deadline fallback used by
                    // WebContents: retain its source dimensions and clip it,
                    // never scale it. The browser-owned revision fence above
                    // still prevents a tab-state mismatch.
                    surface_available && revision_ready
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
    Tick(BeginFrame),
    SwapStarted,
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
            crate::engine::set_presentation_thread_priority();
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
    post_swap_started: &Arc<dyn Fn() + Send + Sync>,
    post_ack: &Arc<dyn Fn(u64) + Send + Sync>,
) -> io::Result<Vec<display::Effect>> {
    // Completion tokens have their own queue and are consumed by the Display
    // owner before any queued draw, exactly as in the previous executor.
    let mut effects = Vec::new();
    while let Ok(id) = acks.try_recv() {
        effects.extend(display.Handle(
            display::Message::SwapAck(id),
            post_swap_started,
            post_ack,
        )?);
    }
    if let DisplayWork::Command(message) = work {
        effects.extend(display.Handle(message, post_swap_started, post_ack)?);
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
    let swap_started_reply = reply.clone();
    let post_swap_started: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        let _ = swap_started_reply.send(Command::SwapStarted);
    });
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
        let post_swap_started = post_swap_started.clone();
        let post_ack = post_ack.clone();
        std::thread::Builder::new()
            .name("window-display".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                crate::engine::set_presentation_thread_priority();
                let mut display = Display::New(target, output.clone(), config.capture_frames);
                let mut pending = VecDeque::new();
                while let Ok(work) = ReceiveDisplayWork(&receiver, &mut pending) {
                    if matches!(work, DisplayWork::Command(display::Message::Stop)) {
                        break;
                    }
                    let result = apply_display_work(
                        &mut display,
                        &ack_receiver,
                        work,
                        &post_swap_started,
                        &post_ack,
                    );
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
            for _ in apply_display_work(display, acks, work, &post_swap_started, &post_ack)? {
                source.request_begin_frame();
            }
        }
        Ok(())
    };
    // Chromium keeps kAllowMultipleSwapsPerVsync feature-gated. Rechrom opts
    // in explicitly at this application assembly point because Page and
    // Display share physical BeginFrameIds and the typed swap ACK path below.
    let mut scheduler = viz::DisplayScheduler::new(viz::PendingSwapParams::default(), false, true);
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
                let damage_frame_id = match &message {
                    display::Message::Prepare { submitted } => {
                        let frame_id = submitted.frame.frame.metadata.begin_frame_ack.frame_id;
                        viz::BeginFrameId {
                            source_id: frame_id.source_id,
                            sequence_number: frame_id.sequence_number,
                        }
                    }
                    _ => viz::BeginFrameId::default(),
                };
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
                    // A previous-size fallback satisfies the desktop resize
                    // deadline. Only missing required surfaces or a different
                    // browser-owned tab revision keep this root pending.
                    if all_surfaces_ready && surfaces.has_interaction_damage() {
                        scheduler.SetDamageDueToInteraction(true);
                    }
                    scheduler.OnPendingSurfacesChanged(!all_surfaces_ready);
                    if let Some(frame) = scheduler.OnDisplayDamaged(damage_frame_id, Instant::now())
                    {
                        surfaces.did_schedule_draw();
                        dispatch(DisplayWork::Command(display::Message::Submit { frame }))?;
                    }
                }
                if all_surfaces_ready.is_some() {
                    source.request_begin_frame();
                }
            }
            Ok(Command::Tick(frame)) => {
                if let Some(previous) = scheduler.BeginFrame(
                    frame,
                    viz::FrameTiming {
                        frame_id: viz::BeginFrameId {
                            source_id: frame.source_id,
                            sequence_number: frame.sequence_number,
                        },
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
            Ok(Command::SwapStarted) => {
                scheduler.DidSwapBuffers();
            }
            Ok(Command::Ack(_id)) => {
                scheduler.DidReceiveSwapBuffersAck();
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
        page_presentation::{DeviceRect, Placement, Scene},
    };

    #[test]
    fn resize_damage_uses_available_fallback_surfaces() {
        let toolbar = (0, 0);
        let content = (1, 1);
        let mut state = SurfaceReadiness::default();
        state.observe(&display::Message::Scene(Scene {
            viewport: Viewport {
                width: 1000,
                height: 700,
                scale: 1.0,
            },
            browser_root: crate::chrome::browser_root_frame(Viewport {
                width: 1000,
                height: 700,
                scale: 1.0,
            }),
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
            toolbar_revision: 0,
        }));

        state.viewports.insert(
            toolbar,
            Viewport {
                width: 1000,
                height: 86,
                scale: 1.0,
            },
        );
        assert!(state.observe_page(true));
        assert!(state.has_ready_damage());
        assert!(state.has_interaction_damage());
        state.viewports.insert(
            content,
            Viewport {
                width: 900,
                height: 614,
                scale: 1.0,
            },
        );
        assert!(state.observe_page(false));
        assert!(state.has_ready_damage());
        state.viewports.insert(
            content,
            Viewport {
                width: 1000,
                height: 614,
                scale: 1.0,
            },
        );
        assert!(state.observe_page(false));
        assert!(state.has_ready_damage());
        assert!(state.has_interaction_damage());

        state.did_schedule_draw();
        assert!(!state.has_ready_damage());
        assert!(!state.has_interaction_damage());

        // A toolbar Page may finish the next browser mutation before the host
        // Scene message reaches Display. The newer toolbar must wait for its
        // matching root commit instead of appearing over the previous tab.
        state.embedder_revisions.insert(toolbar, 1);
        assert!(!state.observe_page(false));
        assert!(!state.has_ready_damage());
        let mut next_toolbar_scene = state.scene.clone().unwrap();
        next_toolbar_scene.toolbar_revision = 1;
        assert_eq!(
            state.observe(&display::Message::Scene(next_toolbar_scene)),
            Some(true)
        );
        state.did_schedule_draw();

        let next_content = (1, 2);
        assert_eq!(
            state.observe(&display::Message::Scene(Scene {
                viewport: Viewport {
                    width: 1000,
                    height: 700,
                    scale: 1.0,
                },
                browser_root: crate::chrome::browser_root_frame(Viewport {
                    width: 1000,
                    height: 700,
                    scale: 1.0,
                }),
                placements: vec![
                    Placement {
                        document: toolbar,
                        destination: DeviceRect::new(0, 0, 1000, 86),
                    },
                    Placement {
                        document: next_content,
                        destination: DeviceRect::new(0, 86, 1000, 614),
                    },
                ],
                live_documents: vec![toolbar, next_content],
                metadata: Vec::new(),
                toolbar_revision: 1,
            })),
            Some(true),
            "the previous document surface keeps a same-tab navigation drawable"
        );
        assert!(state.has_ready_damage());
        assert!(!state.has_interaction_damage());
    }

    #[test]
    fn resized_root_uses_stale_toolbar_without_latching_interaction() {
        let toolbar = (0, 0);
        let content = (1, 1);
        let mut state = SurfaceReadiness::default();
        state.viewports.insert(
            toolbar,
            Viewport {
                width: 1000,
                height: 86,
                scale: 1.0,
            },
        );
        state.viewports.insert(
            content,
            Viewport {
                width: 1000,
                height: 614,
                scale: 1.0,
            },
        );
        let resized = Scene {
            viewport: Viewport {
                width: 1100,
                height: 700,
                scale: 1.0,
            },
            browser_root: crate::chrome::browser_root_frame(Viewport {
                width: 1100,
                height: 700,
                scale: 1.0,
            }),
            placements: vec![
                Placement {
                    document: toolbar,
                    destination: DeviceRect::new(0, 0, 1100, 86),
                },
                Placement {
                    document: content,
                    destination: DeviceRect::new(0, 86, 1100, 614),
                },
            ],
            live_documents: vec![toolbar, content],
            metadata: Vec::new(),
            toolbar_revision: 0,
        };

        assert_eq!(state.observe(&display::Message::Scene(resized)), Some(true));
        assert!(state.has_ready_damage());
        assert!(!state.has_interaction_damage());

        // The retained toolbar and WebContents surfaces make the resized root
        // drawable immediately. A later Page frame can use the interaction
        // deadline without waiting for an exact toolbar size.
        assert!(state.observe_page(true));
        assert!(state.has_ready_damage());
        assert!(state.has_interaction_damage());

        state.viewports.insert(
            toolbar,
            Viewport {
                width: 1100,
                height: 86,
                scale: 1.0,
            },
        );
        assert!(state.observe_page(false));
        assert!(state.has_ready_damage());
        assert!(state.has_interaction_damage());
    }
}
