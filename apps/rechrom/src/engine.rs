//! Public host protocol. Every document is executed by its own Page runtime;
//! the window host only connects Page handles to the display service.
use crate::begin_frame_source::NativeBeginFrame;
use browser::page::{Page, PageClient};
#[cfg(test)]
use dom::dom_mutation::{DOMMutation, DOMMutationType};
#[cfg(test)]
use event_loop::{
    EventLoopEngine, EventLoopExecutor, EventLoopMutation, ExecutionContextId, ScheduledTask,
    TaskPriority, TaskToken, WakeRequest,
};
#[cfg(test)]
use foundation::begin_frame::{BeginFrameArgs, BeginFrameSource};
use interaction::input_event::*;
use layoutng_assembly::internal::layout_input::Size;
#[cfg(test)]
use page_mutation::PageMutation;
use std::{
    cell::RefCell,
    io,
    rc::Rc,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};

#[cfg(test)]
use layoutng_assembly::{fragment_tree::FragmentNode, internal::layout_input::Offset};
#[cfg(test)]
use std::{
    cell::Cell,
    collections::VecDeque,
    sync::atomic::{AtomicBool, Ordering},
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

pub(crate) fn memory_diagnostics_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RECHROM_MEMORY_DIAGNOSTICS").is_some())
}
pub(crate) fn diagnostic_time_ns() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos())
}
/// Immutable RGBA readback used by screenshots and diagnostics. Presentation
/// has already completed before this value is created; it carries no renderer,
/// raster-engine or drawing-library capability.
pub struct CapturedSurface {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl CapturedSurface {
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<u8>) -> io::Result<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|length| length.checked_mul(4))
            .ok_or_else(|| io::Error::other("captured surface dimensions overflow"))?;
        if pixels.len() != expected {
            return Err(io::Error::other(
                "captured surface pixels do not match its dimensions",
            ));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

pub struct WindowFrame {
    pub provenance: Option<crate::page_presentation::FrameProvenance>,
    pub viewport: Viewport,
    pub toolbar: CapturedSurface,
    pub content: CapturedSurface,
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
    CloseWindow,
    FrameReady,
    ChromeDragRegions {
        viewport: Viewport,
        frame_sequence: u64,
        regions: Vec<crate::chrome::DragRegion>,
    },
    /// Native swap completion. This acknowledges ownership transfer to the
    /// platform compositor; it is not physical display feedback.
    FrameSwapCompleted(Viewport),
    Location(String),
    Message(String),
    Fatal(FatalError),
    PageError(crate::page_error::PageError),
    CursorChanged(browser::page::Cursor),
    CaretChanged(Option<browser::page::CaretRect>),
}
/// Opaque application routing key carried by a generic event-loop wake.
/// The scheduler never interprets this value or learns which engine emitted it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EngineWakeToken(pub u64);

#[cfg(test)]
const OPEN_ENGINE_WAKE: EngineWakeToken = EngineWakeToken(1);

#[derive(Clone)]
pub enum Command {
    Navigate(String),
    Resize(Viewport),
    VSyncDisplayChanged(u32),
    Redraw,
    Input(InputEvent),
    NativeWheel(crate::input::NativeWheelSample),
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
    Wake(EngineWakeToken),
    BeginFrame(NativeBeginFrame),
}
#[cfg(test)]
impl Command {
    fn diagnostic_name(&self) -> &'static str {
        match self {
            Self::Navigate(_) => "Host.Navigate",
            Self::Resize(_) => "Host.Resize",
            Self::VSyncDisplayChanged(_) => "Host.VSyncDisplayChanged",
            Self::Redraw => "Host.Redraw",
            Self::Input(_) | Self::QueuedInput { .. } | Self::NativeWheel(_) => "Host.Input",
            Self::FocusAddress => "Host.FocusAddress",
            Self::SetActive(_) => "Host.SetActive",
            Self::Reload => "Host.Reload",
            Self::Evaluate { .. } => "Host.Evaluate",
            Self::NewTab => "Host.NewTab",
            Self::CloseActiveTab => "Host.CloseActiveTab",
            Self::CycleTab(_) => "Host.CycleTab",
            Self::Back => "Host.Back",
            Self::Forward => "Host.Forward",
            Self::Stop => "Host.Stop",
            Self::Wake(_) => "Host.Wake",
            Self::BeginFrame(_) => "Host.BeginFrame",
        }
    }
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
    #[cfg(test)]
    fn message(&self, error: impl std::fmt::Display) {
        (self.notify)(UserEvent::Message(error.to_string()));
    }
}
// MainThreadSchedulerImpl keeps a presentation ThreadType lease on its page
// thread. PlatformThreadApple maps that type to USER_INTERACTIVE; an ordinary
// std::thread otherwise inherits default QoS, even while composing a window.
pub(crate) fn set_page_thread_presentation_priority() {
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

pub fn spawn(
    viewport: Viewport,
    address: String,
    output: Output,
    window_target: crate::window_surface::DisplayTarget,
    fallback_interval: Duration,
    display_id: Option<u32>,
    presentation_config: crate::presentation_runtime::Config,
) -> io::Result<mpsc::Sender<Command>> {
    crate::browser_host::spawn(
        viewport,
        address,
        output,
        window_target,
        fallback_interval,
        display_id,
        presentation_config,
    )
}

fn configure_scrollbar_mode(
    metrics: &mut layoutng_assembly::internal::layout_input_types::ScrollbarThemeMetrics,
    mode: Option<&str>,
) -> io::Result<()> {
    match mode {
        None => {}
        Some("classic") => metrics.uses_overlay_scrollbars = false,
        Some("overlay") => metrics.uses_overlay_scrollbars = true,
        Some(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "RECHROM_SCROLLBAR_MODE must be classic or overlay",
            ))
        }
    }
    Ok(())
}

