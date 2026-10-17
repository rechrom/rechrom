//! One independent Page domain. Rc document engines are constructed and used
//! only on its document executor. Its impl executor owns compositor/raster
//! state and exports only CompositorFrame plus immutable resource read leases.
//! Source roles: content/renderer/renderer_main.cc creates the document task
//! scheduler; cc/trees/layer_tree_host.cc::CreateThreaded binds main and impl
//! task runners. Assigning a separate pair to every Page is embedder policy;
//! Chromium may co-locate documents. No engine assumes a physical thread.
use crate::{
    begin_frame_source::NativeBeginFrame,
    engine::Viewport,
    page_dom::{chrome_hover_chain, element_id, find_id, set_hover_class},
    page_error::{error_kind, panic_error, PageError},
    page_presentation::{
        self as presentation, Document, PageCommand, PagePresentation, PageRasterReady,
        PageSnapshot,
    },
    window_runtime,
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
    Main(compositor::MainThreadEvent<NativeBeginFrame>),
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
    Core(PageCommand),
    Tick(NativeBeginFrame),
    Ready(PageRasterReady),
    Stop,
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
        if let InputEvent::Wheel(event) = input {
            let _ = self.impl_sender.send(ImplCommand::Core(PageCommand::Wheel {
                event,
                queued_at,
                samples,
            }));
        } else {
            self.send(Command::Input {
                input,
                queued_at,
                samples,
            });
        }
    }
    pub fn visible(&self, visible: bool) {
        if self.failed() {
            return;
        }
        let was_visible = self.visible.swap(visible, Ordering::AcqRel);
        if visible && !was_visible {
            self.source.request_begin_frame();
            let _ = self
                .impl_sender
                .send(ImplCommand::Core(PageCommand::Redraw));
        }
    }
    pub fn resize(&self, viewport: Viewport) {
        if self.failed() {
            return;
        }
        let _ = self
            .impl_sender
            .send(ImplCommand::Core(PageCommand::Resize(viewport)));
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
    let impl_threads = threads.clone();
    let document = params.document;
    let (doc_sender, doc_receiver) = mpsc::channel();
    let (impl_sender, impl_receiver) = mpsc::channel();
    let requested = Arc::new(AtomicBool::new(true));
    let source: Arc<dyn BeginFrameSource> = Arc::new(FrameDemand {
        requested: requested.clone(),
        native: params.source.clone(),
    });
    // The parent registers Surface placement before enabling frame delivery.
    let visible = Arc::new(AtomicBool::new(false));
    let ready_sender = impl_sender.clone();
    let main_sender = doc_sender.clone();
    let impl_source = params.source.clone();
    let frames = params.frames.clone();
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
                    document,
                    impl_receiver,
                    ready_sender,
                    main_sender,
                    requested,
                    impl_source,
                    frames,
                    &impl_reporter,
                    &impl_threads,
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
    let commits = impl_sender.clone();
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
                    commits,
                    doc_source,
                    &doc_reporter,
                    threads,
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
fn run_impl(
    document: Document,
    receiver: mpsc::Receiver<ImplCommand>,
    ready_sender: mpsc::Sender<ImplCommand>,
    main: mpsc::Sender<Command>,
    requested: Arc<AtomicBool>,
    source: Arc<dyn BeginFrameSource>,
    frames: mpsc::Sender<window_runtime::Command>,
    reporter: &ErrorReporter,
    threads: &browser::page::PageThreads,
) -> Result<(), RuntimeFailure> {
    let reply = Arc::new(move |ready| {
        let _ = ready_sender.send(ImplCommand::Ready(ready));
    });
    let mut owner = PagePresentation::with_threads(document, reply, threads)?;
    let mut scheduler = viz::DisplayScheduler::default();
    let route = |owner: &mut PagePresentation,
                 scheduler: &mut viz::DisplayScheduler<NativeBeginFrame>,
                 effects: Vec<presentation::Effect>|
     -> io::Result<()> {
        for effect in effects {
            match effect {
                presentation::Effect::MainThread(event) => {
                    let _ = main.send(Command::Main(event));
                }
                presentation::Effect::FrameProduced {
                    frame,
                    submitted,
                    resources,
                    scroll_active,
                } => {
                    frames
                        .send(window_runtime::Command::Display(
                            crate::display::Message::Prepare {
                                frame,
                                submitted,
                                resources,
                                scroll_active,
                            },
                        ))
                        .map_err(|_| io::Error::other("window display stopped"))?;
                }
                presentation::Effect::RequestBeginFrame => source.request_begin_frame(),
                presentation::Effect::VisualDamage => scheduler.NotifyVisualDamage(Instant::now()),
                _ => {}
            }
        }
        let _ = owner;
        Ok(())
    };
    loop {
        let timeout = scheduler
            .NextDeadline()
            .map_or(Duration::from_secs(1), |t| {
                t.saturating_duration_since(Instant::now())
            });
        let effects = match receiver.recv_timeout(timeout) {
            Ok(ImplCommand::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            Ok(ImplCommand::Core(command)) => owner.Handle(command)?,
            Ok(ImplCommand::Ready(ready)) => {
                let ready = ready.take();
                let rejected = ready
                    .result
                    .as_ref()
                    .err()
                    .map(|e| reporter.event("raster", error_kind(e), e.to_string(), None));
                let effects = owner
                    .Handle(PageCommand::RasterReady(ready))
                    .map_err(|error| RuntimeFailure {
                        stage: "raster",
                        error,
                    })?;
                if let Some(error) = rejected {
                    reporter.emit(error);
                }
                effects
            }
            Ok(ImplCommand::Tick(frame)) => {
                if let Some(previous) = scheduler.BeginFrame(
                    frame,
                    viz::FrameTiming {
                        frame_time: frame.frame_time,
                        interval: frame.interval,
                        source_deadline: frame.deadline,
                    },
                ) {
                    if !previous.IsPrepared() {
                        let effects = owner.Prepare(previous.Payload())?;
                        route(&mut owner, &mut scheduler, effects)?;
                    }
                }
                if requested.swap(false, Ordering::AcqRel) {
                    let effects = owner.Handle(PageCommand::BeginFrame(frame))?;
                    if !effects.iter().any(|e| {
                        matches!(
                            e,
                            presentation::Effect::MainThread(
                                compositor::MainThreadEvent::BeginFrame { .. }
                            )
                        )
                    }) {
                        requested.store(true, Ordering::Release);
                    }
                    effects
                } else {
                    vec![]
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => vec![],
        };
        route(&mut owner, &mut scheduler, effects)?;
        if scheduler.NeedsPrepare(Instant::now()) {
            let pending = scheduler.PendingFrame().unwrap();
            let effects = owner.Prepare(pending.Payload())?;
            let produced = effects
                .iter()
                .any(|e| matches!(e, presentation::Effect::FrameProduced { .. }));
            scheduler.MarkPrepared(pending, produced);
            route(&mut owner, &mut scheduler, effects)?;
        }
        scheduler.TakeIfDrawDue(Instant::now());
    }
}
// Each Page has its own instance of the existing task-selection engine.
// Rendering opportunities keep priority over ordinary resource/parser tasks;
// the transport pump only blocks and delivers typed commands.
struct DocumentQueue {
    engine: event_loop::EventLoopEngine<Command, compositor::MainThreadEvent<NativeBeginFrame>>,
    token: u64,
    task_queued: bool,
    collection_queued: bool,
    javascript_collection_queued: bool,
}
struct SelectedCommand(Option<Command>);
impl event_loop::EventLoopExecutor<Command, compositor::MainThreadEvent<NativeBeginFrame>>
    for SelectedCommand
{
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
        frame: compositor::MainThreadEvent<NativeBeginFrame>,
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
        if let Command::Main(event @ compositor::MainThreadEvent::BeginFrame { .. }) = command {
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
    commits: mpsc::Sender<ImplCommand>,
    source: Arc<dyn BeginFrameSource>,
    reporter: &ErrorReporter,
    threads: browser::page::PageThreads,
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
    page.SetBeginFrameSource(Some(source));
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
    let mut status = None;
    let mut last_memory_report: Option<Instant> = None;
    let mut hovered = Vec::new();
    let mut press_discarded = false;
    let mut queue = DocumentQueue::new();
    loop {
        let mut main_complete = None;
        let command = queue.next(&receiver, page.NextTaskDeadline(Instant::now()));
        let frame_context = match &command {
            Ok(Command::Main(compositor::MainThreadEvent::BeginFrame { frame, .. })) => {
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
            Ok(Command::Main(compositor::MainThreadEvent::BeginFrame {
                frame, scroll, ..
            })) => {
                let started = Instant::now();
                if let Some(update) = scroll {
                    page.ApplyCompositorScrollDelta(
                        update.sequence,
                        update.owner_client_id,
                        update.delta,
                    )?;
                }
                let result = page.UpdateRendering(frame.args);
                main_complete = Some((frame.source_id, frame.sequence_number));
                if let Err(error) = result {
                    let _ = commits.send(ImplCommand::Core(PageCommand::MainFrameComplete {
                        source_id: frame.source_id,
                        sequence_number: frame.sequence_number,
                    }));
                    return Err(error);
                }
                browser_tracing::interval(
                    "frame",
                    "MainFrameCycle",
                    started,
                    Instant::now(),
                    &[("view_id", params.document.0 as f64)],
                );
            }
            Ok(Command::Main(compositor::MainThreadEvent::Wheel {
                event,
                queued_at,
                samples,
            })) => {
                if page.CurrentFrame().is_some() {
                    page.QueueFrameInput(InputEvent::Wheel(event), queued_at, samples)?;
                }
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
            let identity = (frame.sequence, frame.compositor_scroll_sequence, viewport);
            if published != Some(identity) {
                let blocking = page.BlockingWheelEventRegions();
                let snapshot = PageSnapshot {
                    document: params.document,
                    viewport,
                    frame_time: Instant::now(),
                    sequence: frame.sequence,
                    compositor_scroll_sequence: frame.compositor_scroll_sequence,
                    content: compositor::ContentUpdate::Replace(frame.display_items.clone()),
                    transparent_background: params.transparent,
                    metadata: (),
                    async_root_scroll: params.scripting,
                    blocking_wheel_regions: compositor::BlockingWheelEventRegions {
                        covers_viewport: blocking.covers_viewport,
                        scrolling_rects: blocking.scrolling_rects,
                        fixed_rects: blocking.fixed_rects,
                    },
                };
                let _ = commits.send(ImplCommand::Core(PageCommand::Snapshot(snapshot)));
                published = Some(identity);
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
        (params.events)(
            params.document,
            Event::Feedback {
                cursor: page.Cursor(),
                caret: page.Caret().map(|c| c.rect),
            },
        );
        if let Some((source_id, sequence_number)) = main_complete {
            let _ = commits.send(ImplCommand::Core(PageCommand::MainFrameComplete {
                source_id,
                sequence_number,
            }));
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

#[cfg(test)]
mod failure_tests {
    use super::*;

    #[test]
    fn page_failures_emit_identity_stop_their_actors_and_leave_other_pages_rendering() {
        let source =
            crate::begin_frame_source::create_offscreen(Duration::from_millis(16)).unwrap();
        let routes: FrameRoutes = Arc::new(Mutex::new(BTreeMap::new()));
        let clocks = routes.clone();
        source
            .GetFrameSource(1)
            .unwrap()
            .SetClient(Arc::new(move |frame| {
                let senders: Vec<_> = clocks
                    .lock()
                    .unwrap()
                    .values()
                    .filter(|(_, v)| v.load(Ordering::Acquire))
                    .map(|(s, _)| s.clone())
                    .collect();
                for sender in senders {
                    let _ = sender.send(ImplCommand::Tick(frame));
                }
            }))
            .unwrap();
        let (frames, frame_receiver) = mpsc::channel();
        let (events, event_receiver) = mpsc::channel();
        let events: Arc<dyn Fn(Document, Event) + Send + Sync> = Arc::new(move |id, event| {
            let _ = events.send((id, event));
        });
        let viewport = Viewport {
            width: 320,
            height: 160,
            scale: 1.0,
        };
        let new_page = |document, scripting| {
            let page=spawn(Params{document,url:format!("https://page.test/{}",document.0),viewport,request:url_loader::URLRequest{url:crate::engine::data_url("<!doctype html><title>alive</title><body style='margin:0;background:#ace'>alive</body>"),..Default::default()},scripting,threads:Default::default(),transparent:false,controls:false,source:source.clone(),routes:routes.clone(),frames:frames.clone(),events:events.clone()}).unwrap();
            page.visible(true);
            page
        };
        let frame_for = |document, after| {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let message = frame_receiver
                    .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .expect("Page frame deadline");
                if let window_runtime::Command::Display(crate::display::Message::Prepare {
                    submitted,
                    ..
                }) = message
                {
                    if submitted.provenance.document == document
                        && submitted.content.is_some()
                        && submitted
                            .provenance
                            .content
                            .is_some_and(|p| p.sequence > after)
                    {
                        return submitted.provenance.content.unwrap().sequence;
                    }
                }
            }
        };
        let failed_event = |document| {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let (id, event) = event_receiver
                    .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .expect("Page error deadline");
                if let Event::Error(error) = event {
                    if id == document && error.terminal {
                        error.emit_stderr();
                        return error;
                    }
                }
            }
        };
        let healthy = new_page((1, 1), true);
        let mut sequence = frame_for(healthy.document, 0);
        let mut failures = Vec::new();
        for (id, stage) in [(2, "compositor"), (3, "document"), (4, "raster")] {
            let page = new_page((id, 1), false);
            frame_for(page.document, 0);
            if stage == "document" {
                page.send(Command::TestTask(Box::new(|| {
                    std::panic::panic_any(dom::error::DOMException {
                        kind: dom::error::DOMExceptionKind::LogicError,
                        message: "injected document failure",
                    })
                })));
            } else {
                let mut artifact = paint::paint_engine::PaintArtifact::default();
                // A draw without semantic records is rejected by the real
                // raster planner, on this Page's own raster executor.
                artifact.items = Arc::new(vec![Default::default()]);
                let snapshot = PageSnapshot {
                    document: if stage == "compositor" {
                        (99, 1)
                    } else {
                        page.document
                    },
                    viewport,
                    frame_time: Instant::now(),
                    sequence: 10000,
                    compositor_scroll_sequence: 0,
                    content: compositor::ContentUpdate::Replace(Arc::new(artifact)),
                    transparent_background: false,
                    metadata: (),
                    async_root_scroll: false,
                    blocking_wheel_regions: Default::default(),
                };
                page.impl_sender
                    .send(ImplCommand::Core(PageCommand::Snapshot(snapshot)))
                    .unwrap();
            }
            let error = failed_event(page.document);
            assert_eq!(error.document, page.document);
            assert_eq!(error.stage, stage);
            assert_eq!(error.url, format!("https://page.test/{id}"));
            assert_eq!(error.json()["view_id"], id);
            assert!(page.failed());
            assert!(!routes.lock().unwrap().contains_key(&page.document));
            if stage == "document" {
                assert_eq!(error.kind, "panic");
                assert!(error.message.contains("injected document failure"));
            }
            if stage == "raster" {
                assert!(error.message.contains("missing-semantic-records"));
            }
            let (reply, result) = mpsc::channel();
            page.send(Command::Evaluate {
                source: "throw Error('must not run')".into(),
                reply,
            });
            assert!(result
                .recv_timeout(Duration::from_millis(500))
                .unwrap()
                .unwrap_err()
                .contains("stopped"));
            let (reply, result) = mpsc::channel();
            healthy.send(Command::Evaluate {
                source: format!("document.body.textContent='alive after {stage}';"),
                reply,
            });
            result
                .recv_timeout(Duration::from_secs(5))
                .unwrap()
                .unwrap();
            sequence = frame_for(healthy.document, sequence);
            assert!(!healthy.failed());
            failures.push(page);
        }
        // A fresh document generation can render again without restarting the
        // host, a healthy Page, or the shared physical clock.
        let replacement = new_page((2, 2), false);
        frame_for(replacement.document, 0);
        assert!(!replacement.failed());
        drop((replacement, failures, healthy));
        assert!(routes.lock().unwrap().is_empty());
    }
}
