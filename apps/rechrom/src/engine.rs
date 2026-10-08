//! DOM, layout, JS and raster presentation stay on one browser thread.
//! The UI thread sends commands and receives presentation notifications; owned
//! RGBA frames remain available through the diagnostic headless mailbox.
use crate::begin_frame_source::NativeBeginFrame;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use event_loop::{
    EventLoopExecutor, EventLoopEngine, EventLoopMutation, ExecutionContextId, ScheduledTask, TaskId,
    TaskSource, WakeRequest,
};
use foundation::begin_frame::{BeginFrameArgs, BeginFrameSource};
use interaction::input_event::*;
use layoutng_assembly::{
    fragment_tree::FragmentNode,
    internal::layout_input::{Offset, Size},
};
use page_mutation::PageMutation;
use rechrom::page::{Page, PageClient};
use skia::compat::surface::RasterSurface;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    io,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};

pub const TOOLBAR_HEIGHT: f64 = crate::chrome::HEIGHT;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub scale: f64,
}
impl Viewport {
    pub fn toolbar_pixels(self) -> u32 {
        (TOOLBAR_HEIGHT * self.scale).round() as u32
    }
    pub fn logical_width(self) -> f64 {
        f64::from(self.width) / self.scale
    }
    pub fn content_height(self) -> f64 {
        f64::from(self.height.saturating_sub(self.toolbar_pixels()).max(1)) / self.scale
    }
    pub fn valid(self) -> bool {
        self.width > 0
            && self.height > self.toolbar_pixels()
            && self.scale.is_finite()
            && self.scale > 0.0
            && u64::from(self.width) * u64::from(self.height) <= 64 * 1024 * 1024
    }
}
pub struct WindowFrame {
    pub viewport: Viewport,
    pub toolbar: RasterSurface,
    pub content: RasterSurface,
}
pub type FrameMailbox = Arc<Mutex<Option<WindowFrame>>>;
#[derive(Debug)]
pub struct FatalError {
    pub source: &'static str,
    pub message: String,
}
impl FatalError {
    pub fn new(source: &'static str, error: impl std::fmt::Display) -> Self {
        Self {
            source,
            message: error.to_string(),
        }
    }
}
#[derive(Debug)]
pub enum UserEvent {
    OpenDevTools,
    FrameReady,
    ChromeDragRegions {
        viewport: Viewport,
        frame_sequence: u64,
        regions: Vec<crate::chrome::DragRegion>,
    },
    FramePresented(Viewport),
    Location(String),
    Message(String),
    Fatal(FatalError),
    CursorChanged(rechrom::page::Cursor),
    CaretChanged(Option<rechrom::page::CaretRect>),
}
#[derive(Clone)]
pub enum Command {
    Navigate(String),
    Resize(Viewport),
    VSyncDisplayChanged(u32),
    Redraw,
    Input(InputEvent),
    QueuedInput {
        input: InputEvent,
        queued_at: Instant,
        samples: usize,
    },
    FocusAddress,
    SetActive(bool),
    Reload,
    Evaluate {
        source: String,
        reply: mpsc::Sender<Result<(), String>>,
    },
    NewTab,
    CloseActiveTab,
    CycleTab(isize),
    Back,
    Forward,
    Stop,
    WakeLoading,
    BeginFrame(NativeBeginFrame),
}
impl Command {
    fn input(&self) -> Option<&InputEvent> {
        match self {
            Self::Input(input) | Self::QueuedInput { input, .. } => Some(input),
            _ => None,
        }
    }
    fn input_mut(&mut self) -> Option<&mut InputEvent> {
        match self {
            Self::Input(input) | Self::QueuedInput { input, .. } => Some(input),
            _ => None,
        }
    }
    fn merge_input_timing(&mut self, next: &Self) {
        if let Self::QueuedInput {
            queued_at, samples, ..
        } = self
        {
            if let Self::QueuedInput {
                queued_at: next_at,
                samples: next_samples,
                ..
            } = next
            {
                *queued_at = (*queued_at).min(*next_at);
                *samples += next_samples;
            } else {
                *samples += 1;
            }
        }
    }
}
#[derive(Clone)]
pub struct Output {
    pub mailbox: FrameMailbox,
    pub notify: Arc<dyn Fn(UserEvent) + Send + Sync>,
}
impl Output {
    pub(crate) fn publish(&self, frame: WindowFrame) {
        let mut mailbox = self.mailbox.lock().unwrap();
        let wake = mailbox.is_none();
        *mailbox = Some(frame);
        drop(mailbox);
        if wake {
            (self.notify)(UserEvent::FrameReady);
        }
    }
    fn message(&self, error: impl std::fmt::Display) {
        (self.notify)(UserEvent::Message(error.to_string()));
    }
}
// MainThreadSchedulerImpl keeps a presentation ThreadType lease on its page
// thread. PlatformThreadApple maps that type to USER_INTERACTIVE; an ordinary
// std::thread otherwise inherits default QoS, even while composing a window.
fn set_page_thread_presentation_priority() {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
        }
        const QOS_CLASS_USER_INTERACTIVE: u32 = 0x21;
        // Call only on the Page owner. This changes scheduling class, never
        // realtime policy. Focus is not visibility: the owner's presentation
        // lease remains active while an unfocused window can still present.
        let result = unsafe { pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0) };
        if result != 0 {
            eprintln!(
                "page-thread-priority: {}",
                io::Error::from_raw_os_error(result)
            );
        }
    }
}

struct PageBeginFrameSource {
    native: Arc<dyn BeginFrameSource>,
    requested: Arc<AtomicBool>,
}
impl BeginFrameSource for PageBeginFrameSource {
    fn request_begin_frame(&self) {
        self.requested.store(true, Ordering::Release);
        self.native.request_begin_frame();
    }
    fn set_display(&self, display_id: u32) {
        self.native.set_display(display_id);
    }
}

pub fn spawn(
    viewport: Viewport,
    address: String,
    output: Output,
    window_target: crate::window_surface::WindowTarget,
    fallback_interval: Duration,
    display_id: Option<u32>,
    presentation_config: crate::presentation_runtime::Config,
) -> io::Result<mpsc::Sender<Command>> {
    let (sender, receiver) = mpsc::channel();
    let page_frame_requested = Arc::new(AtomicBool::new(true));
    let frame_source = crate::begin_frame_source::create(fallback_interval, display_id)?;
    let input_frames = frame_source.GetFrameSource(0)?;
    let compositor_frames = frame_source.GetFrameSource(1)?;
    let display_frames = frame_source.GetFrameSource(2)?;
    let page_frame_source: Arc<dyn BeginFrameSource> = Arc::new(PageBeginFrameSource {
        native: frame_source.clone(),
        requested: page_frame_requested.clone(),
    });
    let (page_sender, page_receiver) = mpsc::channel();
    let begin_frame_page = page_sender.clone();
    let begin_frame_requested = page_frame_requested.clone();
    let begin_main_frame: crate::presentation_runtime::BeginMainFrameClient =
        Arc::new(move |frame| {
            if begin_frame_requested.swap(false, Ordering::AcqRel) {
                let _ = begin_frame_page.send(Command::BeginFrame(frame));
            }
        });
    let compositor = crate::presentation_runtime::Spawn(
        presentation_config,
        window_target,
        output.clone(),
        input_frames,
        display_frames,
        begin_main_frame,
    )?;
    let begin_frame_compositor = compositor.clone();
    compositor_frames.SetClient(Arc::new(move |frame| {
        let _ = begin_frame_compositor.send(crate::compositor::Command::BeginFrame(frame));
    }))?;
    let loading_sender = page_sender.clone();
    let compositor_dispatch = compositor.clone();
    let page_dispatch = page_sender.clone();
    std::thread::Builder::new()
        .name("browser-host-dispatch".into())
        .spawn(move || {
            while let Ok(command) = receiver.recv() {
                match &command {
                    Command::BeginFrame(frame) => {
                        let _ = compositor_dispatch
                            .send(crate::compositor::Command::BeginFrame(*frame));
                        if !page_frame_requested.swap(false, Ordering::AcqRel) {
                            continue;
                        }
                    }
                    Command::QueuedInput {
                        input: InputEvent::Wheel(event),
                        queued_at,
                        ..
                    } => {
                        let _ = compositor_dispatch.send(crate::compositor::Command::Wheel {
                            event: event.clone(),
                            queued_at: *queued_at,
                        });
                    }
                    Command::Input(InputEvent::Wheel(event)) => {
                        let _ = compositor_dispatch.send(crate::compositor::Command::Wheel {
                            event: event.clone(),
                            queued_at: Instant::now(),
                        });
                    }
                    Command::Resize(viewport) => {
                        let _ =
                            compositor_dispatch.send(crate::compositor::Command::Resize(*viewport));
                    }
                    Command::Redraw => {
                        let _ = compositor_dispatch.send(crate::compositor::Command::Redraw);
                    }
                    Command::Stop => {
                        let _ = compositor_dispatch.send(crate::compositor::Command::Stop);
                    }
                    _ => {}
                }
                let stop = matches!(command, Command::Stop);
                if page_dispatch.send(command).is_err() || stop {
                    break;
                }
            }
        })?;
    std::thread::Builder::new()
        .name("browser-page".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            set_page_thread_presentation_priority();
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run(
                    page_receiver,
                    loading_sender,
                    viewport,
                    address,
                    output.clone(),
                    page_frame_source,
                    compositor,
                )
            }));
            match outcome {
                Ok(Ok(())) => {}
                Ok(Err(error)) => (output.notify)(UserEvent::Fatal(FatalError::new("page", error))),
                Err(payload) => (output.notify)(UserEvent::Fatal(FatalError::new(
                    "page",
                    panic_message(payload),
                ))),
            }
        })?;
    Ok(sender)
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(error) = payload.downcast_ref::<dom::error::DOMException>() {
        return format!("browser thread {:?}: {}", error.kind, error.message);
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return format!("browser thread panicked: {message}");
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return format!("browser thread panicked: {message}");
    }
    "browser thread panicked with a non-string payload".into()
}

fn run(
    receiver: mpsc::Receiver<Command>,
    loading_sender: mpsc::Sender<Command>,
    viewport: Viewport,
    address: String,
    output: Output,
    frame_source: Arc<dyn BeginFrameSource>,
    compositor_sender: mpsc::Sender<crate::compositor::Command>,
) -> io::Result<()> {
    let _target_scope = browser_tracing::scope(browser_tracing::Context {
        target_id: 1,
        ..Default::default()
    });
    let mut state = BrowserState::WithPresentation(
        viewport,
        output,
        PresentationHost::Composited(compositor_sender),
    )?;
    state.loading_sender = Some(loading_sender);
    state
        .toolbar
        .SetBeginFrameSource(Some(frame_source.clone()));
    state.frame_source = Some(frame_source);
    state.navigate(&address, true)?;
    let mut pending = VecDeque::new();
    let mut event_loop = EventLoopEngine::<BrowserEventLoopTask, NativeBeginFrame>::New();
    let mut scheduling = BrowserEventLoopScheduling::default();
    QueueForegroundTurn(&mut event_loop, &mut scheduling, Instant::now());
    loop {
        if state
            .late_scroll
            .as_ref()
            .is_some_and(|frame| Instant::now() >= frame.deadline)
        {
            if let Err(error) = state.finish_late_scroll(false) {
                state.output.message(error);
            }
        }
        let now = Instant::now();
        let event_loop_deadline = match event_loop.NextWakeRequest(now) {
            WakeRequest::Now => Some(now),
            WakeRequest::At(deadline) => Some(deadline),
            WakeRequest::None => state.NextTaskDeadline(now),
            WakeRequest::Stop => break,
        };
        let timeout = state.late_scroll.as_ref().map_or_else(
            || {
                event_loop_deadline.map_or(Duration::from_secs(60 * 60), |deadline| {
                    deadline.saturating_duration_since(now)
                })
            },
            |frame| {
                let frame_deadline = frame
                    .input_deadline
                    .filter(|deadline| now < *deadline)
                    .unwrap_or(frame.deadline)
                    .min(frame.deadline);
                event_loop_deadline
                    .map_or(frame_deadline, |deadline| deadline.min(frame_deadline))
                    .saturating_duration_since(now)
            },
        );
        let mut received_external_work = false;
        match receive_command(&receiver, &mut pending, timeout) {
            Ok(Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(Command::BeginFrame(frame)) => {
                event_loop.Apply(
                    EventLoopMutation::BeginMainFrame {
                        context: ExecutionContextId(1),
                        frame,
                    },
                    Instant::now(),
                );
                received_external_work = true;
            }
            Ok(command) => {
                let source = BrowserTaskSource(&command);
                QueueBrowserTask(
                    &mut event_loop,
                    &mut scheduling,
                    source,
                    BrowserEventLoopTask::Command(command),
                    None,
                    Instant::now(),
                );
                received_external_work = true;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if state
                    .late_scroll
                    .as_ref()
                    .is_some_and(|frame| Instant::now() >= frame.deadline)
                {
                    if let Err(error) = state.finish_late_scroll(false) {
                        state.output.message(error);
                    }
                } else if state.late_scroll.as_ref().is_some_and(|frame| {
                    frame
                        .input_deadline
                        .is_some_and(|deadline| Instant::now() >= deadline)
                }) {
                    // WAIT_FOR_SCROLL ends frame-production blocking. It does
                    // not end the display cycle or force an early submit.
                    if let Err(error) = state.resume_late_scroll_production(false) {
                        state.output.message(error);
                    }
                } else if matches!(
                    event_loop.NextWakeRequest(Instant::now()),
                    WakeRequest::None
                ) {
                    // A Page task source reached the exact deadline returned
                    // by NextTaskDeadline. Execute one complete foreground
                    // turn; further ready work requests another immediate wake.
                    QueueForegroundTurn(&mut event_loop, &mut scheduling, Instant::now());
                }
            }
        }

        if state.late_scroll.is_none()
            && received_external_work
            && matches!(
                event_loop.NextWakeRequest(Instant::now()),
                WakeRequest::None
            )
        {
            QueueForegroundTurn(&mut event_loop, &mut scheduling, Instant::now());
        }

        if !matches!(event_loop.NextWakeRequest(Instant::now()), WakeRequest::Now) {
            continue;
        }
        let mut executor = BrowserEventLoopExecutor {
            state: &mut state,
            receiver: &receiver,
            pending: &mut pending,
            scheduling: &mut scheduling,
            request_foreground: false,
            request_background: false,
        };
        if let Err(error) = event_loop.OnWake(Instant::now(), &mut executor) {
            executor.state.output.message(error);
        }
        let request_foreground = executor.request_foreground;
        let request_background = executor.request_background;
        drop(executor);
        if state.late_scroll.is_none() && request_foreground {
            QueueForegroundTurn(&mut event_loop, &mut scheduling, Instant::now());
        }
        if request_background {
            QueueBackgroundTurn(&mut event_loop, &mut scheduling, Instant::now());
        }
    }
    Ok(())
}

enum BrowserEventLoopTask {
    Command(Command),
    Foreground,
    Background,
}

#[derive(Default)]
struct BrowserEventLoopScheduling {
    next_task_id: u64,
    foreground_queued: bool,
    background_queued: bool,
}

fn BrowserTaskSource(command: &Command) -> TaskSource {
    if command.input().is_some()
        || matches!(
            command,
            Command::FocusAddress
                | Command::Navigate(_)
                | Command::Reload
                | Command::NewTab
                | Command::CloseActiveTab
                | Command::CycleTab(_)
                | Command::Back
                | Command::Forward
        )
    {
        TaskSource::UserInteraction
    } else if matches!(command, Command::WakeLoading) {
        TaskSource::Networking
    } else {
        TaskSource::Internal
    }
}

fn QueueBrowserTask(
    event_loop: &mut EventLoopEngine<BrowserEventLoopTask, NativeBeginFrame>,
    scheduling: &mut BrowserEventLoopScheduling,
    source: TaskSource,
    payload: BrowserEventLoopTask,
    ready_at: Option<Instant>,
    now: Instant,
) {
    let id = TaskId(scheduling.next_task_id);
    scheduling.next_task_id = scheduling.next_task_id.wrapping_add(1);
    event_loop.Apply(
        EventLoopMutation::PostTask(ScheduledTask {
            id,
            context: ExecutionContextId(1),
            source,
            ready_at,
            payload,
        }),
        now,
    );
}

fn QueueForegroundTurn(
    event_loop: &mut EventLoopEngine<BrowserEventLoopTask, NativeBeginFrame>,
    scheduling: &mut BrowserEventLoopScheduling,
    now: Instant,
) {
    if scheduling.foreground_queued {
        return;
    }
    scheduling.foreground_queued = true;
    QueueBrowserTask(
        event_loop,
        scheduling,
        TaskSource::DomManipulation,
        BrowserEventLoopTask::Foreground,
        None,
        now,
    );
}

fn QueueBackgroundTurn(
    event_loop: &mut EventLoopEngine<BrowserEventLoopTask, NativeBeginFrame>,
    scheduling: &mut BrowserEventLoopScheduling,
    now: Instant,
) {
    if scheduling.background_queued {
        return;
    }
    scheduling.background_queued = true;
    QueueBrowserTask(
        event_loop,
        scheduling,
        TaskSource::Internal,
        BrowserEventLoopTask::Background,
        None,
        now,
    );
}

struct BrowserEventLoopExecutor<'a> {
    state: &'a mut BrowserState,
    receiver: &'a mpsc::Receiver<Command>,
    pending: &'a mut VecDeque<Command>,
    scheduling: &'a mut BrowserEventLoopScheduling,
    request_foreground: bool,
    request_background: bool,
}

impl EventLoopExecutor<BrowserEventLoopTask, NativeBeginFrame> for BrowserEventLoopExecutor<'_> {
    type Error = io::Error;

    fn RunTask(
        &mut self,
        _context: ExecutionContextId,
        _source: TaskSource,
        task: BrowserEventLoopTask,
    ) -> io::Result<()> {
        match task {
            BrowserEventLoopTask::Command(command) => {
                dispatch_host_command(self.state, command, self.receiver, self.pending)?;
                self.request_foreground = true;
            }
            BrowserEventLoopTask::Foreground => {
                self.scheduling.foreground_queued = false;
                self.state.publish()?;
                let gesture_frame_pending = self.state.has_priority_frame();
                self.state.tick_foreground(|| {
                    native_command_waiting(self.receiver, self.pending, gesture_frame_pending)
                })?;
                self.state.publish()?;
                self.request_background = true;
            }
            BrowserEventLoopTask::Background => {
                self.scheduling.background_queued = false;
                if native_command_waiting(
                    self.receiver,
                    self.pending,
                    self.state.has_priority_frame(),
                ) {
                    self.request_background = true;
                } else {
                    self.state.tick_background()?;
                    self.state.publish()?;
                }
            }
        }
        Ok(())
    }

    fn UpdateRendering(
        &mut self,
        _context: ExecutionContextId,
        frame: NativeBeginFrame,
    ) -> io::Result<()> {
        dispatch_host_command(
            self.state,
            Command::BeginFrame(frame),
            self.receiver,
            self.pending,
        )?;
        self.request_foreground = true;
        Ok(())
    }
}
fn dispatch_host_command(
    state: &mut BrowserState,
    command: Command,
    receiver: &mpsc::Receiver<Command>,
    pending: &mut VecDeque<Command>,
) -> io::Result<()> {
    let Command::BeginFrame(args) = command else {
        let wheel = matches!(command.input(), Some(InputEvent::Wheel(_)));
        let aligned = command.input().is_some_and(
            interaction::frame_aligned_input_queue::FrameAlignedInputQueue::IsFrameAligned,
        );
        // Preserve discrete input/browser-action ordering before the held draw.
        if state.late_scroll.is_some()
            && !wheel
            && !aligned
            && !matches!(&command, Command::WakeLoading)
        {
            state.finish_late_scroll(false)?;
        }
        if matches!(
            &command,
            Command::Reload
                | Command::NewTab
                | Command::CloseActiveTab
                | Command::CycleTab(_)
                | Command::Back
                | Command::Forward
                | Command::FocusAddress
                | Command::SetActive(false)
                | Command::Resize(_)
        ) {
            state.scroll_active = false;
        }
        state.command(command)?;
        return Ok(());
    };
    state.finish_late_scroll(false)?;
    let args = prepare_begin_frame(args, receiver, pending, |input| {
        if let Err(error) = state.command(input) {
            state.output.message(error);
        }
    });
    state.command(Command::BeginFrame(args))
}

// Chromium's main-thread input queue snapshots ready continuous input when
// dispatching its frame callback. Our input and pulses share a host channel:
// transfer the already-ready continuous prefix into Page before that callback,
// rather than rendering the old offset and leaving wheel for the next tick.
// Discrete events remain ordering boundaries; cap work so incoming input cannot
// indefinitely postpone this frame. This only enqueues input, not its JS task.
fn prepare_begin_frame(
    mut args: NativeBeginFrame,
    receiver: &mpsc::Receiver<Command>,
    pending: &mut VecDeque<Command>,
    mut queue_input: impl FnMut(Command),
) -> NativeBeginFrame {
    for _ in 0..64 {
        let Some(next) = pending.pop_front().or_else(|| receiver.try_recv().ok()) else {
            break;
        };
        if let Command::BeginFrame(newer) = next {
            args = latest_native_frame(args, newer);
        } else if next.input().is_some_and(
            interaction::frame_aligned_input_queue::FrameAlignedInputQueue::IsFrameAligned,
        ) {
            queue_input(next);
        } else {
            pending.push_front(next);
            break;
        }
    }
    args
}

fn latest_native_frame(current: NativeBeginFrame, next: NativeBeginFrame) -> NativeBeginFrame {
    // Current-cycle demand and CV callbacks can enqueue from different threads.
    // A delayed older notification must not replace a newer pulse.
    if current.source_id == next.source_id && current.sequence_number >= next.sequence_number {
        current
    } else {
        next
    }
}
// Ordinary tasks remain independent of display ticks. publish only requests
// a frame here; native raster/compose/present runs inside BeginFrame.
#[cfg(test)]
fn finish_host_turn(
    state: &mut BrowserState,
    receiver: &mpsc::Receiver<Command>,
    pending: &mut VecDeque<Command>,
) -> io::Result<()> {
    state.publish()?;
    // Input dispatch is itself a task with its microtask checkpoint. Like
    // Chromium's main-thread gesture priority, choose queued input before
    // another ordinary task. Probe after raster, since events can arrive
    // during it; retain the command so ordering and delivery stay intact.
    let priority_waiting = native_command_waiting(receiver, pending, state.has_priority_frame());
    if defer_foreground_for_priority(priority_waiting) {
        return Ok(());
    }
    // A completed foreground task must reach the window before an unrelated
    // background task. Both remain ordinary, uninterruptible Page tasks.
    let gesture_frame_pending = state.has_priority_frame();
    // If new input arrives while toolbar work runs, leave the complete content
    // task queued for the next host turn rather than starting behind it.
    if let Err(error) = state.tick_foreground(|| {
        !priority_waiting && native_command_waiting(receiver, pending, gesture_frame_pending)
    }) {
        state.output.message(error);
    }
    state.publish()?;
    if native_command_waiting(receiver, pending, state.has_priority_frame()) {
        return Ok(());
    }
    if let Err(error) = state.tick_background() {
        state.output.message(error);
    }
    state.publish()
}
fn native_command_waiting(
    receiver: &mpsc::Receiver<Command>,
    pending: &mut VecDeque<Command>,
    gesture_frame_pending: bool,
) -> bool {
    // MainThreadSchedulerImpl::kMainThreadGesture prioritizes compositor work
    // for the active gesture, including a pulse between wheel samples.
    // Animation-only pulses still leave loading/script tasks runnable.
    let priority = |command: &Command| {
        !matches!(command, Command::WakeLoading | Command::BeginFrame(_))
            || (gesture_frame_pending && matches!(command, Command::BeginFrame(_)))
    };
    // Loading wakes and animation-only frame pulses are control work.
    // A continuing animation must leave ordinary loading/JS tasks runnable.
    // Look through the bounded wake prefix without reordering or consuming it:
    // a queued wake must not conceal a mouse/key/resize behind it.
    if pending.iter().any(priority) {
        return true;
    }
    for _ in 0..64 {
        let Ok(command) = receiver.try_recv() else {
            return false;
        };
        let native = priority(&command);
        pending.push_back(command);
        if native {
            return true;
        }
    }
    // Yield on an unexpectedly long control prefix too; the normal dispatcher
    // will process it, and this bounded scan never drains an unbounded queue.
    true
}
// The host's ScheduleWork notification is separate from URL/body events.
// Chromium's WorkDeduplicator coalesces requests while a wake is pending.
fn loading_wake_callback(
    sender: mpsc::Sender<Command>,
    pending: Arc<AtomicBool>,
) -> Arc<dyn Fn() + Send + Sync> {
    Arc::new(move || {
        if !pending.swap(true, Ordering::AcqRel) {
            if sender.send(Command::WakeLoading).is_err() {
                pending.store(false, Ordering::Release);
            }
        }
    })
}

