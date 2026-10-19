//! One independent Page domain. Document tasks and Page rendering execute only
//! on its document owner. Frame delivery merely enqueues rendering opportunities;
//! completed pixel allocations cross into the window presentation domain.
use crate::{
    begin_frame_source::NativeBeginFrame,
    engine::Viewport,
    page_dom::{chrome_hover_chain, element_id, find_id, set_hover_class},
    page_error::{error_kind, panic_error, PageError},
    page_presentation::{ContentFrameProvenance, Document, FrameProvenance, PageFrame},
    window_runtime,
};
use browser::page::{
    CompositorMainThreadEffect, PageCompositorFrame, PageFramePresenter, PageRenderPipeline,
    PageReturnedResource,
};
use browser::page::{Page, PageClient};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use foundation::begin_frame::BeginFrameSource;
use interaction::input_event::*;
use page_mutation::PageMutation;
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    io,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};

pub(crate) type FrameRoutes =
    Arc<Mutex<BTreeMap<Document, (mpsc::Sender<ImplCommand>, Arc<AtomicBool>)>>>;
#[derive(Clone)]
pub(crate) enum Event {
    Status {
        url: String,
        title: String,
        loading: bool,
    },
    Navigate(browser::page::NavigationRequest),
    Input {
        input: InputEvent,
        target: Option<String>,
        focused: Option<String>,
        value: String,
        prevented: bool,
    },
    Feedback {
        cursor: browser::page::Cursor,
        caret: Option<browser::page::CaretRect>,
    },
    DragRegions(Vec<crate::chrome::DragRegion>),
    Error(PageError),
}
#[derive(Clone)]
pub(crate) enum Patch {
    Attribute {
        id: String,
        name: String,
        value: Option<String>,
    },
    Html {
        id: String,
        value: String,
    },
    Value {
        id: String,
        value: String,
    },
    Focus {
        id: String,
        select_all: bool,
    },
}
pub(crate) enum Command {
    Input {
        input: InputEvent,
        queued_at: Instant,
        samples: usize,
    },
    ExternalWheel {
        wheel: WheelEvent,
        queued_at: Instant,
        samples: usize,
        effects: Vec<CompositorMainThreadEffect>,
        main_frame_epoch: Option<u64>,
    },
    CompositorEffects {
        effects: Vec<CompositorMainThreadEffect>,
        main_frame_epoch: Option<u64>,
    },
    Main(MainThreadEvent),
    Resize(Viewport),
    Active(bool),
    Patch(Patch),
    StopLoading,
    Evaluate {
        source: String,
        reply: mpsc::Sender<Result<(), String>>,
    },
    Wake,
    #[cfg(test)]
    TestTask(Box<dyn FnOnce() + Send>),
    Task,
    CollectLayoutHeap,
    CollectJavaScriptHeap,
    Stop,
}
pub(crate) enum ImplCommand {
    Tick(NativeBeginFrame),
    Pump,
    Commit {
        artifact: Arc<paint::PaintArtifact>,
        content: ContentFrameProvenance,
        viewport: Viewport,
    },
    Wheel {
        wheel: WheelEvent,
        queued_at: Instant,
        samples: usize,
    },
    Reclaim(Vec<PageReturnedResource>),
    RendererFailed(String),
    Stop,
}

struct WakingExecutor {
    inner: Arc<dyn executor::Executor>,
    sender: mpsc::Sender<ImplCommand>,
    pending: Arc<AtomicBool>,
}
impl WakingExecutor {
    fn new(
        inner: Arc<dyn executor::Executor>,
        sender: mpsc::Sender<ImplCommand>,
        pending: Arc<AtomicBool>,
    ) -> Self {
        Self {
            inner,
            sender,
            pending,
        }
    }

    fn wrapped(&self, task: executor::Task) -> executor::Task {
        let sender = self.sender.clone();
        let pending = self.pending.clone();
        Box::new(move || {
            task();
            if !pending.swap(true, Ordering::AcqRel) && sender.send(ImplCommand::Pump).is_err() {
                pending.store(false, Ordering::Release);
            }
        })
    }
}
impl executor::Executor for WakingExecutor {
    fn execute(&self, task: executor::Task) -> io::Result<()> {
        self.inner.execute(self.wrapped(task))
    }

    fn execute_after(&self, delay: Duration, task: executor::Task) -> io::Result<()> {
        self.inner.execute_after(delay, self.wrapped(task))
    }
}
enum PresenterCommand {
    Frame {
        frame: PageCompositorFrame,
        content: ContentFrameProvenance,
        viewport: Viewport,
    },
    Stop,
}
#[derive(Clone, Copy)]
pub(crate) enum MainThreadEvent {
    BeginFrame { frame: NativeBeginFrame },
}
struct FrameDemand {
    requested: Arc<AtomicBool>,
    native: Arc<dyn BeginFrameSource>,
}
impl BeginFrameSource for FrameDemand {
    fn request_begin_frame(&self) {
        self.requested.store(true, Ordering::Release);
        self.native.request_begin_frame();
    }
    fn set_display(&self, id: u32) {
        self.native.set_display(id);
    }
}
pub(crate) struct Handle {
    pub document: Document,
    document_sender: mpsc::Sender<Command>,
    impl_sender: mpsc::Sender<ImplCommand>,
    visible: Arc<AtomicBool>,
    routes: FrameRoutes,
    source: Arc<dyn BeginFrameSource>,
    controls: bool,
    pub loading: bool,
    failure: Arc<Mutex<Option<PageError>>>,
}
impl Handle {
    pub fn send(&self, command: Command) {
        if !matches!(command, Command::Stop) {
            if let Some(error) = self.failure.lock().unwrap().as_ref() {
                if let Command::Evaluate { reply, .. } = command {
                    let _ = reply.send(Err(error.unavailable()));
                }
                return;
            }
        }
        if let Err(mpsc::SendError(Command::Evaluate { reply, .. })) =
            self.document_sender.send(command)
        {
            let _ = reply.send(Err("Page document executor disconnected".into()));
        }
    }
    pub fn failed(&self) -> bool {
        self.failure.lock().unwrap().is_some()
    }
    pub fn input(&self, input: InputEvent, queued_at: Instant, samples: usize) {
        if self.failed() {
            return;
        }
        if !self.controls {
            if let InputEvent::Wheel(wheel) = input {
                let _ = self.impl_sender.send(ImplCommand::Wheel {
                    wheel,
                    queued_at,
                    samples,
                });
                return;
            }
        }
        self.send(Command::Input {
            input,
            queued_at,
            samples,
        });
    }

