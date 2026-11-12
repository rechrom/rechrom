//! One independent Page domain. Document tasks and Page rendering execute only
//! on its document owner. Frame delivery merely enqueues rendering opportunities;
//! move-only CompositorFrames cross into the window-owned Viz domain.
use crate::{
    begin_frame_source::BeginFrame,
    engine::{ResizeRequest, Viewport},
    page_dom::{chrome_hover_chain, element_id, find_id, set_hover_class},
    page_error::{error_kind, panic_error, PageError},
    page_presentation::{
        ContentFrameProvenance, Document, FrameProvenance, PageFrame, ResizeProvenance,
    },
    window_runtime,
};
use browser::page::{
    CompositorMainThreadEffect, CompositorWheelResult, PageChildCompositorFrame,
    PageRenderPipeline, PageReturnedResource,
};
use browser::page::{Page, PageClient};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use foundation::begin_frame::BeginFrameSource;
use interaction::input::ui::events::gesture_detection::filtered_gesture_provider::GestureTouchActionFilterHandle;
use interaction::input::{
    components::input::{
        fling_controller::{FlingControllerOutput, GeneratedFlingEvent},
        gesture_event_queue::{
            DebounceEffect, ForwardedGesture, GestureEventAck, GestureEventQueue,
        },
    },
    public::{
        common::input::web_mouse_wheel_event::WebMouseWheelEvent,
        mojom::input::input_event_result::{InputEventResultSource, InputEventResultState},
    },
};
use interaction::input_event::*;
use page_mutation::PageMutation;
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    io,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};

pub(crate) type FrameRoutes = Arc<Mutex<BTreeMap<Document, Arc<FrameRoute>>>>;

pub(crate) struct FrameRoute {
    sender: mpsc::Sender<ImplCommand>,
    visible: Arc<AtomicBool>,
}

impl FrameRoute {
    pub(crate) fn visible(&self) -> bool {
        self.visible.load(Ordering::Acquire)
    }