// Like a browser input queue, adjacent mouse moves retain the latest position.
// Blink WebMouseWheelEvent coalesces compatible wheel deltas before dispatch.
// This host keeps native phases, modifiers, units and pointer/target fixed.
// A redundant same-position move between wheels is retained once after the
// merged wheel so hover sees the scrolled content; real moves remain boundaries.
fn receive_command(
    receiver: &mpsc::Receiver<Command>,
    pending: &mut VecDeque<Command>,
    timeout: Duration,
) -> Result<Command, mpsc::RecvTimeoutError> {
    let mut command = match pending.pop_front() {
        Some(command) => command,
        None => receiver.recv_timeout(timeout)?,
    };
    if matches!(command, Command::BeginFrame(_)) {
        while let Some(next) = pending.pop_front().or_else(|| receiver.try_recv().ok()) {
            if let Command::BeginFrame(next_args) = next {
                let Command::BeginFrame(current_args) = command else {
                    unreachable!()
                };
                command = Command::BeginFrame(latest_native_frame(current_args, next_args));
            } else {
                pending.push_front(next);
                break;
            }
        }
        return Ok(command);
    }
    // Resize represents the newest visual properties, not a task for every
    // intermediate native drag size. Like RenderWidgetHost's deferred visual
    // properties update, consume only the pending adjacent resize/redraw run.
    // Input, activation, navigation, loading wakes and Stop remain boundaries.
    if matches!(&command, Command::Resize(viewport) if viewport.valid())
        || matches!(&command, Command::Redraw)
    {
        for _ in 0..4096 {
            let next = match pending.pop_front().or_else(|| receiver.try_recv().ok()) {
                Some(next) => next,
                None => break,
            };
            match next {
                Command::Resize(viewport) if viewport.valid() => {
                    command = Command::Resize(viewport)
                }
                Command::Redraw => {} // The selected resize also requires a full repaint.
                next => {
                    pending.push_front(next);
                    break;
                }
            }
        }
        return Ok(command);
    }
    if !matches!(command.input(), Some(InputEvent::Mouse(event)) if event.r#type == MouseEventType::kMove)
        && !matches!(command.input(), Some(InputEvent::Wheel(event)) if matches!(event.phase, WheelPhase::kNone | WheelPhase::kChanged))
    {
        return Ok(command);
    }
    let mut stationary_move = None;
    for _ in 0..4096 {
        let next = match pending.pop_front().or_else(|| receiver.try_recv().ok()) {
            Some(next) => next,
            None => break,
        };
        match (command.input_mut(), next.input()) {
            (Some(InputEvent::Mouse(a)), Some(InputEvent::Mouse(b)))
                if b.r#type == MouseEventType::kMove
                    && a.button == b.button
                    && a.target_node_id == b.target_node_id
                    && a.modifiers == b.modifiers =>
            {
                *a = b.clone();
            }
            (Some(InputEvent::Wheel(a)), Some(InputEvent::Mouse(b)))
                if b.r#type == MouseEventType::kMove
                    && b.button == MouseButton::kNone
                    && a.position == b.position
                    && a.target_node_id == b.target_node_id
                    && a.modifiers == b.modifiers =>
            {
                stationary_move = Some(next);
                continue;
            }
            (Some(InputEvent::Wheel(a)), Some(InputEvent::Wheel(b)))
                if a.CanCoalesce(b)
                    && a.position == b.position
                    && a.target_node_id == b.target_node_id
                    && compatible_wheel_delta(a.delta, b.delta) =>
            {
                a.delta.x += b.delta.x;
                a.delta.y += b.delta.y;
                a.native = b.native.clone();
            }
            _ => {
                pending.push_front(next);
                break;
            }
        }
        command.merge_input_timing(&next);
    }
    if let Some(moved) = stationary_move {
        pending.push_front(moved);
    }
    Ok(command)
}
fn compatible_wheel_delta(a: Offset, b: Offset) -> bool {
    let axis = |a: f64, b: f64| a.is_finite() && b.is_finite() && (a + b).is_finite();
    axis(a.x, b.x) && axis(a.y, b.y)
}

// Page publishes platform-independent feedback at the input boundary. The
// adapter routes only the page currently under the pointer to the native view.
struct HostFeedback {
    in_toolbar: Cell<bool>,
    active_tab: Cell<u64>,
    navigations: RefCell<VecDeque<(u64, u64, rechrom::page::NavigationRequest)>>,
    cursor: Cell<rechrom::page::Cursor>,
    output: Output,
}
impl HostFeedback {
    fn publish(&self, cursor: rechrom::page::Cursor, toolbar: bool, force: bool) {
        if self.in_toolbar.get() != toolbar {
            return;
        }
        let changed = self.cursor.replace(cursor) != cursor;
        // Native focus/leave handling can reset the OS cursor independently.
        // Reassert feedback for a fresh input, even if Page's value is cached.
        if force || changed {
            if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some() {
                eprintln!("cursor-notify {cursor:?}");
            }
            (self.output.notify)(UserEvent::CursorChanged(cursor));
        }
    }
}

struct Client {
    pointer: Rc<HostFeedback>,
    toolbar: bool,
    tab_id: u64,
    document_generation: u64,
    trace: bool,
    started: Instant,
}
impl PageClient for Client {
    fn DidRequestNavigation(&mut self, request: &rechrom::page::NavigationRequest) {
        if !self.toolbar {
            self.pointer.navigations.borrow_mut().push_back((
                self.tab_id,
                self.document_generation,
                request.clone(),
            ));
        }
    }
    fn DidChangeCursor(&mut self, cursor: rechrom::page::Cursor) {
        if self.toolbar || self.pointer.active_tab.get() == self.tab_id {
            self.pointer.publish(cursor, self.toolbar, true);
        }
    }
    fn DidCommit(&mut self, url: &str) {
        if self.trace {
            eprintln!(
                "loading-commit ms={:.3} url={url}",
                self.started.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
    fn DidPresentFrame(&mut self, frame: &rechrom::page::PageFrame) {
        if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some() {
            if let Some(caret) = frame.display_items.caret {
                eprintln!(
                    "caret-paint sequence={} node={} visible={} rect={:?}",
                    frame.sequence, caret.node_id, caret.visible, caret.rect
                );
            }
        }
        if self.trace {
            eprintln!(
                "loading-frame ms={:.3} sequence={} items={}",
                self.started.elapsed().as_secs_f64() * 1000.0,
                frame.sequence,
                frame.display_items.items.len()
            );
        }
    }
    fn DidFinishLoad(&mut self) {
        if self.trace {
            eprintln!(
                "loading-finish ms={:.3}",
                self.started.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
    fn DidFailResource(&mut self, url: &str, error: &str) {
        eprintln!("resource: {url}: {error}");
    }
    fn DidReportScriptError(
        &mut self,
        error: &javascript::javascript_runtime::JavaScriptException,
    ) {
        eprintln!("script: {}: {}", error.source_name, error.message);
        if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some() {
            eprintln!("script-stack: {}", error.stack);
        }
    }
}
fn create_page(
    width: f64,
    height: f64,
    scale: f64,
    scripting: bool,
    pointer: Rc<HostFeedback>,
    toolbar: bool,
    tab_id: u64,
    document_generation: u64,
) -> io::Result<Page> {
    let mut options = url_loader::DefaultURLLoaderOptions::default();
    // Chromium can multiplex many subresource requests over HTTP/2. Keep the
    // embedder-wide scheduler from letting deferred scripts and media occupy
    // every slot while render-blocking stylesheets wait behind them.
    options.max_parallel_requests = 32;
    options.trace_requests = std::env::var_os("BROWSER_APP_TRACE_NETWORK").is_some();
    let loader = Rc::new(RefCell::new(url_loader::DefaultURLLoader::new(
        options.clone(),
    )?));
    let images = Rc::new(RefCell::new(
        image_decoder::skia_image_decoder::SkiaImageDecoder,
    ));
    let assembly = rechrom::CreateLayoutAssembly();
    let mut constraints =
        rechrom::CreateBrowserConstraints(width.ceil() as u32, height.ceil() as u32);
    constraints.available_size = Size { width, height };
    constraints.device_pixel_ratio = scale;
    let document_images = Rc::new(RefCell::new(
        document_image::SVGImageDecoder::new_with_constraints(&assembly, &constraints),
    ));
    let environment = if scripting {
        Some(rechrom::page::ScriptEnvironment {
            // This embedding owns a 16 MiB thread. The unoptimized translated
            // VM uses about 87 KiB per call; retain 8 MiB for host/layout frames.
            runtime: Box::new(javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_native_stack_budget(8 * 1024 * 1024)),
            xhr: xhr_transport::CreateHTTPXMLHttpRequestTransport(&options)?,
            user_agent: options.user_agent,
        })
    } else {
        None
    };
    Ok(Page::Create(
        loader,
        images,
        document_images,
        constraints,
        environment,
        Some(Rc::new(RefCell::new(Client {
            pointer,
            toolbar,
            tab_id,
            document_generation,
            trace: scripting && std::env::var_os("BROWSER_APP_TRACE_LOADING").is_some(),
            started: Instant::now(),
        }))),
    ))
}
struct ChromePopup {
    page: Page,
    x: f64,
    width: f64,
    height: f64,
    actions: Vec<String>,
    selected: Option<usize>,
    hovered: Option<u64>,
}
#[derive(Clone, Copy)]
struct NativeInputTrace {
    queued_at: Instant,
    samples: usize,
    commands: usize,
}
impl NativeInputTrace {
    fn merge(&mut self, queued_at: Instant, samples: usize) {
        self.queued_at = self.queued_at.min(queued_at);
        self.samples += samples;
        self.commands += 1;
    }
}
struct LateScrollFrame {
    frame: NativeBeginFrame,
    started: Instant,
    opened_at: Instant,
    input_deadline: Option<Instant>,
    // DisplayScheduler deadline after reserving the final third for draw/swap.
    deadline: Instant,
    submitted_before: u64,
    input_attempted_at: Option<Instant>,
    input_dispatched: bool,
    production_started: bool,
    // Completion of Page/paint-artifact production, before the window
    // compositor rasterizes and composes the final IOSurface.
    compositor_input_ready_at: Option<Instant>,
}

enum PresentationHost {
    Readback,
    Composited(mpsc::Sender<crate::compositor::Command>),
}

impl PresentationHost {
    fn IsComposited(&self) -> bool {
        matches!(self, Self::Composited(_))
    }

    fn Compositor(&self) -> Option<&mpsc::Sender<crate::compositor::Command>> {
        match self {
            Self::Readback => None,
            Self::Composited(sender) => Some(sender),
        }
    }
}

fn late_scroll_deadline(
    frame: NativeBeginFrame,
    active: bool,
    has_wheel: bool,
    now: Instant,
) -> Option<Instant> {
    // cc::Scheduler WAIT_FOR_SCROLL and kWaitForLateScrollEventsDeadlineRatio.
    // Ready input is never held. Only an empty active-scroll cycle can accept
    // real late input; no extrapolated deltas or additional frame are created.
    if !active || has_wheel {
        return None;
    }
    let window = frame.interval.checked_mul(333)? / 1000;
    let deadline = frame.frame_time.checked_add(window)?.min(frame.deadline);
    (now < deadline).then_some(deadline)
}

fn display_draw_deadline(frame: NativeBeginFrame) -> Instant {
    viz::FrameTiming {
        frame_time: frame.frame_time,
        interval: frame.interval,
        source_deadline: frame.deadline,
    }
    .Deadlines()
    .draw_and_swap
}

// Chromium raises compositor/input work above ordinary main-thread work during
// a gesture; it does not suspend the main-thread queues for the whole gesture.
// This single-owner host therefore yields while priority work is actually
// queued, then uses the first queue gap for one complete Page task. Keeping the
// gesture bit itself as a blanket veto loses DocumentLoader wakeups and can
// leave a progressively loaded page frozen until scrolling stops.
#[cfg(test)]
fn defer_foreground_for_priority(priority_waiting: bool) -> bool {
    priority_waiting
}

struct BrowserState {
    loading_sender: Option<mpsc::Sender<Command>>,
    loading_wake_pending: Arc<AtomicBool>,
    viewport: Viewport,
    toolbar: Page,
    popup: Option<ChromePopup>,
    chrome_hovered: Vec<u64>,
    reload_loading: Option<bool>,
    content_press_discarded: bool,
    presented_sequence: u64,
    pending_native_input: Option<NativeInputTrace>,
    dispatching_native_input: bool,
    dispatch_input_timing: Option<(Instant, usize)>,
    deferred_native_input: bool,
    frame_source: Option<Arc<dyn BeginFrameSource>>,
    handling_begin_frame: bool,
    scroll_active: bool,
    late_scroll: Option<LateScrollFrame>,
    address_id: u64,
    // Static toolbar controls survive tabstrip replacement, like address_id.
    chrome_id: u64,
    omnibox_id: u64,
    dragspace_id: u64,
    tabs: crate::tabs::Tabs<Page>,
    tabstrip_id: u64,
    tabstrip_html: String,
    background_turn: usize,
    toolbar_focused: bool,
    content_pointer_position: Option<Offset>,
    pointer: Rc<HostFeedback>,
    caret_rect: Option<rechrom::page::CaretRect>,
    active: bool,
    last_frame: Option<(u64, u64, Viewport)>,
    failed_frame: Option<(u64, u64, Viewport)>,
    output: Output,
    presentation: PresentationHost,
}
impl BrowserState {
    fn NextTaskDeadline(&self, now: Instant) -> Option<Instant> {
        let mut wake = self.toolbar.NextTaskDeadline(now);
        let mut include = |candidate: Option<Instant>| {
            if let Some(candidate) = candidate {
                wake = Some(wake.map_or(candidate, |current| current.min(candidate)));
            }
        };
        include(
            self.tabs
                .active
                .page
                .as_ref()
                .and_then(|page| page.NextTaskDeadline(now)),
        );
        if let Some(popup) = &self.popup {
            include(popup.page.NextTaskDeadline(now));
        }
        for tab in &self.tabs.background {
            include(
                tab.page
                    .as_ref()
                    .and_then(|page| page.NextTaskDeadline(now)),
            );
        }
        wake
    }
    #[cfg(test)]
    fn new(viewport: Viewport, output: Output) -> io::Result<Self> {
        Self::WithPresentation(viewport, output, PresentationHost::Readback)
    }

    fn WithPresentation(
        viewport: Viewport,
        output: Output,
        presentation: PresentationHost,
    ) -> io::Result<Self> {
        if !viewport.valid() {
            return Err(io::Error::other("invalid window viewport"));
        }
        let pointer = Rc::new(HostFeedback {
            in_toolbar: Cell::new(false),
            active_tab: Cell::new(1),
            navigations: RefCell::new(VecDeque::new()),
            cursor: Cell::new(rechrom::page::Cursor::kDefault),
            output: output.clone(),
        });
        let mut toolbar = create_page(
            viewport.logical_width(),
            TOOLBAR_HEIGHT,
            viewport.scale,
            false,
            pointer.clone(),
            true,
            0,
            0,
        )?;
        // Open runs the normal stylesheet collection path, including <style>.
        toolbar.Open(&data_url(&crate::chrome::document()), 16384, 4096)?;
        while toolbar.IsLoading() {
            toolbar.RunTask()?;
        }
        toolbar.SetActive(false)?;
        let address_id = find_id(&toolbar, "address")
            .ok_or_else(|| io::Error::other("toolbar has no address input"))?;
        let tabstrip_id = find_id(&toolbar, "tabs").unwrap();
        let chrome_id = find_id(&toolbar, "chrome").unwrap();
        let omnibox_id = find_id(&toolbar, "omnibox").unwrap();
        let dragspace_id = find_id(&toolbar, "dragspace").unwrap();
        Ok(Self {
            loading_sender: None,
            loading_wake_pending: Arc::new(AtomicBool::new(false)),
            viewport,
            toolbar,
            popup: None,
            chrome_hovered: Vec::new(),
            reload_loading: None,
            content_press_discarded: false,
            presented_sequence: 0,
            pending_native_input: None,
            dispatching_native_input: false,
            dispatch_input_timing: None,
            deferred_native_input: false,
            frame_source: None,
            handling_begin_frame: false,
            scroll_active: false,
            late_scroll: None,
            address_id,
            chrome_id,
            omnibox_id,
            dragspace_id,
            tabs: crate::tabs::Tabs::new(),
            tabstrip_id,
            tabstrip_html: String::new(),
            background_turn: 0,
            toolbar_focused: false,
            content_pointer_position: None,
            pointer,
            caret_rect: None,
            active: true,
            last_frame: None,
            failed_frame: None,
            output,
            presentation,
        })
    }
    #[cfg(test)]
    fn tick(&mut self) -> io::Result<()> {
        self.tick_foreground(|| false)?;
        self.tick_background()
    }
    fn has_pending_frame_input(&self) -> bool {
        self.toolbar.HasPendingFrameInput()
            || self
                .tabs
                .active
                .page
                .as_ref()
                .is_some_and(Page::HasPendingFrameInput)
            || self
                .popup
                .as_ref()
                .is_some_and(|popup| popup.page.HasPendingFrameInput())
    }
    fn async_root_scroll_eligible(&self) -> bool {
        self.presentation.IsComposited() && self.popup.is_none()
    }
    fn has_priority_frame(&self) -> bool {
        if !self.async_root_scroll_eligible()
            || self
                .tabs
                .active
                .page
                .as_ref()
                .is_some_and(Page::HasBlockingWheelListener)
        {
            return self.scroll_active || self.has_pending_frame_input();
        }
        // Threaded scrolling is already consumed by the display owner.  Its
        // matching main-thread wheel delivery must make forward progress, but
        // must not classify every following BeginFrame as a reason to starve
        // DocumentLoader/JS.  Non-wheel input retains main-thread priority.
        self.toolbar.HasPendingFrameInput()
            || self
                .popup
                .as_ref()
                .is_some_and(|popup| popup.page.HasPendingFrameInput())
            || self.tabs.active.page.as_ref().is_some_and(|page| {
                page.HasPendingFrameInput() && !page.HasPendingWheelFrameInput()
            })
    }
    fn tick_foreground(
        &mut self,
        mut native_input_waiting: impl FnMut() -> bool,
    ) -> io::Result<()> {
        let _trace = browser_tracing::span("task", "ForegroundTasks");
        let started = Instant::now();
        {
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 2,
                ..Default::default()
            });
            let _trace = browser_tracing::span("task", "ToolbarTasks");
            self.toolbar.RunTask()?;
        }
        // Chrome and content are separate complete Page tasks. Input which
        // arrived during Chrome's task gets the next host turn before starting
        // another content task; never interrupt either task/checkpoint.
        if native_input_waiting() {
            return Ok(());
        }
        let result = if let Some(page) = &mut self.tabs.active.page {
            let _trace = browser_tracing::span("task", "ContentTasks");
            page.RunTask()
        } else {
            Ok(())
        };
        if std::env::var_os("BROWSER_APP_TRACE_LOADING").is_some()
            && started.elapsed() >= Duration::from_millis(16)
        {
            eprintln!(
                "loading-task ms={:.3}",
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        if let Err(error) = result {
            self.output.message(&error);
            // Blink DocumentLoader::LoadFailed stops the parser after commit;
            // it does not replace a received document with a navigation error.
            let committed_load_failure =
                self.tabs.active.page.as_ref().is_some_and(|page| {
                    page.HasCommittedDocument() && page.LoadingFailure().is_some()
                });
            if !committed_load_failure {
                if let Some(mut old) = self.tabs.active.page.take() {
                    old.StopLoading();
                }
                self.tabs.active.document_generation =
                    self.tabs.active.document_generation.wrapping_add(1);
                let html = format!("<!doctype html><style>body{{font:16px sans-serif;padding:32px}}h1{{font-size:26px}}</style><h1>Unable to load page</h1><p>{}</p><pre>{}</pre>", escape(&self.tabs.active.location), escape(&error.to_string()));
                let mut page = create_page(
                    self.viewport.logical_width(),
                    self.viewport.content_height(),
                    self.viewport.scale,
                    false,
                    self.pointer.clone(),
                    false,
                    self.tabs.active.id,
                    self.tabs.active.document_generation,
                )?;
                page.Open(&data_url(&html), 16384, 4096)?;
                page.SetActive(self.active && !self.toolbar_focused)?;
                page.SetBeginFrameSource(self.frame_source.clone());
                self.tabs.active.page = Some(page);
                self.last_frame = None;
                self.failed_frame = None;
                return Ok(());
            }
        }
        if let Some(page) = &self.tabs.active.page {
            let url = page.URL();
            if !url.is_empty()
                && !url.starts_with("data:")
                && !self.tabs.active.location.starts_with("about:")
                && self.tabs.active.location != url
            {
                self.tabs.active.location = url.to_owned();
                if let Some(entry) = self
                    .tabs
                    .active
                    .history
                    .get_mut(self.tabs.active.history_index)
                {
                    *entry = self.tabs.active.location.clone();
                }
                self.set_address(&self.tabs.active.location.clone())?;
                (self.output.notify)(UserEvent::Location(self.tabs.active.location.clone()));
            }
        }
        update_tab_metadata(&mut self.tabs.active);
        self.refresh_tabstrip()?;
        self.drain_navigation_requests()?;
        Ok(())
    }

    fn produce_frame_lifecycle(
        &mut self,
        args: BeginFrameArgs,
        run_toolbar: bool,
    ) -> io::Result<()> {
        if run_toolbar {
            let _target = browser_tracing::scope(browser_tracing::Context {
                target_id: 2,
                ..Default::default()
            });
            self.toolbar.UpdateRendering(args)?;
        }
        if let Some(page) = &mut self.tabs.active.page {
            page.UpdateRendering(args)?;
        }
        if let Some(popup) = &mut self.popup {
            popup.page.UpdateRendering(args)?;
        }
        self.record_frame_input_dispatches();
        self.drain_navigation_requests()?;
        Ok(())
    }

    fn prepare_compositor_input(&self) -> io::Result<()> {
        if self.presentation.IsComposited() {
            return self.send_compositor_snapshot();
        }
        // Protocol readback renders through Page::PaintInto when publish()
        // materializes its image. It has no native presentation surface.
        Ok(())
    }

    fn send_compositor_snapshot(&self) -> io::Result<()> {
        let Some(sender) = self.presentation.Compositor() else {
            return Ok(());
        };
        let toolbar = self
            .toolbar
            .CurrentFrame()
            .ok_or_else(|| io::Error::other("toolbar has no frame"))?;
        let page = self.tabs.active.page.as_ref().and_then(Page::CurrentFrame);
        let content = self
            .popup
            .as_ref()
            .map(|popup| {
                crate::chrome::content_with_popup(
                    page.map(|frame| frame.display_items.as_ref()),
                    popup.page.CurrentFrame().unwrap().display_items.as_ref(),
                )
            })
            .transpose()?
            .map(Arc::new)
            .or_else(|| page.map(|frame| frame.display_items.clone()));
        let mut drag_regions = vec![crate::chrome::DragRegion {
            x: 92.0,
            y: 0.0,
            width: (self.viewport.logical_width() - 92.0).max(0.0),
            height: 6.0,
        }];
        let id = self.dragspace_id;
        fn bounds(
            fragment: &FragmentNode,
            id: u64,
            parent: Offset,
        ) -> Option<crate::chrome::DragRegion> {
            let origin = Offset {
                x: parent.x + fragment.offset.x,
                y: parent.y + fragment.offset.y,
            };
            if fragment.node_id == id {
                return Some(crate::chrome::DragRegion {
                    x: origin.x,
                    y: origin.y,
                    width: fragment.size.width,
                    height: fragment.size.height,
                });
            }
            fragment
                .children
                .iter()
                .find_map(|child| bounds(child, id, origin))
        }
        if let Some(region) = bounds(&toolbar.fragments, id, Offset::default()) {
            drag_regions.push(region);
        }
        let signature = self.current_frame_signature()?;
        let async_root_scroll = self.async_root_scroll_eligible();
        let blocking_wheel_regions = self
            .tabs
            .active
            .page
            .as_ref()
            .map(Page::BlockingWheelEventRegions)
            .unwrap_or_default();
        sender
            .send(crate::compositor::Command::Snapshot(
                crate::compositor::ArtifactSnapshot {
                    document: (self.tabs.active.id, self.tabs.active.document_generation),
                    viewport: self.viewport,
                    frame_time: Instant::now(),
                    signature,
                    toolbar: toolbar.display_items.clone(),
                    content,
                    drag_regions,
                    async_root_scroll,
                    blocking_wheel_regions,
                },
            ))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "compositor stopped"))
    }

    fn resume_late_scroll_production(&mut self, input_arrived: bool) -> io::Result<()> {
        let args = {
            let Some(frame) = &mut self.late_scroll else {
                return Ok(());
            };
            if frame.production_started {
                return Ok(());
            }
            frame.production_started = true;
            frame.input_deadline = None;
            if input_arrived {
                frame.input_attempted_at.get_or_insert_with(Instant::now);
                frame.input_dispatched = true;
            }
            frame.frame.args
        };
        let previous = std::mem::replace(&mut self.handling_begin_frame, true);
        let result = self
            .produce_frame_lifecycle(args, false)
            .and_then(|_| self.prepare_compositor_input());
        if result.is_ok() {
            // Once WAIT_FOR_SCROLL closes, this cycle's compositor input is
            // immutable. Produce the native candidate now, during the middle
            // third of the interval, but keep publication at the fixed display
            // deadline. This preserves cadence while avoiding a full software
            // composition starting only after the draw deadline.
            if let Some(frame) = &mut self.late_scroll {
                frame.compositor_input_ready_at = Some(Instant::now());
            }
        }
        self.handling_begin_frame = previous;
        result
    }

    fn begin_frame(&mut self, frame: impl Into<NativeBeginFrame>) -> io::Result<()> {
        let frame = frame.into();
        let args = frame.args;
        let _frame_scope = browser_tracing::scope(browser_tracing::Context {
            target_id: 1,
            source_id: args.source_id,
            frame_id: args.sequence_number,
            input_id: self
                .pending_native_input
                .as_ref()
                .map_or(0, |input| browser_tracing::instant_id(input.queued_at)),
            ..Default::default()
        });
        let mut frame_trace = browser_tracing::span("frame", "BeginFrame");
        let started = Instant::now();
        let draw_deadline = display_draw_deadline(frame);
        frame_trace.set("scroll_active", self.scroll_active as u8 as f64);
        frame_trace.set("interval_ms", args.interval.as_secs_f64() * 1000.0);
        frame_trace.set(
            "source_deadline_ms",
            args.deadline
                .saturating_duration_since(args.frame_time)
                .as_secs_f64()
                * 1000.0,
        );
        frame_trace.set(
            "deadline_ms",
            draw_deadline
                .saturating_duration_since(args.frame_time)
                .as_secs_f64()
                * 1000.0,
        );
        frame_trace.set(
            "start_age_ms",
            started
                .saturating_duration_since(args.frame_time)
                .as_secs_f64()
                * 1000.0,
        );
        browser_tracing::interval("frame", "BeginFrameQueue", args.frame_time, started, &[]);
        if self.presentation.IsComposited() {
            // Threaded scrolling and Draw/Swap have their own display owner.
            // A BeginMainFrame performs only main-thread input/lifecycle and
            // publishes an immutable artifact; it never waits for Viz's draw
            // deadline.  This is the essential scheduler boundary in Chromium.
            self.handling_begin_frame = true;
            let result = self
                .produce_frame_lifecycle(args, true)
                .and_then(|_| self.send_compositor_snapshot());
            self.handling_begin_frame = false;
            let finished = Instant::now();
            browser_tracing::interval(
                "frame",
                "MainFrameCycle",
                started,
                finished,
                &[
                    ("interval_ms", args.interval.as_secs_f64() * 1000.0),
                    (
                        "start_age_ms",
                        started
                            .saturating_duration_since(args.frame_time)
                            .as_secs_f64()
                            * 1000.0,
                    ),
                    ("succeeded", result.is_ok() as u8 as f64),
                ],
            );
            frame_trace.set("succeeded", result.is_ok() as u8 as f64);
            return result;
        }
        let submitted_before = self.presented_sequence;
        // LayerTreeHostImpl::WillBeginImplFrame uses HasQueuedInput, not
        // whether input was received in the preceding cycle. A dispatched
        // gesture boundary must not close this frame's late-input window.
        let has_wheel = self
            .tabs
            .active
            .page
            .as_ref()
            .is_some_and(Page::HasPendingWheelFrameInput);
        let retained = self.presentation.IsComposited()
            && self.popup.is_none()
            && self
                .tabs
                .active
                .page
                .as_ref()
                .and_then(Page::CurrentFrame)
                .is_some();
        let input_deadline = late_scroll_deadline(
            frame,
            self.scroll_active && retained,
            has_wheel,
            Instant::now(),
        );
        let schedule_draw = self.scroll_active && retained && Instant::now() < draw_deadline;
        browser_tracing::instant(
            "frame",
            "ScrollFrameDecision",
            &[
                ("scroll_active", self.scroll_active as u8 as f64),
                ("retained_content", retained as u8 as f64),
                ("has_wheel", has_wheel as u8 as f64),
                ("toolbar_focused", self.toolbar_focused as u8 as f64),
                (
                    "display_target_known",
                    frame.display_time.is_some() as u8 as f64,
                ),
                ("late_window_open", input_deadline.is_some() as u8 as f64),
                ("display_draw_scheduled", schedule_draw as u8 as f64),
                ("input_deadline_ms", frame.interval.as_secs_f64() * 333.0),
                (
                    "draw_deadline_ms",
                    draw_deadline
                        .saturating_duration_since(frame.frame_time)
                        .as_secs_f64()
                        * 1000.0,
                ),
                (
                    "source_deadline_ms",
                    frame
                        .deadline
                        .saturating_duration_since(frame.frame_time)
                        .as_secs_f64()
                        * 1000.0,
                ),
            ],
        );
        self.handling_begin_frame = true;
        let result = (|| {
            {
                let _target = browser_tracing::scope(browser_tracing::Context {
                    target_id: 2,
                    ..Default::default()
                });
                self.toolbar.UpdateRendering(args)?;
            }
            if schedule_draw && input_deadline.is_some() {
                // Chromium blocks BeginMainFrame while WAIT_FOR_SCROLL is
                // active, so main-thread effects observe the accepted scroll.
                self.late_scroll = Some(LateScrollFrame {
                    frame,
                    started,
                    opened_at: Instant::now(),
                    input_deadline,
                    deadline: draw_deadline,
                    submitted_before,
                    input_attempted_at: None,
                    input_dispatched: false,
                    production_started: false,
                    compositor_input_ready_at: None,
                });
            } else {
                self.produce_frame_lifecycle(args, false)?;
                self.prepare_compositor_input()?;
                if schedule_draw {
                    let compositor_input_ready_at = Instant::now();
                    self.late_scroll = Some(LateScrollFrame {
                        frame,
                        started,
                        opened_at: Instant::now(),
                        input_deadline: None,
                        deadline: draw_deadline,
                        submitted_before,
                        input_attempted_at: has_wheel.then(Instant::now),
                        input_dispatched: has_wheel,
                        production_started: true,
                        compositor_input_ready_at: Some(compositor_input_ready_at),
                    });
                } else {
                    let draw_started = Instant::now();
                    self.publish()?;
                    self.profile_begin_frame(
                        frame,
                        started,
                        draw_started,
                        submitted_before,
                        Duration::ZERO,
                    );
                }
            }
            // Keep the compositor's native cycle alive during a real gesture,
            // so the next empty cycle can receive an input that arrives late.
            if self.scroll_active {
                if let Some(source) = &self.frame_source {
                    source.request_begin_frame();
                }
            }
            Ok(())
        })();
        self.handling_begin_frame = false;
        frame_trace.set(
            "submitted",
            if self.presented_sequence != submitted_before {
                1.0
            } else {
                0.0
            },
        );
        frame_trace.set(
            "late_scroll_pending",
            if self.late_scroll.is_some() { 1.0 } else { 0.0 },
        );
        frame_trace.set("succeeded", if result.is_ok() { 1.0 } else { 0.0 });
        if std::env::var_os("BROWSER_PROFILE_FRAMES").is_some() && result.is_err() {
            eprintln!("begin-frame source={} sequence={} interval_ms={:.3} age_ms={:.3} present_failed=true",
                args.source_id, args.sequence_number, args.interval.as_secs_f64() * 1000.0,
                args.frame_time.elapsed().as_secs_f64() * 1000.0);
        }
        result
    }
    fn finish_late_scroll(&mut self, input_arrived: bool) -> io::Result<()> {
        // Remove the held cycle before dispatch, so an input/navigation error
        // cannot leave it open for a second dispatch in the same native cycle.
        let Some(mut frame) = self.late_scroll.take() else {
            return Ok(());
        };
        let _frame_scope = browser_tracing::scope(browser_tracing::Context {
            target_id: 1,
            source_id: frame.frame.source_id,
            frame_id: frame.frame.sequence_number,
            input_id: self
                .pending_native_input
                .as_ref()
                .map_or(0, |input| browser_tracing::instant_id(input.queued_at)),
            ..Default::default()
        });
        let mut frame_trace = browser_tracing::span("frame", "LateScrollFinish");
        let draw_started = Instant::now();
        let input_wait_end = frame.input_attempted_at.unwrap_or_else(|| {
            frame
                .input_deadline
                .unwrap_or(draw_started)
                .min(draw_started)
        });
        let input_wait = input_wait_end.saturating_duration_since(frame.opened_at);
        let wait = draw_started.saturating_duration_since(frame.opened_at);
        let display_wait_start = frame
            .compositor_input_ready_at
            .unwrap_or(frame.opened_at)
            .min(draw_started);
        frame_trace.set("source_id", frame.frame.source_id as f64);
        frame_trace.set("input_arrived", if input_arrived { 1.0 } else { 0.0 });
        frame_trace.set("input_dispatched", frame.input_dispatched as u8 as f64);
        frame_trace.set("wait_ms", wait.as_secs_f64() * 1000.0);
        frame_trace.set("input_wait_ms", input_wait.as_secs_f64() * 1000.0);
        browser_tracing::interval(
            "frame",
            "LateScrollWait",
            frame.opened_at,
            input_wait_end,
            &[],
        );
        // Candidate production may now overlap the middle third of the frame.
        // Show only the time after a complete candidate exists as display wait;
        // PrepareWindowFrame/ComposeDraw already account for active CPU work.
        browser_tracing::interval(
            "frame",
            "DisplayDrawWait",
            display_wait_start,
            draw_started,
            &[],
        );
        self.handling_begin_frame = true;
        let result = (|| {
            if !frame.production_started {
                frame.production_started = true;
                self.produce_frame_lifecycle(frame.frame.args, false)?;
                self.prepare_compositor_input()?;
                frame.compositor_input_ready_at = Some(Instant::now());
            } else if frame.input_dispatched {
                self.drain_navigation_requests()?;
            }
            let ready_at = frame.compositor_input_ready_at.unwrap_or_else(Instant::now);
            let ready_before_deadline_ms = if frame.deadline >= ready_at {
                frame.deadline.duration_since(ready_at).as_secs_f64() * 1000.0
            } else {
                -ready_at.duration_since(frame.deadline).as_secs_f64() * 1000.0
            };
            browser_tracing::instant_at(
                "frame",
                "CompositorInputReady",
                ready_at,
                &[("ready_before_draw_deadline_ms", ready_before_deadline_ms)],
            );
            if ready_at > frame.deadline {
                // At the regular DisplayScheduler deadline no fresh upstream
                // frame was available. Repeat the previous display frame and
                // let the following BeginFrame consume the latest state. This
                // is the readiness decision; final window composition has not
                // started and therefore no completed IOSurface is discarded.
                browser_tracing::instant(
                    "frame",
                    "FrameRepeated",
                    &[
                        ("reason", 2.0),
                        (
                            "compositor_input_late_by_ms",
                            ready_at.duration_since(frame.deadline).as_secs_f64() * 1000.0,
                        ),
                    ],
                );
                if let Some(source) = &self.frame_source {
                    source.request_begin_frame();
                }
                Ok(())
            } else {
                // Display owns DrawAndSwap and swap throttling. Page only
                // publishes the immutable compositor input selected for this
                // frame; there is no second native presentation path here.
                Ok(())
            }
        })();
        self.handling_begin_frame = false;
        frame_trace.set("succeeded", if result.is_ok() { 1.0 } else { 0.0 });
        self.profile_begin_frame(
            frame.frame,
            frame.started,
            draw_started,
            frame.submitted_before,
            wait,
        );
        result
    }
    fn profile_begin_frame(
        &self,
        frame: NativeBeginFrame,
        started: Instant,
        draw_started: Instant,
        submitted_before: u64,
        waited: Duration,
    ) {
        let profile = std::env::var_os("BROWSER_PROFILE_FRAMES").is_some();
        if !profile && !browser_tracing::enabled() {
            return;
        }
        let args = frame.args;
        let draw_deadline = display_draw_deadline(frame);
        let submitted = self.presented_sequence != submitted_before;
        // Native readiness and swap timing belong to Display and are traced
        // there. Page owns only main-frame/compositor-input readiness.
        let ready_margin_ms = f64::NAN;
        let target_offset_ms = frame.display_time.map_or(f64::NAN, |target| {
            target
                .saturating_duration_since(args.frame_time)
                .as_secs_f64()
                * 1000.0
        });
        let finished = Instant::now();
        browser_tracing::interval(
            "frame",
            "FrameCycle",
            started,
            finished,
            &[
                ("scroll_active", self.scroll_active as u8 as f64),
                ("interval_ms", args.interval.as_secs_f64() * 1000.0),
                (
                    "source_deadline_ms",
                    args.deadline
                        .saturating_duration_since(args.frame_time)
                        .as_secs_f64()
                        * 1000.0,
                ),
                (
                    "deadline_ms",
                    draw_deadline
                        .saturating_duration_since(args.frame_time)
                        .as_secs_f64()
                        * 1000.0,
                ),
                (
                    "start_age_ms",
                    started
                        .saturating_duration_since(args.frame_time)
                        .as_secs_f64()
                        * 1000.0,
                ),
                (
                    "draw_start_age_ms",
                    draw_started
                        .saturating_duration_since(args.frame_time)
                        .as_secs_f64()
                        * 1000.0,
                ),
                (
                    "deadline_missed",
                    if draw_started > draw_deadline {
                        1.0
                    } else {
                        0.0
                    },
                ),
                ("submitted", if submitted { 1.0 } else { 0.0 }),
                ("late_scroll_wait_ms", waited.as_secs_f64() * 1000.0),
                ("ready_to_target_ms", ready_margin_ms),
            ],
        );
        if !profile {
            return;
        }
        eprintln!("begin-frame source={} sequence={} interval_ms={:.3} age_ms={:.3} presented_sequence={} start_age_ms={:.3} work_ms={:.3} deadline_missed={} submitted={} display_target_offset_ms={:.3} ready_to_target_ms={:.3} late_scroll_wait_ms={:.3}",
            args.source_id, args.sequence_number, args.interval.as_secs_f64() * 1000.0,
            args.frame_time.elapsed().as_secs_f64() * 1000.0, self.presented_sequence,
            started.saturating_duration_since(args.frame_time).as_secs_f64() * 1000.0,
            started.elapsed().saturating_sub(waited).as_secs_f64() * 1000.0, draw_started > draw_deadline,
            submitted, target_offset_ms, ready_margin_ms, waited.as_secs_f64() * 1000.0);
    }
    fn record_frame_input_dispatches(&mut self) {
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
        let record = |target_id, event: rechrom::page::DispatchedFrameInput| {
            let _input_scope = browser_tracing::scope(browser_tracing::Context {
                target_id,
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
            if !profile {
                return;
            }
            let kind = match event.input {
                InputEvent::Wheel(_) => "wheel",
                InputEvent::Mouse(_) => "mouse",
                _ => "other",
            };
            eprintln!(
                "native-input-dispatch kind={kind} samples={} queue_ms={:.3} frame_aligned=true",
                event.samples,
                event
                    .dispatch_started
                    .saturating_duration_since(event.queued_at)
                    .as_secs_f64()
                    * 1000.0
            );
            eprintln!("native-input-complete kind={kind} samples={} dispatch_ms={:.3} enqueue_to_dispatch_complete_ms={:.3}", event.samples,
                event.dispatch_finished.duration_since(event.dispatch_started).as_secs_f64() * 1000.0,
                event.dispatch_finished.saturating_duration_since(event.queued_at).as_secs_f64() * 1000.0);
        };
        for event in self.toolbar.TakeDispatchedFrameInputs() {
            record(2, event);
        }
        if let Some(page) = &mut self.tabs.active.page {
            for event in page.TakeDispatchedFrameInputs() {
                record(1, event);
            }
        }
        if let Some(popup) = &mut self.popup {
            for event in popup.page.TakeDispatchedFrameInputs() {
                record(1, event);
            }
        }
    }
    fn tick_background(&mut self) -> io::Result<()> {
        if self.tabs.background.is_empty() {
            return Ok(());
        }
        let mut trace = browser_tracing::span("task", "BackgroundTasks");
        let started = Instant::now();
        // One background Page task per host turn, round robin. Do not drain a
        // background document's task queue before yielding to native input.
        if !self.tabs.background.is_empty() {
            let index = self.background_turn % self.tabs.background.len();
            self.background_turn = self.background_turn.wrapping_add(1);
            let tab = &mut self.tabs.background[index];
            trace.set("tab_id", tab.id as f64);
            if let Some(page) = &mut tab.page {
                if let Err(error) = page.RunTask() {
                    self.output.message(error);
                }
            }
            update_tab_metadata(tab);
        }
        if std::env::var_os("BROWSER_APP_TRACE_LOADING").is_some()
            && started.elapsed() >= Duration::from_millis(16)
        {
            eprintln!(
                "background-task ms={:.3}",
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        self.refresh_tabstrip()?;
        self.drain_navigation_requests()?;
        Ok(())
    }
    fn navigate(&mut self, address: &str, record: bool) -> io::Result<()> {
        self.navigate_request(
            url_loader::URLRequest {
                url: address.into(),
                ..Default::default()
            },
            record,
            true,
        )
    }
    fn navigate_request(
        &mut self,
        mut request: url_loader::URLRequest,
        record: bool,
        present: bool,
    ) -> io::Result<()> {
        let _open_trace = browser_tracing::span("navigation", "Open");
        self.scroll_active = false;
        if present {
            self.popup = None;
        }
        let address = crate::options::normalize_address(&request.url)?;
        if present {
            self.set_address(&address)?;
            (self.output.notify)(UserEvent::Location(format!("Loading — {address}")));
            self.publish()?;
        }
        if let Some(mut old) = self.tabs.active.page.take() {
            old.StopLoading(); // Navigation discards this document/realm only.
        }
        self.tabs.active.document_generation = self.tabs.active.document_generation.wrapping_add(1);
        let mut page = create_page(
            self.viewport.logical_width(),
            self.viewport.content_height(),
            self.viewport.scale,
            true,
            self.pointer.clone(),
            false,
            self.tabs.active.id,
            self.tabs.active.document_generation,
        )?;
        request.url = match address.as_str() {
            "about:home" => data_url(HOME),
            "about:blank" => data_url("<!doctype html><html><body></body></html>"),
            _ => address.clone(),
        };
        if let Some(sender) = &self.loading_sender {
            page.SetLoadingWakeCallback(loading_wake_callback(
                sender.clone(),
                self.loading_wake_pending.clone(),
            ));
        }
        if let Err(error) = page.OpenRequest(&request, 16384, 4096) {
            self.output.message(&error);
            drop(page);
            page = create_page(
                self.viewport.logical_width(),
                self.viewport.content_height(),
                self.viewport.scale,
                false,
                self.pointer.clone(),
                false,
                self.tabs.active.id,
                self.tabs.active.document_generation,
            )?;
            let html = format!("<!doctype html><style>body{{font:16px sans-serif;padding:32px;color:#263445}}h1{{font-size:26px}}</style><h1>Unable to load page</h1><p>{}</p><pre>{}</pre>", escape(&address), escape(&error.to_string()));
            page.Open(&data_url(&html), 16384, 4096)?;
        }
        self.tabs.active.location = if address.starts_with("about:")
            || page.URL().starts_with("data:")
            || page.URL().is_empty()
        {
            address
        } else {
            page.URL().to_owned()
        };
        page.SetActive(present && self.active)?;
        page.SetLastMousePosition(self.content_pointer_position);
        page.SetBeginFrameSource(self.frame_source.clone());
        self.tabs.active.page = Some(page);
        self.tabs.active.viewport = Some(self.viewport);
        self.tabs.active.title = location_title(&self.tabs.active.location);
        self.tabs.active.title_node_id = None;
        self.tabs.active.metadata_nodes = 0;
        self.tabs.active.metadata_sequence = 0;
        if present {
            self.toolbar_focused = false;
            self.toolbar.SetActive(false)?;
            self.last_frame = None;
            self.failed_frame = None;
        }
        if record {
            if !self.tabs.active.history.is_empty() {
                self.tabs
                    .active
                    .history
                    .truncate(self.tabs.active.history_index + 1);
            }
            self.tabs
                .active
                .history
                .push(self.tabs.active.location.clone());
            self.tabs.active.history_index = self.tabs.active.history.len() - 1;
        }
        if present {
            self.set_address(&self.tabs.active.location.clone())?;
            self.refresh_tabstrip()?;
            (self.output.notify)(UserEvent::Location(self.tabs.active.location.clone()));
            self.publish()?;
        }
        Ok(())
    }
    fn replace_current_history_entry(&mut self) {
        if let Some(entry) = self
            .tabs
            .active
            .history
            .get_mut(self.tabs.active.history_index)
        {
            *entry = self.tabs.active.location.clone();
        }
    }
    fn drain_navigation_requests(&mut self) -> io::Result<()> {
        // These callbacks can run inside JS/input dispatch. Start navigation
        // only after that Page call has returned, retaining its source tab and
        // document generation so closed/replaced documents cannot navigate it.
        loop {
            let next = self.pointer.navigations.borrow_mut().pop_front();
            let Some((tab_id, generation, navigation)) = next else {
                return Ok(());
            };
            let valid = self
                .tabs
                .ordered()
                .iter()
                .any(|tab| tab.id == tab_id && tab.document_generation == generation);
            if !valid {
                continue;
            }
            if navigation.target.eq_ignore_ascii_case("_blank") {
                self.new_tab_request(navigation.request, false)?;
            } else if tab_id == self.tabs.active.id {
                self.navigate_request(navigation.request, !navigation.replace_history, true)?;
                if navigation.replace_history {
                    self.replace_current_history_entry();
                }
            } else {
                let foreground = self.tabs.active.id;
                self.tabs.activate(tab_id);
                let result =
                    self.navigate_request(navigation.request, !navigation.replace_history, false);
                if result.is_ok() && navigation.replace_history {
                    self.replace_current_history_entry();
                }
                self.tabs.activate(foreground);
                result?;
                self.refresh_tabstrip()?;
            }
            // One navigation per host turn; later requests stay queued. This
            // preserves task/input yielding even when script opens many tabs.
            return Ok(());
        }
    }
    fn refresh_tabstrip(&mut self) -> io::Result<()> {
        let html = crate::chrome::tab_strip(&self.tabs);
        let loading = self.tabs.active.page.as_ref().is_some_and(Page::IsLoading);
        if self.reload_loading != Some(loading) {
            let id = find_id(&self.toolbar, "reload").unwrap();
            self.toolbar.Apply(PageMutation::DOMMutation(DOMMutation {
                mutation_type: DOMMutationType::kSetInnerHTML,
                target_node_id: id,
                value: crate::chrome::reload_icon(loading).into(),
                ..Default::default()
            }))?;
            self.set_chrome_attribute(
                "reload",
                "aria-label",
                Some(if loading { "Stop loading" } else { "Reload" }),
            )?;
            self.set_chrome_attribute(
                "reload",
                "title",
                Some(if loading {
                    "Stop loading"
                } else {
                    "Reload (⌘R)"
                }),
            )?;
            self.reload_loading = Some(loading);
        }
        self.set_navigation_enabled("back", self.tabs.active.history_index > 0)?;
        self.set_navigation_enabled(
            "forward",
            self.tabs.active.history_index + 1 < self.tabs.active.history.len(),
        )?;
        self.set_chrome_attribute(
            "tabs",
            "style",
            Some(&format!(
                "flex-basis:{}px",
                self.tabs.ordered().len() * 238 - 6
            )),
        )?;
        if html == self.tabstrip_html {
            return Ok(());
        }
        self.toolbar.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kSetInnerHTML,
            target_node_id: self.tabstrip_id,
            value: html.clone(),
            ..Default::default()
        }))?;
        self.tabstrip_html = html;
        Ok(())
    }
    fn set_navigation_enabled(&mut self, name: &str, enabled: bool) -> io::Result<()> {
        self.set_chrome_attribute(name, "disabled", (!enabled).then_some(""))?;
        // Views changes icon ink alongside Button::SetEnabled. Use SVG's
        // presentation attribute so this does not depend on :disabled support.
        let paths = {
            let owner = self.toolbar.Document();
            let doc = owner.GetDocument();
            fn visit(
                doc: &dom::persistent_document::PersistentDocument,
                i: usize,
                out: &mut Vec<u64>,
            ) {
                let node = doc.Node(i);
                if node.Name() == "path" {
                    out.push(node.Id());
                }
                for &child in node.Children() {
                    visit(doc, child, out);
                }
            }
            let mut paths = Vec::new();
            if let Some(id) = find_id(&self.toolbar, name) {
                visit(doc, doc.FindNodeById(id).unwrap(), &mut paths);
            }
            paths
        };
        let ink = if enabled { "#444746" } else { "#bfc1c3" };
        for id in paths {
            let equal = {
                let owner = self.toolbar.Document();
                let doc = owner.GetDocument();
                doc.Node(doc.FindNodeById(id).unwrap())
                    .FindAttribute("fill")
                    .is_some_and(|attr| attr.value == ink)
            };
            if !equal {
                self.toolbar.Apply(PageMutation::DOMMutation(DOMMutation {
                    mutation_type: DOMMutationType::kSetAttribute,
                    target_node_id: id,
                    name: "fill".into(),
                    value: ink.into(),
                    ..Default::default()
                }))?;
            }
        }
        Ok(())
    }
    fn open_popup(&mut self, kind: &str) -> io::Result<()> {
        if self.popup.take().is_some() {
            self.last_frame = None;
            self.failed_frame = None;
            return Ok(());
        }
        let rows = match kind {
            "tabsearch" => self
                .tabs
                .ordered()
                .iter()
                .map(|tab| {
                    (
                        format!("activate-{}", tab.id),
                        tab.title.clone(),
                        String::new(),
                    )
                })
                .collect::<Vec<_>>(),
            "site" => vec![(
                "info".into(),
                self.tabs.active.location.clone(),
                String::new(),
            )],
            _ => vec![
                ("newtab".into(), "New tab".into(), "⌘T".into()),
                ("reload".into(), "Reload".into(), "⌘R".into()),
                ("devtools".into(), "Developer tools".into(), "F12".into()),
                ("home".into(), "Home".into(), String::new()),
                ("close".into(), "Close tab".into(), "⌘W".into()),
            ],
        };
        let width = 280.0_f64.min(self.viewport.logical_width() - 16.0);
        let x = if kind == "tabsearch" {
            92.0_f64.min(self.viewport.logical_width() - width - 8.0)
        } else if kind == "site" {
            121.0_f64.min(self.viewport.logical_width() - width - 8.0)
        } else {
            self.viewport.logical_width() - width - 8.0
        };
        let mut page = create_page(
            self.viewport.logical_width(),
            self.viewport.content_height(),
            self.viewport.scale,
            false,
            self.pointer.clone(),
            true,
            0,
            0,
        )?;
        page.Open(
            &data_url(&crate::chrome::popup_document(&rows, width, x)),
            16384,
            4096,
        )?;
        while page.IsLoading() {
            page.RunTask()?;
        }
        page.SetBeginFrameSource(self.frame_source.clone());
        self.popup = Some(ChromePopup {
            page,
            x,
            width,
            height: (rows.len() as f64 * 32.0 + 18.0).min(self.viewport.content_height()),
            actions: rows.iter().map(|row| row.0.clone()).collect(),
            selected: None,
            hovered: None,
        });
        self.last_frame = None;
        self.failed_frame = None;
        Ok(())
    }
    fn chrome_action(&mut self, id: &str) -> io::Result<()> {
        match id {
            "devtools" => {
                self.popup = None;
                (self.output.notify)(UserEvent::OpenDevTools);
                Ok(())
            }
            "back" => self.history(-1),
            "forward" => self.history(1),
            "reload" if self.tabs.active.page.as_ref().is_some_and(Page::IsLoading) => {
                if let Some(page) = &mut self.tabs.active.page {
                    page.StopLoading();
                }
                self.refresh_tabstrip()
            }
            "reload" => self.navigate(&self.tabs.active.location.clone(), false),
            "home" => self.navigate("about:home", true),
            "go" => self.navigate(&self.address(), true),
            "newtab" => self.new_tab("about:home", true),
            "close" => self.close_tab(self.tabs.active.id),
            "menu" | "tabsearch" | "site" => self.open_popup(id),
            id if id.starts_with("close-") => {
                id[6..].parse().map_or(Ok(()), |id| self.close_tab(id))
            }
            id if id.starts_with("activate-") => {
                id[9..].parse().map_or(Ok(()), |id| self.activate_tab(id))
            }
            id if id.starts_with("tab-") || id.starts_with("title-") => id
                .split_once('-')
                .and_then(|(_, id)| id.parse().ok())
                .map_or(Ok(()), |id| self.activate_tab(id)),
            _ => Ok(()),
        }
    }
    fn set_chrome_attribute(
        &mut self,
        name: &str,
        attribute: &str,
        value: Option<&str>,
    ) -> io::Result<()> {
        let id = match name {
            "chrome" => self.chrome_id,
            "omnibox" => self.omnibox_id,
            _ => {
                let Some(id) = find_id(&self.toolbar, name) else {
                    return Ok(());
                };
                id
            }
        };
        let unchanged = {
            let owner = self.toolbar.Document();
            let doc = owner.GetDocument();
            let node = doc.Node(doc.FindNodeById(id).unwrap());
            node.FindAttribute(attribute).map(|a| a.value.as_str()) == value
        };
        if unchanged {
            return Ok(());
        }
        self.toolbar.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: if value.is_some() {
                DOMMutationType::kSetAttribute
            } else {
                DOMMutationType::kRemoveAttribute
            },
            target_node_id: id,
            name: attribute.into(),
            value: value.unwrap_or_default().into(),
            ..Default::default()
        }))
    }
    fn deactivate_tab(&mut self) -> io::Result<()> {
        if let Some(page) = &mut self.tabs.active.page {
            page.FlushFrameInputs()?;
            page.SetActive(false)?;
            // Hidden pages retain the injected source. Ordinary tasks continue,
            // but rAF/lifecycle wait until the page receives frames again.
            // Detaching would enable the headless timer/paint fallback.
        }
        Ok(())
    }
    fn activate_tab(&mut self, id: u64) -> io::Result<()> {
        if self.tabs.active.id == id {
            return self.show_active_tab();
        }
        if !self.tabs.background.iter().any(|tab| tab.id == id) {
            return Ok(());
        }
        self.deactivate_tab()?;
        self.tabs.activate(id);
        self.show_active_tab()
    }
    fn show_active_tab(&mut self) -> io::Result<()> {
        self.popup = None;
        self.pointer.active_tab.set(self.tabs.active.id);
        self.toolbar_focused = false;
        self.last_frame = None;
        self.failed_frame = None;
        if self.tabs.active.page.is_none() {
            return self.navigate("about:home", true);
        }
        if let Some(page) = &mut self.tabs.active.page {
            page.SetBeginFrameSource(self.frame_source.clone());
        }
        // Hidden tabs retain their layout at its last viewport. Apply the
        // current visual properties before this Page receives input again.
        if self.tabs.active.viewport != Some(self.viewport) {
            if let Some(page) = &mut self.tabs.active.page {
                page.ResizeViewport(
                    self.viewport.logical_width(),
                    self.viewport.content_height(),
                    self.viewport.scale,
                )?;
                let _ = page.Evaluate(
                    "window.dispatchEvent(new Event('resize'))",
                    "rechrom:activate-resize",
                );
            }
            self.tabs.active.viewport = Some(self.viewport);
        }
        self.update_active_pages()?;
        self.set_address(&self.tabs.active.location.clone())?;
        self.refresh_tabstrip()?;
        (self.output.notify)(UserEvent::Location(self.tabs.active.location.clone()));
        self.publish()
    }
    fn new_tab(&mut self, address: &str, focus_address: bool) -> io::Result<()> {
        self.new_tab_request(
            url_loader::URLRequest {
                url: address.into(),
                ..Default::default()
            },
            focus_address,
        )
    }
    fn new_tab_request(
        &mut self,
        request: url_loader::URLRequest,
        focus_address: bool,
    ) -> io::Result<()> {
        // Validate first so a malformed URL cannot replace the visible tab.
        crate::options::normalize_address(&request.url)?;
        self.deactivate_tab()?;
        self.tabs.add();
        self.pointer.active_tab.set(self.tabs.active.id);
        self.last_frame = None;
        self.failed_frame = None;
        self.navigate_request(request, true, true)?;
        if focus_address {
            self.command(Command::FocusAddress)?;
        }
        Ok(())
    }
    fn close_tab(&mut self, id: u64) -> io::Result<()> {
        if id == self.tabs.active.id {
            if let Some(page) = &mut self.tabs.active.page {
                page.StopLoading();
            }
        } else if let Some(tab) = self.tabs.background.iter_mut().find(|tab| tab.id == id) {
            if let Some(page) = &mut tab.page {
                page.StopLoading();
            }
        }
        if self.tabs.close(id) {
            self.show_active_tab()
        } else {
            self.toolbar_focused = false;
            self.update_active_pages()?;
            self.refresh_tabstrip()
        }
    }
    fn follow_link(&mut self, request: crate::navigation::NavigationRequest) -> io::Result<()> {
        match request.disposition {
            crate::navigation::Disposition::CurrentTab => self.navigate(&request.url, true),
            crate::navigation::Disposition::NewTab => self.new_tab(&request.url, false),
        }
    }
    fn set_address(&mut self, value: &str) -> io::Result<()> {
        self.toolbar.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kSetControlValue,
            target_node_id: self.address_id,
            value: value.into(),
            ..Default::default()
        }))
    }
    fn address(&self) -> String {
        let owner = self.toolbar.Document();
        let doc = owner.GetDocument();
        doc.ControlValue(doc.FindNodeById(self.address_id).unwrap())
    }
    fn flush_frame_inputs(&mut self) -> io::Result<()> {
        self.toolbar.FlushFrameInputs()?;
        if let Some(page) = &mut self.tabs.active.page {
            page.FlushFrameInputs()?;
        }
        if let Some(popup) = &mut self.popup {
            popup.page.FlushFrameInputs()?;
        }
        self.record_frame_input_dispatches();
        Ok(())
    }
    fn command(&mut self, command: Command) -> io::Result<()> {
        // Cross-Page browser actions must not overtake queued content input.
        // Frame pulses and ordinary loading wakes are not input boundaries.
        if matches!(
            &command,
            Command::FocusAddress
                | Command::SetActive(_)
                | Command::Navigate(_)
                | Command::Reload
                | Command::NewTab
                | Command::CloseActiveTab
                | Command::CycleTab(_)
                | Command::Back
                | Command::Forward
                | Command::Resize(_)
        ) {
            self.flush_frame_inputs()?;
        }
        match command {
            Command::Navigate(address) => self.navigate(&address, true),
            Command::Redraw => {
                self.last_frame = None;
                self.failed_frame = None;
                Ok(())
            }
            Command::Reload => self.navigate(&self.tabs.active.location.clone(), false),
            Command::Evaluate { source, reply } => {
                let result = match self.tabs.active.page.as_mut() {
                    Some(page) => match page.Evaluate(&source, "devtools:Runtime.evaluate") {
                        Ok(result) => match result.exception {
                            Some(exception) => Err(exception.message),
                            None => Ok(()),
                        },
                        Err(error) => Err(error.to_string()),
                    },
                    None => Err("No active page".into()),
                };
                let _ = reply.send(result);
                Ok(())
            }
            Command::Back => self.history(-1),
            Command::Forward => self.history(1),
            Command::NewTab => self.new_tab("about:home", true),
            Command::CloseActiveTab => self.close_tab(self.tabs.active.id),
            Command::CycleTab(delta) => {
                let tabs = self.tabs.ordered();
                let index = tabs
                    .iter()
                    .position(|tab| tab.id == self.tabs.active.id)
                    .unwrap();
                let next = (index as isize + delta).rem_euclid(tabs.len() as isize) as usize;
                self.activate_tab(tabs[next].id)
            }
            Command::Resize(viewport) => {
                self.popup = None;
                if !viewport.valid() {
                    return Ok(());
                }
                // An adjacent native redraw may have been merged with this
                // visual-properties update, including one at the current size.
                self.last_frame = None;
                self.failed_frame = None;
                if viewport == self.viewport {
                    return Ok(());
                }
                #[cfg(feature = "profile_paint")]
                let started = Instant::now();
                self.viewport = viewport;
                self.toolbar.ResizeViewport(
                    viewport.logical_width(),
                    TOOLBAR_HEIGHT,
                    viewport.scale,
                )?;
                #[cfg(feature = "profile_paint")]
                let toolbar_done = Instant::now();
                #[cfg(feature = "profile_paint")]
                let mut page_layout_ms = 0.0;
                if let Some(page) = &mut self.tabs.active.page {
                    page.ResizeViewport(
                        viewport.logical_width(),
                        viewport.content_height(),
                        viewport.scale,
                    )?;
                    #[cfg(feature = "profile_paint")]
                    {
                        page_layout_ms = toolbar_done.elapsed().as_secs_f64() * 1000.0;
                    }
                    let _ = page.Evaluate(
                        "window.dispatchEvent(new Event('resize'))",
                        "rechrom:resize",
                    );
                }
                self.tabs.active.viewport = Some(viewport);
                #[cfg(feature = "profile_paint")]
                if std::env::var_os("BROWSER_APP_PROFILE_RESIZE").is_some() {
                    eprintln!("window-resize width={} height={} scale={} toolbar_ms={:.3} page_layout_ms={:.3} resize_js_ms={:.3} total_ms={:.3}", viewport.width, viewport.height, viewport.scale,
                        (toolbar_done-started).as_secs_f64()*1000.0, page_layout_ms,
                        toolbar_done.elapsed().as_secs_f64()*1000.0-page_layout_ms,
                        started.elapsed().as_secs_f64()*1000.0);
                }
                Ok(())
            }
            Command::FocusAddress => {
                self.popup = None;
                self.last_frame = None;
                self.failed_frame = None;
                self.toolbar_focused = true;
                self.update_active_pages()?;
                self.toolbar.Dispatch(&InputEvent::Focus(FocusEvent {
                    r#type: FocusEventType::kFocus,
                    target_node_id: self.address_id,
                    ..Default::default()
                }))?;
                self.toolbar.Dispatch(&InputEvent::Key(KeyEvent {
                    key: "a".into(),
                    modifiers: EventModifiers {
                        control: true,
                        ..Default::default()
                    },
                    ..Default::default()
                }))?;
                Ok(())
            }
            Command::SetActive(active) => {
                if !active {
                    self.popup = None;
                    self.last_frame = None;
                    self.failed_frame = None;
                    self.content_press_discarded = false;
                }
                self.active = active;
                self.update_active_pages()
            }
            Command::BeginFrame(args) => self.begin_frame(args),
            Command::VSyncDisplayChanged(display_id) => {
                if let Some(source) = &self.frame_source {
                    source.set_display(display_id);
                }
                Ok(())
            }
            Command::Input(input) => self.input(input),
            Command::QueuedInput {
                input,
                queued_at,
                samples,
            } => {
                let profile = std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some()
                    || std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
                let toolbar = match &input {
                    InputEvent::Mouse(event) if event.r#type == MouseEventType::kLeave => {
                        self.pointer.in_toolbar.get()
                    }
                    InputEvent::Mouse(event) => event.position.y < TOOLBAR_HEIGHT,
                    InputEvent::Wheel(event) => event.position.y < TOOLBAR_HEIGHT,
                    _ => self.toolbar_focused,
                };
                let _input_scope = browser_tracing::scope(browser_tracing::Context {
                    target_id: if toolbar { 2 } else { 1 },
                    input_id: browser_tracing::instant_id(queued_at),
                    ..Default::default()
                });
                let mut input_trace = browser_tracing::span("input", "NativeInputReceive");
                input_trace.set("samples", samples as f64);
                input_trace.set(
                    "kind",
                    match &input {
                        InputEvent::Wheel(_) => 1.0,
                        InputEvent::Mouse(_) => 2.0,
                        _ => 3.0,
                    },
                );
                if let InputEvent::Wheel(wheel) = &input {
                    browser_tracing::instant_at(
                        "input",
                        "WheelSample",
                        queued_at,
                        &[
                            ("samples", samples as f64),
                            ("delta_x", wheel.delta.x),
                            ("delta_y", wheel.delta.y),
                            ("phase", wheel.phase as u8 as f64),
                            (
                                "precise",
                                (wheel.delta_units == ScrollGranularity::kScrollByPrecisePixel)
                                    as u8 as f64,
                            ),
                            (
                                "native_phase",
                                wheel.native.as_ref().map_or(f64::NAN, |n| n.phase as f64),
                            ),
                            (
                                "momentum_phase",
                                wheel
                                    .native
                                    .as_ref()
                                    .map_or(f64::NAN, |n| n.momentum_phase as f64),
                            ),
                            (
                                "native_timestamp_seconds",
                                wheel
                                    .native
                                    .as_ref()
                                    .map_or(f64::NAN, |n| n.timestamp_seconds),
                            ),
                        ],
                    );
                }
                if profile || browser_tracing::enabled() {
                    match &mut self.pending_native_input {
                        Some(trace) => trace.merge(queued_at, samples),
                        None => {
                            self.pending_native_input = Some(NativeInputTrace {
                                queued_at,
                                samples,
                                commands: 1,
                            })
                        }
                    }
                }
                self.dispatching_native_input = true;
                self.dispatch_input_timing = Some((queued_at, samples));
                self.deferred_native_input = false;
                let started = Instant::now();
                browser_tracing::interval(
                    "input",
                    "InputOwnerQueue",
                    queued_at,
                    started,
                    &[("samples", samples as f64)],
                );
                let kind = match &input {
                    InputEvent::Mouse(_) => "mouse",
                    InputEvent::Wheel(_) => "wheel",
                    InputEvent::Key(_) => "key",
                    InputEvent::TextInput(_) => "text",
                    _ => "other",
                };
                // Timestamp is captured by the native UI thread, before mpsc.
                // A coalesced event retains the oldest enqueue time, so long
                // script tasks cannot disappear from the queue-latency trace.
                if profile {
                    eprintln!(
                        "native-input-received kind={kind} samples={samples} queue_ms={:.3}",
                        (started - queued_at).as_secs_f64() * 1000.0
                    );
                    if let InputEvent::Wheel(wheel) = &input {
                        eprintln!(
                            "native-wheel-sample phase={:?} units={:?} delta_x={:.3} delta_y={:.3}",
                            wheel.phase, wheel.delta_units, wheel.delta.x, wheel.delta.y
                        );
                    }
                }
                let dispatch_started = Instant::now();
                let result = self.input(input);
                let dispatch_finished = Instant::now();
                self.dispatching_native_input = false;
                self.dispatch_input_timing = None;
                self.record_frame_input_dispatches();
                input_trace.set(
                    "deferred",
                    if self.deferred_native_input { 1.0 } else { 0.0 },
                );
                input_trace.set("succeeded", if result.is_ok() { 1.0 } else { 0.0 });
                if !self.deferred_native_input {
                    browser_tracing::interval(
                        "input",
                        "InputQueue",
                        queued_at,
                        dispatch_started,
                        &[("samples", samples as f64), ("frame_aligned", 0.0)],
                    );
                    browser_tracing::interval(
                        "input",
                        "InputDispatch",
                        dispatch_started,
                        dispatch_finished,
                        &[("samples", samples as f64), ("frame_aligned", 0.0)],
                    );
                    if profile {
                        eprintln!("native-input-complete kind={kind} samples={samples} dispatch_ms={:.3} enqueue_to_dispatch_complete_ms={:.3}",
                        started.elapsed().as_secs_f64() * 1000.0, queued_at.elapsed().as_secs_f64() * 1000.0);
                    }
                }
                result
            }
            Command::WakeLoading => {
                // A wake announces ready work, not one unit of body data.
                // Acknowledge before pumping so newly arriving work can wake
                // the following turn. Bytes/events stay in each loader queue.
                self.loading_wake_pending.store(false, Ordering::Release);
                Ok(())
            }
            Command::Stop => Ok(()),
        }
    }
    fn history(&mut self, delta: isize) -> io::Result<()> {
        let index = self.tabs.active.history_index as isize + delta;
        if index < 0 || index >= self.tabs.active.history.len() as isize {
            return Ok(());
        }
        let index = index as usize;
        self.navigate(&self.tabs.active.history[index].clone(), false)?;
        self.tabs.active.history_index = index;
        self.refresh_tabstrip()
    }
    fn input(&mut self, mut input: InputEvent) -> io::Result<()> {
        if let InputEvent::Wheel(wheel) = &input {
            // Wheel targets follow pointer location, independently of keyboard
            // focus. Only a content gesture drives the retained content cycle.
            let content = wheel.position.y >= TOOLBAR_HEIGHT;
            match wheel.phase {
                WheelPhase::kBegan
                    if content && wheel.delta_units == ScrollGranularity::kScrollByPrecisePixel =>
                {
                    self.scroll_active = true
                }
                WheelPhase::kEnded | WheelPhase::kCancelled => self.scroll_active = false,
                _ => {}
            }
        }
        if !interaction::frame_aligned_input_queue::FrameAlignedInputQueue::IsFrameAligned(&input) {
            self.flush_frame_inputs()?;
        }
        if let Some(popup) = &mut self.popup {
            if matches!(&input, InputEvent::Key(event) if event.r#type == KeyEventType::kDown && event.key == "Escape")
            {
                self.popup = None;
                self.last_frame = None;
                self.failed_frame = None;
                return Ok(());
            }
            if let InputEvent::Key(event) = &input {
                if (event.r#type == KeyEventType::kDown
                    || (event.r#type == KeyEventType::kUp && event.key == " "))
                    && !popup.actions.is_empty()
                {
                    match event.key.as_str() {
                        "ArrowDown" | "ArrowUp" | "Tab" => {
                            let reverse = event.key == "ArrowUp"
                                || (event.key == "Tab" && event.modifiers.shift);
                            let index = popup.selected.map_or(
                                if reverse { popup.actions.len() - 1 } else { 0 },
                                |index| {
                                    if reverse {
                                        (index + popup.actions.len() - 1) % popup.actions.len()
                                    } else {
                                        (index + 1) % popup.actions.len()
                                    }
                                },
                            );
                            popup.selected = Some(index);
                            let id = find_id(&popup.page, &popup.actions[index]).unwrap();
                            if let Some(old) = popup.hovered.replace(id) {
                                set_hover_class(&mut popup.page, old, false)?;
                            }
                            set_hover_class(&mut popup.page, id, true)?;
                            popup.page.Dispatch(&InputEvent::Focus(FocusEvent {
                                r#type: FocusEventType::kFocus,
                                target_node_id: id,
                                ..Default::default()
                            }))?;
                            return Ok(());
                        }
                        "Enter" | " "
                            if event.key == "Enter" || event.r#type == KeyEventType::kUp =>
                        {
                            if let Some(index) = popup.selected {
                                let id = popup.actions[index].clone();
                                self.popup = None;
                                self.last_frame = None;
                                self.failed_frame = None;
                                return self.chrome_action(&id);
                            }
                        }
                        _ => {}
                    }
                }
            }
            if matches!(
                &input,
                InputEvent::TextInput(_) | InputEvent::Composition(_)
            ) {
                return Ok(());
            }
            if let InputEvent::Mouse(event) = &input {
                let point = event.position;
                let inside = point.x >= popup.x
                    && point.x < popup.x + popup.width
                    && point.y >= TOOLBAR_HEIGHT + 4.0
                    && point.y < TOOLBAR_HEIGHT + 4.0 + popup.height;
                if inside {
                    let mut local = event.clone();
                    local.position.y -= TOOLBAR_HEIGHT;
                    let result = popup.page.Dispatch(&InputEvent::Mouse(local))?;
                    if event.r#type == MouseEventType::kMove {
                        let id = result
                            .target_node_id
                            .and_then(|id| element_id(&popup.page, id))
                            .and_then(|name| find_id(&popup.page, &name));
                        if id != popup.hovered {
                            if let Some(old) = popup.hovered {
                                set_hover_class(&mut popup.page, old, false)?;
                            }
                            if let Some(id) = id {
                                set_hover_class(&mut popup.page, id, true)?;
                            }
                            popup.hovered = id;
                        }
                    }
                    if event.r#type == MouseEventType::kClick {
                        let id = result
                            .target_node_id
                            .and_then(|id| element_id(&popup.page, id));
                        self.popup = None;
                        self.last_frame = None;
                        self.failed_frame = None;
                        if let Some(id) = id {
                            return self.chrome_action(&id);
                        }
                    }
                    return Ok(());
                }
                if event.r#type == MouseEventType::kDown && point.y >= TOOLBAR_HEIGHT {
                    self.popup = None;
                    self.last_frame = None;
                    self.failed_frame = None;
                }
            }
        }
        let was_in_toolbar = self.pointer.in_toolbar.get();
        let toolbar = match &mut input {
            InputEvent::Mouse(event) => {
                let toolbar = if event.r#type == MouseEventType::kLeave {
                    self.pointer.in_toolbar.get()
                } else {
                    event.position.y < TOOLBAR_HEIGHT
                };
                self.pointer.in_toolbar.set(toolbar);
                if event.r#type == MouseEventType::kDown {
                    self.toolbar_focused = toolbar;
                    self.content_press_discarded = false;
                }
                if !toolbar {
                    event.position.y -= TOOLBAR_HEIGHT;
                }
                self.content_pointer_position = if event.r#type == MouseEventType::kLeave || toolbar
                {
                    None
                } else {
                    Some(event.position)
                };
                toolbar
            }
            InputEvent::Wheel(event) => {
                let toolbar = event.position.y < TOOLBAR_HEIGHT;
                if !toolbar {
                    event.position.y -= TOOLBAR_HEIGHT;
                }
                toolbar
            }
            _ => self.toolbar_focused,
        };
        // The toolbar and content are separate Page roots. Crossing that host
        // boundary must leave the old content document even though the native
        // window itself only delivered a move.
        if toolbar && !was_in_toolbar && matches!(&input, InputEvent::Mouse(_)) {
            if let Some(page) = &mut self.tabs.active.page {
                if page.CurrentFrame().is_some() {
                    page.Dispatch(&InputEvent::Mouse(MouseEvent {
                        r#type: MouseEventType::kLeave,
                        position: Offset { x: -1.0, y: -1.0 },
                        ..Default::default()
                    }))?;
                }
            }
        }
        if !toolbar && matches!(&input, InputEvent::Mouse(_)) {
            for id in std::mem::take(&mut self.chrome_hovered) {
                set_hover_class(&mut self.toolbar, id, false)?;
            }
        }
        self.update_active_pages()?;
        if toolbar {
            let result = self.toolbar.Dispatch(&input)?;
            if matches!(&input, InputEvent::Mouse(event) if matches!(event.r#type, MouseEventType::kMove | MouseEventType::kLeave))
            {
                let target = if matches!(&input, InputEvent::Mouse(event) if event.r#type == MouseEventType::kLeave)
                {
                    None
                } else {
                    result.target_node_id
                };
                let hovered = chrome_hover_chain(&self.toolbar, target);
                if hovered != self.chrome_hovered {
                    for id in std::mem::take(&mut self.chrome_hovered) {
                        set_hover_class(&mut self.toolbar, id, false)?;
                    }
                    for &id in &hovered {
                        set_hover_class(&mut self.toolbar, id, true)?;
                    }
                    self.chrome_hovered = hovered;
                }
            }
            if let InputEvent::Key(event) = &input {
                if !result.default_prevented {
                    let focus = self.toolbar.FocusedNodeId();
                    if event.r#type == KeyEventType::kDown
                        && event.key == "Enter"
                        && focus == Some(self.address_id)
                    {
                        return self.navigate(&self.address(), true);
                    }
                    if (event.r#type == KeyEventType::kDown && event.key == "Enter")
                        || (event.r#type == KeyEventType::kUp
                            && event.key == " "
                            && focus != Some(self.address_id))
                    {
                        if let Some(id) = focus.and_then(|id| element_id(&self.toolbar, id)) {
                            return self.chrome_action(&id);
                        }
                    }
                }
            }
            if let InputEvent::Mouse(event) = &input {
                if event.r#type == MouseEventType::kDown && self.popup.is_some() {
                    let id = result
                        .target_node_id
                        .and_then(|id| element_id(&self.toolbar, id));
                    if matches!(id.as_deref(), Some("dragspace" | "tabrow" | "tabs")) {
                        self.popup = None;
                        self.last_frame = None;
                        self.failed_frame = None;
                    }
                }
                if event.r#type == MouseEventType::kClick && !result.default_prevented {
                    if let Some(id) = result
                        .target_node_id
                        .and_then(|id| element_id(&self.toolbar, id))
                    {
                        return self.chrome_action(&id);
                    }
                }
            }
        } else if let Some(page) = &mut self.tabs.active.page {
            // There is no content hit-test tree before first paint. Native
            // pointer input during streaming navigation is not a fatal error.
            if page.CurrentFrame().is_none() {
                if matches!(&input, InputEvent::Mouse(event) if event.r#type == MouseEventType::kDown)
                {
                    self.content_press_discarded = true;
                }
                self.pointer
                    .publish(rechrom::page::Cursor::kDefault, false, false);
                return Ok(());
            }
            if self.content_press_discarded {
                if let InputEvent::Mouse(event) = &input {
                    if matches!(
                        event.r#type,
                        MouseEventType::kClick
                            | MouseEventType::kContextMenu
                            | MouseEventType::kDoubleClick
                    ) {
                        self.content_press_discarded = false;
                        return Ok(());
                    }
                    if event.r#type == MouseEventType::kUp {
                        return Ok(());
                    }
                }
            }
            let continuous = matches!(&input, InputEvent::Wheel(event) if matches!(event.phase, WheelPhase::kNone | WheelPhase::kChanged))
                || matches!(&input, InputEvent::Mouse(event) if event.r#type == MouseEventType::kMove);
            if continuous && self.frame_source.is_some() {
                let (queued_at, samples) = self
                    .dispatch_input_timing
                    .unwrap_or_else(|| (Instant::now(), 1));
                let wheel = matches!(&input, InputEvent::Wheel(_));
                page.QueueFrameInput(input, queued_at, samples)?;
                self.deferred_native_input = true;
                if wheel {
                    if let Some(frame) = &mut self.late_scroll {
                        let now = Instant::now();
                        let live_deadline = frame.input_deadline;
                        let admitted = live_deadline.is_some_and(|deadline| now < deadline);
                        let deadline_margin_ms = live_deadline.map_or(f64::NAN, |deadline| {
                            if deadline >= now {
                                deadline.duration_since(now).as_secs_f64() * 1000.0
                            } else {
                                -now.duration_since(deadline).as_secs_f64() * 1000.0
                            }
                        });
                        let _decision_scope = browser_tracing::scope(browser_tracing::Context {
                            target_id: 1,
                            source_id: frame.frame.source_id,
                            frame_id: frame.frame.sequence_number,
                            input_id: browser_tracing::instant_id(queued_at),
                            ..Default::default()
                        });
                        browser_tracing::instant(
                            "input",
                            "LateScrollAdmissionDecision",
                            &[
                                ("admitted", admitted as u8 as f64),
                                ("window_open", live_deadline.is_some() as u8 as f64),
                                (
                                    "arrival_age_ms",
                                    now.saturating_duration_since(frame.frame.frame_time)
                                        .as_secs_f64()
                                        * 1000.0,
                                ),
                                (
                                    "input_deadline_ms",
                                    frame.frame.interval.as_secs_f64() * 333.0,
                                ),
                                ("deadline_margin_ms", deadline_margin_ms),
                            ],
                        );
                        if !admitted {
                            return Ok(());
                        }
                        // InputHandlerProxy queues, checks its live deadline,
                        // then resumes frame production immediately. Page's
                        // BeginMainFrame has not run yet, so this wheel is
                        // drained before rAF and lifecycle work.
                        frame.input_attempted_at = Some(now);
                        self.resume_late_scroll_production(true)?;
                    }
                }
                return Ok(());
            }
            let result = page.Dispatch(&input)?;
            if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some() {
                let owner = page.Document();
                let document = owner.GetDocument();
                let node = result
                    .target_node_id
                    .and_then(|id| document.FindNodeById(id))
                    .map(|i| document.Node(i));
                eprintln!("input-trace event={input:?} target={:?} tag={:?} id={:?} class={:?} cursor={:?} prevented={}",result.target_node_id,node.map(|n|n.Name()),node.and_then(|n|n.FindAttribute("id")).map(|a|a.value.as_str()),node.and_then(|n|n.FindAttribute("class")).map(|a|a.value.as_str()),page.Cursor(),result.default_prevented);
                eprintln!(
                    "input-state value={:?} rendering_ready={} caret={:?}",
                    result
                        .target_node_id
                        .and_then(|id| document.FindNodeById(id))
                        .filter(|&i| document.Node(i).IsHTMLElement("input")
                            || document.Node(i).IsHTMLElement("textarea"))
                        .map(|i| document.ControlValue(i)),
                    page.IsRenderingReady(),
                    page.Caret()
                );
            }
            if !result.default_prevented {
                if let InputEvent::Wheel(event) = &input {
                    let started = Instant::now();
                    scroll(page, event, self.viewport.content_height())?;
                    if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                        eprintln!(
                            "scroll-default-profile ms={:.3} sequence={}",
                            started.elapsed().as_secs_f64() * 1000.0,
                            page.CurrentFrame().map_or(0, |frame| frame.sequence)
                        );
                    }
                }
                if let InputEvent::Mouse(event) = &input {
                    if event.r#type == MouseEventType::kClick
                        && event.button == MouseButton::kPrimary
                    {
                        if let Some(address) = result
                            .target_node_id
                            .and_then(|id| crate::navigation::link_request(page, id))
                        {
                            return self.follow_link(address);
                        }
                    }
                }
            }
        }
        // Deliver pointer feedback before the host drains more commands or runs loading tasks.
        self.publish_cursor();
        self.drain_navigation_requests()?;
        Ok(())
    }
    fn update_active_pages(&mut self) -> io::Result<()> {
        self.toolbar
            .SetActive(self.active && self.toolbar_focused)?;
        if let Some(page) = &mut self.tabs.active.page {
            page.SetActive(self.active && !self.toolbar_focused)?;
        }
        Ok(())
    }
    fn publish_caret(&mut self) {
        let caret = if self.toolbar_focused {
            self.toolbar.Caret().map(|caret| caret.rect)
        } else {
            self.tabs
                .active
                .page
                .as_ref()
                .and_then(Page::Caret)
                .map(|caret| rechrom::page::CaretRect {
                    y: caret.rect.y + TOOLBAR_HEIGHT,
                    ..caret.rect
                })
        };
        if caret != self.caret_rect {
            self.caret_rect = caret;
            (self.output.notify)(UserEvent::CaretChanged(caret));
        }
    }
    fn publish_cursor(&mut self) {
        let toolbar = self.pointer.in_toolbar.get();
        let cursor = if toolbar {
            self.toolbar.Cursor()
        } else {
            self.tabs
                .active
                .page
                .as_ref()
                .map_or(rechrom::page::Cursor::kDefault, Page::Cursor)
        };
        self.pointer.publish(cursor, toolbar, false);
    }
    fn finish_native_input_trace(&mut self, outcome: &str, submitted_at: Instant) {
        // A no-op publish within navigation dispatch is not the final input
        // outcome. Wait until dispatch has returned and the host flushes it.
        if outcome == "no-frame" && self.dispatching_native_input {
            return;
        }
        let Some(trace) = self.pending_native_input.take() else {
            return;
        };
        let _input_scope = browser_tracing::scope(browser_tracing::Context {
            input_id: browser_tracing::instant_id(trace.queued_at),
            ..Default::default()
        });
        let event_name = match outcome {
            "no-frame" => "InputNoFrame",
            "presentation-failed" => "InputPresentationFailed",
            "headless-raster-only" => "InputHeadlessRaster",
            _ => "InputOutcome",
        };
        browser_tracing::interval(
            "input",
            event_name,
            trace.queued_at,
            submitted_at,
            &[
                ("samples", trace.samples as f64),
                ("commands", trace.commands as f64),
                ("present_sequence", self.presented_sequence as f64),
            ],
        );
        if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_none()
            && std::env::var_os("BROWSER_PROFILE_INPUT").is_none()
        {
            return;
        }
        let elapsed_ms = submitted_at.duration_since(trace.queued_at).as_secs_f64() * 1000.0;
        eprintln!("native-input-frame outcome={outcome} frame_sequence={} samples={} commands={} enqueue_to_outcome_ms={elapsed_ms:.3}",
            self.presented_sequence, trace.samples, trace.commands);
    }

    fn current_frame_signature(&self) -> io::Result<(u64, u64, Viewport)> {
        let toolbar = self
            .toolbar
            .CurrentFrame()
            .ok_or_else(|| io::Error::other("toolbar has no frame"))?;
        let popup_sequence = self.popup.as_ref().map_or(0, |popup| {
            popup.page.CurrentFrame().map_or(0, |frame| frame.sequence)
        });
        let page_sequence = self
            .tabs
            .active
            .page
            .as_ref()
            .and_then(Page::CurrentFrame)
            .map_or(0, |frame| frame.sequence);
        Ok((
            toolbar.sequence + popup_sequence,
            page_sequence,
            self.viewport,
        ))
    }

    fn publish(&mut self) -> io::Result<()> {
        self.set_chrome_attribute(
            "chrome",
            "class",
            Some(if self.active { "" } else { "inactive" }),
        )?;
        let address_focused = self.active
            && self.toolbar_focused
            && self.toolbar.FocusedNodeId() == Some(self.address_id);
        self.set_chrome_attribute(
            "omnibox",
            "class",
            Some(if address_focused {
                "focused"
            } else if self.chrome_hovered.contains(&self.omnibox_id) {
                "hovered"
            } else {
                ""
            }),
        )?;
        // Pointer feedback must be delivered even when the frame is unchanged.
        self.publish_cursor();
        self.publish_caret();
        if self.late_scroll.is_some() {
            return Ok(());
        }
        if self.presentation.IsComposited() {
            if !self.handling_begin_frame {
                if let Some(source) = &self.frame_source {
                    source.request_begin_frame();
                }
            } else {
                self.send_compositor_snapshot()?;
            }
            return Ok(());
        }
        // This is an explicit protocol readback host. The
        // native application always uses the compositor/Viz/Display pipeline;
        // there is no Page-direct window fallback.
        let toolbar = self
            .toolbar
            .CurrentFrame()
            .ok_or_else(|| io::Error::other("toolbar has no frame"))?;
        let page = self.tabs.active.page.as_ref().and_then(Page::CurrentFrame);
        let signature = (
            toolbar.sequence
                + self
                    .popup
                    .as_ref()
                    .map_or(0, |popup| popup.page.CurrentFrame().unwrap().sequence),
            page.map_or(0, |frame| frame.sequence),
            self.viewport,
        );
        if self.last_frame == Some(signature) {
            if !self
                .tabs
                .active
                .page
                .as_ref()
                .is_some_and(Page::HasPendingFrameInput)
            {
                self.finish_native_input_trace("no-frame", Instant::now());
            }
            return Ok(());
        }
        // A permanently unsupported paint artifact cannot become renderable by
        // requesting another display tick. Report its failure once; a newer
        // artifact/viewport or explicit redraw permits another attempt.
        if self.failed_frame == Some(signature) {
            self.finish_native_input_trace("presentation-failed", Instant::now());
            return Ok(());
        }
        if !self.handling_begin_frame {
            if let Some(source) = &self.frame_source {
                source.request_begin_frame();
                return Ok(());
            }
        }
        let overlay_content = self
            .popup
            .as_ref()
            .map(|popup| {
                crate::chrome::content_with_popup(
                    page.map(|frame| frame.display_items.as_ref()),
                    popup.page.CurrentFrame().unwrap().display_items.as_ref(),
                )
            })
            .transpose()?
            .map(Arc::new);
        let mut drag_regions = vec![crate::chrome::DragRegion {
            x: 92.0,
            y: 0.0,
            width: (self.viewport.logical_width() - 92.0).max(0.0),
            height: 6.0,
        }];
        {
            let id = self.dragspace_id;
            fn bounds(
                fragment: &FragmentNode,
                id: u64,
                parent: Offset,
            ) -> Option<crate::chrome::DragRegion> {
                let origin = Offset {
                    x: parent.x + fragment.offset.x,
                    y: parent.y + fragment.offset.y,
                };
                if fragment.node_id == id {
                    return Some(crate::chrome::DragRegion {
                        x: origin.x,
                        y: origin.y,
                        width: fragment.size.width,
                        height: fragment.size.height,
                    });
                }
                fragment
                    .children
                    .iter()
                    .find_map(|child| bounds(child, id, origin))
            }
            if let Some(region) = bounds(&toolbar.fragments, id, Offset::default()) {
                drag_regions.push(region);
            }
        }
        // Protocol readback shares the same Page-owned tile path as native presentation.
        let readback =
            |page: &Page, artifact: Option<&Arc<paint::paint_engine::PaintArtifact>>, height| {
                let mut pixels = vec![u32::MAX; self.viewport.width as usize * height as usize];
                if let Some(artifact) = artifact {
                    page.PaintArtifactInto(
                        artifact,
                        self.viewport.width,
                        height,
                        self.viewport.scale,
                        &mut pixels,
                        skia::PixelFormat::Rgba8888,
                        self.viewport.width as usize,
                    )?;
                } else {
                    page.PaintInto(
                        self.viewport.width,
                        height,
                        self.viewport.scale,
                        &mut pixels,
                        skia::PixelFormat::Rgba8888,
                        self.viewport.width as usize,
                    )?;
                }
                RasterSurface::from_pixels(
                    self.viewport.width,
                    height,
                    pixels.into_iter().flat_map(u32::to_ne_bytes).collect(),
                )
            };
        let toolbar_surface = readback(&self.toolbar, None, self.viewport.toolbar_pixels())?;
        let content_height = self.viewport.height - self.viewport.toolbar_pixels();
        let content = if let Some(page) = self.tabs.active.page.as_ref() {
            readback(page, overlay_content.as_ref(), content_height)?
        } else {
            RasterSurface::new(self.viewport.width, content_height)?
        };
        self.presented_sequence = self.presented_sequence.wrapping_add(1);
        self.failed_frame = None;
        self.output.publish(WindowFrame {
            viewport: self.viewport,
            toolbar: toolbar_surface,
            content,
        });
        self.last_frame = Some(signature);
        self.finish_native_input_trace("headless-raster-only", Instant::now());
        (self.output.notify)(UserEvent::ChromeDragRegions {
            viewport: self.viewport,
            frame_sequence: self.presented_sequence,
            regions: drag_regions,
        });
        Ok(())
    }
}
fn location_title(location: &str) -> String {
    if location == "about:home" {
        return "New tab".into();
    }
    if location == "about:blank" {
        return "Blank page".into();
    }
    url::Url::parse(location)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_else(|| {
            if location.starts_with("data:") {
                "Document".into()
            } else {
                location.chars().take(60).collect()
            }
        })
}
fn update_tab_metadata(tab: &mut crate::tabs::Tab<Page>) {
    if tab.location == "about:home" {
        tab.title = "New tab".into();
        return;
    }
    let Some(page) = &tab.page else {
        return;
    };
    if !tab.location.starts_with("about:")
        && !page.URL().is_empty()
        && !page.URL().starts_with("data:")
        && tab.location != page.URL()
    {
        tab.location = page.URL().into();
        if let Some(entry) = tab.history.get_mut(tab.history_index) {
            *entry = tab.location.clone();
        }
    }
    let Some(frame) = page.CurrentFrame() else {
        return;
    };
    if tab.metadata_sequence == frame.sequence {
        return;
    }
    tab.metadata_sequence = frame.sequence;
    let owner = page.Document();
    let document = owner.GetDocument();
    fn connected(document: &dom::persistent_document::PersistentDocument, index: usize) -> bool {
        let mut next = Some(index);
        while let Some(index) = next {
            if index == document.Root() {
                return true;
            }
            next = document.Node(index).Parent();
        }
        false
    }
    if tab
        .title_node_id
        .and_then(|id| document.FindNodeById(id))
        .is_some_and(|index| !connected(document, index))
    {
        tab.title_node_id = None;
    }
    if tab.title_node_id.is_none() && tab.metadata_nodes != document.NodeCount() {
        tab.title_node_id = (0..document.NodeCount()).find_map(|i| {
            (document.Node(i).IsHTMLElement("title") && connected(document, i))
                .then_some(document.Node(i).Id())
        });
    }
    tab.metadata_nodes = document.NodeCount();
    if let Some(index) = tab.title_node_id.and_then(|id| document.FindNodeById(id)) {
        fn collect(
            document: &dom::persistent_document::PersistentDocument,
            index: usize,
            text: &mut String,
        ) {
            let node = document.Node(index);
            if node.Type() == dom::persistent_document::DOMNodeType::kText {
                text.push_str(node.Data());
            }
            for &child in node.Children() {
                collect(document, child, text);
            }
        }
        let mut title = String::new();
        collect(document, index, &mut title);
        let title = title.trim();
        if !title.is_empty() {
            tab.title = title.chars().take(100).collect();
        }
    }
}
fn find_id(page: &Page, name: &str) -> Option<u64> {
    let owner = page.Document();
    let document = owner.GetDocument();
    fn visit(
        document: &dom::persistent_document::PersistentDocument,
        index: usize,
        name: &str,
    ) -> Option<u64> {
        let node = document.Node(index);
        if node
            .FindAttribute("id")
            .is_some_and(|attribute| attribute.value == name)
        {
            return Some(node.Id());
        }
        node.Children()
            .iter()
            .find_map(|&child| visit(document, child, name))
    }
    // InnerHTML leaves detached arena nodes alive; only the connected tree is
    // the current chrome. An old tab label may have the same element id.
    visit(document, document.Root(), name)
}
fn chrome_hover_chain(page: &Page, target: Option<u64>) -> Vec<u64> {
    let owner = page.Document();
    let doc = owner.GetDocument();
    let mut index = target.and_then(|id| doc.FindNodeById(id));
    let mut chain = Vec::new();
    while let Some(i) = index {
        let node = doc.Node(i);
        if let Some(id) = node.FindAttribute("id") {
            if id.value.starts_with("tab-")
                || id.value.starts_with("close-")
                || matches!(
                    id.value.as_str(),
                    "omnibox"
                        | "back"
                        | "forward"
                        | "reload"
                        | "site"
                        | "menu"
                        | "tabsearch"
                        | "newtab"
                )
            {
                chain.push(node.Id());
            }
        }
        index = node.Parent();
    }
    chain
}
fn set_hover_class(page: &mut Page, id: u64, hovered: bool) -> io::Result<()> {
    let value = {
        let owner = page.Document();
        let doc = owner.GetDocument();
        let Some(i) = doc.FindNodeById(id) else {
            return Ok(());
        };
        let mut classes = doc.Node(i).FindAttribute("class").map_or(Vec::new(), |a| {
            a.value
                .split_whitespace()
                .filter(|class| *class != "hovered")
                .map(str::to_owned)
                .collect::<Vec<_>>()
        });
        if hovered {
            classes.push("hovered".into());
        }
        classes.join(" ")
    };
    page.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type: DOMMutationType::kSetAttribute,
        target_node_id: id,
        name: "class".into(),
        value,
        ..Default::default()
    }))
}
fn element_id(page: &Page, target: u64) -> Option<String> {
    let owner = page.Document();
    let document = owner.GetDocument();
    let mut index = document.FindNodeById(target);
    while let Some(i) = index {
        let node = document.Node(i);
        if let Some(id) = node.FindAttribute("id") {
            return Some(id.value.clone());
        }
        index = node.Parent();
    }
    None
}
#[cfg(test)]
fn scroll_candidate(
    fragment: &FragmentNode,
    point: Offset,
    parent: Offset,
    delta: Offset,
    viewport_height: f64,
    root: bool,
) -> Option<(u64, Offset, Offset)> {
    scroll_chain(fragment, point, parent, delta, viewport_height, root)?
        .first()
        .copied()
}