    pub fn visible(&self, visible: bool) {
        if self.failed() {
            return;
        }
        let was_visible = self.visible.swap(visible, Ordering::AcqRel);
        if visible && !was_visible {
            self.source.request_begin_frame();
        }
    }
    pub fn resize(&self, viewport: Viewport) {
        if self.failed() {
            return;
        }
        self.send(Command::Resize(viewport));
    }
    pub fn patch(&self, patch: Patch) {
        self.send(Command::Patch(patch));
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        self.routes.lock().unwrap().remove(&self.document);
        self.visible.store(false, Ordering::Release);
        // Closing never joins a Page running script or tearing down caches.
        self.send(Command::Stop);
        let _ = self.impl_sender.send(ImplCommand::Stop);
    }
}
pub(crate) struct Params {
    pub document: Document,
    pub url: String,
    pub viewport: Viewport,
    pub request: url_loader::URLRequest,
    pub scripting: bool,
    pub threads: browser::page::PageThreads,
    pub transparent: bool,
    pub controls: bool,
    pub source: Arc<dyn BeginFrameSource>,
    pub routes: FrameRoutes,
    pub frames: mpsc::Sender<window_runtime::Command>,
    pub events: Arc<dyn Fn(Document, Event) + Send + Sync>,
}
pub(crate) fn spawn(params: Params) -> io::Result<Handle> {
    let threads = params.threads.clone();
    let document = params.document;
    let controls = params.controls;
    let (doc_sender, doc_receiver) = mpsc::channel();
    let (impl_sender, impl_receiver) = mpsc::channel();
    let requested = Arc::new(AtomicBool::new(true));
    let source: Arc<dyn BeginFrameSource> = Arc::new(FrameDemand {
        requested: requested.clone(),
        native: params.source.clone(),
    });
    // The parent registers Surface placement before enabling frame delivery.
    let visible = Arc::new(AtomicBool::new(false));
    let main_sender = doc_sender.clone();
    let failure = Arc::new(Mutex::new(None));
    let reporter = ErrorReporter {
        document,
        url: Arc::new(Mutex::new(params.url.clone())),
        events: params.events.clone(),
        failure: failure.clone(),
        visible: visible.clone(),
        routes: params.routes.clone(),
        document_sender: doc_sender.clone(),
        impl_sender: impl_sender.clone(),
    };
    params
        .routes
        .lock()
        .unwrap()
        .insert(document, (impl_sender.clone(), visible.clone()));
    let impl_reporter = reporter.clone();
    let impl_self = impl_sender.clone();
    let impl_source = params.source.clone();
    let impl_frames = params.frames.clone();
    let impl_executor = threads.executor.clone();
    let impl_viewport = params.viewport;
    let impl_external_compositor = !params.controls;
    std::thread::Builder::new()
        .name(format!("page-compositor-{}-{}", document.0, document.1))
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            crate::engine::set_page_thread_presentation_priority();
            let _scope = browser_tracing::scope(browser_tracing::Context {
                target_id: if document.0 == 0 { 2 } else { 1 },
                ..Default::default()
            });
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_impl(
                    impl_receiver,
                    impl_self,
                    main_sender,
                    requested,
                    document,
                    impl_viewport,
                    impl_external_compositor,
                    impl_executor,
                    impl_source,
                    impl_frames,
                )
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => impl_reporter.terminal(error.stage, error.error),
                Err(payload) => impl_reporter.terminal("compositor", panic_error(payload)),
            }
        })
        .map_err(|error| {
            reporter.terminal("compositor", io::Error::other(error.to_string()));
            error
        })?;
    let doc_source = source.clone();
    let wake = doc_sender.clone();
    let doc_impl_sender = impl_sender.clone();
    let routes = params.routes.clone();
    let doc_reporter = reporter.clone();
    let doc_spawn = std::thread::Builder::new()
        .name(format!("page-document-{}-{}", document.0, document.1))
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            crate::engine::set_page_thread_presentation_priority();
            let _scope = browser_tracing::scope(browser_tracing::Context {
                target_id: if document.0 == 0 { 2 } else { 1 },
                ..Default::default()
            });
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_document(
                    &params,
                    doc_receiver,
                    wake,
                    doc_source,
                    &doc_reporter,
                    threads,
                    doc_impl_sender,
                )
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => doc_reporter.terminal("document", error),
                Err(payload) => doc_reporter.terminal("document", panic_error(payload)),
            }
        });
    if let Err(error) = doc_spawn {
        reporter.terminal("document", io::Error::other(error.to_string()));
        return Err(error);
    }
    source.request_begin_frame();
    Ok(Handle {
        document,
        document_sender: doc_sender,
        impl_sender,
        visible,
        routes,
        source,
        controls,
        loading: true,
        failure,
    })
}
// Equivalent responsibility to WebContentsImpl::RenderViewTerminated and its
// PrimaryMainFrameRenderProcessGone notification: terminate this producer,
// publish its status, leave other producers and the window event loop alive.
#[derive(Clone)]
struct ErrorReporter {
    document: Document,
    url: Arc<Mutex<String>>,
    events: Arc<dyn Fn(Document, Event) + Send + Sync>,
    failure: Arc<Mutex<Option<PageError>>>,
    visible: Arc<AtomicBool>,
    routes: FrameRoutes,
    document_sender: mpsc::Sender<Command>,
    impl_sender: mpsc::Sender<ImplCommand>,
}
impl ErrorReporter {
    fn event(
        &self,
        stage: &'static str,
        kind: &'static str,
        message: String,
        resource_url: Option<String>,
    ) -> PageError {
        PageError {
            document: self.document,
            url: self.url.lock().unwrap().clone(),
            stage,
            kind,
            terminal: false,
            message,
            resource_url,
            stack: None,
        }
    }
    fn emit(&self, error: PageError) {
        browser_tracing::instant_with_text(
            "error",
            "PageError",
            &format!("{}: {}", error.stage, error.message),
            &[
                ("view_id", self.document.0 as f64),
                ("document_generation", self.document.1 as f64),
                ("terminal", error.terminal as u8 as f64),
            ],
        );
        (self.events)(self.document, Event::Error(error));
    }
    fn terminal(&self, stage: &'static str, error: io::Error) {
        let mut event = self.event(stage, error_kind(&error), error.to_string(), None);
        event.terminal = true;
        let mut failure = self.failure.lock().unwrap();
        if failure.is_some() {
            return;
        }
        *failure = Some(event.clone());
        drop(failure);
        self.visible.store(false, Ordering::Release);
        self.routes.lock().unwrap().remove(&self.document);
        let _ = self.document_sender.send(Command::Stop);
        let _ = self.impl_sender.send(ImplCommand::Stop);
        self.emit(event);
    }
}
struct RuntimeFailure {
    stage: &'static str,
    error: io::Error,
}
impl From<io::Error> for RuntimeFailure {
    fn from(error: io::Error) -> Self {
        Self {
            stage: if crate::page_error::is_raster_error(&error) {
                "raster"
            } else {
                "compositor"
            },
            error,
        }
    }
}
struct ImplPresentation {
    pipeline: PageRenderPipeline,
    presenter: mpsc::Sender<PresenterCommand>,
    artifact: Option<Arc<paint::PaintArtifact>>,
    content: Option<ContentFrameProvenance>,
    viewport: Viewport,
    needs_publish: bool,
    required_main_frame_epoch: u64,
    responded_main_frame_epoch: u64,
}