pub(crate) fn create_document(
    width: f64,
    height: f64,
    scale: f64,
    scripting: bool,
    client: Rc<RefCell<dyn PageClient>>,
    threads: browser::page::PageThreads,
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
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints =
        browser::CreateBrowserConstraints(width.ceil() as u32, height.ceil() as u32);
    // The host selects a theme; pure layout receives only immutable metrics.
    // Leave the established overlay default unchanged unless explicitly asked.
    let scrollbar_mode = std::env::var("RECHROM_SCROLLBAR_MODE")
        .map(Some)
        .or_else(|error| match error {
            std::env::VarError::NotPresent => Ok(None),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid RECHROM_SCROLLBAR_MODE",
            )),
        })?;
    configure_scrollbar_mode(
        constraints
            .scrollbar_theme
            .as_mut()
            .expect("browser scrollbar metrics"),
        scrollbar_mode.as_deref(),
    )?;
    constraints.available_size = Size { width, height };
    constraints.device_pixel_ratio = scale;
    // The embedder supplies the viewport in CSS pixels. Device scale belongs
    // to media-resolution evaluation and the raster transform; applying it a
    // second time as layout zoom makes Layout see a 640 CSS-pixel viewport
    // while media queries see 1280 CSS pixels at DPR 2. Chromium's
    // use-zoom-for-dsf path compensates by changing the LayoutView input size
    // as well. Rechrom keeps the equivalent, simpler CSS-pixel contract at
    // this boundary and reserves layout_zoom_factor for actual page zoom.
    constraints.layout_zoom_factor = 1.0;
    let document_images = Rc::new(RefCell::new(browser::CreateSVGImageDecoderWithConstraints(
        &assembly,
        &constraints,
    )));
    let environment = if scripting {
        Some(browser::page::ScriptEnvironment {
            // This embedding owns a 16 MiB thread. The unoptimized translated
            // VM uses about 87 KiB per call; retain 8 MiB for host/layout frames.
            runtime: Box::new(javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_native_stack_budget(8 * 1024 * 1024)),
            xhr: xhr_transport::CreateHTTPXMLHttpRequestTransport(&options)?,
            user_agent: options.user_agent,
        })
    } else {
        None
    };
    let mut page = Page::CreateWithThreads(
        loader,
        images,
        document_images,
        constraints,
        environment,
        Some(client),
        threads,
    );
    if scripting && std::env::var("RECHROM_DIAGNOSTIC_READ_AHEAD").as_deref() == Ok("1") {
        page.EnableDiagnosticReadAhead(scripting);
    }
    Ok(page)
}
pub(crate) fn data_url(html: &str) -> String {
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

pub(crate) const HOME: &str = r#"<!doctype html><html><head><title>New Tab</title><style>
*{box-sizing:border-box}html,body{margin:0;width:100%;height:100%}
body{display:flex;align-items:center;justify-content:center;background:#fdfefe;color:#062142;font:14px Arial,sans-serif}
main{width:560px;margin:0}
.brand{display:flex;align-items:center;justify-content:center;gap:12px;margin-bottom:42px}
.brand svg{width:48px;height:42px}
h1{margin:0;font-size:30px;line-height:1;font-weight:700;letter-spacing:-1.1px}
form{position:relative;width:100%}
.search-icon{position:absolute;left:17px;top:15px;width:17px;height:17px;color:#687d99;pointer-events:none}
input{display:block;width:100%;height:48px;padding:0 18px 0 48px;border:1px solid #cbd7e1;border-radius:8px;background:#fff;color:#062142;font:15px Arial,sans-serif;outline:none}
input:focus{border-color:#15919d}
.favorites{margin-top:66px}
.favorites h2{display:none}
.sites{display:flex;justify-content:center;gap:18px}
.site{display:flex;flex-direction:column;align-items:center;gap:10px;width:88px;padding:10px 4px;border-radius:8px;color:#062142;text-decoration:none}
.site:hover{background:#eef7f8}
.icon{display:flex;align-items:center;justify-content:center;width:38px;height:38px;border:1px solid #d7e2e8;border-radius:8px;background:#fff;color:#15919d}
.icon svg{width:24px;height:22px}
.github svg{width:21px;height:21px;fill:#181717}
.google svg{width:22px;height:22px}
.label{font-size:13px}
</style></head><body><main><header class="brand"><svg viewBox="0 0 220 190" aria-hidden="true"><g transform="translate(-111.5 -66.5) scale(.35)"><path fill='#15919d' d="M519 324V282C519 244 551 211 590 216L849 259C883 265 910 290 910 326V545C910 584 880 609 844 602L790 594V538L830 545C849 549 861 538 861 521V354C861 337 850 327 834 325L597 283C580 280 568 290 568 306V331Z"/><path fill='#062142' d="M355 388C355 347 386 320 427 327L728 376C757 381 775 403 775 432V476C760 454 745 446 717 441L441 392C422 388 409 399 409 419V575C409 591 418 602 434 605L701 648C714 651 722 645 722 631V461C750 467 775 486 775 512V651C775 692 747 716 710 709L410 660C377 654 355 630 355 595Z"/></g></svg><h1>Rechrom</h1></header><form action="https://www.google.com/search" method="get"><svg class="search-icon" viewBox="0 0 20 20" aria-hidden="true"><circle cx="8.5" cy="8.5" r="5.5" fill="none" stroke="currentColor" stroke-width="1.7"/><path d="M12.5 12.5L17 17" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"/></svg><input name="q" placeholder="Search the web" autofocus></form><section class="favorites"><h2>Favorites</h2><nav class="sites"><a class="site" href="https://rechrom.dev"><span class="icon"><svg viewBox="0 0 220 190" aria-hidden="true"><g transform="translate(-111.5 -66.5) scale(.35)"><path fill='#15919d' d="M519 324V282C519 244 551 211 590 216L849 259C883 265 910 290 910 326V545C910 584 880 609 844 602L790 594V538L830 545C849 549 861 538 861 521V354C861 337 850 327 834 325L597 283C580 280 568 290 568 306V331Z"/><path fill='#062142' d="M355 388C355 347 386 320 427 327L728 376C757 381 775 403 775 432V476C760 454 745 446 717 441L441 392C422 388 409 399 409 419V575C409 591 418 602 434 605L701 648C714 651 722 645 722 631V461C750 467 775 486 775 512V651C775 692 747 716 710 709L410 660C377 654 355 630 355 595Z"/></g></svg></span><span class="label">Rechrom</span></a><a class="site" href="https://www.google.com"><span class="icon google"><svg viewBox="0 0 48 48" aria-hidden="true"><path fill='#ffc107' d="M43.6 20H24v8h11.3C33.7 32.7 29.2 36 24 36c-6.6 0-12-5.4-12-12s5.4-12 12-12c3.1 0 5.8 1.2 8 3.1l5.7-5.7C34.1 6.1 29.3 4 24 4 13 4 4 13 4 24s9 20 20 20 20-9 20-20c0-1.3-.1-2.7-.4-4Z"/><path fill='#ff3d00' d="M6.3 14.7l6.6 4.8C14.7 15.1 19 12 24 12c3.1 0 5.8 1.2 8 3.1l5.7-5.7C34.1 6.1 29.3 4 24 4c-7.7 0-14.3 4.3-17.7 10.7Z"/><path fill='#4caf50' d="M24 44c5.2 0 9.9-2 13.4-5.2l-6.2-5.2C29.1 35.1 26.7 36 24 36c-5.2 0-9.6-3.3-11.3-7.9l-6.5 5C9.6 39.5 16.3 44 24 44Z"/><path fill='#1976d2' d="M43.6 20H24v8h11.3c-.8 2.3-2.3 4.2-4.2 5.6l6.2 5.2C41 35.5 44 30.2 44 24c0-1.3-.1-2.7-.4-4Z"/></svg></span><span class="label">Google</span></a><a class="site" href="https://github.com/rechrom/rechrom"><span class="icon github"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 .7C5.6.7.4 5.9.4 12.3c0 5.1 3.3 9.4 7.8 10.9.6.1.8-.3.8-.6v-2.2c-3.2.7-3.9-1.4-3.9-1.4-.5-1.4-1.3-1.7-1.3-1.7-1.1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1.1 1.8 2.8 1.3 3.5 1 .1-.8.4-1.3.7-1.6-2.5-.3-5.2-1.3-5.2-5.7 0-1.3.4-2.3 1.2-3.1-.1-.3-.5-1.5.1-3.1 0 0 1-.3 3.2 1.2.9-.3 1.9-.4 2.9-.4s2 .1 2.9.4c2.2-1.5 3.2-1.2 3.2-1.2.6 1.6.2 2.8.1 3.1.8.9 1.2 1.9 1.2 3.1 0 4.4-2.7 5.4-5.2 5.7.4.3.8 1 .8 2v3c0 .4.2.7.8.6 4.5-1.5 7.8-5.8 7.8-10.9C23.6 5.9 18.4.7 12 .7Z"/></svg></span><span class="label">GitHub</span></a></nav></section></main></body></html>"#;