    /// CVDisplayLinkMac posts the frame directly to the compositor task
    /// runner. The impl mailbox is that runner's sequenced ingress here; an
    /// extra hop through cc's worker executor introduces a second queue and can
    /// leave a requested BeginMainFrame behind self-posted deadline work.
    pub(crate) fn begin_frame(&self, frame: BeginFrame) {
        let _ = self.sender.send(ImplCommand::Tick(frame));
    }
}
#[derive(Clone)]
pub(crate) enum Event {
    Status {
        url: String,
        title: String,
        loading: bool,
    },
    HistoryCommitted {
        item: Arc<navigation_history::HistoryItemSnapshot>,
        load_type: navigation_history::FrameLoadType,
    },
    ChildHistoryCommitted {
        item: Arc<navigation_history::HistoryItemSnapshot>,
        load_type: navigation_history::FrameLoadType,
    },
    HistoryStateUpdated(Arc<navigation_history::HistoryItemSnapshot>),
    NavigationPrepared {
        token: u64,
    },
    HistoryNavigation(i32),
    Navigate(browser::page::NavigationRequest),
    AuxiliaryNavigationTarget(browser::page::AuxiliaryNavigationTargetRequest),
    DropNavigate(browser::page::DropNavigationRequest),
    StartSystemDrag(crate::native_drag::Start),
    Input {
        input: InputEvent,
        target: Option<String>,
        focused: Option<String>,
        value: String,
        prevented: bool,
    },
    ValidationMessage(browser::page::ValidationMessageUpdate),
    ValidationMessageVisibility(browser::page::ValidationMessageVisibility),
    ValidationBubbleSize {
        width: f64,
        height: f64,
    },
    CursorChanged(interaction::Cursor),
    ControlActivated {
        target: Option<String>,
    },
    EventHandlerChrome(browser::page::EventHandlerChromeEffect),
    CaptureMouseEventsToWidget(bool),
    Autoscroll(interaction::adapter::page_mouse_autoscroll::AutoscrollHostEffect),
    TouchDisposition {
        unique_touch_event_id: u32,
        consumed: bool,
    },
    FlingDisposition {
        gesture: WebGestureEvent,
        result: InputEventResultState,
    },
    GestureDisposition {
        gesture: WebGestureEvent,
        source: InputEventResultSource,
        result: InputEventResultState,
    },
    Feedback {
        caret: Option<browser::page::CaretRect>,
        text_input: Option<interaction::adapter::interaction::TextInputState>,
    },
    DragRegions {
        regions: Vec<crate::chrome::DragRegion>,
        root_viewport: Option<Viewport>,
    },
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
    Text {
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
    PrepareNavigation { token: u64 },
    CompleteAuxiliaryNavigationTarget {
        request_id: u64,
        accepted: bool,
    },
    DirectoryFilesChosen(browser::page::DirectoryEnumerationResult),
    SystemDragTarget {
        target: crate::native_drag::Target,
        action: interaction::default::core::page::drag_controller::DragTargetAction,
    },
    SystemDragEnded(crate::native_drag::Completion),
    BindCompositorPresentationClock(browser::page::PageCompositorClockMapping),
    EventTimingDidSwap {
        requests: Vec<u64>,
        frame_token: u32,
        swap_time: Instant,
    },
    EventTimingDidNotSwap {
        requests: Vec<u64>,
        swap_time: Instant,
    },
    EventTimingDidCommit(Instant),
    EventTimingDidPresent(browser::page::PageFramePresentationFeedback),
    Visibility {
        visible: bool,
        changed_at: Instant,
    },
    Input {
        input: InputEvent,
        queued_at: Instant,
        samples: usize,
    },
    ExternalWheel {
        document_generation: u64,
        wheel: WheelEvent,
        queued_at: Instant,
        samples: usize,
        effects: Vec<CompositorMainThreadEffect>,
        main_frame_epoch: Option<u64>,
    },
    ExternalCanonicalWheel {
        document_generation: u64,
        event: WebMouseWheelEvent,
        wheel: WheelEvent,
        queued_at: Instant,
        effects: Vec<CompositorMainThreadEffect>,
        main_frame_epoch: Option<u64>,
    },
    FlingDisposition(WebGestureEvent),
    GestureDisposition(GestureEventAck),
    ExternalGesture { document_generation: u64, dispatch_id: u64, gesture: WebGestureEvent },
    CompositorEffects {
        effects: Vec<CompositorMainThreadEffect>,
        main_frame_epoch: Option<u64>,
    },
    Main(MainThreadEvent),
    Resize(ResizeRequest),
    SystemCursorSize(crate::cursor_screen_info::SystemCursorSize),
    ContextMenuWidgetGeometry(interaction::adapter::page_nonlocated_context_menu_coordinates::NativeContextMenuWidgetGeometry),
    InteractionTextSettings(crate::interaction_text_settings::InteractionTextSettings),
    ResizeTask,
    Active(bool),
    Patch(Vec<(u64, Patch)>),
    StopLoading,
    TraverseHistory {
        item: Arc<navigation_history::HistoryItemSnapshot>,
        state: navigation_history::SessionHistoryState,
    },
    SessionHistoryState {
        item: Arc<navigation_history::HistoryItemSnapshot>,
        state: navigation_history::SessionHistoryState,
    },
    Evaluate {
        source: String,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Document-sequence half of an offscreen CopyOutput ordering fence. The
    /// impl input owner posts this only after it has consumed all preceding
    /// compositor input, so reaching it also follows preceding direct Page
    /// input in the document queue.
    CopyOutputFence,
    Wake,
    ClickActivated { source_document: u128, target_node_id: u64 },
    #[cfg(test)]
    TestTask(Box<dyn FnOnce() + Send>),
    Task,
    CollectLayoutHeap,
    CollectJavaScriptHeap,
    Stop,
}
pub(crate) enum ImplCommand {
    BindGestureTouchActionFilter(GestureTouchActionFilterHandle),
    WheelAck {
        document_generation: u64,
        result: InputEventResultState,
    },
    GestureAck {
        dispatch_id: u64,
        result: InputEventResultState,
    },
    DebounceExpired,
    DocumentDetached {
        generation: u64,
    },
    StopFling,
    EventTimingRequests {
        requests: Vec<u64>,
        request_main_frame: bool,
    },
    RequestEventTimingFrame,
    PresentationFeedback(browser::page::PageFramePresentationFeedback),
    Tick(BeginFrame),
    Pump,
    RequestMainFrame,
    /// Cross the compositor input sequence before requesting the document
    /// main frame used to satisfy an offscreen CopyOutput request.
    CopyOutputFence,
    FinishMainFrameWithoutUpdates {
        main_frame_epoch: u64,
    },
    Commit {
        artifact: Arc<paint::PaintArtifact>,
        child_frames: Vec<PageChildCompositorFrame>,
        content: ContentFrameProvenance,
        viewport: Viewport,
    },
    Wheel {
        wheel: WheelEvent,
        queued_at: Instant,
        samples: usize,
    },
    Gesture {
        gesture: interaction::input_event::WebGestureEvent,
        queued_at: Instant,
        samples: usize,
    },
    InputContinuation(compositor::CompositorMutation),
    Reclaim(Vec<PageReturnedResource>),
    Stop,
}

struct WakingExecutor {
    inner: Arc<dyn executor::Executor>,
    sender: mpsc::Sender<ImplCommand>,
    pending: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
    trace_name: &'static str,
}
impl WakingExecutor {
    fn new(
        inner: Arc<dyn executor::Executor>,
        sender: mpsc::Sender<ImplCommand>,
        pending: Arc<AtomicBool>,
        generation: Arc<AtomicU64>,
        trace_name: &'static str,
    ) -> Self {
        Self {
            inner,
            sender,
            pending,
            generation,
            trace_name,
        }
    }

    fn wrapped(&self, task: executor::Task) -> executor::Task {
        let sender = self.sender.clone();
        let pending = self.pending.clone();
        let generation = self.generation.clone();
        let trace_name = self.trace_name;
        // A posted task keeps the Page target identity of its origin turn.
        // Chromium's trace task context crosses PostTask with the closure;
        // without this scope, raster and compositor worker spans are recorded
        // as target 0 and disappear from a target-scoped Performance trace.
        let trace_context = browser_tracing::context();
        Box::new(move || {
            let _trace_context = browser_tracing::scope(trace_context);
            let _task_trace = browser_tracing::span("compositor", trace_name);
            task();
            generation.fetch_add(1, Ordering::Release);
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
#[derive(Clone, Copy)]
pub(crate) enum MainThreadEvent {
    BeginFrame {
        frame: BeginFrame,
        /// The compositor transaction dispatched with this BeginMainFrame.
        /// Keep it on the rendering opportunity itself: the document event
        /// loop is allowed to prioritize rendering over already-posted tasks.
        main_frame_epoch: u64,
    },
}
struct FrameDemand {
    requested: Arc<AtomicBool>,
    frame_source: Arc<dyn BeginFrameSource>,
    compositor: mpsc::Sender<ImplCommand>,
}
impl FrameDemand {
    /// A request originating from cc's SEND_BEGIN_MAIN_FRAME already has a
    /// Scheduler commit request. It only needs to wake the Page lifecycle.
    fn request_compositor_main_frame(&self) {
        self.requested.store(true, Ordering::Release);
        self.frame_source.request_begin_frame();
    }
}
impl BeginFrameSource for FrameDemand {
    fn request_begin_frame(&self) {
        // ProxyMain::SetNeedsCommit always enters cc's Scheduler. The widget
        // viewport is initialized before this source is attached, so initial
        // load and later resource/input updates share one transaction path.
        if !self.requested.load(Ordering::Acquire) {
            let _ = self.compositor.send(ImplCommand::RequestMainFrame);
        }
    }
    fn set_display(&self, id: u32) {
        self.frame_source.set_display(id);
    }
}

/// Browser-side VisualProperties throttle.
///
/// Source: content/browser/renderer_host/render_widget_host_impl.cc
/// SynchronizeVisualProperties and OnLocalSurfaceIdChanged. Size-changing
/// updates are limited to one in flight; while its CompositorFrame ACK is
/// pending only the latest requested properties are retained.
struct VisualPropertiesThrottle {
    document_sender: mpsc::Sender<Command>,
    last_sent: Viewport,
    ack_pending: bool,
    pending_latest: Option<ResizeRequest>,
}

impl VisualPropertiesThrottle {
    fn new(document_sender: mpsc::Sender<Command>, initial: Viewport) -> Self {
        Self {
            document_sender,
            last_sent: initial,
            // The renderer is created with initial VisualProperties. Its first
            // accepted CompositorFrame is the corresponding ACK.
            ack_pending: true,
            pending_latest: None,
        }
    }

    fn send(&mut self, resize: ResizeRequest) {
        let size_changed = self.last_sent.width != resize.viewport.width
            || self.last_sent.height != resize.viewport.height;
        self.last_sent = resize.viewport;
        self.ack_pending = size_changed;
        let _ = self.document_sender.send(Command::Resize(resize));
    }

    fn resize(&mut self, resize: ResizeRequest) {
        if self.ack_pending {
            if let Some(pending) = &mut self.pending_latest {
                pending.merge(resize);
            } else {
                self.pending_latest = Some(resize);
            }
            return;
        }
        if resize.viewport != self.last_sent {
            self.send(resize);
        }
    }

    fn ack(&mut self) {
        self.ack_pending = false;
        let Some(pending) = self.pending_latest.take() else {
            return;
        };
        if pending.viewport != self.last_sent {
            self.send(pending);
        }
    }

    fn hidden(&mut self) {
        // Source: RenderWidgetHostImpl::WasHidden. A hidden renderer must not
        // leave a visual-properties ACK blocking its later activation.
        self.ack_pending = false;
        self.pending_latest = None;
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
    failure: Arc<Mutex<Option<PageError>>>,
    next_patch_revision: AtomicU64,
    visual_properties: Arc<Mutex<VisualPropertiesThrottle>>,
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
    pub fn copy_output_fence(&self) {
        if !self.failed() {
            let _ = self.impl_sender.send(ImplCommand::CopyOutputFence);
        }
    }
    pub fn input(&self, input: InputEvent, queued_at: Instant, samples: usize) {
        if self.failed() {
            return;
        }
        if matches!(&input, InputEvent::Key(_)) {
            let _ = self.impl_sender.send(ImplCommand::StopFling);
        }
        if let InputEvent::Gesture(gesture) = &input {
            let _ = self.impl_sender.send(ImplCommand::Gesture {
                gesture: gesture.clone(),
                queued_at,
                samples,
            });
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
        if visible != was_visible {
            self.send(Command::Visibility {
                visible,
                changed_at: Instant::now(),
            });
        }
        if !visible {
            self.visual_properties.lock().unwrap().hidden();
        }
        if visible && !was_visible {
            self.source.request_begin_frame();
        }
    }
    pub fn resize(&self, resize: ResizeRequest) {
        if self.failed() {
            return;
        }
        if self.controls {
            // BrowserView/ToolbarView consumes native bounds on the browser UI
            // sequence; RenderWidgetHostImpl's one-in-flight renderer ACK does
            // not apply to browser chrome. Our controls remain Page-backed,
            // but their document queue already coalesces adjacent resize work.
            // Applying the WebContents throttle here would add a renderer ACK
            // round trip to every browser-root resize.
            self.send(Command::Resize(resize));
        } else {
            self.visual_properties.lock().unwrap().resize(resize);
        }
    }
    pub fn patch(&self, patch: Patch) -> u64 {
        let revision = self.next_patch_revision.fetch_add(1, Ordering::AcqRel) + 1;
        self.send(Command::Patch(vec![(revision, patch)]));
        revision
    }
    pub fn patches(&self, patches: Vec<Patch>) -> u64 {
        if patches.is_empty() {
            return self.next_patch_revision.load(Ordering::Acquire);
        }
        let first = self
            .next_patch_revision
            .fetch_add(patches.len() as u64, Ordering::AcqRel)
            + 1;
        let mut revision = first;
        let patches = patches
            .into_iter()
            .map(|patch| {
                let current = revision;
                revision += 1;
                (current, patch)
            })
            .collect();
        self.send(Command::Patch(patches));
        revision - 1
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        self.routes.lock().unwrap().remove(&self.document);
        self.visible.store(false, Ordering::Release);
        // Closing never joins a Page running script or tearing down caches.
        self.send(Command::Stop);
        // Stop the producer before its consumer. An in-flight document turn
        // may still commit after this handle is dropped during navigation.
        // Its executor closes the compositor after that turn unwinds, which
        // then closes the presenter. Otherwise an ordinary navigation races
        // Commit against a disconnected impl receiver and emits a fatal error.
        // Chromium: content/renderer/render_frame_impl.cc FrameDetached closes
        // the WebLocalFrame before destroying its owning renderer objects.
    }
}
pub(crate) struct Params {
    pub document: Document,
    pub url: String,
    pub viewport: Viewport,
    pub request: url_loader::URLRequest,
    pub scripting: bool,
    pub prefers_reduced_motion: bool,
    pub system_cursor_size: crate::cursor_screen_info::SystemCursorSize,
    pub text_settings: crate::interaction_text_settings::InteractionTextSettings,
    pub font_backend_factory: browser::BrowserFontBackendFactory,
    pub primary_clipboard: Arc<crate::primary_clipboard::PrimaryClipboard>,
    pub native_drag_window: bool,
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
    let visual_properties = Arc::new(Mutex::new(VisualPropertiesThrottle::new(
        doc_sender.clone(),
        params.viewport,
    )));
    let visual_properties_ack: Arc<dyn Fn() + Send + Sync> = {
        let visual_properties = visual_properties.clone();
        Arc::new(move || visual_properties.lock().unwrap().ack())
    };
    let requested = Arc::new(AtomicBool::new(false));
    let source = Arc::new(FrameDemand {
        requested: requested.clone(),
        frame_source: params.source.clone(),
        compositor: impl_sender.clone(),
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
    params.routes.lock().unwrap().insert(
        document,
        Arc::new(FrameRoute {
            sender: impl_sender.clone(),
            visible: visible.clone(),
        }),
    );
    let impl_reporter = reporter.clone();
    let impl_self = impl_sender.clone();
    let impl_source = params.source.clone();
    let impl_frames = params.frames.clone();
    let impl_executor = threads.compositor_executor.clone();
    let impl_raster_executor = threads.raster_executor.clone();
    let impl_viewport = params.viewport;
    let impl_transparent = params.transparent;
    std::thread::Builder::new()
        .name(format!("page-compositor-{}-{}", document.0, document.1))
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            crate::engine::set_presentation_thread_priority();
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
                    impl_transparent,
                    impl_executor,
                    impl_raster_executor,
                    impl_source,
                    impl_frames,
                    visual_properties_ack,
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
    let doc_impl_shutdown = impl_sender.clone();
    let routes = params.routes.clone();
    let doc_reporter = reporter.clone();
    let doc_spawn = std::thread::Builder::new()
        .name(format!("page-document-{}-{}", document.0, document.1))
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            crate::engine::set_presentation_thread_priority();
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
            // The document is no longer a frame producer, including when its
            // task failed or panicked. Host teardown remains asynchronous.
            let _ = doc_impl_shutdown.send(ImplCommand::Stop);
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
        failure,
        next_patch_revision: AtomicU64::new(0),
        visual_properties,
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
    input_document_generation: u64,
    cursor_control:
        interaction::input::platform::widget::input::cursor_control_handler::CursorControlHandler,
    gestures: GestureEventQueue,
    gesture_touch_action: Option<GestureTouchActionFilterHandle>,
    pipeline: PageRenderPipeline,
    frames: mpsc::Sender<window_runtime::Command>,
    document: Document,
    document_sender: mpsc::Sender<Command>,
    /// Only requests not yet bound to a cc submission live here. The
    /// document's WidgetPresentationTiming owns token matching and retries.
    pending_event_timing_requests: Vec<u64>,
    reclaim: Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
    /// VideoFrameSubmitter runs while document resource mutations are applied,
    /// one sequence before this impl-owned parent producer. Keep each latest
    /// delegated Surface frame until a root CompositorFrame can carry its
    /// graph reference into the window-owned Viz service.
    pending_child_frames: VecDeque<PageChildCompositorFrame>,
    artifact: Option<Arc<paint::PaintArtifact>>,
    content: Option<ContentFrameProvenance>,
    viewport: Viewport,
    needs_publish: bool,
    revision_redraw_pending: bool,
    copy_output_redraw_pending: bool,
    required_main_frame_epoch: u64,
    responded_main_frame_epoch: u64,
    required_scroll_sequence: u64,
    pending_resize: Option<ResizeProvenance>,
    visual_properties_ack: Arc<dyn Fn() + Send + Sync>,
    presentation_feedback: Arc<dyn Fn(browser::page::PageFramePresentationFeedback) + Send + Sync>,
}

impl ImplPresentation {
    fn new(
        pipeline: PageRenderPipeline,
        frames: mpsc::Sender<window_runtime::Command>,
        document: Document,
        document_sender: mpsc::Sender<Command>,
        reclaim: Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync>,
        viewport: Viewport,
        visual_properties_ack: Arc<dyn Fn() + Send + Sync>,
        presentation_feedback: Arc<
            dyn Fn(browser::page::PageFramePresentationFeedback) + Send + Sync,
        >,
    ) -> Self {
        Self {
            cursor_control: Default::default(),
            input_document_generation: 1,
            gestures: GestureEventQueue::default(),
            gesture_touch_action: None,
            presentation_feedback,
            pipeline,
            frames,
            document,
            document_sender,
            pending_event_timing_requests: Vec::new(),
            reclaim,
            pending_child_frames: VecDeque::new(),
            artifact: None,
            content: None,
            viewport,
            needs_publish: false,
            revision_redraw_pending: false,
            copy_output_redraw_pending: false,
            required_main_frame_epoch: 0,
            responded_main_frame_epoch: 0,
            required_scroll_sequence: 0,
            pending_resize: None,
            visual_properties_ack,
        }
    }

    fn require_main_frame(&mut self, epoch: u64) {
        self.required_main_frame_epoch = self.required_main_frame_epoch.max(epoch);
    }

    fn event_timing_did_not_swap(&mut self) {
        if !self.pending_event_timing_requests.is_empty() {
            let _ = self.document_sender.send(Command::EventTimingDidNotSwap {
                requests: std::mem::take(&mut self.pending_event_timing_requests),
                swap_time: Instant::now(),
            });
        }
    }

    fn deliver_input_effects(
        &mut self,
        effects: Vec<CompositorMainThreadEffect>,
        main: &mpsc::Sender<Command>,
        epoch: &mut u64,
    ) -> Option<u64> {
        self.observe_main_thread_effects(&effects);
        let requested = effects
            .iter()
            .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame));
        let requested_epoch = requested.then(|| {
            *epoch = epoch.wrapping_add(1).max(1);
            self.require_main_frame(*epoch);
            *epoch
        });
        if !effects.is_empty() {
            let _ = main.send(Command::CompositorEffects {
                effects,
                main_frame_epoch: requested_epoch,
            });
        }
        requested_epoch
    }

    /// Assembly only: the input package owns the curve and event generation;
    /// this Page producer routes typed values through its existing ingress.
    fn deliver_fling_output(
        &mut self,
        output: FlingControllerOutput,
        main: &mpsc::Sender<Command>,
        epoch: &mut u64,
    ) -> Result<(), RuntimeFailure> {
        for generated in output.generated {
            match generated {
                GeneratedFlingEvent::Gesture(mut gesture) => {
                    if let Some(filter) = &self.gesture_touch_action {
                        if filter.FilterGeneratedGesture(&mut gesture) {
                            continue;
                        }
                    }
                    let now = self.pipeline.InputTimeForInstant(Instant::now());
                    if let Some(dispatch) = self.gestures.DebounceOrForwardEvent(gesture, (), now) {
                        self.forward_gesture(dispatch, main, epoch)?;
                    }
                }
                GeneratedFlingEvent::Wheel(event) => {
                    let (mut wheel, result) = self.pipeline.CanonicalFlingWheel(&event)?;
                    wheel.compositor_owned = result.compositor_owned;
                    wheel.compositor_nonblocking = result.dom_ack.is_none();
                    if let Some(request) = result.dom_ack {
                        wheel.compositor_ack.push(request);
                    }
                    self.gesture_acks(&result.gesture_acks, main);
                    self.observe_main_thread_effects(&result.effects);
                    let requested = result
                        .effects
                        .iter()
                        .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame));
                    let main_frame_epoch = requested.then(|| {
                        *epoch = epoch.wrapping_add(1).max(1);
                        self.require_main_frame(*epoch);
                        *epoch
                    });
                    let _ = main.send(Command::ExternalCanonicalWheel {
                        document_generation: self.input_document_generation,
                        event,
                        wheel,
                        queued_at: Instant::now(),
                        effects: result.effects,
                        main_frame_epoch,
                    });
                }
            }
        }
        if let Some(event) = output.acknowledged {
            let _ = main.send(Command::FlingDisposition(event));
        }
        Ok(())
    }

    fn gesture_acks(&mut self, acks: &[(u64, bool)], main: &mpsc::Sender<Command>) {
        for &(dispatch_id, handled) in acks {
            self.ack_gesture(
                dispatch_id,
                InputEventResultSource::kCompositorThread,
                if handled {
                    InputEventResultState::kConsumed
                } else {
                    InputEventResultState::kNotConsumed
                },
                main,
            );
        }
    }
    fn ack_gesture(
        &mut self,
        dispatch_id: u64,
        source: InputEventResultSource,
        result: InputEventResultState,
        main: &mpsc::Sender<Command>,
    ) {
        for ack in self.gestures.ProcessGestureAck(dispatch_id, source, result) {
            let _ = main.send(Command::GestureDisposition(ack));
        }
    }
    fn forward_gesture(
        &mut self,
        dispatch: ForwardedGesture<()>,
        main: &mpsc::Sender<Command>,
        epoch: &mut u64,
    ) -> Result<(), RuntimeFailure> {
        use interaction::input::platform::widget::input::cursor_control_handler::CursorControlDisposition;
        match self.cursor_control.ObserveInputEvent(&dispatch.event) {
            Some(CursorControlDisposition::MainThread) => {
                let _ = main.send(Command::ExternalGesture {
                    document_generation: self.input_document_generation,
                    dispatch_id: dispatch.dispatch_id,
                    gesture: dispatch.event,
                });
                return Ok(());
            }
            Some(CursorControlDisposition::Drop) => {
                self.ack_gesture(
                    dispatch.dispatch_id,
                    InputEventResultSource::kCompositorThread,
                    InputEventResultState::kNoConsumerExists,
                    main,
                );
                return Ok(());
            }
            None => {}
        }
        if dispatch.event.IsScrollEvent() {
            let result = self
                .pipeline
                .GestureWithDispatchId(&dispatch.event, dispatch.dispatch_id)?;
            self.gesture_acks(&result.gesture_acks, main);
            self.deliver_input_effects(result.effects, main, epoch);
        } else {
            let _ = main.send(Command::ExternalGesture {
                document_generation: self.input_document_generation,
                dispatch_id: dispatch.dispatch_id,
                gesture: dispatch.event,
            });
        }
        Ok(())
    }
    fn expire_gesture_debounce(
        &mut self,
        now: interaction::input::base::time::time::TimeTicks,
        main: &mpsc::Sender<Command>,
        epoch: &mut u64,
    ) -> Result<(), RuntimeFailure> {
        let effects = self.gestures.ExpireDebounce(now, !self.viewport.valid());
        for effect in effects {
            match effect {
                DebounceEffect::Fling(output) => self.deliver_fling_output(output, main, epoch)?,
                DebounceEffect::Forward(dispatch) => self.forward_gesture(dispatch, main, epoch)?,
                DebounceEffect::NotDispatched(_) => {}
            }
        }
        Ok(())
    }

    fn main_frame_pending(&self) -> bool {
        self.required_main_frame_epoch > self.responded_main_frame_epoch
    }

    fn finish_main_frame_without_updates(
        &mut self,
        main_frame_epoch: u64,
    ) -> Result<Option<CompositorWheelResult>, RuntimeFailure> {
        if main_frame_epoch == 0 || main_frame_epoch <= self.responded_main_frame_epoch {
            return Ok(None);
        }
        if main_frame_epoch > self.required_main_frame_epoch {
            return Err(RuntimeFailure {
                stage: "compositor",
                error: io::Error::other(format!(
                    "BeginMainFrame epoch {main_frame_epoch} has no matching scheduler request {}",
                    self.required_main_frame_epoch
                )),
            });
        }
        let result = self.pipeline.FinishMainFrameWithoutUpdates()?;
        self.responded_main_frame_epoch = self.responded_main_frame_epoch.max(main_frame_epoch);
        Ok(Some(result))
    }

    fn observe_main_thread_effects(&mut self, effects: &[CompositorMainThreadEffect]) {
        self.send_event_timing_commit_finishes();
        let _target_scope = browser_tracing::enabled().then(|| {
            browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            })
        });
        for effect in effects {
            if let CompositorMainThreadEffect::ScrollOffsetChanged {
                sequence,
                element,
                offset,
                ..
            } = effect
            {
                self.required_scroll_sequence = self.required_scroll_sequence.max(*sequence);
                // DidCompositorScroll is the canonical impl-side mutation.
                // Main-thread Page::ScrollApplied observes only its later
                // reflection, so record the resident active-tree offset here
                // as well; otherwise compositor-owned scroll motion is absent
                // from the smoothness report even though pixels moved.
                browser_tracing::instant(
                    "input",
                    "ScrollApplied",
                    &[
                        ("scroll_node", element.GetInternalValue() as f64),
                        ("offset_x", f64::from(offset.x())),
                        ("offset_y", f64::from(offset.y())),
                        ("sequence", *sequence as f64),
                        ("compositor_owned", 1.0),
                    ],
                );
            }
        }
    }

    fn send_event_timing_commit_finishes(&mut self) {
        for commit_finish_time in self.pipeline.TakeCommitFinishTimes() {
            let _ = self
                .document_sender
                .send(Command::EventTimingDidCommit(commit_finish_time));
        }
    }

    fn observe_resize(&mut self, resize: Option<ResizeProvenance>) {
        merge_resize(&mut self.pending_resize, resize);
    }

    fn needs_main_frame_for(&self, content: ContentFrameProvenance) -> bool {
        self.artifact.is_some()
            && content.main_frame_epoch <= self.responded_main_frame_epoch
            && self.required_main_frame_epoch <= self.responded_main_frame_epoch
    }

    fn is_unsolicited(&self, content: ContentFrameProvenance) -> bool {
        self.artifact.is_some() && content.main_frame_epoch <= self.responded_main_frame_epoch
    }

    fn queue_child_frames(&mut self, frames: Vec<PageChildCompositorFrame>) {
        for frame in frames {
            if let Some(pending) = self
                .pending_child_frames
                .iter_mut()
                .find(|pending| pending.surface_id == frame.surface_id)
            {
                // A delegated Surface has one pending frame. Dropping the
                // superseded move-only frame releases its immutable SharedImage
                // lease before it ever crosses the FrameSink boundary.
                *pending = frame;
            } else {
                self.pending_child_frames.push_back(frame);
            }
        }
    }

    fn commit(
        &mut self,
        artifact: Arc<paint::PaintArtifact>,
        child_frames: Vec<PageChildCompositorFrame>,
        mut content: ContentFrameProvenance,
        viewport: Viewport,
    ) -> bool {
        // Queue the child independently of main-frame admission. A rejected
        // stale artifact may be followed by a newer parent that references the
        // same stable video Surface, and Chromium retains that latest child
        // frame at the FrameSink boundary rather than dropping it with the
        // stale main-tree response.
        self.queue_child_frames(child_frames);
        // ProxyImpl::data_for_commit_ pairs a commit with the BeginMainFrame
        // that produced it. An older Page artifact cannot satisfy a newer BMF:
        // doing so advances SyncedScrollOffset's reflected slot without the
        // corresponding main-thread delta and rolls the active offset back.
        // The active tree keeps drawing while the matching artifact is built.
        // Source: cc/trees/proxy_impl.cc:450-501,801-814.
        if content.main_frame_epoch < self.required_main_frame_epoch
            || content.compositor_scroll_sequence < self.required_scroll_sequence
        {
            return false;
        }
        // Chromium attaches CopyOutputRequest to a compositor render pass and
        // schedules a draw even when the main-thread PaintArtifact is retained
        // byte-for-byte. The document fence above supplies the fresh commit;
        // this flag supplies only its normal impl-side SetNeedsRedraw, without
        // inventing paint damage or a second readback path.
        if self.copy_output_redraw_pending {
            self.revision_redraw_pending = true;
            self.copy_output_redraw_pending = false;
        }
        self.revision_redraw_pending |= self.content.is_some_and(|previous| {
            previous.embedder_revision != content.embedder_revision
                && self
                    .artifact
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, &artifact))
                && self.viewport == viewport
        });
        content.resize = self.pending_resize.take();
        self.artifact = Some(artifact);
        self.content = Some(content);
        self.viewport = viewport;
        self.needs_publish = true;
        true
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
        if self.revision_redraw_pending {
            self.pipeline.RequestRedraw()?;
            self.revision_redraw_pending = false;
        }
        self.needs_publish = false;
        self.flush_frames()
    }

    fn flush_frames(&mut self) -> Result<bool, RuntimeFailure> {
        // Completion originates at cc's FinishCommit, including asynchronous
        // commits progressed by Pump. Preserve its timestamp and send it
        // before any swap notification for frames produced in the same turn.
        self.send_event_timing_commit_finishes();
        let Some(content) = self.content else {
            return Ok(false);
        };
        let frames = self.pipeline.TakeFrames();
        if frames.is_empty() {
            // Child SurfaceDrawQuads are published atomically with the next
            // parent CompositorFrame. Do not drain their move-only resources
            // while no parent can carry the reference into external Viz.
            return Ok(false);
        }
        let pipeline_child_frames = self.pipeline.TakeChildFrames();
        self.queue_child_frames(pipeline_child_frames);
        let mut child_frames = self.pending_child_frames.drain(..).collect::<Vec<_>>();
        let mut submitted = false;
        for (index, frame) in frames.into_iter().enumerate() {
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
            let is_handling_interaction = frame.frame.metadata.is_handling_interaction;
            // ReportTimeSwapPromise::DidSwap runs on the compositor sequence
            // for this exact frame token. Post it before handing the frame to
            // Display: a native presenter may return feedback synchronously.
            // This is a swap timestamp, never a presentation timestamp.
            if !self.pending_event_timing_requests.is_empty() {
                let _ = self.document_sender.send(Command::EventTimingDidSwap {
                    requests: std::mem::take(&mut self.pending_event_timing_requests),
                    frame_token: frame.frame.metadata.frame_token,
                    swap_time: Instant::now(),
                });
            }
            let command = window_runtime::Command::Display(crate::display::Message::Prepare {
                submitted: PageFrame {
                    provenance: FrameProvenance {
                        document: self.document,
                        content: Some(content),
                    },
                    viewport,
                    is_handling_interaction,
                    // A flush normally contains one root. If cc queued
                    // more than one, submitting every child before the
                    // first parent preserves Viz's child-before-parent
                    // activation invariant for the complete batch.
                    child_frames: if index == 0 {
                        std::mem::take(&mut child_frames)
                    } else {
                        Vec::new()
                    },
                    frame,
                    reclaim: self.reclaim.clone(),
                    // Delegated video resources are immutable SharedImage
                    // leases created outside cc's ResourceProvider. Viz owns
                    // the sole strong frame reference; replacement/destruction
                    // drops that lease directly, so returned ids must not be
                    // routed into the parent cc provider where their numeric
                    // namespace could collide with a tile resource.
                    child_reclaim: Arc::new(|_| {}),
                    visual_properties_ack: self.visual_properties_ack.clone(),
                    presentation_feedback: self.presentation_feedback.clone(),
                },
            });
            if let Err(mpsc::SendError(window_runtime::Command::Display(
                crate::display::Message::Prepare { submitted },
            ))) = self.frames.send(command)
            {
                submitted.ReclaimAll();
                return Err(RuntimeFailure {
                    stage: "compositor",
                    error: io::Error::other("window display stopped"),
                });
            }
            submitted = true;
        }
        if submitted {
            if let Some(content) = &mut self.content {
                content.resize = None;
            }
        }
        Ok(submitted)
    }
}

impl Drop for ImplPresentation {
    fn drop(&mut self) {
        // Break unswapped promises on executor failure or teardown as well as
        // explicit Stop. Already-swapped requests remain owned solely by the
        // widget timing owner and die with that widget's weak registrations.
        self.event_timing_did_not_swap();
    }
}

fn merge_resize(target: &mut Option<ResizeProvenance>, incoming: Option<ResizeProvenance>) {
    let Some(incoming) = incoming else {
        return;
    };
    if let Some(current) = target {
        current.merge(incoming);
    } else {
        *target = Some(incoming);
    }
}

fn only_begins_main_frame(effects: &[CompositorMainThreadEffect]) -> bool {
    !effects.is_empty()
        && effects
            .iter()
            .all(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame))
}

// The Page impl executor owns cc scheduling and input. Its move-only frame
// output crosses the Page FrameSink queue to the window-owned Viz sequence.
fn run_impl(
    receiver: mpsc::Receiver<ImplCommand>,
    self_sender: mpsc::Sender<ImplCommand>,
    main: mpsc::Sender<Command>,
    requested: Arc<AtomicBool>,
    document: Document,
    viewport: Viewport,
    transparent: bool,
    executor: Arc<dyn executor::Executor>,
    raster_executor: Arc<dyn executor::Executor>,
    source: Arc<dyn BeginFrameSource>,
    frames: mpsc::Sender<window_runtime::Command>,
    visual_properties_ack: Arc<dyn Fn() + Send + Sync>,
) -> Result<(), RuntimeFailure> {
    let pump_pending = Arc::new(AtomicBool::new(false));
    let pump_generation = Arc::new(AtomicU64::new(0));
    let compositor_executor: Arc<dyn executor::Executor> = Arc::new(WakingExecutor::new(
        executor,
        self_sender.clone(),
        pump_pending.clone(),
        pump_generation.clone(),
        "CompositorExecutor.Task",
    ));
    let raster_executor: Arc<dyn executor::Executor> = Arc::new(WakingExecutor::new(
        raster_executor,
        self_sender.clone(),
        pump_pending.clone(),
        pump_generation.clone(),
        "RasterExecutor.Task",
    ));
    let mut pipeline =
        PageRenderPipeline::new_external(compositor_executor, raster_executor, transparent);
    let _ = main.send(Command::BindCompositorPresentationClock(
        pipeline.PresentationClockMapping(),
    ));
    pipeline.InitializeViewport(viewport.width, viewport.height, viewport.scale)?;
    let reclaim_sender = self_sender.clone();
    let reclaim: Arc<dyn Fn(Vec<PageReturnedResource>) + Send + Sync> =
        Arc::new(move |resources| {
            let _ = reclaim_sender.send(ImplCommand::Reclaim(resources));
        });
    let presentation_sender = self_sender.clone();
    let presentation_feedback: Arc<
        dyn Fn(browser::page::PageFramePresentationFeedback) + Send + Sync,
    > = Arc::new(move |feedback| {
        let _ = presentation_sender.send(ImplCommand::PresentationFeedback(feedback));
    });
    let mut presentation = ImplPresentation::new(
        pipeline,
        frames,
        document,
        main.clone(),
        reclaim,
        viewport,
        visual_properties_ack,
        presentation_feedback,
    );
    let mut main_frame_epoch = 0u64;
    let mut queued_commands = VecDeque::new();
    // Once input from a snapshot has run ahead of an already observed
    // BeginFrame, later arrivals cannot overtake that frame. Chromium's
    // CompositorThreadEventQueue similarly closes a batch at BeginFrame.
    let mut tick_due_after_input = false;
    'commands: loop {
        if queued_commands.is_empty() {
            let command = if let Some(deadline) = presentation.gestures.NextDebounceDeadline() {
                let now = presentation.pipeline.InputTimeForInstant(Instant::now());
                let delay = Duration::from_micros(
                    deadline
                        .ToInternalValue()
                        .saturating_sub(now.ToInternalValue())
                        .max(0) as u64,
                );
                match receiver.recv_timeout(delay) {
                    Ok(command) => command,
                    Err(mpsc::RecvTimeoutError::Timeout) => ImplCommand::DebounceExpired,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            } else {
                let Ok(command) = receiver.recv() else {
                    break;
                };
                command
            };
            queued_commands.push_back(command);
        }
        while let Ok(command) = receiver.try_recv() {
            queued_commands.push_back(command);
        }
        let now = presentation.pipeline.InputTimeForInstant(Instant::now());
        presentation.expire_gesture_debounce(now, &main, &mut main_frame_epoch)?;

        // Source: cc/scheduler/compositor_priority_experiments.h and
        // content/renderer/input/input_handler_proxy.cc. Input on the impl
        // sequence outranks ordinary compositor tasks. BeginFrames do not
        // accumulate: when the sequence was busy only the newest physical
        // opportunity is useful. Executor wakeups are likewise level
        // triggered because Poll drains every currently posted cc command.
        let drained = queued_commands.drain(..).collect::<Vec<_>>();
        let last_tick = drained
            .iter()
            .rposition(|pending| matches!(pending, ImplCommand::Tick(_)));
        let last_commit = drained
            .iter()
            .rposition(|pending| matches!(pending, ImplCommand::Commit { .. }));
        let mut compact = VecDeque::with_capacity(drained.len());
        let mut has_pump = false;
        for (index, pending) in drained.into_iter().enumerate() {
            match pending {
                // BeginFrames are latest-value opportunities, but the newest
                // one retains its source-sequence position. Moving it to the
                // tail can let a later deadline Pump overtake an earlier
                // physical frame; moving it to the front can let it overtake
                // a ready main-thread commit.
                ImplCommand::Tick(_) if Some(index) != last_tick => {}
                // Page can produce several immutable artifacts while the impl
                // sequence is busy. ProxyMain only needs the newest response
                // that includes all preceding Page state; admitting every
                // intermediate artifact makes the emulated main phase block
                // impl-thread input repeatedly. The surviving commit must keep
                // its position relative to Scheduler's deadline Pump and the
                // next physical Tick: Chromium's single impl sequence never
                // reverses those already-ready tasks.
                ImplCommand::Commit { .. } if Some(index) != last_commit => {}
                ImplCommand::Pump if has_pump => {}
                ImplCommand::Pump => {
                    has_pump = true;
                    compact.push_back(ImplCommand::Pump);
                }
                ImplCommand::Wheel {
                    wheel,
                    queued_at,
                    samples,
                } => {
                    let mut incoming = Some((wheel, queued_at, samples));
                    for existing in compact.iter_mut().rev() {
                        let ImplCommand::Wheel {
                            wheel: previous,
                            queued_at: previous_at,
                            samples: previous_samples,
                        } = existing
                        else {
                            break;
                        };
                        let (next, next_at, next_samples) = incoming.as_ref().unwrap();
                        if matches!(previous.phase, WheelPhase::kNone | WheelPhase::kChanged)
                            && matches!(next.phase, WheelPhase::kNone | WheelPhase::kChanged)
                            && previous.CanCoalesce(next)
                            && previous.position == next.position
                            && previous.target_node_id == next.target_node_id
                            && (previous.delta.x + next.delta.x).is_finite()
                            && (previous.delta.y + next.delta.y).is_finite()
                        {
                            previous.delta.x += next.delta.x;
                            previous.delta.y += next.delta.y;
                            previous.native = next.native;
                            previous.timestamp = next.timestamp;
                            *previous_at = (*previous_at).min(*next_at);
                            *previous_samples = previous_samples.saturating_add(*next_samples);
                            incoming = None;
                        }
                        break;
                    }
                    if let Some((wheel, queued_at, samples)) = incoming {
                        compact.push_back(ImplCommand::Wheel {
                            wheel,
                            queued_at,
                            samples,
                        });
                    }
                }
                pending => compact.push_back(pending),
            }
        }
        // cc's pending-tree/ack state decides when a main frame can commit.
        // A scroll gesture is not a commit barrier: delaying its response until
        // WheelPhase::kEnded also stalls scroll events and scroll-linked paint.
        // Source: cc/scheduler/scheduler_state_machine.cc
        //         ShouldSendBeginMainFrame / ShouldCommit.
        queued_commands = compact;

        // A document replacement is an input-order barrier. Prioritized
        // input may run ahead of ordinary tasks within its original document,
        // but may not overtake that document's target teardown.
        let input_boundary = queued_commands
            .iter()
            .position(|command| matches!(command, ImplCommand::DocumentDetached { .. }))
            .unwrap_or(queued_commands.len());
        let pending_tick = queued_commands
            .iter()
            .take(input_boundary)
            .position(|command| matches!(command, ImplCommand::Tick(_)));
        let urgent = queued_commands
            .iter()
            .position(|command| matches!(command, ImplCommand::Stop))
            .or_else(|| tick_due_after_input.then_some(pending_tick).flatten())
            .or_else(|| {
                queued_commands
                    .iter()
                    .take(input_boundary)
                    .position(|command| matches!(command, ImplCommand::Wheel { .. }))
            })
            .unwrap_or(0);
        let command = queued_commands
            .remove(urgent)
            .expect("selected queued impl command");
        if matches!(command, ImplCommand::Wheel { .. }) && pending_tick.is_some() {
            tick_due_after_input = true;
        } else if matches!(command, ImplCommand::Tick(_)) {
            tick_due_after_input = false;
        }
        match command {
            ImplCommand::BindGestureTouchActionFilter(filter) => {
                presentation.gesture_touch_action = Some(filter)
            }
            ImplCommand::WheelAck {
                document_generation,
                result,
            } => {
                if document_generation == presentation.input_document_generation {
                    presentation.gestures.OnWheelEventAck(result);
                }
            }
            ImplCommand::GestureAck {
                dispatch_id,
                result,
            } => {
                presentation.ack_gesture(
                    dispatch_id,
                    InputEventResultSource::kMainThread,
                    result,
                    &main,
                );
            }
            ImplCommand::DebounceExpired => {
                let now = presentation.pipeline.InputTimeForInstant(Instant::now());
                presentation.expire_gesture_debounce(now, &main, &mut main_frame_epoch)?;
                let _ = presentation.flush_frames()?;
            }
            ImplCommand::DocumentDetached { generation } => {
                presentation.input_document_generation = generation;
                presentation.cursor_control = Default::default();
                // Retire target-bound input without publishing StopFling's
                // old synthetic WheelEnd to the new document owner.
                for ack in presentation.gestures.DocumentDetached() {
                    let _ = main.send(Command::GestureDisposition(ack));
                }
            }
            ImplCommand::StopFling => {
                let now = presentation.pipeline.InputTimeForInstant(Instant::now());
                let output = presentation.gestures.StopFling(now);
                presentation.deliver_fling_output(output, &main, &mut main_frame_epoch)?;
                let _ = presentation.flush_frames()?;
            }
            ImplCommand::Stop => {
                presentation.event_timing_did_not_swap();
                break 'commands;
            }
            ImplCommand::EventTimingRequests {
                requests,
                request_main_frame,
            } => {
                // Frames already produced before this request belong to the
                // preceding swap promises. Drain them before admitting it.
                let _ = presentation.flush_frames()?;
                presentation.pending_event_timing_requests.extend(requests);
                // A matching Commit or no-update response follows requests
                // from a rendering turn. Do not create a newer BMF before
                // that response arrives: it would reject the valid commit.
                if request_main_frame {
                    let _ = self_sender.send(ImplCommand::RequestEventTimingFrame);
                }
            }
            ImplCommand::Pump => {
                let _impl_trace = browser_tracing::span("compositor", "Impl.Pump");
                // Keep the wake marked pending for the entire Poll. Otherwise
                // every compositor/raster callback completed during Poll can
                // enqueue another Pump ahead of input. Chromium coalesces
                // origin-sequence wakeups and posts one follow-up only when
                // work arrived after the drain began.
                let observed_generation = pump_generation.load(Ordering::Acquire);
                let profile = std::env::var_os("BROWSER_PROFILE_INPUT")
                    .is_some()
                    .then(Instant::now);
                let result = presentation.pipeline.Pump()?;
                if let Some(started) = profile {
                    eprintln!(
                        "page-compositor-pump-profile total_ms={:.3} submitted={} effects={}",
                        started.elapsed().as_secs_f64() * 1000.0,
                        result.submitted,
                        result.effects.len(),
                    );
                }
                presentation.gesture_acks(&result.gesture_acks, &main);
                if !result.effects.is_empty() {
                    presentation.observe_main_thread_effects(&result.effects);
                    let requested = result
                        .effects
                        .iter()
                        .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame));
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
                // AcceptOutput already moved any produced compositor frame
                // into the Page FrameSink queue. A submitted impl frame is
                // not a new PaintArtifact commit; publish() will drain it
                // through flush_frames() while preserving an independently
                // pending main-thread commit, if one exists.
                let _ = presentation.publish()?;
                pump_pending.store(false, Ordering::Release);
                if pump_generation.load(Ordering::Acquire) != observed_generation
                    && !pump_pending.swap(true, Ordering::AcqRel)
                    && self_sender.send(ImplCommand::Pump).is_err()
                {
                    pump_pending.store(false, Ordering::Release);
                }
            }
            request @ (ImplCommand::RequestMainFrame | ImplCommand::RequestEventTimingFrame) => {
                if matches!(request, ImplCommand::RequestEventTimingFrame)
                    && (presentation.pending_event_timing_requests.is_empty()
                        || presentation.main_frame_pending())
                {
                    continue;
                }
                let _impl_trace = browser_tracing::span("compositor", "Impl.RequestMainFrame");
                let result = presentation.pipeline.RequestMainFrame()?;
                presentation.gesture_acks(&result.gesture_acks, &main);
                if !result.effects.is_empty() {
                    let direct_main_frame = only_begins_main_frame(&result.effects);
                    presentation.observe_main_thread_effects(&result.effects);
                    let requested_main_frame = result
                        .effects
                        .iter()
                        .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame));
                    let epoch = requested_main_frame.then(|| {
                        main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                        presentation.require_main_frame(main_frame_epoch);
                        main_frame_epoch
                    });
                    if direct_main_frame {
                        // ProxyImpl posts a pure BeginMainFrame directly;
                        // there are no compositor deltas for Page to apply
                        // in a preceding task.
                        requested.store(true, Ordering::Release);
                        source.request_begin_frame();
                    } else {
                        let _ = main.send(Command::CompositorEffects {
                            effects: result.effects,
                            main_frame_epoch: epoch,
                        });
                    }
                }
                let _ = presentation.publish()?;
            }
            ImplCommand::CopyOutputFence => {
                // InputHandlerProxy and ProxyMain live on different sequences.
                // Forwarding the fence from here preserves every earlier impl
                // input/callback before the document asks cc for the capture
                // transaction; the document queue supplies the corresponding
                // ordering for direct mouse/key/text input.
                presentation.copy_output_redraw_pending = true;
                let _ = main.send(Command::CopyOutputFence);
            }
            ImplCommand::FinishMainFrameWithoutUpdates {
                main_frame_epoch: responded_epoch,
            } => {
                let mut impl_trace =
                    browser_tracing::span("compositor", "Impl.FinishMainFrameWithoutUpdates");
                impl_trace.set("main_frame_epoch", responded_epoch as f64);
                impl_trace.set(
                    "required_main_frame_epoch",
                    presentation.required_main_frame_epoch as f64,
                );
                impl_trace.set(
                    "responded_main_frame_epoch",
                    presentation.responded_main_frame_epoch as f64,
                );
                if let Some(result) =
                    presentation.finish_main_frame_without_updates(responded_epoch)?
                {
                    presentation.gesture_acks(&result.gesture_acks, &main);
                    presentation.observe_main_thread_effects(&result.effects);
                    let direct_main_frame = only_begins_main_frame(&result.effects);
                    let requested_main_frame = result
                        .effects
                        .iter()
                        .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame));
                    let epoch = requested_main_frame.then(|| {
                        main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                        presentation.require_main_frame(main_frame_epoch);
                        main_frame_epoch
                    });
                    if direct_main_frame {
                        requested.store(true, Ordering::Release);
                        source.request_begin_frame();
                    } else if !result.effects.is_empty() {
                        let _ = main.send(Command::CompositorEffects {
                            effects: result.effects,
                            main_frame_epoch: epoch,
                        });
                    }
                    let _ = presentation.publish()?;
                    presentation.event_timing_did_not_swap();
                }
            }
            ImplCommand::PresentationFeedback(feedback) => {
                // Keep the platform timestamp, interval, failure and flags
                // intact. WidgetPresentationTiming, on the document owner,
                // resolves only valid successful presentation feedback.
                let _ = main.send(Command::EventTimingDidPresent(feedback));
                let result = presentation
                    .pipeline
                    .ReceivePresentationFeedback(feedback)?;
                presentation.gesture_acks(&result.gesture_acks, &main);
                presentation.observe_main_thread_effects(&result.effects);
                if !result.effects.is_empty() {
                    let _ = main.send(Command::CompositorEffects {
                        effects: result.effects,
                        main_frame_epoch: None,
                    });
                }
                let _ = presentation.flush_frames()?;
            }
            ImplCommand::Reclaim(resources) => {
                let _impl_trace = browser_tracing::span("compositor", "Impl.Reclaim");
                presentation.pipeline.ReclaimResources(resources)?;
                let _ = presentation.flush_frames()?;
            }
            ImplCommand::Tick(frame) => {
                let _impl_trace = browser_tracing::span("compositor", "Impl.Tick");
                let now = presentation.pipeline.InputTimeForInstant(frame.frame_time);
                let output = presentation.gestures.ProgressFling(now, None);
                presentation.deliver_fling_output(output, &main, &mut main_frame_epoch)?;
                if presentation.pipeline.NeedsBeginFrames() {
                    let result = presentation.pipeline.BeginPhysicalFrame(frame.args)?;
                    presentation.gesture_acks(&result.gesture_acks, &main);
                    if !result.effects.is_empty() {
                        let direct_main_frame = only_begins_main_frame(&result.effects);
                        presentation.observe_main_thread_effects(&result.effects);
                        let requested_main_frame = result.effects.iter().any(|effect| {
                            matches!(effect, CompositorMainThreadEffect::BeginMainFrame)
                        });
                        let epoch = requested_main_frame.then(|| {
                            main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                            presentation.require_main_frame(main_frame_epoch);
                            main_frame_epoch
                        });
                        if direct_main_frame {
                            requested.store(true, Ordering::Release);
                        } else {
                            let _ = main.send(Command::CompositorEffects {
                                effects: result.effects,
                                main_frame_epoch: epoch,
                            });
                        }
                    }
                }
                let _ = presentation.publish()?;
                if requested.swap(false, Ordering::AcqRel) {
                    let main_frame_epoch = presentation.required_main_frame_epoch;
                    let _ = main.send(Command::Main(MainThreadEvent::BeginFrame {
                        frame,
                        main_frame_epoch,
                    }));
                }
            }
            ImplCommand::Commit {
                artifact,
                child_frames,
                content,
                viewport,
            } => {
                let _resize_scope = content.resize.map(|resize| {
                    browser_tracing::scope(browser_tracing::Context {
                        task_id: resize.id,
                        ..Default::default()
                    })
                });
                let mut impl_trace = browser_tracing::span("compositor", "Impl.Commit");
                presentation.observe_resize(content.resize);
                impl_trace.set("content_main_frame_epoch", content.main_frame_epoch as f64);
                impl_trace.set(
                    "required_main_frame_epoch",
                    presentation.required_main_frame_epoch as f64,
                );
                impl_trace.set(
                    "responded_main_frame_epoch",
                    presentation.responded_main_frame_epoch as f64,
                );
                impl_trace.set(
                    "content_scroll_sequence",
                    content.compositor_scroll_sequence as f64,
                );
                impl_trace.set(
                    "required_scroll_sequence",
                    presentation.required_scroll_sequence as f64,
                );
                // Chromium admits a main-tree update to ProxyImpl only as
                // the response to a dispatched BeginMainFrame.  Page may
                // produce animation artifacts between those transactions;
                // committing one directly would advance SyncedProperty's
                // reflected slot with an old absolute scroll base.
                let request_main = presentation.needs_main_frame_for(content);
                if request_main {
                    let result = presentation.pipeline.RequestMainFrame()?;
                    presentation.gesture_acks(&result.gesture_acks, &main);
                    presentation.observe_main_thread_effects(&result.effects);
                    let requested = result
                        .effects
                        .iter()
                        .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame));
                    let epoch = requested.then(|| {
                        main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                        presentation.require_main_frame(main_frame_epoch);
                        main_frame_epoch
                    });
                    if !result.effects.is_empty() {
                        let _ = main.send(Command::CompositorEffects {
                            effects: result.effects,
                            main_frame_epoch: epoch,
                        });
                    }
                }
                let unsolicited = presentation.is_unsolicited(content);
                impl_trace.set("unsolicited", unsolicited as u8 as f64);
                if !unsolicited {
                    let accepted = presentation.commit(artifact, child_frames, content, viewport);
                    impl_trace.set("accepted", accepted as u8 as f64);
                } else {
                    // The latest delegated video frame remains useful to the
                    // already-active parent Surface even when this duplicate
                    // main-tree artifact itself needs no commit.
                    presentation.queue_child_frames(child_frames);
                }
                let _ = presentation.publish()?;
            }
            ImplCommand::Gesture {
                gesture,
                queued_at,
                samples,
            } => {
                browser_tracing::interval(
                    "input",
                    "InputOwnerQueue",
                    queued_at,
                    Instant::now(),
                    &[("samples", samples as f64)],
                );
                let now = presentation.pipeline.InputTimeForInstant(Instant::now());
                let output = presentation.gestures.PassToFlingController(
                    &gesture,
                    now,
                    !presentation.viewport.valid(),
                );
                let consumed = output.consumed;
                presentation.deliver_fling_output(output, &main, &mut main_frame_epoch)?;
                if !consumed {
                    if let Some(dispatch) =
                        presentation
                            .gestures
                            .DebounceOrForwardEvent(gesture, (), now)
                    {
                        presentation.forward_gesture(dispatch, &main, &mut main_frame_epoch)?;
                    }
                }
                let _ = presentation.flush_frames()?;
            }
            ImplCommand::InputContinuation(input) => {
                let result = presentation.pipeline.ContinueCompositorMutation(input)?;
                assert!(
                    result.dom_ack.is_none(),
                    "source continuation cannot redispatch its DOM wheel"
                );
                presentation.gesture_acks(&result.gesture_acks, &main);
                presentation.observe_main_thread_effects(&result.effects);
                let requested = result
                    .effects
                    .iter()
                    .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame));
                let epoch = requested.then(|| {
                    main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                    presentation.require_main_frame(main_frame_epoch);
                    main_frame_epoch
                });
                if !result.effects.is_empty() {
                    let _ = main.send(Command::CompositorEffects {
                        effects: result.effects,
                        main_frame_epoch: epoch,
                    });
                }
                let _ = presentation.flush_frames()?;
            }
            ImplCommand::Wheel {
                mut wheel,
                queued_at,
                samples,
            } => {
                let _input_scope = browser_tracing::enabled().then(|| {
                    browser_tracing::scope(browser_tracing::Context {
                        target_id: 1,
                        input_id: browser_tracing::instant_id(queued_at),
                        ..Default::default()
                    })
                });
                let owner_started = Instant::now();
                browser_tracing::interval(
                    "input",
                    "InputOwnerQueue",
                    queued_at,
                    owner_started,
                    &[("samples", samples as f64)],
                );
                let mut input_trace = browser_tracing::span("input", "CompositorWheel");
                input_trace.set("samples", samples as f64);
                input_trace.set("delta_x", wheel.delta.x);
                input_trace.set("delta_y", wheel.delta.y);
                input_trace.set(
                    "phase",
                    match wheel.phase {
                        WheelPhase::kNone => 0.0,
                        WheelPhase::kBegan => 1.0,
                        WheelPhase::kChanged => 2.0,
                        WheelPhase::kEnded => 3.0,
                        WheelPhase::kCancelled => 4.0,
                    },
                );
                let mut result = presentation.pipeline.Wheel(&wheel)?;
                input_trace.set("compositor_owned", result.compositor_owned as u8 as f64);
                input_trace.set("submitted", result.submitted as u8 as f64);
                wheel.compositor_owned = result.compositor_owned;
                wheel.compositor_nonblocking = result.dom_ack.is_none();
                if let Some(request) = result.dom_ack {
                    wheel.compositor_ack.push(request);
                }
                // Blocking raw wheel awaits its canonical DOM ack. The
                // gesture and any main-thread hit-test callback return to this
                // same impl InputHandler; no main-thread offset fallback runs.
                presentation.gesture_acks(&result.gesture_acks, &main);
                presentation.observe_main_thread_effects(&result.effects);
                let requested = result
                    .effects
                    .iter()
                    .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame));
                let epoch = requested.then(|| {
                    main_frame_epoch = main_frame_epoch.wrapping_add(1).max(1);
                    presentation.require_main_frame(main_frame_epoch);
                    main_frame_epoch
                });
                let _ = main.send(Command::ExternalWheel {
                    document_generation: presentation.input_document_generation,
                    wheel,
                    queued_at,
                    samples,
                    effects: result.effects,
                    main_frame_epoch: epoch,
                });
                // InputHandler mutates the resident active tree immediately,
                // but an input callback is not a BeginFrame lifecycle.  In
                // Chromium, InputHandlerProxy can forward a compositor frame
                // which cc already produced, while a pending main-tree commit
                // remains scheduled for its BeginFrame.  Calling publish()
                // here also commits that pending PaintArtifact and can keep
                // the impl sequence inside commit/raster work for tens of
                // milliseconds while newer wheel samples queue behind it.
                // Forward only output already produced by this input; Tick or
                // the scheduler Pump owns the pending commit.
                //
                // Source: content/renderer/input/input_handler_proxy.cc;
                //         cc/trees/proxy_impl.cc ScheduledActionDraw.
                let _ = presentation.flush_frames()?;
            }
        }
        let effects = presentation.pipeline.TakeMainThreadEffects();
        if !effects.is_empty() {
            presentation.observe_main_thread_effects(&effects);
            let requested = effects
                .iter()
                .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame));
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
        if presentation.pipeline.NeedsPump()
            && !pump_pending.swap(true, Ordering::AcqRel)
            && self_sender.send(ImplCommand::Pump).is_err()
        {
            pump_pending.store(false, Ordering::Release);
        }
        if presentation.needs_publish
            || presentation.pipeline.NeedsBeginFrames()
            || presentation.gestures.fling_in_progress()
        {
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
    resize_queued: bool,
    pending_resize: Option<ResizeRequest>,
    patches_queued: bool,
    pending_patches: Vec<(u64, Patch)>,
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
            resize_queued: false,
            pending_resize: None,
            patches_queued: false,
            pending_patches: Vec::new(),
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
            self.schedule_collection(Instant::now());
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
        let command = match command {
            Command::Resize(resize) => {
                if let Some(pending) = &mut self.pending_resize {
                    pending.merge(resize);
                } else {
                    self.pending_resize = Some(resize);
                }
                if self.resize_queued {
                    return;
                }
                self.resize_queued = true;
                Command::ResizeTask
            }
            Command::Patch(mut patches) => {
                self.pending_patches.append(&mut patches);
                if self.patches_queued {
                    return;
                }
                self.patches_queued = true;
                Command::Patch(Vec::new())
            }
            command => command,
        };
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
                Command::CollectLayoutHeap | Command::CollectJavaScriptHeap => {
                    TaskPriority::Background
                }
                Command::Task | Command::Wake | Command::Evaluate { .. } => TaskPriority::Normal,
                _ => TaskPriority::UserBlocking,
            };
            self.post_task(command, priority, None, now);
        }
    }
    fn post_task(
        &mut self,
        command: Command,
        priority: event_loop::TaskPriority,
        ready_at: Option<Instant>,
        now: Instant,
    ) -> event_loop::TaskToken {
        use event_loop::*;
        self.token += 1;
        let token = TaskToken(self.token);
        self.engine.Apply(
            EventLoopMutation::PostTask(ScheduledTask {
                token,
                context: ExecutionContextId(1),
                priority,
                ready_at,
                payload: command,
            }),
            now,
        );
        token
    }
    fn schedule_collection(&mut self, now: Instant) {
        // cppgc posts its foreground incremental sweeper immediately at
        // kUserVisible priority and also posts a delayed kUserBlocking
        // fallback.  This single-owner heap needs only the first path: input
        // and rendering still run before Normal tasks, while continuous input
        // cannot cancel and perpetually postpone collection.
        self.post_task(
            Command::CollectLayoutHeap,
            event_loop::TaskPriority::Normal,
            None,
            now,
        );
        self.collection_queued = true;
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
                // A resource/worker completion is already an ordinary task,
                // not merely a request to ensure that some ordinary task
                // exists. Page admits one loading completion per task turn;
                // folding this into an already queued Task can consume that
                // turn on a different image/script response and permanently
                // hide this completion.
                Ok(Command::Wake) => self.post(Command::Wake),
                Ok(command) => self.post(command),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(mpsc::RecvTimeoutError::Disconnected);
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
                    Ok(Command::Wake) => self.post(Command::Wake),
                    Ok(command) => self.post(command),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        return Err(mpsc::RecvTimeoutError::Disconnected);
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
                } else if matches!(command, Command::ResizeTask) {
                    self.resize_queued = false;
                    self.ordinary();
                    return Ok(Command::Resize(
                        self.pending_resize
                            .take()
                            .expect("queued resize owns a viewport"),
                    ));
                } else if matches!(command, Command::Patch(_)) {
                    self.patches_queued = false;
                    self.ordinary();
                    return Ok(Command::Patch(std::mem::take(&mut self.pending_patches)));
                } else if matches!(command, Command::Wake) {
                    // The completion itself is Page's ordinary task. Work
                    // left by the bounded turn can enqueue its own next wake.
                } else {
                    self.ordinary();
                }
                return Ok(command);
            }
        }
    }
}
struct Client {
    document_sender: mpsc::Sender<Command>,
    controls: bool,
    native_drag_window: bool,
    accepts_load_drops: bool,
    primary_clipboard: Arc<crate::primary_clipboard::PrimaryClipboard>,
    document: Document,
    events: Arc<dyn Fn(Document, Event) + Send + Sync>,
    reporter: ErrorReporter,
}