impl ImplPresentation {
    fn new(
        pipeline: PageRenderPipeline,
        presenter: mpsc::Sender<PresenterCommand>,
        viewport: Viewport,
    ) -> Self {
        Self {
            pipeline,
            presenter,
            artifact: None,
            content: None,
            viewport,
            needs_publish: false,
            required_main_frame_epoch: 0,
            responded_main_frame_epoch: 0,
        }
    }

    fn require_main_frame(&mut self, epoch: u64) {
        self.required_main_frame_epoch = self.required_main_frame_epoch.max(epoch);
    }

    fn commit(
        &mut self,
        artifact: Arc<paint::PaintArtifact>,
        content: ContentFrameProvenance,
        viewport: Viewport,
    ) {
        // ProxyImpl::data_for_commit_ pairs a commit with the BeginMainFrame
        // that produced it. An older Page artifact cannot satisfy a newer BMF:
        // doing so advances SyncedScrollOffset's reflected slot without the
        // corresponding main-thread delta and rolls the active offset back.
        // The active tree keeps drawing while the matching artifact is built.
        // Source: cc/trees/proxy_impl.cc:450-501,801-814.
        if content.main_frame_epoch < self.required_main_frame_epoch {
            return;
        }
        self.artifact = Some(artifact);
        self.content = Some(content);
        self.viewport = viewport;
        self.needs_publish = true;
    }