#[cfg(test)]
fn scroll_chain(
    fragment: &FragmentNode,
    point: Offset,
    parent: Offset,
    delta: Offset,
    viewport_height: f64,
    root: bool,
) -> Option<Vec<(u64, Offset, Offset)>> {
    use layoutng_assembly::internal::layout_input::Overflow;
    if fragment.paint.hidden {
        return None;
    }
    let origin = Offset {
        x: parent.x + fragment.offset.x,
        y: parent.y + fragment.offset.y,
    };
    let inside_x = point.x >= origin.x && point.x < origin.x + fragment.size.width;
    let inside_y = point.y >= origin.y && point.y < origin.y + fragment.size.height;
    // A clipped-away descendant must not capture wheel input. Visible
    // overflow can still contain a hit descendant outside the parent's box.
    if !root
        && ((fragment.paint.overflow_x != Overflow::kVisible && !inside_x)
            || (fragment.paint.overflow_y != Overflow::kVisible && !inside_y))
    {
        return None;
    }
    let offset = fragment.paint.scroll_offset;
    let mut child_origin = origin;
    if fragment.paint.establishes_paint_state {
        child_origin.x -= offset.x;
        child_origin.y -= offset.y;
    }
    let mut chain = None;
    for child in fragment.children.iter().rev() {
        if let Some(value) = scroll_chain(child, point, child_origin, delta, viewport_height, false)
        {
            chain = Some(value);
            break;
        }
    }
    let user_scrollable = |overflow| {
        matches!(overflow, Overflow::kAuto | Overflow::kScroll)
            || (root && overflow == Overflow::kVisible)
    };
    let maximum = Offset {
        x: if user_scrollable(fragment.paint.overflow_x) {
            (fragment.paint.scroll_size.width - fragment.size.width).max(0.0)
        } else {
            0.0
        },
        y: if user_scrollable(fragment.paint.overflow_y) {
            if root {
                (fragment
                    .content_size
                    .height
                    .max(fragment.paint.scroll_size.height)
                    - viewport_height)
                    .max(0.0)
            } else {
                (fragment.paint.scroll_size.height - fragment.size.height).max(0.0)
            }
        } else {
            0.0
        },
    };
    let can_move = (delta.x != 0.0 && (offset.x + delta.x).clamp(0.0, maximum.x) != offset.x)
        || (delta.y != 0.0 && (offset.y + delta.y).clamp(0.0, maximum.y) != offset.y);
    // Only real scroll boxes accept wheel input; at a directional boundary
    // keep walking out to an ancestor. In particular, horizontal overflow
    // cannot swallow a vertical gesture (nor can overflow:hidden/clip).
    if (root || inside_x && inside_y)
        && fragment.node_id != 0
        && (root || fragment.paint.establishes_paint_state)
        && can_move
    {
        chain
            .get_or_insert_with(Vec::new)
            .push((fragment.node_id, offset, maximum));
    }
    chain
}
#[cfg(test)]
fn scroll_updates(
    fragments: &FragmentNode,
    wheel: &WheelEvent,
    viewport_height: f64,
) -> Vec<(u64, Offset)> {
    let mut updates: Vec<(u64, Offset)> = Vec::new();
    // Route axes independently and pass unconsumed displacement out through
    // the same ancestor chain. A coalesced gesture must not lose its remainder
    // when an inner scroller reaches its limit partway through the delta.
    for delta in [
        Offset {
            x: wheel.delta.x,
            y: 0.0,
        },
        Offset {
            x: 0.0,
            y: wheel.delta.y,
        },
    ] {
        // A stationary axis cannot select a scroll target. Avoid a second
        // complete fragment-tree walk for ordinary single-axis gestures.
        if delta.x == 0.0 && delta.y == 0.0 {
            continue;
        }
        let mut remaining = delta;
        if let Some(chain) = scroll_chain(
            fragments,
            wheel.position,
            Offset::default(),
            delta,
            viewport_height,
            true,
        ) {
            let mut visited = Vec::new();
            for (id, before, maximum) in chain {
                if visited.contains(&id) {
                    continue;
                }
                visited.push(id);
                let offset = Offset {
                    x: (before.x + remaining.x).clamp(0.0, maximum.x),
                    y: (before.y + remaining.y).clamp(0.0, maximum.y),
                };
                remaining.x -= offset.x - before.x;
                remaining.y -= offset.y - before.y;
                if let Some((_, merged)) = updates.iter_mut().find(|(node, _)| *node == id) {
                    if delta.x != 0.0 {
                        merged.x = offset.x;
                    }
                    if delta.y != 0.0 {
                        merged.y = offset.y;
                    }
                } else if offset != before {
                    updates.push((id, offset));
                }
                if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some() {
                    eprintln!("scroll-target node={id} before={before:?} after={offset:?} maximum={maximum:?} delta={delta:?}");
                }
                if remaining.x == 0.0 && remaining.y == 0.0 {
                    break;
                }
            }
        }
    }
    updates
}
fn scroll(page: &mut Page, wheel: &WheelEvent, _viewport_height: f64) -> io::Result<()> {
    page.ScrollWheelDefault(wheel)
}