fn loading_wake_callback(wake: mpsc::Sender<Command>) -> url_loader::URLWakeCallback {
    Arc::new(move || {
        // Resource and compilation completions are distinct networking/
        // worker tasks in Chromium. Page intentionally admits one loading
        // completion per task, so coalescing callbacks here can strand every
        // ready operation except the one that bounded turn happens to poll.
        let _ = wake.send(Command::Wake);
    })
}

impl PageClient for Client {
    fn DidDispatchClick(&mut self, source_document: u128, target_node_id: u64) {
        if self.controls {
            let _ = self.document_sender.send(Command::ClickActivated {
                source_document,
                target_node_id,
            });
        }
    }
    fn DidEventHandlerChromeEffect(&mut self, effect: browser::page::EventHandlerChromeEffect) {
        (self.events)(self.document, Event::EventHandlerChrome(effect));
    }

    fn CurrentKeyboardModifiers(
        &self,
    ) -> Result<
        interaction::core::input::keyboard_event_manager::NativeKeyboardModifiers,
        &'static str,
    > {
        crate::native_keyboard_modifiers::current()
    }
    fn DidChangeValidationMessageVisibility(
        &mut self,
        event: browser::page::ValidationMessageVisibility,
    ) {
        (self.events)(self.document, Event::ValidationMessageVisibility(event));
    }
    fn DidChangeValidationMessage(&mut self, update: browser::page::ValidationMessageUpdate) {
        (self.events)(self.document, Event::ValidationMessage(update));
    }