    fn publish(&mut self) -> Result<bool, RuntimeFailure> {
        if !self.needs_publish {
            return self.flush_frames();
        }
        let Some(artifact) = self.artifact.as_ref() else {
            return Ok(false);
        };
        let Some(content) = self.content else {
            return Ok(false);
        };
        let viewport = self.viewport;
        let responds_to_main_frame = content.main_frame_epoch > self.responded_main_frame_epoch;
        let render = if responds_to_main_frame {
            self.pipeline.CommitArtifactForMainFrame(
                artifact,
                viewport.width,
                viewport.height,
                viewport.scale,
            )
        } else {
            self.pipeline
                .CommitArtifact(artifact, viewport.width, viewport.height, viewport.scale)
        };
        match render {
            Ok(_) => {
                if responds_to_main_frame {
                    self.responded_main_frame_epoch = content.main_frame_epoch;
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) => return Err(error.into()),
        }
        self.needs_publish = false;
        self.flush_frames()
    }

    fn flush_frames(&mut self) -> Result<bool, RuntimeFailure> {
        let Some(content) = self.content else {
            return Ok(false);
        };
        let mut submitted = false;
        for frame in self.pipeline.TakeFrames() {
            // A frame owns the pixel-space size that cc submitted to its
            // FrameSink.  Do not relabel an older queued frame with a newer
            // Page viewport during resize: Viz surface generations are keyed
            // by these surface parameters in Chromium.
            //
            // Source: components/viz/common/quads/compositor_frame.h;
            //         cc/trees/unbounded_frame_sink_handler.cc:56-77.
            let viewport = frame
                .frame
                .render_pass_list
                .last()
                .filter(|pass| pass.output_rect.width > 0 && pass.output_rect.height > 0)
                .map(|pass| Viewport {
                    width: pass.output_rect.width as u32,
                    height: pass.output_rect.height as u32,
                    scale: if frame.frame.metadata.device_scale_factor.is_finite()
                        && frame.frame.metadata.device_scale_factor > 0.0
                    {
                        f64::from(frame.frame.metadata.device_scale_factor)
                    } else {
                        self.viewport.scale
                    },
                })
                .unwrap_or(self.viewport);
            self.presenter
                .send(PresenterCommand::Frame {
                    frame,
                    content,
                    viewport,
                })
                .map_err(|_| RuntimeFailure {
                    stage: "compositor",
                    error: io::Error::other("Page renderer stopped"),
                })?;
            submitted = true;
        }
        Ok(submitted)
    }
}

fn run_presenter(
    receiver: mpsc::Receiver<PresenterCommand>,
    mut presenter: PageFramePresenter,
    compositor: mpsc::Sender<ImplCommand>,
    document: Document,
    frames: mpsc::Sender<window_runtime::Command>,
) -> io::Result<()> {
    let mut pixels = Arc::new(Vec::<u32>::new());
    let mut retained_target: Option<(u32, u32)> = None;
    let mut presentation_sequence = 0u64;
    while let Ok(first) = receiver.recv() {
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(Instant::now);
        let PresenterCommand::Frame {
            frame,
            content,
            viewport,
        } = first
        else {
            break;
        };
        let (accepted, returned) = presenter.Submit(frame)?;
        if !returned.is_empty() {
            let _ = compositor.send(ImplCommand::Reclaim(returned));
        }
        let mut latest = accepted.then_some((content, viewport));
        let mut stop = false;
        let mut coalesced = 0usize;
        while let Ok(command) = receiver.try_recv() {
            match command {
                PresenterCommand::Frame {
                    frame,
                    content,
                    viewport,
                } => {
                    coalesced += 1;
                    let (accepted, returned) = presenter.Submit(frame)?;
                    if !returned.is_empty() {
                        let _ = compositor.send(ImplCommand::Reclaim(returned));
                    }
                    if accepted {
                        latest = Some((content, viewport));
                    }
                }
                PresenterCommand::Stop => {
                    stop = true;
                    break;
                }
            }
        }
        let Some((content, viewport)) = latest else {
            if stop {
                break;
            }
            continue;
        };
        let retained = retained_target == Some((viewport.width, viewport.height));
        let pixel_count = viewport.width as usize * viewport.height as usize;
        let target = Arc::make_mut(&mut pixels);
        if target.len() != pixel_count {
            target.resize(pixel_count, 0);
        }
        presenter.RenderCurrent(viewport.width, viewport.height, target, retained)?;
        if let Some(started) = profile {
            eprintln!(
                "page-presenter-profile total_ms={:.3} coalesced={} width={} height={}",
                started.elapsed().as_secs_f64() * 1000.0,
                coalesced,
                viewport.width,
                viewport.height,
            );
        }
        retained_target = Some((viewport.width, viewport.height));
        presentation_sequence = presentation_sequence
            .wrapping_add(1)
            .max(content.sequence)
            .max(1);
        frames
            .send(window_runtime::Command::Display(
                crate::display::Message::Prepare {
                    submitted: PageFrame {
                        provenance: FrameProvenance {
                            document,
                            content: Some(ContentFrameProvenance {
                                sequence: presentation_sequence,
                                recording_revision: content.recording_revision,
                                main_frame_epoch: content.main_frame_epoch,
                            }),
                        },
                        viewport,
                        pixels: pixels.clone(),
                    },
                },
            ))
            .map_err(|_| io::Error::other("window display stopped"))?;
        if stop {
            break;
        }
    }
    Ok(())
}

// The Page impl executor owns cc scheduling and input. Its move-only frame
// output crosses the Page FrameSink queue to a Page-owned Viz/Renderer
// sequence, so software drawing cannot stall impl-thread wheel dispatch.
fn run_impl(
    receiver: mpsc::Receiver<ImplCommand>,
    self_sender: mpsc::Sender<ImplCommand>,
    main: mpsc::Sender<Command>,
    requested: Arc<AtomicBool>,
    document: Document,
    viewport: Viewport,
    external_compositor: bool,
    executor: Arc<dyn executor::Executor>,
    source: Arc<dyn BeginFrameSource>,
    frames: mpsc::Sender<window_runtime::Command>,
) -> Result<(), RuntimeFailure> {
    let mut presenter_sender = None;
    let pump_pending = Arc::new(AtomicBool::new(false));
    let compositor_executor: Arc<dyn executor::Executor> = Arc::new(WakingExecutor::new(
        executor,
        self_sender.clone(),
        pump_pending.clone(),
    ));
    let mut presentation = if external_compositor {
        let (pipeline, presenter) = PageRenderPipeline::new_external(compositor_executor);
        let (sender, presenter_receiver) = mpsc::channel();
        let presenter_self = self_sender.clone();
        std::thread::Builder::new()
            .name(format!("page-renderer-{}-{}", document.0, document.1))
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                crate::engine::set_page_thread_presentation_priority();
                if let Err(error) = run_presenter(
                    presenter_receiver,
                    presenter,
                    presenter_self.clone(),
                    document,
                    frames,
                ) {
                    let _ = presenter_self.send(ImplCommand::RendererFailed(error.to_string()));
                }
            })
            .map_err(|error| RuntimeFailure {
                stage: "renderer",
                error: io::Error::other(error.to_string()),
            })?;
        presenter_sender = Some(sender.clone());
        Some(ImplPresentation::new(pipeline, sender, viewport))
    } else {
        None
    };
    let mut main_frame_epoch = 0u64;
    while let Ok(command) = receiver.recv() {
        match command {
            ImplCommand::Stop => {
                if let Some(sender) = presenter_sender.take() {
                    let _ = sender.send(PresenterCommand::Stop);
                }
                break;
            }
            ImplCommand::RendererFailed(message) => {
                return Err(RuntimeFailure {
                    stage: "renderer",
                    error: io::Error::other(message),
                });
            }
            ImplCommand::Pump => {
                pump_pending.store(false, Ordering::Release);
                if let Some(presentation) = presentation.as_mut() {
                    let result = presentation.pipeline.Pump()?;
                    if !result.effects.is_empty() {
                        let requested = result.effects.iter().any(|effect| {
                            matches!(effect, CompositorMainThreadEffect::RequestMainFrame)
                        });
                        let epoch = requested.then(|| {
                            main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                            presentation.require_main_frame(main_frame_epoch);
                            main_frame_epoch
                        });
                        let _ = main.send(Command::CompositorEffects {
                            effects: result.effects,
                            main_frame_epoch: epoch,
                        });
                    }
                    presentation.needs_publish |= result.submitted;
                    let _ = presentation.publish()?;
                }
            }
            ImplCommand::Reclaim(resources) => {
                if let Some(presentation) = presentation.as_mut() {
                    presentation.pipeline.ReclaimResources(resources)?;
                    let _ = presentation.flush_frames()?;
                }
            }
            ImplCommand::Tick(frame) => {
                if let Some(presentation) = presentation.as_mut() {
                    if presentation.pipeline.NeedsBeginFrames() {
                        let result = presentation.pipeline.BeginPhysicalFrame(frame.args)?;
                        if !result.effects.is_empty() {
                            let requested = result.effects.iter().any(|effect| {
                                matches!(effect, CompositorMainThreadEffect::RequestMainFrame)
                            });
                            let epoch = requested.then(|| {
                                main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                                presentation.require_main_frame(main_frame_epoch);
                                main_frame_epoch
                            });
                            let _ = main.send(Command::CompositorEffects {
                                effects: result.effects,
                                main_frame_epoch: epoch,
                            });
                        }
                        presentation.needs_publish |= result.submitted;
                    }
                    let _ = presentation.publish()?;
                }
                if requested.swap(false, Ordering::AcqRel) {
                    let _ = main.send(Command::Main(MainThreadEvent::BeginFrame { frame }));
                }
            }
            ImplCommand::Commit {
                artifact,
                content,
                viewport,
            } => {
                if let Some(presentation) = presentation.as_mut() {
                    presentation.commit(artifact, content, viewport);
                    let _ = presentation.publish()?;
                }
            }
            ImplCommand::Wheel {
                mut wheel,
                queued_at,
                samples,
            } => {
                let Some(presentation) = presentation.as_mut() else {
                    let _ = main.send(Command::Input {
                        input: InputEvent::Wheel(wheel),
                        queued_at,
                        samples,
                    });
                    continue;
                };
                let result = presentation.pipeline.Wheel(&wheel)?;
                wheel.compositor_handled = result.handled;
                presentation.needs_publish |= result.submitted;
                let requested = result
                    .effects
                    .iter()
                    .any(|effect| matches!(effect, CompositorMainThreadEffect::RequestMainFrame));
                let epoch = requested.then(|| {
                    main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                    presentation.require_main_frame(main_frame_epoch);
                    main_frame_epoch
                });
                let _ = main.send(Command::ExternalWheel {
                    wheel,
                    queued_at,
                    samples,
                    effects: result.effects,
                    main_frame_epoch: epoch,
                });
                let _ = presentation.publish()?;
            }
        }
        if let Some(presentation) = presentation.as_mut() {
            let effects = presentation.pipeline.TakeMainThreadEffects();
            if !effects.is_empty() {
                let requested = effects
                    .iter()
                    .any(|effect| matches!(effect, CompositorMainThreadEffect::RequestMainFrame));
                let epoch = requested.then(|| {
                    main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                    presentation.require_main_frame(main_frame_epoch);
                    main_frame_epoch
                });
                let _ = main.send(Command::CompositorEffects {
                    effects,
                    main_frame_epoch: epoch,
                });
            }
        }
        if presentation.as_ref().is_some_and(|presentation| {
            presentation.needs_publish || presentation.pipeline.NeedsBeginFrames()
        }) {
            source.request_begin_frame();
        }
    }
    Ok(())
}
// Each Page has its own instance of the existing task-selection engine.
// Rendering opportunities keep priority over ordinary resource/parser tasks;
// the transport pump only blocks and delivers typed commands.
struct DocumentQueue {
    engine: event_loop::EventLoopEngine<Command, MainThreadEvent>,
    token: u64,
    task_queued: bool,
    collection_queued: bool,
    javascript_collection_queued: bool,
}
struct SelectedCommand(Option<Command>);
impl event_loop::EventLoopExecutor<Command, MainThreadEvent> for SelectedCommand {
    type Error = std::convert::Infallible;
    fn RunTask(
        &mut self,
        _: event_loop::ExecutionContextId,
        _: event_loop::TaskPriority,
        command: Command,
    ) -> Result<(), Self::Error> {
        self.0 = Some(command);
        Ok(())
    }
    fn UpdateRendering(
        &mut self,
        _: event_loop::ExecutionContextId,
        frame: MainThreadEvent,
    ) -> Result<(), Self::Error> {
        self.0 = Some(Command::Main(frame));
        Ok(())
    }
}
impl DocumentQueue {
    fn new() -> Self {
        let mut queue = Self {
            engine: Default::default(),
            token: 0,
            task_queued: false,
            collection_queued: false,
            javascript_collection_queued: false,
        };
        queue.ordinary();
        queue
    }
    fn ordinary(&mut self) {
        if !self.task_queued {
            self.task_queued = true;
            self.post(Command::Task);
        }
    }
    fn collection(&mut self) {
        if !self.collection_queued && foundation::IsLayoutHeapCollectionRequested() {
            self.collection_queued = true;
            self.post(Command::CollectLayoutHeap);
        }
    }
    fn javascript_collection(&mut self, requested: bool) {
        if !self.javascript_collection_queued && requested {
            self.javascript_collection_queued = true;
            self.post(Command::CollectJavaScriptHeap);
        }
    }
    fn post(&mut self, command: Command) {
        use event_loop::*;
        let now = Instant::now();
        if let Command::Main(event @ MainThreadEvent::BeginFrame { .. }) = command {
            self.engine.Apply(
                EventLoopMutation::BeginMainFrame {
                    context: ExecutionContextId(1),
                    frame: event,
                },
                now,
            );
        } else {
            let priority = match &command {
                Command::CollectJavaScriptHeap => TaskPriority::Background,
                Command::Task
                | Command::Wake
                | Command::Evaluate { .. }
                | Command::CollectLayoutHeap => TaskPriority::Normal,
                _ => TaskPriority::UserBlocking,
            };
            self.token += 1;
            self.engine.Apply(
                EventLoopMutation::PostTask(ScheduledTask {
                    token: TaskToken(self.token),
                    context: ExecutionContextId(1),
                    priority,
                    ready_at: None,
                    payload: command,
                }),
                now,
            );
        }
    }
    fn next(
        &mut self,
        receiver: &mpsc::Receiver<Command>,
        task_deadline: Option<Instant>,
    ) -> Result<Command, mpsc::RecvTimeoutError> {
        use event_loop::WakeRequest;
        loop {
            let now = Instant::now();
            let timeout = match self.engine.NextWakeRequest(now) {
                WakeRequest::Now => Duration::ZERO,
                WakeRequest::At(t) => t.saturating_duration_since(now),
                _ => task_deadline.map_or(Duration::from_secs(3600), |t| {
                    t.saturating_duration_since(now)
                }),
            };
            match receiver.recv_timeout(timeout) {
                Ok(Command::Stop) => return Ok(Command::Stop),
                Ok(Command::Wake) => self.ordinary(),
                Ok(command) => self.post(command),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(mpsc::RecvTimeoutError::Disconnected)
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !matches!(
                        self.engine.NextWakeRequest(Instant::now()),
                        WakeRequest::Now
                    ) {
                        self.ordinary();
                    }
                }
            }
            // Make pending transport input visible to task selection before
            // running ordinary/GC work. Selecting after a single FIFO message
            // can otherwise hide user-blocking input behind a resource wake.
            loop {
                match receiver.try_recv() {
                    Ok(Command::Stop) => return Ok(Command::Stop),
                    Ok(Command::Wake) => self.ordinary(),
                    Ok(command) => self.post(command),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        return Err(mpsc::RecvTimeoutError::Disconnected)
                    }
                }
            }
            let mut selected = SelectedCommand(None);
            self.engine.OnWake(Instant::now(), &mut selected).unwrap();
            if let Some(command) = selected.0 {
                if matches!(command, Command::Task) {
                    self.task_queued = false;
                } else if matches!(command, Command::CollectLayoutHeap) {
                    self.collection_queued = false;
                } else if matches!(command, Command::CollectJavaScriptHeap) {
                    self.javascript_collection_queued = false;
                } else {
                    self.ordinary();
                }
                return Ok(command);
            }
        }
    }
}
struct Client {
    document: Document,
    events: Arc<dyn Fn(Document, Event) + Send + Sync>,
    reporter: ErrorReporter,
}
impl PageClient for Client {
    fn DidRequestNavigation(&mut self, request: &browser::page::NavigationRequest) {
        (self.events)(self.document, Event::Navigate(request.clone()));
    }
    fn DidCommit(&mut self, url: &str) {
        if !url.starts_with("data:") {
            *self.reporter.url.lock().unwrap() = url.into();
        }
        browser_tracing::instant(
            "navigation",
            "DocumentCommitted",
            &[
                ("tab_id", self.document.0 as f64),
                ("document_generation", self.document.1 as f64),
            ],
        );
    }
    fn DidPresentFrame(&mut self, frame: &browser::page::PageFrame) {
        browser_tracing::instant(
            "paint",
            "DocumentPaintArtifact",
            &[
                ("tab_id", self.document.0 as f64),
                ("document_generation", self.document.1 as f64),
                ("content_sequence", frame.sequence as f64),
                (
                    "content_recording_revision",
                    frame.display_items.recording_revision as f64,
                ),
                ("display_items", frame.display_items.items.len() as f64),
            ],
        );
    }
    fn DidFinishLoad(&mut self) {
        browser_tracing::instant(
            "navigation",
            "DocumentLoadFinished",
            &[
                ("tab_id", self.document.0 as f64),
                ("document_generation", self.document.1 as f64),
            ],
        );
    }
    fn DidFail(&mut self, message: &str) {
        self.reporter.emit(
            self.reporter
                .event("navigation", "error", message.into(), None),
        );
    }
    fn DidFailResource(&mut self, url: &str, message: &str) {
        self.reporter.emit(self.reporter.event(
            "resource",
            "error",
            message.into(),
            Some(url.into()),
        ));
    }
    fn DidReportScriptError(
        &mut self,
        error: &javascript::javascript_runtime::JavaScriptException,
    ) {
        let mut event = self.reporter.event(
            "script",
            "exception",
            error.message.clone(),
            Some(error.source_name.clone()),
        );
        event.stack = (!error.stack.is_empty()).then(|| error.stack.clone());
        self.reporter.emit(event);
    }
}
fn apply_patch(page: &mut Page, patch: &Patch) -> io::Result<bool> {
    let id = match patch {
        Patch::Attribute { id, .. }
        | Patch::Html { id, .. }
        | Patch::Value { id, .. }
        | Patch::Focus { id, .. } => id,
    };
    let Some(target) = find_id(page, id) else {
        return Ok(false);
    };
    if let Patch::Attribute { name, value, .. } = patch {
        let owner = page.Document();
        let doc = owner.GetDocument();
        let current = doc
            .FindNodeById(target)
            .and_then(|i| doc.Node(i).FindAttribute(name))
            .map(|a| a.value.as_str());
        if current == value.as_deref() {
            return Ok(true);
        }
    }
    let (mutation_type, name, value) = match patch {
        Patch::Attribute { name, value, .. } => (
            if value.is_some() {
                DOMMutationType::kSetAttribute
            } else {
                DOMMutationType::kRemoveAttribute
            },
            name.clone(),
            value.clone().unwrap_or_default(),
        ),
        Patch::Html { value, .. } => (DOMMutationType::kSetInnerHTML, String::new(), value.clone()),
        Patch::Value { value, .. } => (
            DOMMutationType::kSetControlValue,
            String::new(),
            value.clone(),
        ),
        Patch::Focus { select_all, .. } => {
            if page.CurrentFrame().is_none() {
                return Ok(false);
            }
            page.Dispatch(&InputEvent::Focus(FocusEvent {
                target_node_id: target,
                ..Default::default()
            }))?;
            if *select_all {
                page.Dispatch(&InputEvent::Key(KeyEvent {
                    key: "a".into(),
                    modifiers: EventModifiers {
                        control: true,
                        ..Default::default()
                    },
                    ..Default::default()
                }))?;
            }
            return Ok(true);
        }
    };
    page.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type,
        target_node_id: target,
        name,
        value,
        ..Default::default()
    }))?;
    Ok(true)
}
fn run_document(
    params: &Params,
    receiver: mpsc::Receiver<Command>,
    wake: mpsc::Sender<Command>,
    source: Arc<dyn BeginFrameSource>,
    reporter: &ErrorReporter,
    threads: browser::page::PageThreads,
    impl_sender: mpsc::Sender<ImplCommand>,
) -> io::Result<()> {
    let mut viewport = params.viewport;
    let client = Rc::new(RefCell::new(Client {
        document: params.document,
        events: params.events.clone(),
        reporter: reporter.clone(),
    }));
    let mut page = crate::engine::create_document(
        f64::from(viewport.width) / viewport.scale,
        f64::from(viewport.height) / viewport.scale,
        viewport.scale,
        params.scripting,
        client,
        threads,
    )?;
    page.SetExternalCompositor(!params.controls);
    page.SetBeginFrameSource(Some(source.clone()));
    let wake_pending = Arc::new(AtomicBool::new(false));
    let pending = wake_pending.clone();
    page.SetLoadingWakeCallback(Arc::new(move || {
        if !pending.swap(true, Ordering::AcqRel) {
            let _ = wake.send(Command::Wake);
        }
    }));
    page.OpenRequest(&params.request, 16384, 4096)?;
    let mut patches = VecDeque::new();
    let mut published = None;
    let mut pending_main_frame_epoch = 0u64;
    let mut completed_main_frame_epoch = 0u64;
    let mut status = None;
    let mut feedback = None;
    let mut last_memory_report: Option<Instant> = None;
    let mut hovered = Vec::new();
    let mut press_discarded = false;
    let mut queue = DocumentQueue::new();
    loop {
        let command = queue.next(&receiver, page.NextTaskDeadline(Instant::now()));
        let frame_context = match &command {
            Ok(Command::Main(MainThreadEvent::BeginFrame { frame, .. })) => {
                browser_tracing::Context {
                    source_id: frame.source_id,
                    frame_id: frame.sequence_number,
                    ..Default::default()
                }
            }
            _ => Default::default(),
        };
        let _scope = browser_tracing::scope(frame_context);
        match command {
            Ok(Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(Command::Wake) => {
                wake_pending.store(false, Ordering::Release);
                page.RunTask()?;
            }
            #[cfg(test)]
            Ok(Command::TestTask(task)) => task(),
            Ok(Command::Task) => {
                wake_pending.store(false, Ordering::Release);
                page.RunTask()?;
            }
            Ok(Command::CollectLayoutHeap) => {
                // cppgc::Sweeper::IncrementalSweepTask uses a 5 ms foreground
                // budget. User-blocking input/rendering is selected first;
                // native pointers and finalizers stay on this heap's owner.
                foundation::CollectLayoutHeapIncrementally(Duration::from_millis(5));
            }
            Ok(Command::CollectJavaScriptHeap) => page.CollectJavaScriptGarbage(),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                unreachable!("document queue resolves deadlines")
            }
            Ok(Command::Main(MainThreadEvent::BeginFrame { frame, .. })) => {
                let started = Instant::now();
                let result = page.UpdateRendering(frame.args);
                if let Err(error) = result {
                    return Err(error);
                }
                completed_main_frame_epoch = pending_main_frame_epoch;
                browser_tracing::interval(
                    "frame",
                    "MainFrameCycle",
                    started,
                    Instant::now(),
                    &[("view_id", params.document.0 as f64)],
                );
            }
            Ok(Command::Input {
                input,
                queued_at,
                samples,
            }) => {
                if page.CurrentFrame().is_none() {
                    if matches!(&input,InputEvent::Mouse(e) if e.r#type==MouseEventType::kDown) {
                        press_discarded = true;
                    }
                    continue;
                }
                if press_discarded
                    && matches!(&input,InputEvent::Mouse(e) if matches!(e.r#type,MouseEventType::kUp|MouseEventType::kClick|MouseEventType::kDoubleClick|MouseEventType::kContextMenu))
                {
                    if matches!(&input,InputEvent::Mouse(e) if e.r#type==MouseEventType::kClick) {
                        press_discarded = false;
                    }
                    continue;
                }
                if matches!(&input,InputEvent::Mouse(e) if e.r#type==MouseEventType::kDown) {
                    press_discarded = false;
                }
                if interaction::frame_aligned_input_queue::FrameAlignedInputQueue::IsFrameAligned(
                    &input,
                ) && !params.controls
                {
                    page.QueueFrameInput(input, queued_at, samples)?;
                } else {
                    let started = Instant::now();
                    let address = params.controls.then(|| find_id(&page, "address")).flatten();
                    let was_address = params.controls && page.FocusedNodeId() == address;
                    let result = page.Dispatch(&input)?;
                    if params.controls {
                        if matches!(&input,InputEvent::Mouse(e) if matches!(e.r#type,MouseEventType::kDown|MouseEventType::kDoubleClick) && e.button==MouseButton::kPrimary)
                            && result.target_node_id == address
                            && (!was_address
                                || matches!(&input,InputEvent::Mouse(e) if e.r#type==MouseEventType::kDoubleClick))
                        {
                            apply_patch(
                                &mut page,
                                &Patch::Focus {
                                    id: "address".into(),
                                    select_all: true,
                                },
                            )?;
                        }
                        if matches!(&input,InputEvent::Mouse(e) if matches!(e.r#type,MouseEventType::kMove|MouseEventType::kLeave))
                        {
                            let target = if matches!(&input,InputEvent::Mouse(e) if e.r#type==MouseEventType::kLeave)
                            {
                                None
                            } else {
                                result.target_node_id
                            };
                            let next = chrome_hover_chain(&page, target);
                            if hovered != next {
                                for id in std::mem::take(&mut hovered) {
                                    set_hover_class(&mut page, id, false)?;
                                }
                                for &id in &next {
                                    set_hover_class(&mut page, id, true)?;
                                }
                                hovered = next;
                            }
                        }
                        let value = address
                            .and_then(|id| {
                                let owner = page.Document();
                                let d = owner.GetDocument();
                                d.FindNodeById(id).map(|i| d.ControlValue(i))
                            })
                            .unwrap_or_default();
                        (params.events)(
                            params.document,
                            Event::Input {
                                input: input.clone(),
                                target: result.target_node_id.and_then(|id| element_id(&page, id)),
                                focused: page.FocusedNodeId().and_then(|id| element_id(&page, id)),
                                value,
                                prevented: result.default_prevented,
                            },
                        );
                    } else if !result.default_prevented
                        && matches!(&input,InputEvent::Mouse(e) if e.r#type==MouseEventType::kClick && e.button==MouseButton::kPrimary)
                    {
                        if let Some(link) = result
                            .target_node_id
                            .and_then(|id| crate::navigation::link_request(&page, id))
                        {
                            (params.events)(
                                params.document,
                                Event::Navigate(browser::page::NavigationRequest {
                                    request: url_loader::URLRequest {
                                        url: link.url,
                                        ..Default::default()
                                    },
                                    target: if link.disposition
                                        == crate::navigation::Disposition::NewTab
                                    {
                                        "_blank"
                                    } else {
                                        ""
                                    }
                                    .into(),
                                    replace_history: false,
                                }),
                            );
                        }
                    }
                    browser_tracing::interval(
                        "input",
                        "InputQueue",
                        queued_at,
                        started,
                        &[("samples", samples as f64), ("frame_aligned", 0.0)],
                    );
                    browser_tracing::interval(
                        "input",
                        "InputDispatch",
                        started,
                        Instant::now(),
                        &[("samples", samples as f64), ("frame_aligned", 0.0)],
                    );
                }
            }
            Ok(Command::ExternalWheel {
                wheel,
                queued_at,
                samples,
                effects,
                main_frame_epoch,
            }) => {
                let requests_main = main_frame_epoch.is_some();
                page.ApplyCompositorWheelEffects(effects)?;
                if let Some(epoch) = main_frame_epoch {
                    pending_main_frame_epoch = pending_main_frame_epoch.max(epoch);
                }
                page.QueueFrameInput(InputEvent::Wheel(wheel), queued_at, samples)?;
                if requests_main {
                    source.request_begin_frame();
                }
            }
            Ok(Command::CompositorEffects {
                effects,
                main_frame_epoch,
            }) => {
                let requests_main = main_frame_epoch.is_some();
                page.ApplyCompositorWheelEffects(effects)?;
                if let Some(epoch) = main_frame_epoch {
                    pending_main_frame_epoch = pending_main_frame_epoch.max(epoch);
                }
                if requests_main {
                    source.request_begin_frame();
                }
            }
            Ok(Command::Resize(next)) => {
                viewport = next;
                page.ResizeViewport(
                    f64::from(next.width) / next.scale,
                    f64::from(next.height) / next.scale,
                    next.scale,
                )?;
                if params.scripting {
                    let _ = page.Evaluate(
                        "window.dispatchEvent(new Event('resize'))",
                        "rechrom:resize",
                    );
                }
            }
            Ok(Command::Active(active)) => page.SetActive(active)?,
            Ok(Command::Patch(patch)) => patches.push_back(patch),
            Ok(Command::StopLoading) => page.StopLoading(),
            Ok(Command::Evaluate { source, reply }) => {
                let _task = browser_tracing::span("owner_task", "Page.Evaluate");
                let result = page
                    .Evaluate(&source, "devtools:Runtime.evaluate")
                    .map_err(|e| e.to_string())
                    .and_then(|r| r.exception.map_or(Ok(()), |e| Err(e.message)));
                let _ = reply.send(result);
            }
        }
        let count = patches.len();
        for _ in 0..count {
            let patch = patches.pop_front().unwrap();
            if !apply_patch(&mut page, &patch)? {
                patches.push_back(patch);
            }
        }
        for event in page.TakeDispatchedFrameInputs() {
            let _scope = browser_tracing::scope(browser_tracing::Context {
                input_id: browser_tracing::instant_id(event.queued_at),
                ..Default::default()
            });
            browser_tracing::interval(
                "input",
                "InputQueue",
                event.queued_at,
                event.dispatch_started,
                &[("samples", event.samples as f64), ("frame_aligned", 1.0)],
            );
            browser_tracing::interval(
                "input",
                "InputDispatch",
                event.dispatch_started,
                event.dispatch_finished,
                &[("samples", event.samples as f64), ("frame_aligned", 1.0)],
            );
        }
        if let Some(frame) = page.CurrentFrame() {
            let identity = (
                frame.sequence,
                frame.compositor_scroll_sequence,
                completed_main_frame_epoch,
                viewport,
            );
            if published != Some(identity) {
                if !params.controls {
                    impl_sender
                        .send(ImplCommand::Commit {
                            artifact: frame.display_items.clone(),
                            content: ContentFrameProvenance {
                                sequence: frame.sequence,
                                recording_revision: frame.display_items.recording_revision,
                                main_frame_epoch: completed_main_frame_epoch,
                            },
                            viewport,
                        })
                        .map_err(|_| io::Error::other("Page compositor executor stopped"))?;
                    published = Some(identity);
                } else {
                    let render_started = std::env::var_os("BROWSER_PROFILE_INPUT")
                        .is_some()
                        .then(std::time::Instant::now);
                    let mut pixels = vec![0u32; viewport.width as usize * viewport.height as usize];
                    let allocated_at = render_started.map(|_| std::time::Instant::now());
                    let ready = match page.PaintInto(
                        viewport.width,
                        viewport.height,
                        viewport.scale,
                        &mut pixels,
                        skia::PixelFormat::Bgra8888,
                        viewport.width as usize,
                    ) {
                        Ok(_) => true,
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                            // The Page has retained this commit. Retry through the
                            // same frame-demand owner, keeping the last completed
                            // surface visible until a presentable frame exists.
                            source.request_begin_frame();
                            false
                        }
                        Err(error) => return Err(error),
                    };
                    if ready {
                        let painted_at = render_started.map(|_| std::time::Instant::now());
                        let completed = PageFrame {
                            provenance: FrameProvenance {
                                document: params.document,
                                content: Some(ContentFrameProvenance {
                                    sequence: frame.sequence,
                                    recording_revision: frame.display_items.recording_revision,
                                    main_frame_epoch: completed_main_frame_epoch,
                                }),
                            },
                            viewport,
                            pixels: Arc::new(pixels),
                        };
                        params
                            .frames
                            .send(window_runtime::Command::Display(
                                crate::display::Message::Prepare {
                                    submitted: completed,
                                },
                            ))
                            .map_err(|_| io::Error::other("window display stopped"))?;
                        if let (Some(started), Some(allocated), Some(painted)) =
                            (render_started, allocated_at, painted_at)
                        {
                            eprintln!(
                            "page-publish-profile allocate_ms={:.3} paint_ms={:.3} send_ms={:.3} total_ms={:.3}",
                            (allocated - started).as_secs_f64() * 1000.0,
                            (painted - allocated).as_secs_f64() * 1000.0,
                            painted.elapsed().as_secs_f64() * 1000.0,
                            started.elapsed().as_secs_f64() * 1000.0,
                            );
                        }
                        published = Some(identity);
                    }
                }
                if params.document.0 == 0 {
                    (params.events)(
                        params.document,
                        Event::DragRegions(drag_regions(&page, viewport)),
                    );
                }
                let next = (page.URL().to_string(), title(&page), page.IsLoading());
                if status.as_ref() != Some(&next) {
                    (params.events)(
                        params.document,
                        Event::Status {
                            url: next.0.clone(),
                            title: next.1.clone(),
                            loading: next.2,
                        },
                    );
                    status = Some(next);
                }
            }
        }
        if status.as_ref().is_some_and(|s| s.2 != page.IsLoading()) {
            if let Some((url, title, _)) = status.take() {
                let loading = page.IsLoading();
                (params.events)(
                    params.document,
                    Event::Status {
                        url: url.clone(),
                        title: title.clone(),
                        loading,
                    },
                );
                status = Some((url, title, loading));
            }
        }
        // Page's document executor can run many resource and BeginFrame tasks
        // between native input events. Cursor/caret are state feedback, not a
        // pulse for every owner-thread task; enqueue only a changed value so
        // the window event loop remains able to consume native mouse/wheel
        // input while a document is busy.
        let next_feedback = (page.Cursor(), page.Caret().map(|c| c.rect));
        if feedback != Some(next_feedback) {
            (params.events)(
                params.document,
                Event::Feedback {
                    cursor: next_feedback.0,
                    caret: next_feedback.1,
                },
            );
            feedback = Some(next_feedback);
        }
        queue.collection();
        queue.javascript_collection(page.JavaScriptGarbageCollectionRequested());
        // Keep the existing opt-in accounting on this Page's owner thread.
        // The independent host cannot synchronously inspect another Page's heaps.
        if crate::engine::memory_diagnostics_enabled()
            && last_memory_report.is_none_or(|last| last.elapsed() >= Duration::from_secs(1))
        {
            last_memory_report = Some(Instant::now());
            eprintln!(
                "page-memory time_ns={} tab={} generation={} usage={:?}",
                crate::engine::diagnostic_time_ns(),
                params.document.0,
                params.document.1,
                page.MemoryDiagnostics()
            );
        }
    }
    Ok(())
}
fn title(page: &Page) -> String {
    let owner = page.Document();
    let doc = owner.GetDocument();
    fn text(doc: &dom::persistent_document::PersistentDocument, i: usize, out: &mut String) {
        let n = doc.Node(i);
        if n.Type() == dom::persistent_document::DOMNodeType::kText {
            out.push_str(n.Data());
        }
        for &c in n.Children() {
            text(doc, c, out);
        }
    }
    fn find(doc: &dom::persistent_document::PersistentDocument, i: usize) -> Option<usize> {
        let n = doc.Node(i);
        if n.IsHTMLElement("title") {
            return Some(i);
        }
        n.Children().iter().find_map(|&c| find(doc, c))
    }
    let mut value = String::new();
    if let Some(i) = find(doc, doc.Root()) {
        text(doc, i, &mut value);
    }
    value.trim().chars().take(100).collect()
}
fn drag_regions(page: &Page, viewport: Viewport) -> Vec<crate::chrome::DragRegion> {
    use layoutng_assembly::{fragment_tree::FragmentNode, internal::layout_input::Offset};
    let mut regions = vec![crate::chrome::DragRegion {
        x: 92.0,
        y: 0.0,
        width: (f64::from(viewport.width) / viewport.scale - 92.0).max(0.0),
        height: 6.0,
    }];
    fn bounds(f: &FragmentNode, id: u64, p: Offset) -> Option<crate::chrome::DragRegion> {
        let o = Offset {
            x: p.x + f.offset.x,
            y: p.y + f.offset.y,
        };
        if f.node_id == id {
            return Some(crate::chrome::DragRegion {
                x: o.x,
                y: o.y,
                width: f.size.width,
                height: f.size.height,
            });
        }
        f.children.iter().find_map(|c| bounds(c, id, o))
    }
    if let (Some(frame), Some(id)) = (page.CurrentFrame(), find_id(page, "dragspace")) {
        if let Some(r) = bounds(&frame.fragments, id, Offset::default()) {
            regions.push(r);
        }
    }
    regions
}

#[cfg(test)]
mod queue_tests {
    use super::*;

    #[test]
    fn queued_input_precedes_gc_even_behind_transport_wakes() {
        let (sender, receiver) = mpsc::channel();
        let mut queue = DocumentQueue::new();
        queue.post(Command::CollectLayoutHeap);
        sender.send(Command::Wake).unwrap();
        sender
            .send(Command::Input {
                input: InputEvent::Key(KeyEvent {
                    key: "a".into(),
                    ..Default::default()
                }),
                queued_at: Instant::now(),
                samples: 1,
            })
            .unwrap();
        assert!(matches!(
            queue.next(&receiver, None),
            Ok(Command::Input { .. })
        ));
        assert!(matches!(queue.next(&receiver, None), Ok(Command::Task)));
        assert!(matches!(
            queue.next(&receiver, None),
            Ok(Command::CollectLayoutHeap)
        ));
    }
}