fn data_url(html: &str) -> String {
    let mut url = String::from("data:text/html;charset=utf-8,");
    for byte in html.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            url.push(byte as char);
        } else {
            use std::fmt::Write;
            write!(&mut url, "%{byte:02X}").unwrap();
        }
    }
    url
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const HOME: &str = r#"<!doctype html><html><head><title>Rechrom</title><style>
body{margin:0;background:#f7f9fc;color:#243247;font:17px sans-serif}main{max-width:760px;margin:80px auto;padding:32px;background:white;border:1px solid #dce3ed;border-radius:12px}
h1{font-size:34px;color:#172033}p{line-height:1.7}a{color:#2865c7;margin-right:24px}input{font:17px sans-serif;padding:10px;border:1px solid #bbc8d8;border-radius:5px;width:300px}
</style></head><body><main><h1>Rechrom</h1><p>Window, input and page rendering are connected to the Rechrom engine.</p><p>Enter a URL above, or press Ctrl / Cmd + L. Use Ctrl / Cmd + R to reload.</p><p><a href="https://www.baidu.com">Baidu</a><a href="https://www.qq.com">QQ</a></p><p><input placeholder="Try typing here"></p></main></body></html>"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn active_scroll_prioritizes_ready_input_without_suspending_page_tasks() {
        assert!(defer_foreground_for_priority(true));
        assert!(!defer_foreground_for_priority(false));
    }
    #[test]
    fn late_scroll_wait_is_bounded_and_never_holds_ready_input() {
        let base = Instant::now();
        let interval = Duration::from_micros(16666);
        let frame = NativeBeginFrame {
            args: BeginFrameArgs {
                source_id: 1,
                sequence_number: 1,
                frame_time: base,
                deadline: base + interval,
                interval,
            },
            display_time: Some(base + interval * 2),
        };
        let deadline = base + interval * 333 / 1000;
        assert_eq!(
            late_scroll_deadline(frame, true, false, base),
            Some(deadline)
        );
        assert_eq!(late_scroll_deadline(frame, true, true, base), None);
        assert_eq!(late_scroll_deadline(frame, false, false, base), None);
        assert_eq!(late_scroll_deadline(frame, true, false, deadline), None);
        // WAIT_FOR_SCROLL depends on BeginFrameArgs, not presentation feedback.
        assert_eq!(
            late_scroll_deadline(
                NativeBeginFrame {
                    display_time: None,
                    ..frame
                },
                true,
                false,
                base
            ),
            Some(deadline)
        );
    }

    #[test]
    fn late_scroll_updates_current_frame_without_running_raf_twice() {
        std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(|| {
            struct Source;
            impl BeginFrameSource for Source { fn request_begin_frame(&self) {} }
            let output = Output { mailbox: Arc::new(Mutex::new(None)), notify: Arc::new(|_| {}) };
            let mut state = BrowserState::new(Viewport { width: 320, height: 248, scale: 1.0 }, output).unwrap();
            assert!(!state.has_pending_frame_input());
            state.scroll_active = true;
            let (sender, receiver) = mpsc::channel();
            let now = Instant::now();
            sender.send(Command::BeginFrame(BeginFrameArgs { source_id: 1, sequence_number: 1,
                frame_time: now, deadline: now + Duration::from_millis(16),
                interval: Duration::from_millis(16) }.into())).unwrap();
            let mut pending = VecDeque::new();
            assert!(native_command_waiting(&receiver, &mut pending, state.has_priority_frame()),
                "a ready gesture pulse precedes scripts even between wheel samples");
            state.scroll_active = false;
            assert!(!native_command_waiting(&receiver, &mut pending, state.has_priority_frame()),
                "ending the gesture returns animation-only pulses to ordinary priority");
            let mut page = create_page(320.0, 160.0, 1.0, true, state.pointer.clone(), false, 1, 0).unwrap();
            page.Open(&data_url("<!doctype html><style>body{margin:0;height:4000px}</style>scroll"), 16384, 4096).unwrap();
            while page.IsLoading() { page.RunTask().unwrap(); }
            page.SetBeginFrameSource(Some(Arc::new(Source)));
            assert!(page.Evaluate("var rafCount=0;requestAnimationFrame(()=>{rafCount++;requestAnimationFrame(()=>rafCount++);});", "test:late-scroll").unwrap().Succeeded());
            let base = Instant::now();
            let args = BeginFrameArgs { source_id: 7, sequence_number: 1, frame_time: base,
                interval: Duration::from_secs(1), deadline: base + Duration::from_secs(1) };
            page.UpdateRendering(args).unwrap();
            page.QueueFrameInput(InputEvent::Wheel(WheelEvent { phase: WheelPhase::kChanged,
                delta_units: ScrollGranularity::kScrollByPrecisePixel, position: Offset { x: 10.0, y: 10.0 },
                delta: Offset { x: 0.0, y: 20.0 }, ..Default::default() }), Instant::now(), 1).unwrap();
            assert!(!page.OnLateScrollInput(BeginFrameArgs { sequence_number: 2, ..args }).unwrap());
            assert!(page.HasPendingWheelFrameInput());
            assert!(page.OnLateScrollInput(args).unwrap());
            assert!(!page.HasPendingWheelFrameInput());
            let root = page.CurrentFrame().unwrap().fragments.node_id;
            {
                let owner = page.Document();
                let document = owner.GetDocument();
                assert_eq!(document.ScrollOffsetFor(document.FindNodeById(root).unwrap()).y, 20.0);
            }
            assert!(page.Evaluate("if(rafCount!==1)throw Error('late scroll reran RAF');", "test:late-scroll").unwrap().Succeeded());
            // Dispatch at the queue boundary; later host bookkeeping crossing
            // the held draw deadline must neither defer nor replay this input.
            state.tabs.active.page = Some(page);
            state.frame_source = Some(Arc::new(Source));
            let next_args = BeginFrameArgs { sequence_number: 2, ..args };
            state.late_scroll = Some(LateScrollFrame { frame: next_args.into(), started: base,
                opened_at: base, input_deadline: Some(base + Duration::from_secs(1)),
                deadline: base + Duration::from_secs(1),
                submitted_before: 0, input_attempted_at: None, input_dispatched: false,
                production_started: false, compositor_input_ready_at: None });
            state.input(InputEvent::Wheel(WheelEvent { phase: WheelPhase::kChanged,
                delta_units: ScrollGranularity::kScrollByPrecisePixel,
                position: Offset { x: 10.0, y: TOOLBAR_HEIGHT + 10.0 },
                delta: Offset { x: 0.0, y: 20.0 }, ..Default::default() })).unwrap();
            assert!(state.late_scroll.as_ref().unwrap().input_dispatched);
            assert!(!state.tabs.active.page.as_ref().unwrap().HasPendingWheelFrameInput());
            state.late_scroll.as_mut().unwrap().deadline = Instant::now() - Duration::from_millis(1);
            state.finish_late_scroll(true).unwrap();
            let page = state.tabs.active.page.as_mut().unwrap();
            {
                let owner = page.Document();
                let document = owner.GetDocument();
                assert_eq!(document.ScrollOffsetFor(document.FindNodeById(root).unwrap()).y, 40.0);
            }
            // An input reaching the actual admission point too late stays
            // queued; removing the host's second gate must not relax this one.
            let expired_base = Instant::now() - Duration::from_secs(1);
            let expired = BeginFrameArgs { source_id: 7, sequence_number: 3,
                frame_time: expired_base, interval: Duration::from_millis(16),
                deadline: expired_base + Duration::from_millis(16) };
            page.UpdateRendering(expired).unwrap();
            page.QueueFrameInput(InputEvent::Wheel(WheelEvent { phase: WheelPhase::kChanged,
                delta_units: ScrollGranularity::kScrollByPrecisePixel,
                position: Offset { x: 10.0, y: 10.0 },
                delta: Offset { x: 0.0, y: 20.0 }, ..Default::default() }), Instant::now(), 1).unwrap();
            assert!(!page.OnLateScrollInput(expired).unwrap());
            assert!(page.HasPendingWheelFrameInput());
        }).unwrap().join().unwrap();
    }

    #[test]
    fn native_momentum_continues_frame_aligned_scroll() {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                struct Source;
                impl BeginFrameSource for Source {
                    fn request_begin_frame(&self) {}
                }
                let output = Output {
                    mailbox: Arc::new(Mutex::new(None)),
                    notify: Arc::new(|_| {}),
                };
                let mut state = BrowserState::new(
                    Viewport {
                        width: 320,
                        height: 248,
                        scale: 1.0,
                    },
                    output,
                )
                .unwrap();
                let mut page =
                    create_page(320.0, 160.0, 1.0, true, state.pointer.clone(), false, 1, 0)
                        .unwrap();
                page.Open(
                    &data_url("<!doctype html><style>body{margin:0;height:4000px}</style>scroll"),
                    16384,
                    4096,
                )
                .unwrap();
                while page.IsLoading() {
                    page.RunTask().unwrap();
                }
                let source: Arc<dyn BeginFrameSource> = Arc::new(Source);
                page.SetBeginFrameSource(Some(source.clone()));
                state.tabs.active.page = Some(page);
                state.frame_source = Some(source);
                let mut native = crate::input::InputState::default();
                native.event(
                    &winit::event::WindowEvent::CursorMoved {
                        device_id: winit::event::DeviceId::dummy(),
                        position: winit::dpi::PhysicalPosition::new(10.0, TOOLBAR_HEIGHT + 10.0),
                    },
                    1.0,
                );
                let event =
                    |phase, momentum_phase, dy: f64| winit::event::WindowEvent::MouseWheel {
                        device_id: winit::event::DeviceId::dummy(),
                        delta: winit::event::MouseScrollDelta::PixelDelta(
                            winit::dpi::PhysicalPosition::new(0.0, -dy),
                        ),
                        phase: if phase == 8 || momentum_phase == 8 {
                            winit::event::TouchPhase::Ended
                        } else if phase == 1 || momentum_phase == 1 {
                            winit::event::TouchPhase::Started
                        } else {
                            winit::event::TouchPhase::Moved
                        },
                        native: Some(winit::event::MacOSMouseWheelMetadata {
                            timestamp_seconds: 1.0,
                            phase,
                            momentum_phase,
                        }),
                    };
                let send =
                    |state: &mut BrowserState, native: &mut crate::input::InputState, event| {
                        for command in native.event(&event, 1.0) {
                            state.command(command).unwrap();
                        }
                    };
                let offset = |state: &BrowserState| {
                    let page = state.tabs.active.page.as_ref().unwrap();
                    let root = page.CurrentFrame().unwrap().fragments.node_id;
                    let owner = page.Document();
                    let document = owner.GetDocument();
                    document
                        .ScrollOffsetFor(document.FindNodeById(root).unwrap())
                        .y
                };
                let frame = |state: &mut BrowserState, sequence_number| {
                    let now = Instant::now();
                    state
                        .begin_frame(BeginFrameArgs {
                            source_id: 41,
                            sequence_number,
                            frame_time: now,
                            deadline: now + Duration::from_millis(16),
                            interval: Duration::from_millis(16),
                        })
                        .unwrap();
                };
                send(&mut state, &mut native, event(1, 0, 20.0));
                frame(&mut state, 1);
                assert_eq!(offset(&state), 20.0);
                // Both the last finger update and the first momentum update belong
                // to the existing gesture. Neither may flush the frame queue.
                send(&mut state, &mut native, event(4, 0, 7.0));
                send(&mut state, &mut native, event(8, 0, 5.0));
                send(&mut state, &mut native, event(0, 1, 5.0));
                assert_eq!(
                    offset(&state),
                    20.0,
                    "momentum begin must not flush queued scroll outside BeginFrame"
                );
                assert!(state
                    .tabs
                    .active
                    .page
                    .as_ref()
                    .unwrap()
                    .HasPendingWheelFrameInput());
                assert!(state.scroll_active);
                frame(&mut state, 2);
                assert_eq!(offset(&state), 32.0);
                assert!(!state
                    .tabs
                    .active
                    .page
                    .as_ref()
                    .unwrap()
                    .HasPendingWheelFrameInput());
                send(&mut state, &mut native, event(0, 4, 6.0));
                assert_eq!(offset(&state), 32.0);
                frame(&mut state, 3);
                assert_eq!(offset(&state), 38.0);
                send(&mut state, &mut native, event(0, 8, 0.0));
                assert!(!state.scroll_active);
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn ready_wheel_enters_current_frame_without_crossing_gesture_boundary() {
        let (sender, receiver) = mpsc::channel();
        let now = Instant::now();
        let args = BeginFrameArgs {
            source_id: 1,
            sequence_number: 1,
            frame_time: now,
            deadline: now + Duration::from_millis(16),
            interval: Duration::from_millis(16),
        };
        let wheel = |phase| Command::QueuedInput {
            input: InputEvent::Wheel(WheelEvent {
                phase,
                delta: Offset { x: 0.0, y: 2.0 },
                ..Default::default()
            }),
            queued_at: now,
            samples: 1,
        };
        sender.send(wheel(WheelPhase::kChanged)).unwrap();
        sender
            .send(Command::BeginFrame(
                BeginFrameArgs {
                    sequence_number: 2,
                    ..args
                }
                .into(),
            ))
            .unwrap();
        sender.send(Command::BeginFrame(args.into())).unwrap();
        sender.send(wheel(WheelPhase::kChanged)).unwrap();
        sender.send(wheel(WheelPhase::kEnded)).unwrap();
        sender.send(wheel(WheelPhase::kChanged)).unwrap();
        let mut pending = VecDeque::new();
        let mut inputs = interaction::frame_aligned_input_queue::FrameAlignedInputQueue::default();
        let selected = prepare_begin_frame(args.into(), &receiver, &mut pending, |command| {
            let Command::QueuedInput {
                input,
                queued_at,
                samples,
            } = command
            else {
                panic!()
            };
            inputs.Push(input, queued_at, samples);
        });
        assert_eq!(selected.sequence_number, 2);
        let input = inputs.PopFront().unwrap();
        assert_eq!(input.samples, 2);
        assert!(matches!(input.input, InputEvent::Wheel(wheel) if wheel.delta.y == 4.0));
        assert!(inputs.IsEmpty());
        assert!(
            matches!(pending.pop_front().unwrap().input(), Some(InputEvent::Wheel(wheel))
            if wheel.phase == WheelPhase::kEnded)
        );
        assert!(
            matches!(receiver.try_recv().unwrap().input(), Some(InputEvent::Wheel(wheel))
            if wheel.phase == WheelPhase::kChanged)
        );
        // A continuous producer cannot hold the owner inside preparation.
        for _ in 0..65 {
            sender.send(wheel(WheelPhase::kChanged)).unwrap();
        }
        let mut count = 0;
        prepare_begin_frame(args.into(), &receiver, &mut pending, |_| count += 1);
        assert_eq!(count, 64);
        assert!(receiver.try_recv().is_ok());
    }
    #[test]
    fn begin_frame_presentation_waits_and_reuses_unchanged_frame() {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                struct Source(std::sync::atomic::AtomicUsize);
                impl BeginFrameSource for Source {
                    fn request_begin_frame(&self) {
                        self.0.fetch_add(1, Ordering::Relaxed);
                    }
                }
                let source = Arc::new(Source(std::sync::atomic::AtomicUsize::new(0)));
                let output = Output {
                    mailbox: Arc::new(Mutex::new(None)),
                    notify: Arc::new(|_| {}),
                };
                let mut state = BrowserState::new(
                    Viewport {
                        width: 320,
                        height: 240,
                        scale: 1.0,
                    },
                    output,
                )
                .unwrap();
                state.frame_source = Some(source.clone());
                state.toolbar.SetBeginFrameSource(Some(source.clone()));
                state.publish().unwrap();
                assert_eq!(state.presented_sequence, 0);
                assert!(source.0.load(Ordering::Relaxed) > 0);
                let frame_time = Instant::now();
                let args = BeginFrameArgs {
                    source_id: 1,
                    sequence_number: 1,
                    frame_time,
                    interval: Duration::from_millis(16),
                    deadline: frame_time + Duration::from_millis(16),
                };
                state.begin_frame(args).unwrap();
                assert_eq!(state.presented_sequence, 1);
                state.publish().unwrap();
                state
                    .begin_frame(BeginFrameArgs {
                        sequence_number: 2,
                        ..args
                    })
                    .unwrap();
                assert_eq!(
                    state.presented_sequence, 1,
                    "unchanged frame must not raster/compose again"
                );
                state.failed_frame = state.last_frame;
                state.last_frame = None;
                let requests = source.0.load(Ordering::Relaxed);
                state.publish().unwrap();
                assert_eq!(
                    source.0.load(Ordering::Relaxed),
                    requests,
                    "same failed artifact must not request another tick"
                );
                state.command(Command::Redraw).unwrap();
                state.publish().unwrap();
                assert!(
                    source.0.load(Ordering::Relaxed) > requests,
                    "explicit redraw can retry"
                );
                let (sender, receiver) = mpsc::channel();
                sender.send(Command::BeginFrame(args.into())).unwrap();
                let mut pending = VecDeque::new();
                assert!(
                    !native_command_waiting(&receiver, &mut pending, false),
                    "frames must not starve ordinary loading tasks"
                );
                sender.send(Command::FocusAddress).unwrap();
                assert!(
                    native_command_waiting(&receiver, &mut pending, false),
                    "native input still retains priority"
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn pending_input_frame_precedes_script_work_without_starving_animations() {
        let (sender, receiver) = mpsc::channel();
        let now = Instant::now();
        let args = BeginFrameArgs {
            source_id: 1,
            sequence_number: 1,
            frame_time: now,
            deadline: now + Duration::from_millis(16),
            interval: Duration::from_millis(16),
        };
        sender.send(Command::WakeLoading).unwrap();
        sender.send(Command::BeginFrame(args.into())).unwrap();
        let mut pending = VecDeque::new();
        assert!(!native_command_waiting(&receiver, &mut pending, false));
        assert!(native_command_waiting(&receiver, &mut pending, true));
        assert!(matches!(pending.pop_front(), Some(Command::WakeLoading)));
        assert!(matches!(pending.pop_front(), Some(Command::BeginFrame(_))));
        assert!(!native_command_waiting(&receiver, &mut pending, true));
    }

    fn host_task_pump_does_not_sleep_until_a_future_timer(state: &mut BrowserState) {
        let result = state.tabs.active.page.as_mut().unwrap().Evaluate(
            "var pumpLog=[]; var future=setTimeout(()=>pumpLog.push('future'),1000); setTimeout(()=>pumpLog.push('ready'),0);",
            "test:task-pump",
        ).unwrap();
        assert!(result.Succeeded(), "{:?}", result.exception);
        let started = Instant::now();
        for _ in 0..10 {
            state.tick().unwrap();
        }
        assert!(
            started.elapsed() < Duration::from_millis(500),
            "host pump waited for a future timer"
        );
        let result = state.tabs.active.page.as_mut().unwrap().Evaluate(
            "clearTimeout(future); if(pumpLog.join(',')!=='ready')throw Error('timer readiness');",
            "test:task-pump-check",
        ).unwrap();
        assert!(result.Succeeded(), "{:?}", result.exception);
    }

    #[test]
    fn background_page_feedback_cannot_change_the_visible_cursor() {
        use rechrom::page::Cursor;
        let cursors = Arc::new(Mutex::new(Vec::new()));
        let output = Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: {
                let cursors = cursors.clone();
                Arc::new(move |event| {
                    if let UserEvent::CursorChanged(cursor) = event {
                        cursors.lock().unwrap().push(cursor);
                    }
                })
            },
        };
        let pointer = Rc::new(HostFeedback {
            in_toolbar: Cell::new(false),
            active_tab: Cell::new(2),
            navigations: RefCell::new(VecDeque::new()),
            cursor: Cell::new(Cursor::kDefault),
            output,
        });
        let mut client = Client {
            pointer: pointer.clone(),
            toolbar: false,
            tab_id: 1,
            document_generation: 1,
            trace: false,
            started: Instant::now(),
        };
        client.DidChangeCursor(Cursor::kWait);
        assert!(cursors.lock().unwrap().is_empty());
        client.tab_id = 2;
        client.DidChangeCursor(Cursor::kText);
        assert_eq!(*cursors.lock().unwrap(), [Cursor::kText]);
        pointer.in_toolbar.set(true);
        client.DidChangeCursor(Cursor::kCrosshair);
        assert_eq!(*cursors.lock().unwrap(), [Cursor::kText]);
    }

    #[test]
    fn wheel_target_ignores_visible_overflow_and_non_scrollable_axis() {
        let mut root = FragmentNode {
            node_id: 1,
            size: Size {
                width: 300.0,
                height: 100.0,
            },
            content_size: Size {
                width: 300.0,
                height: 600.0,
            },
            ..Default::default()
        };
        let mut child = FragmentNode {
            node_id: 2,
            size: Size {
                width: 100.0,
                height: 40.0,
            },
            ..Default::default()
        };
        child.paint.has_source = true;
        child.paint.establishes_paint_state = true;
        child.paint.scroll_size = Size {
            width: 200.0,
            height: 200.0,
        };
        root.children.push(child);
        let target = scroll_candidate(
            &root,
            Offset { x: 20.0, y: 20.0 },
            Offset::default(),
            Offset { x: 0.0, y: 10.0 },
            100.0,
            true,
        )
        .unwrap();
        assert_eq!(target.0, 1, "visible overflow is not user-scrollable");
        use layoutng_assembly::internal::layout_input::Overflow;
        for overflow in [Overflow::kHidden, Overflow::kClip] {
            root.children[0].paint.SetOverflow(overflow);
            assert_eq!(
                scroll_candidate(
                    &root,
                    Offset { x: 20.0, y: 20.0 },
                    Offset::default(),
                    Offset { x: 0.0, y: 10.0 },
                    100.0,
                    true
                )
                .unwrap()
                .0,
                1
            );
        }
        root.children[0].paint.overflow_x = Overflow::kAuto;
        root.children[0].paint.overflow_y = Overflow::kHidden;
        root.children[0].paint.scroll_size.height = 40.0;
        assert_eq!(
            scroll_candidate(
                &root,
                Offset { x: 20.0, y: 20.0 },
                Offset::default(),
                Offset { x: 0.0, y: 10.0 },
                100.0,
                true
            )
            .unwrap()
            .0,
            1
        );
        assert_eq!(
            scroll_candidate(
                &root,
                Offset { x: 20.0, y: 20.0 },
                Offset::default(),
                Offset { x: 10.0, y: 0.0 },
                100.0,
                true
            )
            .unwrap()
            .0,
            2
        );
        root.children[0].paint.overflow_y = Overflow::kAuto;
        root.children[0].paint.scroll_size.height = 200.0;
        root.children[0].paint.scroll_offset.y = 160.0;
        assert_eq!(
            scroll_candidate(
                &root,
                Offset { x: 20.0, y: 20.0 },
                Offset::default(),
                Offset { x: 0.0, y: 10.0 },
                100.0,
                true
            )
            .unwrap()
            .0,
            1,
            "at bottom, scroll the ancestor"
        );
        assert_eq!(
            scroll_candidate(
                &root,
                Offset { x: 20.0, y: 20.0 },
                Offset::default(),
                Offset { x: 0.0, y: -10.0 },
                100.0,
                true
            )
            .unwrap()
            .0,
            2,
            "reverse direction returns to inner"
        );
        root.paint.establishes_paint_state = true;
        root.paint.scroll_offset.y = 100.0;
        root.children[0].offset.y = 120.0;
        assert_eq!(
            scroll_candidate(
                &root,
                Offset { x: 20.0, y: 30.0 },
                Offset::default(),
                Offset { x: 0.0, y: -10.0 },
                100.0,
                true
            )
            .unwrap()
            .0,
            2,
            "hit test uses scrolled coordinates"
        );
    }

    #[test]
    fn coalesced_scroll_keeps_remainder_on_the_ancestor_chain() {
        use layoutng_assembly::internal::layout_input::Overflow;
        let mut root = FragmentNode {
            node_id: 1,
            size: Size {
                width: 300.0,
                height: 100.0,
            },
            content_size: Size {
                width: 300.0,
                height: 600.0,
            },
            ..Default::default()
        };
        let mut child = FragmentNode {
            node_id: 2,
            size: Size {
                width: 100.0,
                height: 40.0,
            },
            ..Default::default()
        };
        child.paint.establishes_paint_state = true;
        child.paint.SetOverflow(Overflow::kAuto);
        child.paint.scroll_size = Size {
            width: 100.0,
            height: 200.0,
        };
        child.paint.scroll_offset.y = 150.0;
        root.children.push(child);
        let wheel = WheelEvent {
            position: Offset { x: 20.0, y: 20.0 },
            delta: Offset { x: 0.0, y: 50.0 },
            ..Default::default()
        };
        assert_eq!(
            scroll_updates(&root, &wheel, 100.0),
            [
                (2, Offset { x: 0.0, y: 160.0 }),
                (1, Offset { x: 0.0, y: 40.0 })
            ]
        );
        root.paint.establishes_paint_state = true;
        root.paint.scroll_offset.y = 50.0;
        root.children[0].offset.y = 50.0;
        root.children[0].paint.scroll_offset.y = 10.0;
        assert_eq!(
            scroll_updates(
                &root,
                &WheelEvent {
                    delta: Offset { x: 0.0, y: -30.0 },
                    ..wheel
                },
                100.0
            ),
            [(2, Offset::default()), (1, Offset { x: 0.0, y: 30.0 })]
        );
    }

    #[test]
    fn native_priority_looks_through_wakes_and_retains_order() {
        let (sender, receiver) = mpsc::channel();
        let mut pending = VecDeque::new();
        sender.send(Command::WakeLoading).unwrap();
        sender.send(Command::Reload).unwrap();
        assert!(native_command_waiting(&receiver, &mut pending, false));
        assert!(matches!(pending.pop_front(), Some(Command::WakeLoading)));
        assert!(matches!(pending.pop_front(), Some(Command::Reload)));
        sender.send(Command::WakeLoading).unwrap();
        assert!(!native_command_waiting(&receiver, &mut pending, false));
        assert!(matches!(pending.pop_front(), Some(Command::WakeLoading)));
    }

    #[test]
    fn coalesced_native_input_keeps_oldest_enqueue_time() {
        let (sender, receiver) = mpsc::channel();
        let first = Instant::now();
        for i in 0..3 {
            sender
                .send(Command::QueuedInput {
                    input: InputEvent::Wheel(WheelEvent {
                        delta: Offset { x: 0.0, y: 2.0 },
                        ..Default::default()
                    }),
                    queued_at: first + Duration::from_millis(i),
                    samples: 1,
                })
                .unwrap();
        }
        sender
            .send(Command::Input(InputEvent::Key(KeyEvent {
                key: "Enter".into(),
                ..Default::default()
            })))
            .unwrap();
        let mut pending = VecDeque::new();
        match receive_command(&receiver, &mut pending, Duration::ZERO).unwrap() {
            Command::QueuedInput {
                input: InputEvent::Wheel(wheel),
                queued_at,
                samples,
            } => {
                assert_eq!(wheel.delta.y, 6.0);
                assert_eq!(queued_at, first);
                assert_eq!(samples, 3);
            }
            _ => panic!("lost queued event timing"),
        }
        assert!(matches!(
            pending.pop_front(),
            Some(Command::Input(InputEvent::Key(_)))
        ));
    }

    fn foreground_frame_and_new_input_precede_background_task() {
        let mailbox: FrameMailbox = Arc::new(Mutex::new(None));
        let (sender, receiver) = mpsc::channel();
        let armed = Arc::new(AtomicBool::new(false));
        let output = Output {
            mailbox: mailbox.clone(),
            notify: {
                let armed = armed.clone();
                Arc::new(move |event| {
                    if matches!(event, UserEvent::FrameReady) {
                        mailbox.lock().unwrap().take();
                        if armed.swap(false, Ordering::AcqRel) {
                            sender.send(Command::WakeLoading).unwrap();
                            sender
                                .send(Command::Input(InputEvent::Mouse(MouseEvent::default())))
                                .unwrap();
                        }
                    }
                })
            },
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 320,
                height: 248,
                scale: 1.0,
            },
            output,
        )
        .unwrap();
        let background = data_url("<!doctype html><p>Background</p>");
        state.navigate(&background, true).unwrap();
        while state.tabs.active.page.as_ref().unwrap().IsLoading() {
            state.tick().unwrap();
        }
        state
            .new_tab(
                &data_url("<!doctype html><style>body{background:red}</style><p>Foreground</p>"),
                false,
            )
            .unwrap();
        while state.tabs.active.page.as_ref().unwrap().IsLoading() {
            state.tick().unwrap();
        }
        state.publish().unwrap();
        // No-op input must terminate before unrelated timer rendering.
        let queued_at = Instant::now();
        state.pending_native_input = Some(NativeInputTrace {
            queued_at,
            samples: 3,
            commands: 2,
        });
        state.publish().unwrap();
        assert!(state.pending_native_input.is_none());
        // A nested unchanged publish cannot end an in-progress input.
        state.pending_native_input = Some(NativeInputTrace {
            queued_at,
            samples: 1,
            commands: 1,
        });
        state.dispatching_native_input = true;
        state.publish().unwrap();
        assert!(state.pending_native_input.is_some());
        state.dispatching_native_input = false;
        state.publish().unwrap();
        assert!(state.pending_native_input.is_none());
        assert!(state.tabs.background[0]
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "window.backgroundRan=0;setTimeout(()=>backgroundRan++,0)",
                "test:background-ready"
            )
            .unwrap()
            .Succeeded());
        assert!(state
            .tabs
            .active
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "setTimeout(()=>document.body.style.background='blue',0)",
                "test:foreground-ready"
            )
            .unwrap()
            .Succeeded());
        state.pending_native_input = Some(NativeInputTrace {
            queued_at: Instant::now(),
            samples: 1,
            commands: 1,
        });
        state.last_frame = None;
        state.publish().unwrap();
        assert!(
            state.pending_native_input.is_none(),
            "headless raster must consume the trace without claiming a native presentation"
        );
        armed.store(true, Ordering::Release);
        let mut pending = VecDeque::new();
        finish_host_turn(&mut state, &receiver, &mut pending).unwrap();
        assert!(
            !armed.load(Ordering::Acquire),
            "foreground frame must be presented this turn"
        );
        assert!(state.tabs.background[0]
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "if(backgroundRan!==0)throw Error('background ran before pending input')",
                "test:background-yield"
            )
            .unwrap()
            .Succeeded());
        while let Some(command) = pending.pop_front() {
            state.command(command).unwrap();
        }
        finish_host_turn(&mut state, &receiver, &mut pending).unwrap();
        assert!(state.tabs.background[0]
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "if(backgroundRan!==1)throw Error('background timer was dropped')",
                "test:background-resume"
            )
            .unwrap()
            .Succeeded());
        // History replacement must preserve the current slot immediately,
        // without waiting for the next active-page task.
        let old_len = state.tabs.active.history.len();
        assert!(state
            .tabs
            .active
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "location.replace('about:blank')",
                "test:location-replace-history"
            )
            .unwrap()
            .Succeeded());
        state.drain_navigation_requests().unwrap();
        assert_eq!(state.tabs.active.history.len(), old_len);
        assert_eq!(
            state.tabs.active.history[state.tabs.active.history_index],
            "about:blank"
        );
    }

    #[test]
    fn loading_wakes_coalesce_without_delaying_input_or_losing_rearm() {
        let (sender, receiver) = mpsc::channel();
        let pending = Arc::new(AtomicBool::new(false));
        let wake = loading_wake_callback(sender.clone(), pending.clone());
        // A burst arriving while JS/render work occupies the owner must not
        // create thousands of empty host turns ahead of a later mouse event.
        let workers: Vec<_> = (0..4)
            .map(|_| {
                let wake = wake.clone();
                std::thread::spawn(move || {
                    for _ in 0..12500 {
                        wake();
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
        sender
            .send(Command::Input(InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kMove,
                position: Offset { x: 40.0, y: 120.0 },
                ..Default::default()
            })))
            .unwrap();
        assert!(matches!(receiver.try_recv(), Ok(Command::WakeLoading)));
        assert!(matches!(
            receiver.try_recv(),
            Ok(Command::Input(InputEvent::Mouse(_)))
        ));
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        assert!(pending.load(Ordering::Acquire));
        pending.store(false, Ordering::Release); // host accepts the wake
        wake();
        assert!(matches!(receiver.try_recv(), Ok(Command::WakeLoading)));
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        drop(receiver);
        pending.store(false, Ordering::Release);
        wake();
        assert!(
            !pending.load(Ordering::Acquire),
            "failed delivery must allow rearm"
        );
    }

    #[test]
    #[ignore = "manual complete scroll latency profile; run with --ignored --nocapture"]
    fn local_scroll_latency_profile() {
        std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(|| {
            let output = Output { mailbox: Arc::new(Mutex::new(None)), notify: Arc::new(|_| {}) };
            let viewport = Viewport { width: 2560, height: 1542, scale: 2.0 };
            let mut state = BrowserState::new(viewport, output.clone()).unwrap();
                let direct = std::env::var_os("BROWSER_APP_SCROLL_DIRECT").is_some();
            let format = if std::env::var("BROWSER_APP_SCROLL_FORMAT").as_deref() == Ok("bgra") {
                skia::PixelFormat::Bgra8888
            } else {
                skia::PixelFormat::Bgrx8888
            };
            state.navigate(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/interaction.html"), true).unwrap();
                while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) { state.tick().unwrap(); }
                state.publish().unwrap();
            let mut buffer = vec![0u32; viewport.width as usize * viewport.height as usize];
            let path = std::env::var("BROWSER_APP_SCROLL_CAPTURE").ok();
            println!("scroll-target direct={direct} capture_outside_timer=true");
                for iteration in 0..12 {
                output.mailbox.lock().unwrap().take();
                let sequence = state.tabs.active.page.as_ref().unwrap().CurrentFrame().unwrap().sequence;
                let start = Instant::now();
                state.input(InputEvent::Wheel(WheelEvent {
                    position: Offset { x: 800.0, y: TOOLBAR_HEIGHT + 200.0 },
                    delta: Offset { x: 0.0, y: if iteration < 6 { 32.0 } else { -32.0 } },
                    ..Default::default()
                })).unwrap();
                let input = start.elapsed();
                #[cfg(feature = "profile_paint")]
                if std::env::var("BROWSER_APP_SCROLL_PROFILE").ok().and_then(|v|v.parse::<usize>().ok())==Some(iteration) { profile_paint(&state); }
                assert!(state.tabs.active.page.as_ref().unwrap().CurrentFrame().unwrap().sequence > sequence);
                let start = Instant::now();
                let frame = if direct {
                    let (top,bottom)=buffer.split_at_mut(viewport.width as usize * viewport.toolbar_pixels() as usize);
                    raster::surface::RenderDisplayItemListIntoWindowBufferWithFormat(
                        &state.toolbar.CurrentFrame().unwrap().display_items,viewport.width,viewport.toolbar_pixels(),viewport.scale,top,format,
                    ).unwrap();
                    raster::surface::RenderDisplayItemListIntoWindowBufferWithFormat(
                        &state.tabs.active.page.as_ref().unwrap().CurrentFrame().unwrap().display_items,viewport.width,viewport.height-viewport.toolbar_pixels(),viewport.scale,bottom,format,
                    ).unwrap();
                    None
                } else {
                    state.publish().unwrap();
                    Some(output.mailbox.lock().unwrap().take().unwrap())
                };
                let render = start.elapsed();
                let start = Instant::now();
                if let Some(frame)=&frame {
                    crate::window_surface::compose_layers(&mut buffer, viewport.width, viewport.height, &[
                        crate::window_surface::SurfaceLayer { surface: &frame.toolbar, x: 0, y: 0 },
                        crate::window_surface::SurfaceLayer { surface: &frame.content, x: 0, y: viewport.toolbar_pixels() },
                    ]);
                }
                let copy = if direct { Duration::ZERO } else { start.elapsed() };
                println!("scroll-profile iteration={iteration} input_ms={:.3} render_ms={:.3} window_copy_ms={:.3} total_ms={:.3}", input.as_secs_f64()*1000.0, render.as_secs_f64()*1000.0, copy.as_secs_f64()*1000.0, (input+render+copy).as_secs_f64()*1000.0);
                if let Some(path) = &path {
                    let bytes: Vec<u8> = buffer.iter().flat_map(|v| v.to_le_bytes()).collect();
                    std::fs::write(format!("{path}-{iteration}.rgbx"), bytes).unwrap();
                }
            }
        }).unwrap().join().unwrap();
    }
    #[test]
    #[ignore = "manual button latency profile; run with --ignored --nocapture"]
    fn local_button_latency_profile() {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let output = Output {
                    mailbox: Arc::new(Mutex::new(None)),
                    notify: Arc::new(|_| {}),
                };
                let viewport = Viewport {
                    width: 2560,
                    height: 1542,
                    scale: 2.0,
                };
                let mut state = BrowserState::new(viewport, output.clone()).unwrap();
                let html = include_str!("../fixtures/interaction.html").replace(
                    "'Clicked — JavaScript event works'",
                    "'Clicked '+String(++window.clickCount)",
                );
                state.navigate(&data_url(&html), true).unwrap();
                while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) {
                    state.tick().unwrap();
                }
                state.publish().unwrap();
                state
                    .tabs
                    .active
                    .page
                    .as_mut()
                    .unwrap()
                    .Evaluate("window.clickCount=0", "test:setup")
                    .unwrap();
                let button = find_id(state.tabs.active.page.as_ref().unwrap(), "click").unwrap();
                let mut pixels = Vec::new();
                let toolbar_path = std::env::var("BROWSER_APP_PROFILE_TOOLBAR_PIXELS").ok();
                let mut toolbar = None;
                let repeats = std::env::var("BROWSER_APP_PROFILE_REPEATS")
                    .ok()
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(3);
                for iteration in 0..repeats {
                    output.mailbox.lock().unwrap().take();
                    let total = Instant::now();
                    for kind in [
                        MouseEventType::kMove,
                        MouseEventType::kDown,
                        MouseEventType::kUp,
                        MouseEventType::kClick,
                    ] {
                        let start = Instant::now();
                        state
                            .input(InputEvent::Mouse(MouseEvent {
                                r#type: kind,
                                target_node_id: Some(button),
                                position: Offset {
                                    x: 425.0,
                                    y: TOOLBAR_HEIGHT + 215.0,
                                },
                                button: if kind == MouseEventType::kMove {
                                    MouseButton::kNone
                                } else {
                                    MouseButton::kPrimary
                                },
                                ..Default::default()
                            }))
                            .unwrap();
                        println!(
                            "button-profile iteration={iteration} event={kind:?} dispatch_ms={:.3}",
                            start.elapsed().as_secs_f64() * 1000.0
                        );
                        let start = Instant::now();
                        state.publish().unwrap();
                        println!(
                            "button-profile iteration={iteration} event={kind:?} render_ms={:.3}",
                            start.elapsed().as_secs_f64() * 1000.0
                        );
                        if let Some(frame) = output.mailbox.lock().unwrap().take() {
                            pixels.clear();
                            pixels.extend_from_slice(frame.content.pixels());
                            if toolbar_path.is_some() {
                                toolbar = Some(frame.toolbar);
                            }
                        }
                    }
                    println!(
                        "button-profile iteration={iteration} total_ms={:.3}",
                        total.elapsed().as_secs_f64() * 1000.0
                    );
                }
                let path = std::env::var("BROWSER_APP_PROFILE_PIXELS")
                    .unwrap_or_else(|_| "/tmp/browser-app-button-profile.rgba".into());
                std::fs::write(path, pixels).unwrap();
                if let Some(path) = toolbar_path {
                    std::fs::write(path, toolbar.unwrap().pixels()).unwrap();
                }
                #[cfg(feature = "profile_paint")]
                profile_paint(&state);
                let result = state
                    .tabs
                    .active
                    .page
                    .as_mut()
                    .unwrap()
                    .Evaluate(
                        &format!("if(clickCount!=={repeats})throw Error('lost clicks')"),
                        "test:clicks",
                    )
                    .unwrap();
                assert!(result.Succeeded());
            })
            .unwrap()
            .join()
            .unwrap();
    }
    #[cfg(feature = "profile_paint")]
    fn profile_paint(state: &BrowserState) {
        use raster::pure_replay::{
            ProfileSourceDisplayItemListWithScale, RasterizeSourceDisplayItemListWithScale,
        };
        for (name, page, height) in [
            ("toolbar", &state.toolbar, state.viewport.toolbar_pixels()),
            (
                "content",
                state.tabs.active.page.as_ref().unwrap(),
                state.viewport.height - state.viewport.toolbar_pixels(),
            ),
        ] {
            let list = &page.CurrentFrame().unwrap().display_items;
            let (baseline, profile) = ProfileSourceDisplayItemListWithScale(
                list,
                state.viewport.width,
                height,
                state.viewport.scale,
            );
            #[cfg(target_os = "macos")]
            println!(
                "paint-glyph-cache surface={name} stats={:?}",
                skia::src::ports::SkScalerContext_mac_ct::glyph_cache_stats()
            );
            let mut groups = std::collections::BTreeMap::<String, (usize, f64)>::new();
            for entry in &profile.items {
                let item = &list.items[entry.index];
                let ms = entry.elapsed.as_secs_f64() * 1000.0;
                let group = groups.entry(format!("{:?}", item.r#type)).or_default();
                group.0 += 1;
                group.1 += ms;
                println!("paint-item surface={name} index={} type={:?} ms={ms:.3} aa_clip={} partial_clip_pixels={} rect={:?}", entry.index,item.r#type,entry.antialiased_clip,entry.partial_clip_pixels,item.rect);
                if format!("{:?}", item.r#type) == "kDrawGlyphRun" {
                    println!("paint-glyph-input surface={name} index={} count={} synthetic_bold={} synthetic_italic={} stroke={}", entry.index, item.glyphs.len(), item.synthetic_bold, item.synthetic_italic, item.stroke_glyphs);
                }
            }
            let replay: f64 = profile
                .items
                .iter()
                .map(|entry| entry.elapsed.as_secs_f64() * 1000.0)
                .sum();
            println!("paint-stages surface={name} setup_ms={:.3} replay_ms={replay:.3} readback_ms={:.3}",profile.setup.as_secs_f64()*1000.0,profile.readback.as_secs_f64()*1000.0);
            for (kind, (count, ms)) in groups {
                println!("paint-group surface={name} type={kind} count={count} ms={ms:.3}");
            }
            #[cfg(feature = "profile_native")]
            {
                // Warm up native font and CPU dispatch initialization before
                // three measured repetitions of this exact command list.
                for iteration in 0..4 {
                    let (pixels, stages) =
                        raster::source_replay::ProfileSourceDisplayItemListWithScale(
                            list,
                            state.viewport.width,
                            height,
                            state.viewport.scale,
                        );
                    let total: f64 = stages.iter().map(|t| t.as_secs_f64() * 1000.0).sum();
                    let different = pixels
                        .chunks_exact(4)
                        .zip(baseline.chunks_exact(4))
                        .filter(|(a, b)| a != b)
                        .count();
                    println!("paint-native surface={name} iteration={iteration} total_ms={total:.3} setup_ms={:.3} replay_ms={:.3} readback_ms={:.3} different_pixels={different}", stages[0].as_secs_f64()*1000.0, stages[1].as_secs_f64()*1000.0, stages[2].as_secs_f64()*1000.0);
                    if name == "content" && iteration == 3 {
                        if let Ok(path) = std::env::var("BROWSER_APP_PROFILE_NATIVE_PIXELS") {
                            std::fs::write(path, pixels).unwrap();
                        }
                    }
                }
            }
            // Counterfactuals only: retain the fixture's commands, changing
            // one suspected cost at a time. Pixel differences are reported.
            for experiment in ["binary_viewport_clip", "no_gradient", "no_rounded"] {
                let mut changed = list.clone();
                let mut count = 0;
                for item in
                    std::sync::Arc::make_mut(&mut std::sync::Arc::make_mut(&mut changed).items)
                        .iter_mut()
                {
                    let kind = format!("{:?}", item.r#type);
                    if experiment == "binary_viewport_clip"
                        && kind == "kClipRect"
                        && item.rect.x == 0.0
                        && item.rect.y == 0.0
                        && item.rect.width == state.viewport.logical_width()
                        && item.rect.height == f64::from(height) / state.viewport.scale
                    {
                        item.antialias = false;
                        count += 1;
                    } else if experiment == "no_gradient"
                        && (kind == "kDrawGradientRect" || kind == "kDrawTiledGradient")
                    {
                        // Keep the same clipped area, replace shading with a solid fill.
                        item.r#type = paint_type_rect();
                        item.color = item.paint_shader.as_ref().unwrap().stops[0].color;
                        item.paint_shader = None;
                        count += 1;
                    } else if experiment == "no_rounded"
                        && (kind == "kDrawRoundedRect" || kind == "kClipRoundedRect")
                    {
                        item.corner_radii = Default::default();
                        count += 1;
                    }
                }
                if count == 0 {
                    continue;
                }
                let start = Instant::now();
                let pixels = RasterizeSourceDisplayItemListWithScale(
                    &changed,
                    state.viewport.width,
                    height,
                    state.viewport.scale,
                );
                let ms = start.elapsed().as_secs_f64() * 1000.0;
                let differences = pixels
                    .chunks_exact(4)
                    .zip(baseline.chunks_exact(4))
                    .filter(|(a, b)| a != b)
                    .count();
                println!("paint-experiment surface={name} name={experiment} changed_items={count} render_ms={ms:.3} different_pixels={differences}");
            }
            if name == "content" {
                let body = &list.items[2];
                for (variant, duration, equal) in raster::pure_replay::ProfileConstantMaskBlit(
                    body.color,
                    state.viewport.width,
                    height - (body.rect.y * state.viewport.scale) as u32,
                ) {
                    println!(
                        "paint-solid-micro variant={variant} ms={:.3} pixel_equal={equal}",
                        duration.as_secs_f64() * 1000.0
                    );
                }
                let card = list
                    .items
                    .iter()
                    .find(|item| {
                        item.r#type == paint::paint_engine::DisplayItemType::kDrawRoundedRect
                    })
                    .unwrap();
                let (duration, rejected) = raster::pure_replay::ProfileRoundedRectAttempt(
                    card.rect,
                    card.corner_radii,
                    state.viewport.width,
                    height,
                    state.viewport.scale,
                );
                println!(
                    "paint-rounded-micro rejected={rejected} attempt_ms={:.3}",
                    duration.as_secs_f64() * 1000.0
                );
            }
        }
    }
    #[cfg(feature = "profile_paint")]
    fn paint_type_rect() -> paint::paint_engine::DisplayItemType {
        paint::paint_engine::DisplayItemType::kDrawRect
    }
    #[test]
    #[ignore = "manual queued window resize profile; no speed assertion"]
    fn queued_resize_latency_profile() {
        std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(|| {
            let output = Output { mailbox: Arc::new(Mutex::new(None)), notify: Arc::new(|_| {}) };
            let viewport = Viewport { width: 1280, height: 900, scale: 1.0 };
            let mut state = BrowserState::new(viewport, output).unwrap();
            let cards = (0..240).map(|i| format!("<article><h3>Article {i}</h3><p>Text.</p></article>")).collect::<String>();
            let html = format!("<!doctype html><style>body{{margin:0}}article{{display:block;width:30%;margin:1%;padding:8px;background:#eeeeee}}h3,p{{margin:0}}@media(max-width:1000px){{article{{width:45%}}}}</style>{cards}<script>window.resizeCount=0;window.addEventListener('resize',()=>resizeCount++);</script>");
            state.navigate(&data_url(&html), true).unwrap();
            while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) { state.tick().unwrap(); }
            state.publish().unwrap();
            let (tx, rx) = mpsc::channel();
            for i in 0..12 { tx.send(Command::Resize(Viewport { width: 1120 - i * 20, height: 800 + i * 4, scale: 1.0 })).unwrap(); tx.send(Command::Redraw).unwrap(); }
            tx.send(Command::Stop).unwrap();
            let mut pending = VecDeque::new();
            let sample_gate = std::env::var_os("BROWSER_APP_RESIZE_SAMPLE_GATE").is_some();
            if sample_gate {
                eprintln!("resize-profile ready-for-sample");
                std::thread::sleep(Duration::from_secs(1));
            }
            let total = Instant::now();
            let mut commands = 0;
            loop {
                let command = receive_command(&rx, &mut pending, Duration::ZERO).unwrap();
                if matches!(command, Command::Stop) { break; }
                let start = Instant::now();
                state.command(command).unwrap();
                let layout_ms = start.elapsed().as_secs_f64() * 1000.0;
                let start = Instant::now();
                let width = state.viewport.width;
                let height = state.viewport.height;
                let toolbar_pixels = state.viewport.toolbar_pixels();
                let mut buffer = vec![0u32; width as usize * height as usize];
                let (top, bottom) = buffer.split_at_mut(width as usize * toolbar_pixels as usize);
                raster::surface::RenderDisplayItemListIntoWindowBufferWithFormat(&state.toolbar.CurrentFrame().unwrap().display_items, width, toolbar_pixels, state.viewport.scale, top, skia::PixelFormat::Bgra8888).unwrap();
                raster::surface::RenderDisplayItemListIntoWindowBufferWithFormat(&state.tabs.active.page.as_ref().unwrap().CurrentFrame().unwrap().display_items, width, height-toolbar_pixels, state.viewport.scale, bottom, skia::PixelFormat::Bgra8888).unwrap();
                let raster_ms = start.elapsed().as_secs_f64() * 1000.0;
                commands += 1;
                println!("resize-profile command={commands} width={width} layout_js_ms={layout_ms:.3} allocate_full_raster_ms={raster_ms:.3}");
            }
            println!("resize-profile commands={commands} final_viewport={:?} total_ms={:.3}", state.viewport, total.elapsed().as_secs_f64()*1000.0);
            assert!(state.tabs.active.page.as_mut().unwrap().Evaluate(&format!("if(innerWidth!==900||innerHeight!=={}||devicePixelRatio!==1)throw Error('final viewport');", state.viewport.content_height()), "test:resize-profile").unwrap().Succeeded());
            if sample_gate { std::thread::sleep(Duration::from_secs(1)); }
        }).unwrap().join().unwrap();
    }

    #[test]
    fn input_history_and_resize_use_persistent_pages() {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let output = Output {
                    mailbox: Arc::new(Mutex::new(None)),
                    notify: Arc::new(|_| {}),
                };
                let viewport = Viewport {
                    width: 640,
                    height: 480,
                    scale: 1.0,
                };
                let mut state = BrowserState::new(viewport, output.clone()).unwrap();
                state.navigate("about:home", true).unwrap();
                while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) { state.tick().unwrap(); }
                host_task_pump_does_not_sleep_until_a_future_timer(&mut state);
                // A webpack/AMD-style dependency chain must reach DOM bindings
                // on the embedding's unoptimized translated VM stack.
                let result = state.tabs.active.page.as_mut().unwrap().Evaluate(
                    "function dependency(n){if(n) return dependency(n-1); return document.createElement('a').tagName;} if(dependency(52)!=='A') throw Error('dependency initialization');",
                    "test:module-native-stack",
                ).unwrap();
                assert!(result.Succeeded(), "{:?}", result.exception);
                let overflow = state.tabs.active.page.as_mut().unwrap().Evaluate(
                    "function unbounded(){return unbounded();} unbounded();",
                    "test:module-stack-protection",
                ).unwrap();
                assert!(overflow.exception.is_some_and(|error| error.message.contains("stack overflow")));

                state.publish().unwrap();
                assert_eq!(state.tabs.active.history, vec!["about:home"]);
                assert!(output.mailbox.lock().unwrap().take().is_some());
                state.publish().unwrap();
                assert!(
                    output.mailbox.lock().unwrap().is_none(),
                    "unchanged pages must not reraster"
                );
                state.command(Command::FocusAddress).unwrap();
                state
                    .input(InputEvent::TextInput(TextInputEvent {
                        text: "about:blank".into(),
                        ..Default::default()
                    }))
                    .unwrap();
                state
                    .input(InputEvent::Key(KeyEvent {
                        key: "Enter".into(),
                        ..Default::default()
                    }))
                    .unwrap();
                assert_eq!(state.tabs.active.location, "about:blank");
                state.history(-1).unwrap();
                assert_eq!(state.tabs.active.location, "about:home");
                state
                    .command(Command::Resize(Viewport {
                        width: 900,
                        height: 700,
                        scale: 2.0,
                    }))
                    .unwrap();
                let result = state
                    .tabs.active.page
                    .as_mut()
                    .unwrap()
                    .Evaluate(
                        "if(innerWidth!==450||devicePixelRatio!==2)throw Error('viewport');",
                        "test:viewport",
                    )
                    .unwrap();
                assert!(result.Succeeded(), "{:?}", result.exception);
                state.publish().unwrap();
                let frame = output.mailbox.lock().unwrap().take().unwrap();
                assert_eq!(frame.toolbar.size(), (900, state.viewport.toolbar_pixels()));
                assert_eq!(frame.content.size(), (900, state.viewport.height - state.viewport.toolbar_pixels()));
                state.navigate(&data_url(r#"<!doctype html><style>body{margin:0}input{width:200px;height:30px}#long{height:1800px}</style><input id="edit"><button id="button">Click</button><div id="long">Scrollable</div><script>document.getElementById('button').addEventListener('click',()=>document.getElementById('button').textContent='Clicked');</script>"#), true).unwrap();
                while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) { state.tick().unwrap(); }
                let edit = find_id(state.tabs.active.page.as_ref().unwrap(), "edit").unwrap();
                state.input(InputEvent::Mouse(MouseEvent {
                    r#type: MouseEventType::kDown,
                    target_node_id: Some(edit),
                    position: Offset { x: 10.0, y: TOOLBAR_HEIGHT + 10.0 },
                    button: MouseButton::kPrimary,
                    ..Default::default()
                })).unwrap();
                state.input(InputEvent::TextInput(TextInputEvent { text: "Rust input".into(), ..Default::default() })).unwrap();
                let button = find_id(state.tabs.active.page.as_ref().unwrap(), "button").unwrap();
                state.input(InputEvent::Mouse(MouseEvent {
                    r#type: MouseEventType::kClick,
                    target_node_id: Some(button),
                    position: Offset { x: 220.0, y: TOOLBAR_HEIGHT + 10.0 },
                    button: MouseButton::kPrimary,
                    ..Default::default()
                })).unwrap();
                let result = state.tabs.active.page.as_mut().unwrap().Evaluate("if(document.getElementById('edit').value!=='Rust input'||document.getElementById('button').textContent!=='Clicked')throw Error('input routing');", "test:input").unwrap();
                assert!(result.Succeeded(), "{:?}", result.exception);
                let before = state.tabs.active.page.as_ref().unwrap().CurrentFrame().unwrap().sequence;
                state.input(InputEvent::Wheel(WheelEvent {
                    position: Offset { x: 20.0, y: TOOLBAR_HEIGHT + 100.0 },
                    delta: Offset { x: 0.0, y: 200.0 },
                    ..Default::default()
                })).unwrap();
                assert!(state.tabs.active.page.as_ref().unwrap().CurrentFrame().unwrap().sequence > before, "wheel must update the frame");
                coordinate_input_and_cursor_use_resolved_styles();
                native_hover_repaints_without_reflow();
                article_cursor_follows_the_hit_text_and_link_style();
                repeated_cursor_moves_survive_self_posting_script_tasks();
                wheel_routes_axes_and_paints_document_scroll();
                ready_input_frame_is_presented_before_the_next_task();
                foreground_frame_and_new_input_precede_background_task();
                merged_resize_exposes_final_viewport_before_the_next_input();
                tabs_preserve_documents_and_route_link_navigation();
                host_form_requests_keep_post_bytes_source_tab_and_generation();
            })
            .unwrap()
            .join()
            .unwrap();
    }
    fn tabs_preserve_documents_and_route_link_navigation() {
        let output = Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: Arc::new(|_| {}),
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 640,
                height: 480,
                scale: 1.0,
            },
            output,
        )
        .unwrap();
        let primary = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/tabs-primary.html");
        state.navigate(primary, true).unwrap();
        fn loaded(state: &mut BrowserState) {
            for _ in 0..2000 {
                if !state.tabs.active.page.as_ref().unwrap().IsLoading() {
                    return;
                }
                state.tick().unwrap();
            }
            panic!("local tab fixture did not finish loading");
        }
        fn click(state: &mut BrowserState, id: &str) {
            let target = find_id(state.tabs.active.page.as_ref().unwrap(), id).unwrap();
            state
                .input(InputEvent::Mouse(MouseEvent {
                    r#type: MouseEventType::kClick,
                    button: MouseButton::kPrimary,
                    target_node_id: Some(target),
                    position: Offset {
                        x: 20.0,
                        y: TOOLBAR_HEIGHT + 20.0,
                    },
                    ..Default::default()
                }))
                .unwrap();
        }
        loaded(&mut state);
        let first_id = state.tabs.active.id;
        let first_document = {
            let owner = state.tabs.active.page.as_ref().unwrap().Document();
            owner.GetDocument() as *const dom::persistent_document::PersistentDocument
        };
        let edit = find_id(state.tabs.active.page.as_ref().unwrap(), "edit").unwrap();
        state
            .input(InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kDown,
                button: MouseButton::kPrimary,
                target_node_id: Some(edit),
                position: Offset {
                    x: 20.0,
                    y: TOOLBAR_HEIGHT + 20.0,
                },
                ..Default::default()
            }))
            .unwrap();
        state
            .input(InputEvent::TextInput(TextInputEvent {
                text: "Saved input".into(),
                ..Default::default()
            }))
            .unwrap();
        state
            .input(InputEvent::Key(KeyEvent {
                key: "ArrowLeft".into(),
                ..Default::default()
            }))
            .unwrap();
        let page = state.tabs.active.page.as_mut().unwrap();
        assert!(page.Evaluate("window.tabMarker=17;document.getElementById('cancel').addEventListener('click',e=>e.preventDefault());window.addEventListener('resize',()=>{if(document.activeElement.id!=='edit')throw Error('window event lost focus');document.getElementById('edit').focus()});", "test:tab-state").unwrap().Succeeded());
        state
            .input(InputEvent::Wheel(WheelEvent {
                position: Offset {
                    x: 20.0,
                    y: TOOLBAR_HEIGHT + 200.0,
                },
                delta: Offset { x: 0.0, y: 150.0 },
                ..Default::default()
            }))
            .unwrap();
        let root = state
            .tabs
            .active
            .page
            .as_ref()
            .unwrap()
            .CurrentFrame()
            .unwrap()
            .fragments
            .node_id;
        let scroll_before = {
            let owner = state.tabs.active.page.as_ref().unwrap().Document();
            owner
                .GetDocument()
                .ScrollOffsetFor(owner.GetDocument().FindNodeById(root).unwrap())
        };
        assert!(scroll_before.y > 0.0);
        assert!(state.tabs.active.page.as_mut().unwrap().Evaluate("if(document.activeElement.id!=='edit')throw Error('before new-tab focus '+document.activeElement.id);", "test:focus-before-tab").unwrap().Succeeded());
        state.command(Command::NewTab).unwrap();
        assert!(state.tabs.background[0]
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "if(document.activeElement.id!=='edit')throw Error('focus after create');",
                "test:focus-created"
            )
            .unwrap()
            .Succeeded());
        loaded(&mut state);
        assert!(state.tabs.background[0]
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "if(document.activeElement.id!=='edit')throw Error('focus after background pump');",
                "test:focus-background-pump"
            )
            .unwrap()
            .Succeeded());
        let temporary = state.tabs.active.id;
        assert!(state
            .tabs
            .active
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "if(typeof tabMarker!=='undefined')throw Error('new tab shares realm');",
                "test:new-tab"
            )
            .unwrap()
            .Succeeded());
        state
            .command(Command::Resize(Viewport {
                width: 1000,
                height: 800,
                scale: 2.0,
            }))
            .unwrap();
        state.activate_tab(first_id).unwrap();
        assert!(state.tabs.active.page.as_mut().unwrap().Evaluate("if(tabMarker!==17||document.getElementById('edit').value!=='Saved input'||document.activeElement.id!=='edit'||innerWidth!==500||devicePixelRatio!==2)throw Error('focused tab restoration');", "test:focus-restored").unwrap().Succeeded());
        state
            .input(InputEvent::TextInput(TextInputEvent {
                text: "!".into(),
                ..Default::default()
            }))
            .unwrap();
        assert!(state.tabs.active.page.as_mut().unwrap().Evaluate("if(document.getElementById('edit').value!=='Saved inpu!t')throw Error('selection did not survive');", "test:selection-restored").unwrap().Succeeded());
        state.close_tab(temporary).unwrap();
        click(&mut state, "cancel");
        assert_eq!(
            state.tabs.background.len(),
            0,
            "preventDefault cancels _blank as well"
        );
        click(&mut state, "nested");
        loaded(&mut state);
        let second_id = state.tabs.active.id;
        assert_ne!(first_id, second_id, "_blank creates another tab");
        assert!(state.tabs.active.location.ends_with("tabs-secondary.html"));
        assert_eq!(state.tabs.background.len(), 1);
        assert!(state.tabs.active.page.as_mut().unwrap().Evaluate("if(typeof tabMarker!=='undefined')throw Error('shared realm');window.tabMarker=29;", "test:second-tab").unwrap().Succeeded());
        state.activate_tab(first_id).unwrap();
        {
            let owner = state.tabs.active.page.as_ref().unwrap().Document();
            assert_eq!(
                owner.GetDocument() as *const dom::persistent_document::PersistentDocument,
                first_document,
                "switching returns the same document allocation"
            );
            assert_eq!(
                owner
                    .GetDocument()
                    .ScrollOffsetFor(owner.GetDocument().FindNodeById(root).unwrap()),
                scroll_before
            );
        }
        assert!(state.tabs.active.page.as_mut().unwrap().Evaluate("if(tabMarker!==17||document.getElementById('edit').value!=='Saved inpu!t'||innerWidth!==500||devicePixelRatio!==2)throw Error('tab restoration');", "test:first-restored").unwrap().Succeeded());
        assert_eq!(state.tabs.active.history.len(), 1);
        click(&mut state, "same");
        loaded(&mut state);
        assert_eq!(
            state.tabs.active.id, first_id,
            "ordinary links use the current tab"
        );
        assert_eq!(state.tabs.active.history.len(), 2);
        assert_eq!(state.tabs.background.len(), 1);
        state.history(-1).unwrap();
        loaded(&mut state);
        assert!(state.tabs.active.location.ends_with("tabs-primary.html"));
        state.activate_tab(second_id).unwrap();
        assert_eq!(
            state.tabs.active.history.len(),
            1,
            "history belongs to each tab"
        );
        assert!(state
            .tabs
            .active
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "if(tabMarker!==29)throw Error('second realm discarded');",
                "test:second-restored"
            )
            .unwrap()
            .Succeeded());
        state.close_tab(second_id).unwrap();
        assert_eq!(state.tabs.active.id, first_id);
        state.close_tab(first_id).unwrap();
        loaded(&mut state);
        assert_eq!(state.tabs.active.location, "about:home");
        assert!(state.tabs.background.is_empty());
        assert!(find_id(&state.toolbar, "newtab").is_some());
        fn chrome_click(state: &mut BrowserState, name: &str) {
            let id = find_id(&state.toolbar, name).unwrap();
            fn center(fragment: &FragmentNode, id: u64, parent: Offset) -> Option<Offset> {
                let origin = Offset {
                    x: parent.x + fragment.offset.x,
                    y: parent.y + fragment.offset.y,
                };
                if fragment.node_id == id && fragment.size.width > 0.0 && fragment.size.height > 0.0
                {
                    return Some(Offset {
                        x: origin.x + fragment.size.width * 0.5,
                        y: origin.y + fragment.size.height * 0.5,
                    });
                }
                fragment
                    .children
                    .iter()
                    .find_map(|child| center(child, id, origin))
            }
            let position = center(
                &state.toolbar.CurrentFrame().unwrap().fragments,
                id,
                Offset::default(),
            )
            .unwrap_or_else(|| panic!("no chrome fragment for {name}, node {id}"));
            for r#type in [
                MouseEventType::kDown,
                MouseEventType::kUp,
                MouseEventType::kClick,
            ] {
                state
                    .input(InputEvent::Mouse(MouseEvent {
                        r#type,
                        button: MouseButton::kPrimary,
                        position,
                        ..Default::default()
                    }))
                    .unwrap();
            }
        }
        let home_id = state.tabs.active.id;
        chrome_click(&mut state, "newtab");
        loaded(&mut state);
        let created = state.tabs.active.id;
        assert_ne!(
            created, home_id,
            "actual chrome hit testing reaches the new-tab button"
        );
        if let Ok(path) = std::env::var("BROWSER_APP_TABS_CAPTURE") {
            let surface = raster::surface::RenderDisplayItemListToSurface(
                &state.toolbar.CurrentFrame().unwrap().display_items,
                state.viewport.width,
                state.viewport.toolbar_pixels(),
                state.viewport.scale,
            )
            .unwrap();
            std::fs::write(path, surface.pixels()).unwrap();
        }
        chrome_click(&mut state, &format!("title-{home_id}"));
        assert_eq!(
            state.tabs.active.id, home_id,
            "title click activates its tab"
        );
        chrome_click(&mut state, &format!("title-{home_id}"));
        assert!(
            !state.toolbar_focused,
            "clicking the current tab restores content input routing"
        );
        chrome_click(&mut state, &format!("close-{created}"));
        assert!(
            state.tabs.background.is_empty(),
            "close button routes to its own tab"
        );
        assert!(
            !state.toolbar_focused,
            "closing a background tab preserves the content input route"
        );
        state.publish().unwrap();
        let base_items = state
            .tabs
            .active
            .page
            .as_ref()
            .unwrap()
            .CurrentFrame()
            .unwrap()
            .display_items
            .clone();
        let viewport = state.viewport;
        let base = raster::surface::RenderDisplayItemListToSurface(
            &base_items,
            viewport.width,
            viewport.height - viewport.toolbar_pixels(),
            viewport.scale,
        )
        .unwrap();
        chrome_click(&mut state, "menu");
        assert!(state.popup.is_some(), "menu icon opens a host popup Page");
        let popup = state.popup.as_ref().unwrap();
        let combined = crate::chrome::content_with_popup(
            Some(&base_items),
            &popup.page.CurrentFrame().unwrap().display_items,
        )
        .unwrap();
        let over = raster::surface::RenderDisplayItemListToSurface(
            &combined,
            viewport.width,
            viewport.height - viewport.toolbar_pixels(),
            viewport.scale,
        )
        .unwrap();
        assert_eq!(
            &base.pixels()[..4],
            &over.pixels()[..4],
            "popup replay preserves uncovered site pixels"
        );
        assert_ne!(
            base.pixels(),
            over.pixels(),
            "popup paints over the content without clearing it"
        );
        if let Ok(path) = std::env::var("BROWSER_APP_TABS_CAPTURE") {
            let top = raster::surface::RenderDisplayItemListToSurface(
                &state.toolbar.CurrentFrame().unwrap().display_items,
                viewport.width,
                viewport.toolbar_pixels(),
                viewport.scale,
            )
            .unwrap();
            let mut rgba = top.pixels().to_vec();
            rgba.extend_from_slice(over.pixels());
            std::fs::write(format!("{path}.menu"), rgba).unwrap();
        }
        let original = state.tabs.active.id;
        let position = Offset {
            x: popup.x + 60.0,
            y: TOOLBAR_HEIGHT + 4.0 + 8.0 + 16.0,
        };
        for r#type in [
            MouseEventType::kDown,
            MouseEventType::kUp,
            MouseEventType::kClick,
        ] {
            state
                .input(InputEvent::Mouse(MouseEvent {
                    r#type,
                    position,
                    button: MouseButton::kPrimary,
                    ..Default::default()
                }))
                .unwrap();
        }
        assert_ne!(
            state.tabs.active.id, original,
            "actual popup hit creates a new independent tab"
        );
        chrome_click(&mut state, "tabsearch");
        assert!(state.popup.is_some());
        state
            .input(InputEvent::Key(KeyEvent {
                r#type: KeyEventType::kDown,
                key: "Escape".into(),
                ..Default::default()
            }))
            .unwrap();
        assert!(state.popup.is_none(), "Escape dismisses the host popup");
        state.command(Command::FocusAddress).unwrap();
        state.publish().unwrap();
        assert_eq!(state.toolbar.FocusedNodeId(), Some(state.address_id));
        chrome_click(&mut state, "menu");
        let before_menu_key = state.tabs.active.id;
        state
            .input(InputEvent::Key(KeyEvent {
                r#type: KeyEventType::kDown,
                key: "ArrowDown".into(),
                ..Default::default()
            }))
            .unwrap();
        state
            .input(InputEvent::Key(KeyEvent {
                r#type: KeyEventType::kDown,
                key: "Enter".into(),
                ..Default::default()
            }))
            .unwrap();
        assert_ne!(
            state.tabs.active.id, before_menu_key,
            "menu arrows/Enter activate the same new-tab action"
        );
        state
            .input(InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kMove,
                position: Offset {
                    x: 12.0,
                    y: TOOLBAR_HEIGHT + 12.0,
                },
                ..Default::default()
            }))
            .unwrap();
        state
            .input(InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kDown,
                button: MouseButton::kPrimary,
                position: Offset {
                    x: 12.0,
                    y: TOOLBAR_HEIGHT + 12.0,
                },
                ..Default::default()
            }))
            .unwrap();
        loaded(&mut state);
        let fresh_location = state.tabs.active.location.clone();
        let fresh_focus = state.tabs.active.page.as_ref().unwrap().FocusedNodeId();
        for r#type in [MouseEventType::kUp, MouseEventType::kClick] {
            state
                .input(InputEvent::Mouse(MouseEvent {
                    r#type,
                    button: MouseButton::kPrimary,
                    position: Offset {
                        x: 12.0,
                        y: TOOLBAR_HEIGHT + 12.0,
                    },
                    ..Default::default()
                }))
                .unwrap();
        }
        assert_eq!(
            state.tabs.active.location, fresh_location,
            "pre-paint press cannot activate a link appearing before release"
        );
        assert_eq!(
            state.tabs.active.page.as_ref().unwrap().FocusedNodeId(),
            fresh_focus
        );
        // A genuine move updates chrome appearance without changing the site.
        state.publish().unwrap();
        let before_hover = state.toolbar.CurrentFrame().unwrap().display_items.clone();
        let location = state.tabs.active.location.clone();
        state
            .input(InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kMove,
                position: Offset {
                    x: state.viewport.logical_width() - 23.0,
                    y: TOOLBAR_HEIGHT - 23.0,
                },
                ..Default::default()
            }))
            .unwrap();
        assert!(
            state.toolbar.CurrentFrame().unwrap().display_items.items != before_hover.items,
            "hover paints a visible background"
        );
        assert_eq!(state.tabs.active.location, location);
        state
            .input(InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kMove,
                position: Offset {
                    x: 12.0,
                    y: TOOLBAR_HEIGHT + 12.0,
                },
                ..Default::default()
            }))
            .unwrap();
        assert!(
            state.chrome_hovered.is_empty(),
            "leaving chrome clears its hover ink"
        );
        let regions = Arc::new(Mutex::new(Vec::<crate::chrome::DragRegion>::new()));
        let observed = regions.clone();
        let saved = state.output.notify.clone();
        state.output.notify = Arc::new(move |event| {
            if let UserEvent::ChromeDragRegions { regions, .. } = event {
                *observed.lock().unwrap() = regions;
            }
        });
        state.last_frame = None;
        state.publish().unwrap();
        assert!(
            regions
                .lock()
                .unwrap()
                .iter()
                .any(|region| region.contains(state.viewport.logical_width() - 24.0, 20.0)),
            "the actual tabstrip drag-space layout reaches the native host"
        );
        assert!(
            !regions
                .lock()
                .unwrap()
                .iter()
                .any(|region| region.contains(110.0, 23.0)),
            "the tab search button is excluded from native dragging"
        );
        state.output.notify = saved;
    }
    fn host_form_requests_keep_post_bytes_source_tab_and_generation() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let server = std::thread::spawn(move || {
            let mut requests = Vec::new();
            let deadline = Instant::now() + Duration::from_secs(5);
            while requests.len() < 3 && Instant::now() < deadline {
                let (mut stream, _) = match listener.accept() {
                    Ok(value) => value,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(error) => panic!("local HTTP accept: {error}"),
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut bytes = Vec::new();
                let (header_end, length) = loop {
                    let mut buffer = [0; 4096];
                    let count = stream.read(&mut buffer).unwrap();
                    assert!(count > 0, "truncated HTTP request");
                    bytes.extend_from_slice(&buffer[..count]);
                    if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                        let header = String::from_utf8_lossy(&bytes[..index]);
                        let length = header
                            .lines()
                            .find_map(|line| {
                                line.split_once(':')
                                    .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                                    .and_then(|(_, value)| value.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        break (index + 4, length);
                    }
                };
                while bytes.len() < header_end + length {
                    let mut buffer = [0; 4096];
                    let count = stream.read(&mut buffer).unwrap();
                    assert!(count > 0, "truncated HTTP body");
                    bytes.extend_from_slice(&buffer[..count]);
                }
                let head = String::from_utf8_lossy(&bytes[..header_end]);
                let request = (
                    head.lines().next().unwrap().to_owned(),
                    bytes[header_end..header_end + length].to_vec(),
                    head.to_lowercase(),
                );
                let html = if requests.is_empty() {
                    "<!doctype html><title>Form source</title><form id='form' method='post' action='/submit' target='_blank'><input name='q' value='space name'><button id='send' name='submitter' value='Send'>Send</button></form>"
                } else {
                    "<!doctype html><title>Submitted</title><h1>Submitted document</h1>"
                };
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", html.len(), html).unwrap();
                requests.push(request);
            }
            requests
        });
        let output = Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: Arc::new(|_| {}),
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 640,
                height: 480,
                scale: 1.0,
            },
            output,
        )
        .unwrap();
        fn loaded(state: &mut BrowserState) {
            let deadline = Instant::now() + Duration::from_secs(4);
            while state.tabs.active.page.as_ref().unwrap().IsLoading() && Instant::now() < deadline
            {
                state.tick().unwrap();
                std::thread::yield_now();
            }
            assert!(
                !state.tabs.active.page.as_ref().unwrap().IsLoading(),
                "HTTP fixture did not load"
            );
        }
        state
            .navigate(&format!("http://{address}/form"), true)
            .unwrap();
        loaded(&mut state);
        let source_id = state.tabs.active.id;
        let source_generation = state.tabs.active.document_generation;
        let send = find_id(state.tabs.active.page.as_ref().unwrap(), "send").unwrap();
        state
            .input(InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kClick,
                target_node_id: Some(send),
                button: MouseButton::kPrimary,
                position: Offset {
                    x: 20.0,
                    y: TOOLBAR_HEIGHT + 20.0,
                },
                ..Default::default()
            }))
            .unwrap();
        loaded(&mut state);
        let foreground = state.tabs.active.id;
        assert_ne!(
            foreground, source_id,
            "form _blank reaches the host's new-tab path"
        );
        assert!(state.tabs.active.location.ends_with("/submit"));
        let background = state
            .tabs
            .background
            .iter_mut()
            .find(|tab| tab.id == source_id)
            .unwrap();
        assert!(background.page.as_mut().unwrap().Evaluate("var form=document.getElementById('form');form.target='_self';form.action='/background';form.submit();", "test:background-submit").unwrap().Succeeded());
        state.drain_navigation_requests().unwrap();
        assert_eq!(
            state.tabs.active.id, foreground,
            "background navigation cannot activate its source tab"
        );
        assert!(state
            .tabs
            .background
            .iter()
            .find(|tab| tab.id == source_id)
            .unwrap()
            .location
            .ends_with("/background"));
        let stale = rechrom::page::NavigationRequest {
            request: url_loader::URLRequest {
                url: format!("http://{address}/stale"),
                ..Default::default()
            },
            target: "_blank".into(),
            replace_history: false,
        };
        state.pointer.navigations.borrow_mut().push_back((
            source_id,
            source_generation,
            stale.clone(),
        ));
        state
            .pointer
            .navigations
            .borrow_mut()
            .push_back((999, 1, stale));
        state.drain_navigation_requests().unwrap();
        assert_eq!(state.tabs.active.id, foreground);
        assert_eq!(
            state.tabs.background.len(),
            1,
            "old/closed documents cannot open another tab"
        );
        let requests = server.join().unwrap();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].0, "GET /form HTTP/1.1");
        assert_eq!(requests[1].0, "POST /submit HTTP/1.1");
        assert_eq!(requests[1].1, b"q=space+name&submitter=Send");
        assert!(requests[1]
            .2
            .contains("content-type: application/x-www-form-urlencoded"));
        assert_eq!(requests[2].0, "POST /background HTTP/1.1");
        assert_eq!(requests[2].1, b"q=space+name");
    }
    fn ready_input_frame_is_presented_before_the_next_task() {
        let mailbox: FrameMailbox = Arc::new(Mutex::new(None));
        let colors = Arc::new(Mutex::new(Vec::new()));
        let output = Output {
            mailbox: mailbox.clone(),
            notify: {
                let colors = colors.clone();
                Arc::new(move |event| {
                    if matches!(event, UserEvent::FrameReady) {
                        let frame = mailbox.lock().unwrap().take().unwrap();
                        colors
                            .lock()
                            .unwrap()
                            .push(frame.content.pixels()[..4].to_vec());
                    }
                })
            },
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 320,
                height: 248,
                scale: 1.0,
            },
            output,
        )
        .unwrap();
        state.navigate(&data_url("<!doctype html><style>html,body{margin:0}#tail{height:900px;background:red}</style><div id=tail></div>"), true).unwrap();
        while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) {
            state.tick().unwrap();
        }
        state.publish().unwrap();
        colors.lock().unwrap().clear();
        assert!(state
            .tabs
            .active
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "setTimeout(()=>document.getElementById('tail').style.background='blue',0);",
                "test:ready-input-frame"
            )
            .unwrap()
            .Succeeded());
        state
            .input(InputEvent::Wheel(WheelEvent {
                position: Offset {
                    x: 20.0,
                    y: TOOLBAR_HEIGHT + 20.0,
                },
                delta: Offset { x: 0.0, y: 12.0 },
                ..Default::default()
            }))
            .unwrap();
        let (sender, receiver) = mpsc::channel();
        sender
            .send(Command::Input(InputEvent::Wheel(WheelEvent {
                position: Offset {
                    x: 20.0,
                    y: TOOLBAR_HEIGHT + 20.0,
                },
                delta: Offset { x: 0.0, y: 12.0 },
                ..Default::default()
            })))
            .unwrap();
        let mut pending = VecDeque::new();
        finish_host_turn(&mut state, &receiver, &mut pending).unwrap();
        assert_eq!(
            *colors.lock().unwrap(),
            vec![vec![255, 0, 0, 255]],
            "queued input has priority over the unrelated timer"
        );
        state.command(pending.pop_front().unwrap()).unwrap();
        finish_host_turn(&mut state, &receiver, &mut pending).unwrap();
        assert_eq!(
            *colors.lock().unwrap(),
            vec![
                vec![255, 0, 0, 255],
                vec![255, 0, 0, 255],
                vec![0, 0, 255, 255]
            ],
            "present the completed red scroll frame before the timer paints blue"
        );
    }
    fn wheel_routes_axes_and_paints_document_scroll() {
        let output = Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: Arc::new(|_| {}),
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 320,
                height: 248,
                scale: 1.0,
            },
            output,
        )
        .unwrap();
        state.navigate(&data_url("<!doctype html><style>html,body{margin:0}#strip{width:100px;height:40px;overflow-x:auto;overflow-y:hidden}#wide{width:300px;height:40px;background:red}#tail{height:900px;background:blue}</style><div id=strip><div id=wide></div></div><div id=tail></div>"), true).unwrap();
        while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) {
            state.tick().unwrap();
        }
        let strip = find_id(state.tabs.active.page.as_ref().unwrap(), "strip").unwrap();
        let tail = find_id(state.tabs.active.page.as_ref().unwrap(), "tail").unwrap();
        let frame = state
            .tabs
            .active
            .page
            .as_ref()
            .unwrap()
            .CurrentFrame()
            .unwrap();
        let root = frame.fragments.node_id;
        let before_y = frame
            .display_items
            .items
            .iter()
            .find(|i| i.node_id == tail)
            .unwrap()
            .rect
            .y;
        let sequence = frame.sequence;
        state
            .input(InputEvent::Wheel(WheelEvent {
                position: Offset {
                    x: 20.0,
                    y: TOOLBAR_HEIGHT + 20.0,
                },
                delta: Offset { x: 1.5, y: 12.25 },
                ..Default::default()
            }))
            .unwrap();
        let page = state.tabs.active.page.as_ref().unwrap();
        let owner = page.Document();
        let document = owner.GetDocument();
        let root_offset = document.ScrollOffsetFor(document.FindNodeById(root).unwrap());
        let strip_offset = document.ScrollOffsetFor(document.FindNodeById(strip).unwrap());
        assert_eq!(
            root_offset.y, 12.25,
            "vertical wheel must reach the document"
        );
        assert_eq!(
            strip_offset.x, 1.5,
            "horizontal component scrolls the strip"
        );
        assert_eq!(strip_offset.y, 0.0);
        let frame = page.CurrentFrame().unwrap();
        assert!(frame.sequence > sequence);
        let after_y = frame
            .display_items
            .items
            .iter()
            .find(|i| i.node_id == tail)
            .unwrap()
            .rect
            .y;
        assert!(
            after_y < before_y && (after_y - (before_y - 12.25)).abs() <= 0.5,
            "paint must translate the visible content within raster pixel snapping"
        );
        drop(owner);
        let result = state
            .tabs
            .active
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "document.addEventListener('wheel',e=>e.preventDefault());",
                "test:cancel-wheel",
            )
            .unwrap();
        assert!(result.Succeeded());
        state
            .input(InputEvent::Wheel(WheelEvent {
                position: Offset {
                    x: 20.0,
                    y: TOOLBAR_HEIGHT + 80.0,
                },
                delta: Offset { x: 0.0, y: 20.0 },
                ..Default::default()
            }))
            .unwrap();
        let owner = state.tabs.active.page.as_ref().unwrap().Document();
        let document = owner.GetDocument();
        assert_eq!(
            document
                .ScrollOffsetFor(document.FindNodeById(root).unwrap())
                .y,
            12.25,
            "preventDefault must keep the scroll position"
        );
    }

    // Run on the existing browser test's GC owner thread. Separate browser
    // threads in one process cannot share the assembly's Persistent objects.
    fn layout_result_for(
        tree: &layoutng_assembly::internal::layout_object_builder::LayoutObjectTree,
        id: u64,
    ) -> *const layoutng_assembly::layout_result::LayoutResult {
        use layoutng_assembly::internal::{layout_box::LayoutBox, layout_object::LayoutObject};
        let root = tree.Root() as *const LayoutObject;
        let mut object = root as *mut LayoutObject;
        while !object.is_null() {
            let node = unsafe { &*object }.GetNode();
            if !node.is_null() && unsafe { &*node }.InputId() == id {
                let box_ptr = foundation::DynamicTo::<LayoutBox>(object);
                assert!(!box_ptr.is_null());
                return unsafe { &*box_ptr }.GetLayoutResult(0);
            }
            object = unsafe { &*object }.NextInPreOrder(root);
        }
        panic!("no layout object for {id}");
    }
    #[test]
    fn resize_queue_keeps_latest_visual_properties_and_all_task_boundaries() {
        let (tx, rx) = mpsc::channel();
        let viewport = |width, scale| Viewport {
            width,
            height: 800,
            scale,
        };
        let (a, b, c, d) = (
            viewport(1000, 1.0),
            viewport(1800, 2.0),
            viewport(1300, 1.0),
            viewport(2200, 2.0),
        );
        tx.send(Command::Redraw).unwrap();
        for _ in 0..600 {
            tx.send(Command::Resize(a)).unwrap();
            tx.send(Command::Redraw).unwrap();
        }
        tx.send(Command::Resize(b)).unwrap();
        tx.send(Command::Input(InputEvent::Key(KeyEvent {
            key: "Enter".into(),
            ..Default::default()
        })))
        .unwrap();
        tx.send(Command::Resize(c)).unwrap();
        tx.send(Command::WakeLoading).unwrap();
        tx.send(Command::Redraw).unwrap();
        tx.send(Command::Redraw).unwrap();
        tx.send(Command::Reload).unwrap();
        tx.send(Command::Resize(d)).unwrap();
        tx.send(Command::SetActive(false)).unwrap();
        tx.send(Command::Stop).unwrap();
        let mut pending = VecDeque::new();
        assert!(
            matches!(receive_command(&rx, &mut pending, Duration::ZERO).unwrap(), Command::Resize(v) if v==b)
        );
        assert!(
            matches!(receive_command(&rx, &mut pending, Duration::ZERO).unwrap(), Command::Input(InputEvent::Key(k)) if k.key=="Enter")
        );
        assert!(
            matches!(receive_command(&rx, &mut pending, Duration::ZERO).unwrap(), Command::Resize(v) if v==c)
        );
        assert!(matches!(
            receive_command(&rx, &mut pending, Duration::ZERO).unwrap(),
            Command::WakeLoading
        ));
        assert!(matches!(
            receive_command(&rx, &mut pending, Duration::ZERO).unwrap(),
            Command::Redraw
        ));
        assert!(matches!(
            receive_command(&rx, &mut pending, Duration::ZERO).unwrap(),
            Command::Reload
        ));
        assert!(
            matches!(receive_command(&rx, &mut pending, Duration::ZERO).unwrap(), Command::Resize(v) if v==d)
        );
        assert!(matches!(
            receive_command(&rx, &mut pending, Duration::ZERO).unwrap(),
            Command::SetActive(false)
        ));
        assert!(matches!(
            receive_command(&rx, &mut pending, Duration::ZERO).unwrap(),
            Command::Stop
        ));
    }

    fn merged_resize_exposes_final_viewport_before_the_next_input() {
        let output = Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: Arc::new(|_| {}),
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 640,
                height: 480,
                scale: 1.0,
            },
            output.clone(),
        )
        .unwrap();
        state.navigate(&data_url("<!doctype html><style>body{margin:0}#probe{height:50px;width:100%;background:red}@media(max-width:500px){#probe{background:blue}}</style><div id='probe'></div><script>window.resizeCount=0;window.addEventListener('resize',()=>resizeCount++);</script>"), true).unwrap();
        while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) {
            state.tick().unwrap();
        }
        state.publish().unwrap();
        output.mailbox.lock().unwrap().take();
        let final_viewport = Viewport {
            width: 900,
            height: 700,
            scale: 2.0,
        };
        let (tx, rx) = mpsc::channel();
        tx.send(Command::Resize(Viewport {
            width: 1400,
            height: 700,
            scale: 1.0,
        }))
        .unwrap();
        tx.send(Command::Redraw).unwrap();
        tx.send(Command::Resize(final_viewport)).unwrap();
        tx.send(Command::Input(InputEvent::Mouse(MouseEvent::default())))
            .unwrap();
        let mut pending = VecDeque::new();
        state
            .command(receive_command(&rx, &mut pending, Duration::ZERO).unwrap())
            .unwrap();
        assert!(state.tabs.active.page.as_mut().unwrap().Evaluate(&format!("if(innerWidth!==450||innerHeight!=={}||devicePixelRatio!==2||resizeCount!==1)throw Error('merged viewport');", state.viewport.content_height()), "test:merged-resize").unwrap().Succeeded());
        let probe = find_id(state.tabs.active.page.as_ref().unwrap(), "probe").unwrap();
        let items = &state
            .tabs
            .active
            .page
            .as_ref()
            .unwrap()
            .CurrentFrame()
            .unwrap()
            .display_items
            .items;
        assert!(
            items
                .iter()
                .any(|i| i.node_id == probe && i.color.blue == 1.0 && i.color.red == 0.0),
            "final media query must be applied"
        );
        finish_host_turn(&mut state, &rx, &mut pending).unwrap();
        let frame = output.mailbox.lock().unwrap().take().unwrap();
        assert_eq!(frame.viewport, final_viewport);
        assert_eq!(
            frame.content.size(),
            (900, state.viewport.height - state.viewport.toolbar_pixels())
        );
        // A repeated-size update may include an exposure request. It must
        // repaint without dispatching a spurious JS resize event.
        tx.send(Command::Resize(final_viewport)).unwrap();
        tx.send(Command::Redraw).unwrap();
        assert!(matches!(
            receive_command(&rx, &mut pending, Duration::ZERO).unwrap(),
            Command::Input(_)
        ));
        state
            .command(receive_command(&rx, &mut pending, Duration::ZERO).unwrap())
            .unwrap();
        state.publish().unwrap();
        assert!(output.mailbox.lock().unwrap().take().is_some());
        assert!(state
            .tabs
            .active
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                "if(resizeCount!==1)throw Error('duplicate resize event');",
                "test:resize-exposure"
            )
            .unwrap()
            .Succeeded());
    }

    #[test]
    fn mouse_queue_coalesces_only_adjacent_compatible_moves() {
        let (tx, rx) = mpsc::channel();
        let moved = |x, shift| {
            Command::Input(InputEvent::Mouse(MouseEvent {
                position: Offset { x, y: 20.0 },
                modifiers: EventModifiers {
                    shift,
                    ..Default::default()
                },
                ..Default::default()
            }))
        };
        for x in 0..200 {
            tx.send(moved(x as f64, false)).unwrap();
        }
        tx.send(Command::Input(InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kDown,
            position: Offset { x: 199.0, y: 20.0 },
            button: MouseButton::kPrimary,
            ..Default::default()
        })))
        .unwrap();
        tx.send(moved(201.0, false)).unwrap();
        tx.send(moved(202.0, true)).unwrap();
        tx.send(Command::Input(InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kLeave,
            ..Default::default()
        })))
        .unwrap();
        tx.send(Command::Stop).unwrap();
        let mut pending = VecDeque::new();
        let mut events = Vec::new();
        loop {
            match receive_command(&rx, &mut pending, Duration::ZERO).unwrap() {
                Command::Input(InputEvent::Mouse(mouse)) => events.push(mouse),
                Command::Stop => break,
                _ => panic!("input ordering changed"),
            }
        }
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].position.x, 199.0);
        assert_eq!(events[1].r#type, MouseEventType::kDown);
        assert_eq!(events[2].position.x, 201.0);
        assert!(events[3].modifiers.shift);
        assert_eq!(events[4].r#type, MouseEventType::kLeave);
    }

    #[test]
    fn wheel_queue_preserves_displacement_phases_and_input_boundaries() {
        let (tx, rx) = mpsc::channel();
        let wheel = |y, phase, shift| {
            Command::Input(InputEvent::Wheel(WheelEvent {
                position: Offset { x: 40.0, y: 80.0 },
                delta: Offset { x: 0.0, y },
                phase,
                modifiers: EventModifiers {
                    shift,
                    ..Default::default()
                },
                ..Default::default()
            }))
        };
        let moved = |x| {
            Command::Input(InputEvent::Mouse(MouseEvent {
                position: Offset { x, y: 80.0 },
                ..Default::default()
            }))
        };
        tx.send(wheel(0.0, WheelPhase::kBegan, false)).unwrap();
        for _ in 0..200 {
            tx.send(wheel(1.0, WheelPhase::kChanged, false)).unwrap();
            tx.send(moved(40.0)).unwrap();
        }
        tx.send(wheel(0.0, WheelPhase::kEnded, false)).unwrap();
        tx.send(wheel(-5.0, WheelPhase::kChanged, false)).unwrap();
        tx.send(wheel(3.0, WheelPhase::kChanged, false)).unwrap();
        tx.send(wheel(7.0, WheelPhase::kChanged, true)).unwrap();
        tx.send(moved(45.0)).unwrap();
        tx.send(wheel(9.0, WheelPhase::kChanged, true)).unwrap();
        tx.send(Command::Input(InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kDown,
            button: MouseButton::kPrimary,
            ..Default::default()
        })))
        .unwrap();
        tx.send(wheel(10.0, WheelPhase::kChanged, true)).unwrap();
        tx.send(Command::Stop).unwrap();
        let mut pending = VecDeque::new();
        let mut sequence = Vec::new();
        loop {
            match receive_command(&rx, &mut pending, Duration::ZERO).unwrap() {
                Command::Input(InputEvent::Wheel(w)) => sequence.push(format!(
                    "wheel:{:?}:{}:{}",
                    w.phase, w.delta.y, w.modifiers.shift
                )),
                Command::Input(InputEvent::Mouse(m)) => {
                    sequence.push(format!("mouse:{:?}:{}", m.r#type, m.position.x))
                }
                Command::Stop => break,
                _ => panic!("input ordering changed"),
            }
        }
        assert_eq!(
            sequence,
            [
                "wheel:kBegan:0:false",
                "wheel:kChanged:200:false",
                "mouse:kMove:40",
                "wheel:kEnded:0:false",
                "wheel:kChanged:-2:false",
                "wheel:kChanged:7:true",
                "mouse:kMove:45",
                "wheel:kChanged:9:true",
                "mouse:kDown:0",
                "wheel:kChanged:10:true"
            ]
        );
        assert!(!compatible_wheel_delta(
            Offset {
                x: f64::MAX,
                y: 0.0
            },
            Offset {
                x: f64::MAX,
                y: 0.0
            }
        ));
        assert!(!compatible_wheel_delta(
            Offset {
                x: 0.0,
                y: f64::NAN
            },
            Offset::default()
        ));
    }

    fn repeated_cursor_moves_survive_self_posting_script_tasks() {
        use rechrom::page::Cursor;
        let cursors = Arc::new(Mutex::new(Vec::new()));
        let captured = cursors.clone();
        let output = Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: Arc::new(move |event| {
                if let UserEvent::CursorChanged(cursor) = event {
                    captured.lock().unwrap().push(cursor);
                }
            }),
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 640,
                height: 480,
                scale: 1.0,
            },
            output,
        )
        .unwrap();
        state.navigate(&data_url(
            "<!doctype html><style>body{margin:0}a,p{display:block;margin:0;width:300px;height:40px;font:20px sans-serif}</style><a href='https://example.com/'>Article title</a><p>Ordinary prose</p>"
        ), true).unwrap();
        while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) {
            state.tick().unwrap();
        }
        assert!(state
            .tabs
            .active
            .page
            .as_mut()
            .unwrap()
            .Evaluate(
                r#"
            window.turns=0;window.jobs=0;
            window.post=function(){
                var script=document.createElement('script');
                script.textContent='turns++;queueMicrotask(()=>jobs++);if(turns<32)post()';
                document.body.appendChild(script);
            };post();
        "#,
                "test:self-posting-scripts"
            )
            .unwrap()
            .Succeeded());
        for turn in 1..=32 {
            state.tick().unwrap();
            assert!(state.tabs.active.page.as_mut().unwrap().Evaluate(
                &format!("if(turns!=={turn}||jobs!=={turn})throw Error('host task starvation '+turns+'/'+jobs)"),
                "test:script-turn",
            ).unwrap().Succeeded(), "each task must yield after its microtasks");
            let (x, y, expected) = match turn % 3 {
                0 => (20.0, 12.0, Cursor::kPointer),
                1 => (20.0, 52.0, Cursor::kText),
                _ => (500.0, 52.0, Cursor::kDefault),
            };
            state
                .input(InputEvent::Mouse(MouseEvent {
                    position: Offset {
                        x,
                        y: y + TOOLBAR_HEIGHT,
                    },
                    ..Default::default()
                }))
                .unwrap();
            assert_eq!(
                cursors.lock().unwrap().last(),
                Some(&expected),
                "cursor feedback must keep updating between posted scripts (turn {turn})"
            );
        }
    }

    fn article_cursor_follows_the_hit_text_and_link_style() {
        use rechrom::page::Cursor;
        let output = Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: Arc::new(|_| {}),
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 640,
                height: 480,
                scale: 1.0,
            },
            output,
        )
        .unwrap();
        state.navigate(&data_url(
            "<!doctype html><style>body{margin:0}a,p{display:block;margin:0;width:300px;height:40px;font:20px sans-serif}</style><a id='article' href='https://example.com/'><span>Article title</span></a><p>Ordinary prose</p>"
        ), true).unwrap();
        while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) {
            state.tick().unwrap();
        }
        let page = state.tabs.active.page.as_mut().unwrap();
        let move_to = |page: &mut Page, x, y| {
            page.Dispatch(&InputEvent::Mouse(MouseEvent {
                position: Offset { x, y },
                ..Default::default()
            }))
            .unwrap();
        };
        move_to(page, 20.0, 12.0);
        assert_eq!(
            page.Cursor(),
            Cursor::kPointer,
            "article link uses the UA hand cursor"
        );
        move_to(page, 260.0, 12.0);
        assert_eq!(
            page.Cursor(),
            Cursor::kPointer,
            "the link box also uses the hand cursor"
        );
        move_to(page, 20.0, 52.0);
        assert_eq!(page.Cursor(), Cursor::kText, "ordinary text uses an I-beam");
        move_to(page, 500.0, 52.0);
        assert_eq!(
            page.Cursor(),
            Cursor::kDefault,
            "whitespace outside text keeps an arrow"
        );
        assert!(page
            .Evaluate(
                "document.getElementById('article').removeAttribute('href')",
                "test:article-no-href"
            )
            .unwrap()
            .Succeeded());
        move_to(page, 20.0, 12.0);
        assert_eq!(
            page.Cursor(),
            Cursor::kText,
            "an anchor without href is ordinary text"
        );
        assert!(page
            .Evaluate(
                "document.getElementById('article').setAttribute('href','')",
                "test:article-empty-href"
            )
            .unwrap()
            .Succeeded());
        assert_eq!(
            page.Cursor(),
            Cursor::kPointer,
            "an empty href still creates a link"
        );
        assert!(page
            .Evaluate(
                "document.getElementById('article').style.cursor='auto'",
                "test:article-auto"
            )
            .unwrap()
            .Succeeded());
        assert_eq!(
            page.Cursor(),
            Cursor::kText,
            "author auto overrides UA pointer on link text"
        );
        assert!(page
            .Evaluate(
                "document.getElementById('article').style.cursor='crosshair'",
                "test:article-explicit"
            )
            .unwrap()
            .Succeeded());
        assert_eq!(
            page.Cursor(),
            Cursor::kCrosshair,
            "explicit cursor wins over link/text defaults"
        );
    }

    fn native_hover_repaints_without_reflow() {
        let output = Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: Arc::new(|_| {}),
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 320,
                height: 240,
                scale: 1.0,
            },
            output,
        )
        .unwrap();
        state.navigate(&data_url("<!doctype html><style>body{margin:0}button{width:120px;height:40px}</style><button id='theme'>Button</button>"),true).unwrap();
        while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) {
            state.tick().unwrap();
        }
        let page = state.tabs.active.page.as_mut().unwrap();
        let button = find_id(page, "theme").unwrap();
        let geometry = {
            let engine = page.GetLayoutEngine();
            layout_result_for(engine.GetLayoutTree().unwrap(), button)
        };
        let render = |page: &Page| {
            let mut pixels = vec![0; 320 * 192];
            raster::surface::RenderDisplayItemListIntoWindowBuffer(
                &page.CurrentFrame().unwrap().display_items,
                320,
                192,
                1.0,
                &mut pixels,
            )
            .unwrap();
            pixels
        };
        let before = render(page);
        page.Dispatch(&InputEvent::Mouse(MouseEvent {
            position: Offset { x: 20.0, y: 20.0 },
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(
            {
                let engine = page.GetLayoutEngine();
                layout_result_for(engine.GetLayoutTree().unwrap(), button)
            },
            geometry
        );
        let hovered = render(page);
        assert_ne!(
            hovered, before,
            "native hover must update paint even while geometry is cached"
        );
        page.Dispatch(&InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kLeave,
            position: Offset { x: -10.0, y: -10.0 },
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(
            render(page),
            before,
            "leaving must clear native hover appearance"
        );
    }

    fn coordinate_input_and_cursor_use_resolved_styles() {
        use rechrom::page::Cursor;
        let cursors = Arc::new(Mutex::new(Vec::new()));
        let observed = cursors.clone();
        let output = Output {
            mailbox: Arc::new(Mutex::new(None)),
            notify: Arc::new(move |event| {
                if let UserEvent::CursorChanged(cursor) = event {
                    observed.lock().unwrap().push(cursor);
                }
            }),
        };
        let mut state = BrowserState::new(
            Viewport {
                width: 640,
                height: 480,
                scale: 1.0,
            },
            output,
        )
        .unwrap();
        state.navigate(&data_url(r#"<!doctype html><style>
              body{margin:0} #wrap{position:relative;width:250px;height:60px}
              textarea{position:absolute;left:10px;top:10px;width:200px;height:30px;cursor:text}
              #decoration{position:absolute;inset:0;pointer-events:none}
              #off{pointer-events:none} #on{pointer-events:auto;cursor:crosshair;width:80px;height:30px}
              #plain{width:80px;height:30px;cursor:auto}
              </style><div id="wrap"><textarea id="edit"></textarea><div id="decoration"></div></div>
              <div id="off"><div id="on"></div></div><div id="plain"></div>
              <div style="display:none"><input id="ghost"></div>"#), true).unwrap();
        while state.tabs.active.page.as_ref().is_some_and(Page::IsLoading) {
            state.tick().unwrap();
        }
        let edit = find_id(state.tabs.active.page.as_ref().unwrap(), "edit").unwrap();
        let page = state.tabs.active.page.as_mut().unwrap();
        let point = Offset { x: 30.0, y: 20.0 };
        let moved = page
            .Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kMove,
                position: point,
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(
            moved.target_node_id,
            Some(edit),
            "decoration must not capture input"
        );
        assert_eq!(page.Cursor(), Cursor::kText);
        let result = {
            let engine = page.GetLayoutEngine();
            layout_result_for(engine.GetLayoutTree().unwrap(), edit)
        };
        page.Dispatch(&InputEvent::Mouse(MouseEvent {
            position: Offset { x: 30.0, y: 70.0 },
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(
            {
                let engine = page.GetLayoutEngine();
                layout_result_for(engine.GetLayoutTree().unwrap(), edit)
            },
            result,
            "native hover flags must preserve control geometry"
        );
        page.Dispatch(&InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kDown,
            position: point,
            button: MouseButton::kPrimary,
            ..Default::default()
        }))
        .unwrap();
        let empty_caret = page
            .Caret()
            .expect("focused empty control needs layout caret geometry");
        assert!(empty_caret.visible && empty_caret.rect.height > 0.0);
        assert_eq!(empty_caret.rect.width, 1.0);
        let (root, layout_editing) = {
            let engine = page.GetLayoutEngine();
            let tree = engine.GetLayoutTree().unwrap();
            (tree.Root() as *const _, tree.EditingState())
        };
        assert!(layout_editing.selections.Get(edit).unwrap().Collapsed());
        let sibling_id = find_id(page, "on").unwrap();
        let sibling_result = {
            let engine = page.GetLayoutEngine();
            layout_result_for(engine.GetLayoutTree().unwrap(), sibling_id)
        };
        let mut on_pixels = vec![0_u32; 640 * 432];
        raster::surface::RenderDisplayItemListIntoWindowBuffer(
            &page.CurrentFrame().unwrap().display_items,
            640,
            432,
            1.0,
            &mut on_pixels,
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(550));
        page.RunTask().unwrap();
        let off_caret = page.Caret().unwrap();
        assert!(!off_caret.visible);
        assert_eq!(
            off_caret.rect, empty_caret.rect,
            "blink retains layout geometry"
        );
        assert_eq!(
            {
                let engine = page.GetLayoutEngine();
                engine.GetLayoutTree().unwrap().Root() as *const _
            },
            root,
            "blink retains the resident layout root"
        );
        assert_eq!(
            {
                let engine = page.GetLayoutEngine();
                layout_result_for(engine.GetLayoutTree().unwrap(), sibling_id)
            },
            sibling_result,
            "local caret invalidation must retain an unaffected sibling result"
        );
        let mut off_pixels = vec![0_u32; 640 * 432];
        raster::surface::RenderDisplayItemListIntoWindowBuffer(
            &page.CurrentFrame().unwrap().display_items,
            640,
            432,
            1.0,
            &mut off_pixels,
        )
        .unwrap();
        let sample = empty_caret.rect.y as usize * 640 + empty_caret.rect.x as usize;
        assert_ne!(
            on_pixels[sample] & 0x00ff_ffff,
            off_pixels[sample] & 0x00ff_ffff,
            "the caret must actually reach the raster target"
        );
        page.Dispatch(&InputEvent::TextInput(TextInputEvent {
            text: "Rust".into(),
            ..Default::default()
        }))
        .unwrap();
        let end_caret = page.Caret().unwrap();
        assert!(
            end_caret.visible && end_caret.rect.x > empty_caret.rect.x,
            "typing restarts blink and follows shaped text"
        );
        page.Dispatch(&InputEvent::Key(KeyEvent {
            key: "Home".into(),
            ..Default::default()
        }))
        .unwrap();
        assert!(page.Caret().unwrap().rect.x < end_caret.rect.x);
        page.Dispatch(&InputEvent::Key(KeyEvent {
            key: "a".into(),
            modifiers: EventModifiers {
                control: true,
                ..Default::default()
            },
            ..Default::default()
        }))
        .unwrap();
        assert!(
            page.Caret().is_none(),
            "range selections have no insertion caret"
        );
        page.Dispatch(&InputEvent::Key(KeyEvent {
            key: "End".into(),
            ..Default::default()
        }))
        .unwrap();
        page.SetActive(false).unwrap();
        assert!(
            page.Caret().is_none(),
            "inactive host hides caret without changing DOM focus"
        );
        page.SetActive(true).unwrap();
        assert!(page.Caret().unwrap().visible);

        let result = page.Evaluate("if(document.getElementById('edit').value!=='Rust'||document.activeElement.id!=='edit')throw Error('coordinate focus/input');", "test:cursor").unwrap();
        assert!(result.Succeeded(), "{:?}", result.exception);
        let result = page.Evaluate("document.getElementById('ghost').focus();if(document.activeElement.id!=='edit')throw Error('hidden focus stole caret')", "test:hidden-focus").unwrap();
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert!(page.Caret().is_some());
        // A style change under a stationary pointer must refresh feedback.
        let result = page
            .Evaluate(
                "document.getElementById('edit').style.cursor='none'",
                "test:cursor",
            )
            .unwrap();
        assert!(result.Succeeded());
        assert_eq!(page.Cursor(), Cursor::kNone);
        let result = page.Evaluate("document.getElementById('wrap').style.cursor='wait';document.getElementById('edit').style.cursor='inherit'", "test:cursor").unwrap();
        assert!(result.Succeeded());
        assert_eq!(page.Cursor(), Cursor::kWait);
        let result = page
            .Evaluate(
                "document.getElementById('edit').style.cursor='initial'",
                "test:cursor",
            )
            .unwrap();
        assert!(result.Succeeded());
        assert_eq!(page.Cursor(), Cursor::kText);
        let result = page.Evaluate("document.getElementById('edit').style.cursor='url(unavailable-cursor.png), vertical-text'", "test:cursor").unwrap();
        assert!(result.Succeeded());
        assert_eq!(page.Cursor(), Cursor::kVerticalText);
        let on = find_id(page, "on").unwrap();
        let moved = page
            .Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kMove,
                position: Offset { x: 20.0, y: 70.0 },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(
            moved.target_node_id,
            Some(on),
            "auto descendant re-enables hit testing"
        );
        assert_eq!(page.Cursor(), Cursor::kCrosshair);
        page.Dispatch(&InputEvent::Mouse(MouseEvent {
            r#type: MouseEventType::kLeave,
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(page.Cursor(), Cursor::kDefault);
        // The window adapter receives cursor feedback independently of a raster frame.
        state
            .input(InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kMove,
                position: Offset {
                    x: 20.0,
                    y: TOOLBAR_HEIGHT + 70.0,
                },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(cursors.lock().unwrap().last(), Some(&Cursor::kCrosshair));
        state
            .input(InputEvent::Mouse(MouseEvent {
                position: Offset {
                    x: 470.0,
                    y: TOOLBAR_HEIGHT - 24.0,
                },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(cursors.lock().unwrap().last(), Some(&Cursor::kText));
        state
            .input(InputEvent::Mouse(MouseEvent {
                position: Offset {
                    x: 20.0,
                    y: TOOLBAR_HEIGHT + 70.0,
                },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(
            cursors.lock().unwrap().last(),
            Some(&Cursor::kCrosshair),
            "cached page cursor must be restored across surfaces"
        );
        state
            .input(InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kLeave,
                position: Offset { x: -10.0, y: -10.0 },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(
            state.tabs.active.page.as_ref().unwrap().Cursor(),
            Cursor::kDefault
        );
        assert_eq!(cursors.lock().unwrap().last(), Some(&Cursor::kDefault));
        let page = state.tabs.active.page.as_mut().unwrap();
        let result = page.Evaluate("var edit=document.getElementById('edit');edit.style.cursor='initial';edit.addEventListener('mousemove',function(){edit.style.cursor='wait'});", "test:early-cursor").unwrap();
        assert!(result.Succeeded());
        let before = cursors.lock().unwrap().len();
        state
            .input(InputEvent::Mouse(MouseEvent {
                position: Offset {
                    x: 30.0,
                    y: TOOLBAR_HEIGHT + 20.0,
                },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(
            &cursors.lock().unwrap()[before..],
            &[Cursor::kText, Cursor::kWait],
            "native cursor feedback must precede JS, then reflect its style mutation"
        );
        let before = cursors.lock().unwrap().len();
        state
            .input(InputEvent::Mouse(MouseEvent {
                position: Offset {
                    x: 31.0,
                    y: TOOLBAR_HEIGHT + 20.0,
                },
                ..Default::default()
            }))
            .unwrap();
        assert_eq!(
            cursors.lock().unwrap().get(before),
            Some(&Cursor::kWait),
            "fresh input must reassert cached feedback after a native cursor reset"
        );
    }
}