    fn EnumerateDroppedDirectory(
        &self,
        request: browser::page::DirectoryEnumerationRequest,
    ) -> bool {
        let sender = self.document_sender.clone();
        let directory = request.directory.clone();
        directory.EnumerateDirectory(Box::new(move |files| {
            let _ = sender.send(Command::DirectoryFilesChosen(
                browser::page::DirectoryEnumerationResult { request, files },
            ));
        }));
        true
    }

    fn CaptureMouseEventsToWidget(&mut self, capture: bool) {
        (self.events)(self.document, Event::CaptureMouseEventsToWidget(capture));
    }
    fn StartSystemDrag(
        &self,
        request: interaction::default::core::page::drag_controller::SystemDragRequest,
    ) -> interaction::default::core::page::drag_controller::SystemDragSupport {
        use interaction::default::core::page::drag_controller::SystemDragSupport;
        let support = crate::native_drag::support(self.native_drag_window, &request);
        if support == SystemDragSupport::Supported {
            (self.events)(
                self.document,
                Event::StartSystemDrag(crate::native_drag::Start {
                    document: self.document,
                    request: Arc::new(request),
                    origin: (0., 0.),
                    reply: self.document_sender.clone(),
                }),
            );
        }
        support
    }
    fn ReadPrimarySelection(
        &self,
    ) -> Result<
        interaction::default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        use interaction::default::core::editing::commands::clipboard_commands::ClipboardHost;
        self.primary_clipboard.ReadPrimarySelection()
    }
    fn ReadClipboard(
        &self,
    ) -> Result<
        interaction::default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        use interaction::default::core::editing::commands::clipboard_commands::ClipboardHost;
        self.primary_clipboard.ReadClipboard()
    }
    fn WriteClipboard(
        &self,
        snapshot: &interaction::default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
    ) -> Result<
        (),
        interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        use interaction::default::core::editing::commands::clipboard_commands::ClipboardHost;
        self.primary_clipboard.WriteClipboard(snapshot)
    }
    fn DidAutoscroll(
        &mut self,
        effect: interaction::adapter::page_mouse_autoscroll::AutoscrollHostEffect,
    ) {
        (self.events)(self.document, Event::Autoscroll(effect));
    }
    fn DidChangeCursor(&mut self, cursor: interaction::Cursor) {
        (self.events)(self.document, Event::CursorChanged(cursor));
    }
    fn AcceptsLoadDrops(&self) -> bool {
        self.accepts_load_drops
    }
    fn DidRequestDropNavigation(&mut self, request: &browser::page::DropNavigationRequest) {
        (self.events)(self.document, Event::DropNavigate(request.clone()));
    }
    fn DidRequestNavigation(&mut self, request: &browser::page::NavigationRequest) {
        (self.events)(self.document, Event::Navigate(request.clone()));
    }
    fn DidRequestAuxiliaryNavigationTarget(
        &mut self,
        request: &browser::page::AuxiliaryNavigationTargetRequest,
    ) {
        (self.events)(
            self.document,
            Event::AuxiliaryNavigationTarget(request.clone()),
        );
    }
    fn DidRequestHistoryNavigation(&mut self, delta: i32) {
        (self.events)(self.document, Event::HistoryNavigation(delta));
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
    fn DidCommitHistory(
        &mut self,
        item: Arc<navigation_history::HistoryItemSnapshot>,
        load_type: navigation_history::FrameLoadType,
    ) {
        (self.events)(self.document, Event::HistoryCommitted { item, load_type });
    }
    fn DidUpdateHistoryState(&mut self, item: Arc<navigation_history::HistoryItemSnapshot>) {
        (self.events)(self.document, Event::HistoryStateUpdated(item));
    }
    fn DidCommitChildHistory(
        &mut self,
        item: Arc<navigation_history::HistoryItemSnapshot>,
        load_type: navigation_history::FrameLoadType,
    ) {
        (self.events)(
            self.document,
            Event::ChildHistoryCommitted { item, load_type },
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
        | Patch::Text { id, .. }
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
        Patch::Text { value, .. } => (
            DOMMutationType::kSetTextContent,
            String::new(),
            value.clone(),
        ),
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
    source: Arc<FrameDemand>,
    reporter: &ErrorReporter,
    threads: browser::page::PageThreads,
    impl_sender: mpsc::Sender<ImplCommand>,
) -> io::Result<()> {
    let mut viewport = params.viewport;
    let _cursor_metrics_subscription =
        crate::cursor_screen_info::subscribe(wake.clone(), params.system_cursor_size);
    let client = Rc::new(RefCell::new(Client {
        document_sender: wake.clone(),
        controls: params.controls,
        native_drag_window: params.native_drag_window,
        accepts_load_drops: params.native_drag_window && !params.controls,
        primary_clipboard: params.primary_clipboard.clone(),
        document: params.document,
        events: params.events.clone(),
        reporter: reporter.clone(),
    }));
    let mut page = crate::engine::create_document(
        f64::from(viewport.width) / viewport.scale,
        f64::from(viewport.height) / viewport.scale,
        viewport.scale,
        params.scripting,
        params.prefers_reduced_motion,
        &params.text_settings,
        params.font_backend_factory.clone(),
        client,
        threads,
    )?;
    // Source ScreenInfos is present before the first document query, including
    // the empty macOS/Linux source size. Windows updates arrive as host tasks.
    page.SetSystemCursorSize(
        params.system_cursor_size.width_dip as f32,
        params.system_cursor_size.height_dip as f32,
    )?;
    // LocalFrameView::BaseBackgroundColor is an embedding choice independent
    // of CSS. BrowserView paints the toolbar backdrop in its root Viz frame;
    // ordinary WebContents keep Page's default opaque white base.
    page.SetShouldPaintBaseBackgroundColor(!params.transparent);
    page.SetExternalCompositor(true);
    impl_sender
        .send(ImplCommand::BindGestureTouchActionFilter(
            page.GestureTouchActionFilterHandle(),
        ))
        .map_err(|_| io::Error::other("Page input executor stopped"))?;
    page.SetBeginFrameSource(Some(source.clone()));
    page.SetLoadingWakeCallback(loading_wake_callback(wake));
    page.OpenRequest(&params.request, 16384, 4096)?;
    let mut patches = VecDeque::new();
    let mut completed_patch_revisions = std::collections::BTreeSet::new();
    let mut ready_embedder_revision = 0u64;
    let mut applied_embedder_revision = 0u64;
    let mut published: Option<(u64, u64, u64, u64)> = None;
    let mut validation_bubble_size = None;
    let mut completed_main_frame_epoch = 0u64;
    let mut status = None;
    let mut caret_feedback = None;
    let mut text_input_feedback = None;
    let mut last_memory_report: Option<Instant> = None;
    let mut hovered = Vec::new();
    let mut press_discarded = false;
    let mut resize_event_pending = false;
    let mut pending_resize_provenance: Option<ResizeProvenance> = None;
    let mut resize_root_viewport = None;
    let mut copy_output_fence_pending = false;
    let mut queue = DocumentQueue::new();
    loop {
        let command = queue.next(&receiver, page.NextTaskDeadline(Instant::now()));
        let processing_main_frame = matches!(
            &command,
            Ok(Command::Main(MainThreadEvent::BeginFrame { .. }))
        );
        let frame_context = match &command {
            Ok(Command::Main(MainThreadEvent::BeginFrame { frame, .. })) => {
                browser_tracing::Context {
                    source_id: frame.source_id,
                    frame_id: frame.sequence_number,
                    task_id: pending_resize_provenance.map_or(0, |resize| resize.id),
                    ..Default::default()
                }
            }
            Ok(Command::Resize(resize)) => browser_tracing::Context {
                task_id: resize.trace_id(),
                ..Default::default()
            },
            _ => Default::default(),
        };
        let _scope = browser_tracing::scope(frame_context);
        match command {
            Ok(Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(Command::PrepareNavigation { token }) => {
                page.SaveHistoryState();
                (params.events)(params.document, Event::NavigationPrepared { token });
            }
            Ok(Command::CompleteAuxiliaryNavigationTarget {
                request_id,
                accepted,
            }) => page.CompleteAuxiliaryNavigationTarget(request_id, accepted),
            Ok(Command::TraverseHistory { item, state }) => {
                page.TraverseSessionHistory(item, state);
            }
            Ok(Command::SessionHistoryState { item, state }) => {
                page.SetSessionHistoryState(item, state);
            }
            Ok(Command::DirectoryFilesChosen(result)) => {
                page.DirectoryFilesChosen(result)?;
            }
            Ok(Command::SystemDragTarget { target, action }) => {
                let result = page.TargetSystemDrag(
                    action,
                    target.position,
                    target.screen_position,
                    target.modifiers,
                    target.observed_at,
                );
                match result {
                    Ok(operation) => {
                        if target.feedback_generation.load(Ordering::Acquire)
                            == target.request_token
                        {
                            target
                                .operation
                                .store(operation.operation as u32, Ordering::Release);
                        }
                    }
                    Err(error) => {
                        if target.feedback_generation.load(Ordering::Acquire)
                            == target.request_token
                        {
                            target.operation.store(0, Ordering::Release);
                        }
                        return Err(error);
                    }
                }
            }
            Ok(Command::SystemDragEnded(completion)) => {
                if completion.document == params.document {
                    let operation = match completion.outcome {
                        crate::native_drag::Outcome::Ended(operation) => operation,
                        crate::native_drag::Outcome::Cancelled | crate::native_drag::Outcome::Failed(_) =>
                            interaction::dispatch::core::clipboard::data_transfer::DragOperation::None,
                    };
                    if let crate::native_drag::Outcome::Failed(failure) = completion.outcome {
                        eprintln!(
                            "rechrom-native-drag-failed document={:?} reason={failure:?}",
                            completion.document
                        );
                    }
                    page.EndSystemDrag(
                        completion.pointer_id,
                        completion.position,
                        completion.screen_position,
                        completion.completed_at,
                        operation,
                    )?;
                }
            }
            Ok(Command::Wake) => {
                page.RunTask()?;
            }
            #[cfg(test)]
            Ok(Command::TestTask(task)) => task(),
            Ok(Command::Task) => {
                page.RunTask()?;
            }
            Ok(Command::CollectLayoutHeap) => {
                // cppgc::Sweeper::IncrementalSweepTask uses a 5 ms foreground
                // budget. User-blocking input/rendering is selected first;
                // native pointers and finalizers stay on this heap's owner.
                foundation::CollectLayoutHeapIncrementally(Duration::from_millis(5));
            }
            Ok(Command::CollectJavaScriptHeap) => page.CollectJavaScriptGarbage(),
            Ok(Command::CopyOutputFence) => {
                copy_output_fence_pending = true;
                source.request_begin_frame();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                unreachable!("document queue resolves deadlines")
            }
            Ok(Command::Main(MainThreadEvent::BeginFrame {
                frame,
                main_frame_epoch,
            })) => {
                let started = Instant::now();
                let first_response_for_epoch =
                    main_frame_epoch != 0 && main_frame_epoch > completed_main_frame_epoch;
                let artifact_before = page.CurrentFrame().map(|frame| {
                    (
                        Arc::as_ptr(&frame.display_items),
                        frame.display_items.recording_revision,
                    )
                });
                if std::mem::take(&mut resize_event_pending) && params.scripting {
                    let _ = page.Evaluate(
                        "window.dispatchEvent(new Event('resize'))",
                        "rechrom:resize",
                    );
                }
                let result = page.UpdateRendering(frame.args);
                if let Err(error) = result {
                    return Err(error);
                }
                // A browser-owned patch revision belongs to the first frame
                // lifecycle that runs after the DOM mutation. Publishing the
                // revision on the preceding PaintArtifact lets Display accept
                // stale toolbar pixels as the matching root commit during a
                // tab switch. Keep the fence coupled to the frame that has
                // actually consumed the mutation, as Chromium couples its UI
                // state and compositor commit.
                let embedder_revision_changed =
                    applied_embedder_revision != ready_embedder_revision;
                applied_embedder_revision = ready_embedder_revision;
                // Chromium carries BeginMainFrameAndCommitState in the posted
                // BeginMainFrame task. Do not infer that ownership from a
                // separate compositor-effects task: rendering opportunities
                // outrank tasks and may legally overtake it in DocumentQueue.
                completed_main_frame_epoch = completed_main_frame_epoch.max(main_frame_epoch);
                let artifact_unchanged = match (artifact_before, page.CurrentFrame()) {
                    (Some((before, revision)), Some(frame)) => {
                        before == Arc::as_ptr(&frame.display_items)
                            && revision == frame.display_items.recording_revision
                    }
                    (None, None) => true,
                    _ => false,
                };
                let has_no_artifact = page.CurrentFrame().is_none();
                let current_frame_was_published = page.CurrentFrame().is_some_and(|frame| {
                    published.is_some_and(|publication| {
                        publication.0 == frame.sequence
                            && publication.1 == frame.compositor_scroll_sequence
                    })
                });
                if first_response_for_epoch
                    && !embedder_revision_changed
                    && !copy_output_fence_pending
                    && artifact_unchanged
                    // ProxyMain must finish every BeginMainFrame transaction,
                    // including the provisional document's first opportunity
                    // before parsing has produced any PaintArtifact.  Requiring
                    // a previously published frame here leaves cc permanently
                    // waiting on main, so the later dirty parse can never ask
                    // for the BMF that creates the actual first frame.
                    && (has_no_artifact || current_frame_was_published)
                    && (pending_resize_provenance.is_none() || has_no_artifact)
                {
                    send_event_timing_requests(&page, &impl_sender, false)?;
                    impl_sender
                        .send(ImplCommand::FinishMainFrameWithoutUpdates { main_frame_epoch })
                        .map_err(|_| io::Error::other("Page compositor executor stopped"))?;
                    // The no-update response retires cc's reflected scroll
                    // delta. Treat Page's unchanged artifact identity as
                    // acknowledged so the publication tail below does not
                    // send that same Arc back as a synthetic commit.
                    if let Some(frame) = page.CurrentFrame() {
                        published = Some((
                            frame.sequence,
                            frame.compositor_scroll_sequence,
                            completed_main_frame_epoch,
                            applied_embedder_revision,
                        ));
                    }
                }
                browser_tracing::interval(
                    "frame",
                    "MainFrameCycle",
                    started,
                    Instant::now(),
                    &[
                        ("view_id", params.document.0 as f64),
                        ("main_frame_epoch", main_frame_epoch as f64),
                    ],
                );
            }
            Ok(Command::BindCompositorPresentationClock(clock)) => {
                page.BindExternalCompositorPresentationClock(clock)
            }
            Ok(Command::EventTimingDidSwap {
                requests,
                frame_token,
                swap_time,
            }) => page.EventTimingDidSwap(requests, frame_token, swap_time),
            Ok(Command::EventTimingDidNotSwap {
                requests,
                swap_time,
            }) => page.EventTimingDidNotSwap(requests, swap_time),
            Ok(Command::EventTimingDidCommit(commit_finish_time)) => {
                page.EventTimingDidCommit(commit_finish_time)
            }
            Ok(Command::EventTimingDidPresent(feedback)) => page.EventTimingDidPresent(feedback),
            Ok(Command::Visibility {
                visible,
                changed_at,
            }) => {
                page.NotifyPerformancePageVisibilityChanged(visible, changed_at);
            }
            Ok(Command::ClickActivated {
                source_document,
                target_node_id,
            }) => {
                if params.controls
                    && page.Document().GetDocument().RootHandle().Identity() == source_document
                {
                    (params.events)(
                        params.document,
                        Event::ControlActivated {
                            target: element_id(&page, target_node_id),
                        },
                    );
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
                    && matches!(&input,InputEvent::Mouse(e) if matches!(e.r#type,MouseEventType::kUp|MouseEventType::kContextMenu))
                {
                    // Retain the discarded sequence through any platform
                    // context-menu notification; the next down starts a new one.
                    continue;
                }
                if matches!(&input,InputEvent::Mouse(e) if e.r#type==MouseEventType::kDown) {
                    press_discarded = false;
                }
                if !params.controls {
                    // Page owns Chromium's MainThreadEventQueue ordering. In
                    // particular, a non-coalescible mouse-up must flush the
                    // frame-aligned mouse-move that precedes it before the up
                    // is dispatched. Calling Dispatch here bypasses that
                    // barrier and collapses fast native drags to their press.
                    page.QueueFrameInput(input, queued_at, samples)?;
                } else {
                    let started = Instant::now();
                    let address = params.controls.then(|| find_id(&page, "address")).flatten();
                    let was_address = params.controls && page.FocusedNodeId() == address;
                    let result = page.Dispatch(&input)?;
                    if params.controls {
                        if matches!(&input,InputEvent::Mouse(e) if e.r#type==MouseEventType::kDown && e.button==MouseButton::kPrimary)
                            && result.target_node_id == address
                            && (!was_address
                                || matches!(&input,InputEvent::Mouse(e) if e.click_count==2))
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
                document_generation,
                wheel,
                queued_at,
                samples,
                effects,
                main_frame_epoch,
            }) => {
                if document_generation != page.InputDocumentGeneration() {
                    page.DiscardWheelForDetachedDocument(&wheel)?;
                    if let Some(main_frame_epoch) = main_frame_epoch {
                        let _ = impl_sender
                            .send(ImplCommand::FinishMainFrameWithoutUpdates { main_frame_epoch });
                    }
                } else {
                    let requests_main = main_frame_epoch.is_some();
                    page.ApplyCompositorWheelEffects(effects)?;
                    if requests_main {
                        // Mark the compositor-issued main frame before Page queues
                        // the wheel. QueueFrameInput requests the same physical
                        // pulse through BeginFrameSource; this ordering prevents
                        // that request from manufacturing a second cc commit.
                        source.request_compositor_main_frame();
                    }
                    page.QueueFrameInput(InputEvent::Wheel(wheel), queued_at, samples)?;
                }
            }
            Ok(Command::ExternalCanonicalWheel {
                document_generation,
                event,
                wheel,
                queued_at,
                effects,
                main_frame_epoch,
            }) => {
                if document_generation != page.InputDocumentGeneration() {
                    page.DiscardWheelForDetachedDocument(&wheel)?;
                    if let Some(main_frame_epoch) = main_frame_epoch {
                        let _ = impl_sender
                            .send(ImplCommand::FinishMainFrameWithoutUpdates { main_frame_epoch });
                    }
                } else {
                    page.ApplyCompositorWheelEffects(effects)?;
                    if main_frame_epoch.is_some() {
                        source.request_compositor_main_frame();
                    }
                    let packet =
                        interaction::frame_aligned_input_queue::OwnedInputPacket::FromCanonical(
                            ConvertedWebInputEvent::Wheel(event),
                        );
                    page.QueueCanonicalFrameInput(InputEvent::Wheel(wheel), packet, queued_at, 1)?;
                }
            }
            Ok(Command::ExternalGesture {
                document_generation,
                dispatch_id,
                gesture,
            }) => {
                if document_generation == page.InputDocumentGeneration() {
                    page.QueueBrowserGesture(gesture, dispatch_id, Instant::now())?;
                } else {
                    let _ = impl_sender.send(ImplCommand::GestureAck {
                        dispatch_id,
                        result: InputEventResultState::kNoConsumerExists,
                    });
                }
            }
            Ok(Command::GestureDisposition(ack)) => {
                (params.events)(
                    params.document,
                    Event::GestureDisposition {
                        gesture: ack.event,
                        source: ack.source,
                        result: ack.result,
                    },
                );
            }
            Ok(Command::FlingDisposition(gesture)) => {
                (params.events)(
                    params.document,
                    Event::FlingDisposition {
                        gesture,
                        result: InputEventResultState::kConsumed,
                    },
                );
            }
            Ok(Command::CompositorEffects {
                effects,
                main_frame_epoch,
            }) => {
                let requests_main = main_frame_epoch.is_some();
                page.ApplyCompositorWheelEffects(effects)?;
                if requests_main {
                    source.request_compositor_main_frame();
                }
            }
            Ok(Command::Resize(resize)) => {
                let started = Instant::now();
                let next = resize.viewport;
                resize_root_viewport = Some(resize.root_viewport);
                browser_tracing::interval(
                    "resize",
                    "ResizeDocumentQueue",
                    resize.queued_at,
                    started,
                    &[("samples", resize.samples as f64)],
                );
                let mut trace = browser_tracing::span("resize", "Page.ResizeViewport");
                trace.set("samples", resize.samples as f64);
                trace.set("width", next.width as f64);
                trace.set("height", next.height as f64);
                trace.set("device_scale", next.scale);
                viewport = next;
                page.ResizeViewport(
                    f64::from(next.width) / next.scale,
                    f64::from(next.height) / next.scale,
                    next.scale,
                )?;
                let provenance = ResizeProvenance {
                    id: resize.trace_id(),
                    queued_at: resize.queued_at,
                    samples: resize.samples,
                };
                if let Some(pending) = &mut pending_resize_provenance {
                    pending.merge(provenance);
                } else {
                    pending_resize_provenance = Some(provenance);
                }
                resize_event_pending = true;
            }
            Ok(Command::InteractionTextSettings(settings)) => {
                if let Some(locale) = settings.temporal_locale {
                    page.SetTemporalInputLocale(locale);
                }
                page.SetInteractionTextSettings(
                    settings.editing_behavior,
                    &settings.ui_locale,
                    settings.smart_insert_delete_enabled,
                );
            }
            Ok(Command::ContextMenuWidgetGeometry(geometry)) => {
                page.SetContextMenuWidgetGeometry(geometry)?;
            }
            Ok(Command::SystemCursorSize(size)) => {
                page.SetSystemCursorSize(size.width_dip as f32, size.height_dip as f32)?;
            }
            Ok(Command::ResizeTask) => unreachable!("DocumentQueue resolves resize tasks"),
            Ok(Command::Active(active)) => page.SetActive(active)?,
            Ok(Command::Patch(batch)) => patches.extend(batch),
            Ok(Command::StopLoading) => page.StopLoading(),
            Ok(Command::Evaluate { source, reply }) => {
                let _task = browser_tracing::span("owner_task", "Page.Evaluate");
                let result = page
                    .EvaluateForInspector(&source, "devtools:Runtime.evaluate")
                    .map_err(|e| e.to_string())
                    .and_then(|r| r.exception.map_or(Ok(()), |e| Err(e.message)));
                let _ = reply.send(result);
            }
        }
        // WidgetInputHandlerManager.cc:505-520 posts its completed main-thread
        // hit test back to the input owner. WebFrameWidgetImpl.cc:4353-4355
        // posts injected scrollbar gestures to that same sequenced ingress.
        // Wheel acknowledgements and scrollbar begin/update/end retain FIFO
        // order after the synchronous document input transaction completes.
        for disposition in page.TakeWheelDispositions()? {
            let _ = impl_sender.send(ImplCommand::WheelAck {
                document_generation: disposition.document_generation,
                result: disposition.result,
            });
        }
        for input in page.TakePendingBrowserInputs() {
            let command = match input {
                browser::page::PageBrowserInput::DocumentDetached { generation } => {
                    ImplCommand::DocumentDetached { generation }
                }
                browser::page::PageBrowserInput::Compositor(input) => {
                    ImplCommand::InputContinuation(input)
                }
                browser::page::PageBrowserInput::Gesture(gesture) => ImplCommand::Gesture {
                    gesture,
                    queued_at: Instant::now(),
                    samples: 1,
                },
            };
            impl_sender
                .send(command)
                .map_err(|_| io::Error::other("Page input executor stopped"))?;
        }
        let count = patches.len();
        for _ in 0..count {
            let (revision, patch) = patches.pop_front().unwrap();
            if apply_patch(&mut page, &patch)? {
                completed_patch_revisions.insert(revision);
            } else {
                patches.push_back((revision, patch));
            }
        }
        let previous_ready_embedder_revision = ready_embedder_revision;
        while completed_patch_revisions.remove(&(ready_embedder_revision + 1)) {
            ready_embedder_revision += 1;
        }
        if ready_embedder_revision != previous_ready_embedder_revision {
            // Browser UI state and the root scene are fenced by this revision.
            // A patch may be a semantic no-op (for example, clicking content
            // while the omnibox is already unfocused), so DOM invalidation is
            // not guaranteed to request a lifecycle update. Still run a main
            // frame: it publishes the resident artifact with the newly
            // consumed embedder revision and lets Display complete the root
            // commit. This is the Page-backed equivalent of Chromium's
            // browser-UI commit acknowledgement.
            source.request_begin_frame();
        }
        // MainThreadEventQueue owns the original and coalesced callbacks.
        // Publish their completed ACKs in order without waiting on the host.
        for disposition in page.TakeBrowserGestureDispositions() {
            let _ = impl_sender.send(ImplCommand::GestureAck {
                dispatch_id: disposition.dispatch_id,
                result: if disposition.consumed {
                    InputEventResultState::kConsumed
                } else {
                    InputEventResultState::kNotConsumed
                },
            });
        }
        for disposition in page.TakeTouchDispositions() {
            (params.events)(
                params.document,
                Event::TouchDisposition {
                    unique_touch_event_id: disposition.unique_touch_event_id,
                    consumed: disposition.consumed,
                },
            );
        }
        for event in page.TakeDispatchedFrameInputs() {
            let _scope = browser_tracing::scope(browser_tracing::Context {
                input_id: browser_tracing::instant_id(event.queued_at),
                ..Default::default()
            });
            let (kind, compositor_owned, compositor_nonblocking) = match &event.input {
                InputEvent::Wheel(wheel) => (
                    1.0,
                    wheel.compositor_owned as u8 as f64,
                    wheel.compositor_nonblocking as u8 as f64,
                ),
                _ => (0.0, 0.0, 0.0),
            };
            browser_tracing::interval(
                "input",
                "InputQueue",
                event.queued_at,
                event.dispatch_started,
                &[
                    ("samples", event.samples as f64),
                    ("frame_aligned", 1.0),
                    ("kind", kind),
                    ("compositor_owned", compositor_owned),
                    ("compositor_nonblocking", compositor_nonblocking),
                ],
            );
            browser_tracing::interval(
                "input",
                "InputDispatch",
                event.dispatch_started,
                event.dispatch_finished,
                &[
                    ("samples", event.samples as f64),
                    ("frame_aligned", 1.0),
                    ("kind", kind),
                    ("compositor_owned", compositor_owned),
                    ("compositor_nonblocking", compositor_nonblocking),
                ],
            );
        }
        if params.document.0 == u64::MAX - 1 {
            // Browser UI reads only its own resident Layout result. The
            // embedding routes this small immutable size back to the overlay
            // owner; neither document reaches into the other's engine.
            if let Some(control) = find_id(&page, "container") {
                let layout = page.GetLayoutEngine();
                let size = layout
                    .Output()
                    .and_then(|output| output.Node(control))
                    .and_then(|node| node.ClientRects().into_iter().next())
                    .map(|rect| (rect.width, rect.height));
                if let Some((width, height)) = size.filter(|size| {
                    Some(*size) != validation_bubble_size && size.0 > 0.0 && size.1 > 0.0
                }) {
                    validation_bubble_size = Some((width, height));
                    (params.events)(
                        params.document,
                        Event::ValidationBubbleSize { width, height },
                    );
                }
            }
        }
        // Drain every document turn, including input that makes no paint
        // changes. Requests precede the matching artifact on the impl queue.
        // Every independent Page uses its own route; root and embedded Page
        // domains never borrow another Page's mutable timing state.
        let has_pending_commit = page.CurrentFrame().is_some_and(|frame| {
            published
                != Some((
                    frame.sequence,
                    frame.compositor_scroll_sequence,
                    completed_main_frame_epoch,
                    applied_embedder_revision,
                ))
        });
        send_event_timing_requests(&page, &impl_sender, !has_pending_commit)?;
        // ProxyMain admits PaintArtifact state only as the response to the
        // BeginMainFrame that owns its commit state. Input/resource tasks can
        // finish Paint before that rendering opportunity reaches the document
        // queue; retain the new resident artifact here instead of sending it
        // with the preceding epoch and having ProxyImpl reject it as an
        // unsolicited commit. The matching BeginMainFrame publishes it below.
        if processing_main_frame {
            if let Some(frame) = page.CurrentFrame() {
                // A host resize invalidates Page and requests BeginMainFrame, but it
                // does not turn the active PaintArtifact into output for the new
                // viewport. Chromium keeps the old LocalSurfaceId visible until
                // the renderer submits a CompositorFrame with the new surface
                // parameters. Publish again only after Page produces new content
                // or consumes a new main-frame/scroll transaction.
                let identity = (
                    frame.sequence,
                    frame.compositor_scroll_sequence,
                    completed_main_frame_epoch,
                    applied_embedder_revision,
                );
                if published != Some(identity) {
                    let child_frames = page.TakeChildCompositorFrames();
                    impl_sender
                        .send(ImplCommand::Commit {
                            artifact: frame.display_items.clone(),
                            child_frames,
                            content: ContentFrameProvenance {
                                sequence: frame.sequence,
                                recording_revision: frame.display_items.recording_revision,
                                main_frame_epoch: completed_main_frame_epoch,
                                compositor_scroll_sequence: frame.compositor_scroll_sequence,
                                embedder_revision: applied_embedder_revision,
                                resize: pending_resize_provenance.take(),
                            },
                            viewport,
                        })
                        .map_err(|_| io::Error::other("Page compositor executor stopped"))?;
                    copy_output_fence_pending = false;
                    published = Some(identity);
                    if params.document.0 == 0 {
                        (params.events)(
                            params.document,
                            Event::DragRegions {
                                regions: drag_regions(&page, viewport),
                                root_viewport: resize_root_viewport.take(),
                            },
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
        // between native input events. Enqueue caret feedback only when its
        // value changes. Cursor updates arrive through PageClient::SetCursor.
        let next_caret = page.Caret().map(|c| c.rect);
        let next_text_input = page.TextInputState();
        if caret_feedback.as_ref() != Some(&next_caret)
            || text_input_feedback.as_ref() != Some(&next_text_input)
        {
            (params.events)(
                params.document,
                Event::Feedback {
                    caret: next_caret,
                    text_input: next_text_input.clone(),
                },
            );
            caret_feedback = Some(next_caret);
            text_input_feedback = Some(next_text_input);
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
fn send_event_timing_requests(
    page: &Page,
    sender: &mpsc::Sender<ImplCommand>,
    request_main_frame: bool,
) -> io::Result<()> {
    let requests = page.TakeEventTimingPresentationRequests();
    if !requests.is_empty() {
        sender
            .send(ImplCommand::EventTimingRequests {
                requests,
                request_main_frame,
            })
            .map_err(|_| io::Error::other("Page compositor executor stopped"))?;
    }
    Ok(())
}
fn title(page: &Page) -> String {
    let owner = page.Document();
    let doc = owner.GetDocument();
    let mut pending = vec![doc.Root()];
    let mut title = None;
    while let Some(index) = pending.pop() {
        let node = doc.Node(index);
        if node.IsHTMLElement("title") {
            title = Some(index);
            break;
        }
        pending.extend(node.Children().iter().rev().copied());
    }
    let mut value = String::new();
    if let Some(title) = title {
        let mut pending = vec![title];
        while let Some(index) = pending.pop() {
            let node = doc.Node(index);
            if node.Type() == dom::persistent_document::DOMNodeType::kText {
                value.push_str(node.Data());
            }
            pending.extend(node.Children().iter().rev().copied());
        }
    }
    value.trim().chars().take(100).collect()
}
fn drag_regions(page: &Page, viewport: Viewport) -> Vec<crate::chrome::DragRegion> {
    use layoutng_assembly::{internal::layout_input::Offset, layout_engine::PaintFragmentCursor};
    let mut regions = vec![crate::chrome::DragRegion {
        x: 92.0,
        y: 0.0,
        width: (f64::from(viewport.width) / viewport.scale - 92.0).max(0.0),
        height: 6.0,
    }];
    fn bounds(root: PaintFragmentCursor<'_>, id: u64) -> Option<crate::chrome::DragRegion> {
        let mut pending = vec![(root, Offset::default())];
        let mut visited = std::collections::HashSet::new();
        while let Some((fragment, parent_offset)) = pending.pop() {
            if fragment.fragment_instance_id != 0 && !visited.insert(fragment.fragment_instance_id)
            {
                continue;
            }
            let offset = Offset {
                x: parent_offset.x + fragment.offset.x,
                y: parent_offset.y + fragment.offset.y,
            };
            if fragment.node_id == id {
                return Some(crate::chrome::DragRegion {
                    x: offset.x,
                    y: offset.y,
                    width: fragment.size.width,
                    height: fragment.size.height,
                });
            }
            let children = fragment.Children().collect::<Vec<_>>();
            pending.extend(children.into_iter().rev().map(|child| (child, offset)));
        }
        None
    }
    if let (Some(frame), Some(id)) = (page.CurrentFrame(), find_id(page, "dragspace")) {
        let root = PaintFragmentCursor::FromHandle(&frame.fragments);
        if let Some(r) = bounds(root, id) {
            regions.push(r);
        }
    }
    regions
}

#[cfg(test)]
mod queue_tests {
    use super::*;

    #[test]
    fn direct_main_frame_delivery_requires_a_pure_request() {
        assert!(!only_begins_main_frame(&[]));
        assert!(only_begins_main_frame(&[
            CompositorMainThreadEffect::BeginMainFrame,
        ]));
        assert!(!only_begins_main_frame(&[
            CompositorMainThreadEffect::RequestMainFrame,
        ]));
        assert!(!only_begins_main_frame(&[
            CompositorMainThreadEffect::BeginMainFrame,
            CompositorMainThreadEffect::RequestMainFrame,
            CompositorMainThreadEffect::BeginFrameSourcePaused { paused: false },
        ]));
    }

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

    #[test]
    fn each_resource_wake_retains_its_own_page_task() {
        let (sender, receiver) = mpsc::channel();
        let mut queue = DocumentQueue::new();
        sender.send(Command::Wake).unwrap();
        sender.send(Command::Wake).unwrap();

        // The initial bootstrap task remains first.  The two completion tasks
        // must both survive the mailbox drain because Page's loading budget
        // consumes at most one completion in each of them.
        assert!(matches!(queue.next(&receiver, None), Ok(Command::Task)));
        assert!(matches!(queue.next(&receiver, None), Ok(Command::Wake)));
        assert!(matches!(queue.next(&receiver, None), Ok(Command::Wake)));
    }

    #[test]
    fn independent_loading_completions_survive_bootstrap_task() {
        let (sender, receiver) = mpsc::channel();
        let wake = loading_wake_callback(sender);
        let mut queue = DocumentQueue::new();

        // An image response and a module worker can become ready before the
        // owner selects either. They are independent Chromium tasks: the
        // unrelated bootstrap Task and the first bounded Page turn must not
        // consume the second completion's identity.
        wake();
        wake();
        assert!(matches!(queue.next(&receiver, None), Ok(Command::Task)));
        assert!(matches!(queue.next(&receiver, None), Ok(Command::Wake)));
        assert!(matches!(queue.next(&receiver, None), Ok(Command::Wake)));
    }

    #[test]
    fn no_update_response_reopens_the_next_begin_main_frame() {
        let (frames, _frame_receiver) = mpsc::channel();
        let mut presentation = ImplPresentation::new(
            PageRenderPipeline::new_external(executor::inline(), executor::inline(), false),
            frames,
            (1, 1),
            mpsc::channel().0,
            Arc::new(|_| {}),
            Viewport {
                width: 800,
                height: 600,
                scale: 1.0,
            },
            Arc::new(|| {}),
            Arc::new(|_| {}),
        );
        let now = Instant::now();
        let frame = |sequence_number: u64| foundation::begin_frame::BeginFrameArgs {
            source_id: 1,
            sequence_number,
            frame_time: now + Duration::from_millis(16 * sequence_number.saturating_sub(1)),
            deadline: now + Duration::from_millis(16 * sequence_number),
            interval: Duration::from_millis(16),
        };
        let has_main_frame_request = |result: &CompositorWheelResult| {
            result
                .effects
                .iter()
                .any(|effect| matches!(effect, CompositorMainThreadEffect::BeginMainFrame))
        };

        // Scheduler does not dispatch BeginMainFrame for an invisible tree.
        // Prime the same active-artifact state in which the runtime regression
        // occurred, then let its initial commit reach a stable boundary.
        let root = paint::paint_property_tree::PropertyTreeState::Root();
        let artifact = Arc::new(paint::PaintArtifact {
            compositor_viewport: Some(paint::PaintCompositorViewport {
                mode: paint::PaintCompositorViewportMode::SingleLayoutViewport,
                viewport_clip: root.clip.clone(),
                overscroll_elasticity_transform: root.transform.clone(),
                page_scale: root.transform.clone(),
                visual_scroll_translation: root.transform.clone(),
                layout_scroll_translation: Some(root.transform.clone()),
                scroll_translation_nodes: vec![root.transform],
            }),
            ..Default::default()
        });
        let initial = presentation
            .pipeline
            .CommitArtifact(&artifact, 800, 600, 1.0);
        assert!(
            initial.is_ok()
                || initial.is_err_and(|error| error.kind() == io::ErrorKind::WouldBlock)
        );
        for sequence_number in 1..=4 {
            let _ = presentation
                .pipeline
                .BeginPhysicalFrame(frame(sequence_number))
                .unwrap();
            let _ = presentation.pipeline.Pump().unwrap();
        }
        let mut sequence_number = 5;
        let mut dispatch = |presentation: &mut ImplPresentation| {
            for _ in 0..8 {
                let current = sequence_number;
                sequence_number += 1;
                let physical = presentation
                    .pipeline
                    .BeginPhysicalFrame(frame(current))
                    .unwrap();
                if has_main_frame_request(&physical) {
                    return true;
                }
                let pump = presentation.pipeline.Pump().unwrap();
                if has_main_frame_request(&pump) {
                    return true;
                }
            }
            false
        };

        let first_request = presentation.pipeline.RequestMainFrame().unwrap();
        let first_dispatched =
            has_main_frame_request(&first_request) || dispatch(&mut presentation);
        assert!(first_dispatched);
        presentation.require_main_frame(1);

        assert!(presentation
            .finish_main_frame_without_updates(1)
            .map_err(|failure| failure.error)
            .unwrap()
            .is_some());
        assert!(presentation
            .finish_main_frame_without_updates(1)
            .map_err(|failure| failure.error)
            .unwrap()
            .is_none());

        let second_request = presentation.pipeline.RequestMainFrame().unwrap();
        let second_dispatched =
            has_main_frame_request(&second_request) || dispatch(&mut presentation);
        assert!(second_dispatched);
    }

    #[test]
    fn resize_tasks_coalesce_to_the_latest_viewport() {
        let (sender, receiver) = mpsc::channel();
        let mut queue = DocumentQueue::new();
        let started = Instant::now();
        for (sample, width) in [900, 901, 902].into_iter().enumerate() {
            sender
                .send(Command::Resize(ResizeRequest::new(
                    Viewport {
                        width,
                        height: 700,
                        scale: 1.0,
                    },
                    started + Duration::from_millis(sample as u64),
                )))
                .unwrap();
        }
        let Ok(Command::Resize(resize)) = queue.next(&receiver, None) else {
            panic!("expected a coalesced resize");
        };
        assert_eq!(resize.viewport.width, 902);
        assert_eq!(resize.queued_at, started);
        assert_eq!(resize.samples, 3);
    }

    #[test]
    fn visual_properties_wait_for_one_frame_ack_and_keep_latest() {
        let (sender, receiver) = mpsc::channel();
        let initial = Viewport {
            width: 800,
            height: 600,
            scale: 1.0,
        };
        let started = Instant::now();
        let mut throttle = VisualPropertiesThrottle::new(sender, initial);
        throttle.resize(ResizeRequest::new(
            Viewport {
                width: 900,
                ..initial
            },
            started,
        ));
        throttle.resize(ResizeRequest::new(
            Viewport {
                width: 1000,
                ..initial
            },
            started + Duration::from_millis(1),
        ));
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));

        // Like OnLocalSurfaceIdChanged, the ACK is not tied to an exact resize
        // id: even an older accepted frame releases the newest pending value.
        throttle.ack();
        let Command::Resize(resize) = receiver.recv().unwrap() else {
            panic!("expected latest visual properties");
        };
        assert_eq!(resize.viewport.width, 1000);
        assert_eq!(resize.queued_at, started);
        assert_eq!(resize.samples, 2);
    }

    #[test]
    fn identical_visual_properties_do_not_create_an_ack_dependency() {
        let (sender, receiver) = mpsc::channel();
        let initial = Viewport {
            width: 800,
            height: 600,
            scale: 1.0,
        };
        let mut throttle = VisualPropertiesThrottle::new(sender, initial);
        throttle.ack();
        throttle.resize(ResizeRequest::new(initial, Instant::now()));
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        assert!(!throttle.ack_pending);
    }

    #[test]
    fn shown_sync_restores_resize_dropped_while_hidden() {
        let (sender, receiver) = mpsc::channel();
        let initial = Viewport {
            width: 800,
            height: 600,
            scale: 1.0,
        };
        let resized = Viewport {
            width: 1000,
            ..initial
        };
        let started = Instant::now();
        let mut throttle = VisualPropertiesThrottle::new(sender, initial);

        // BrowserView accepts the new root viewport while the renderer still
        // waits for its initial visual-properties ACK.
        throttle.resize(ResizeRequest::new(resized, started));
        assert_eq!(throttle.pending_latest.unwrap().viewport, resized);

        // WasHidden releases the ACK and discards the pending transmission.
        // WasShown/SynchronizeVisualProperties must submit the current value
        // again even though the tab already stores that root viewport.
        throttle.hidden();
        throttle.resize(ResizeRequest::new(
            resized,
            started + Duration::from_millis(1),
        ));
        let Command::Resize(resize) = receiver.recv().unwrap() else {
            panic!("expected shown visual-properties synchronization");
        };
        assert_eq!(resize.viewport, resized);
        assert!(throttle.ack_pending);
        assert!(throttle.pending_latest.is_none());
    }

    #[test]
    fn scale_only_visual_properties_are_not_ack_throttled() {
        let (sender, receiver) = mpsc::channel();
        let initial = Viewport {
            width: 800,
            height: 600,
            scale: 1.0,
        };
        let mut throttle = VisualPropertiesThrottle::new(sender, initial);
        throttle.ack();
        for scale in [1.5, 2.0] {
            throttle.resize(ResizeRequest::new(
                Viewport { scale, ..initial },
                Instant::now(),
            ));
        }
        for scale in [1.5, 2.0] {
            let Command::Resize(resize) = receiver.recv().unwrap() else {
                panic!("expected scale-only visual properties");
            };
            assert_eq!(resize.viewport.scale, scale);
        }
        assert!(!throttle.ack_pending);
    }
}
