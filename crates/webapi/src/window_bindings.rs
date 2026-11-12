#![allow(non_snake_case)]

use crate::url_record::*;
use foundation::begin_frame::BeginFrameSource;
use javascript::javascript_runtime::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
#[path = "window_performance_timing.rs"]
mod window_performance_timing;
pub use window_performance_timing::WindowPerformanceTiming;
#[path = "window_event_timing.rs"]
mod window_event_timing;
use xhr_transport::*;

// cpp: webapi/window_bindings.cc:17-23
const PERFORMANCE: u64 = u64::MAX;
const LOCATION: u64 = PERFORMANCE - 1;
const NAVIGATOR: u64 = PERFORMANCE - 2;
const PERFORMANCE_TIMING: u64 = PERFORMANCE - 3;
const MIME_TYPES: u64 = PERFORMANCE - 4;
const PLUGINS: u64 = PERFORMANCE - 5;
const SCREEN: u64 = PERFORMANCE - 6;
const WINDOW_GLOBAL_NAMES: &[&str] = &[
    "performance",
    "location",
    "navigator",
    "screen",
    "innerWidth",
    "innerHeight",
    "outerWidth",
    "outerHeight",
    "devicePixelRatio",
    "structuredClone",
    "reportError",
    "queueMicrotask",
    "__queueTask",
    "__request",
    "__requestSync",
    "__cancelRequest",
    "__urlParts",
    "__urlDecodeForm",
    "__queueScrollCompletion",
    "__bindEventTiming",
    "__randomHex",
    "__mediaQueryMatches",
    "setTimeout",
    "setInterval",
    "clearTimeout",
    "clearInterval",
    "requestAnimationFrame",
    "cancelAnimationFrame",
];
// Blink DOMTimer uses the same values. Timers installed by deeply nested
// timer callbacks must stop behaving like an unrestricted busy loop.
const MAX_TIMER_NESTING_LEVEL: u32 = 6;
const MINIMUM_NESTED_TIMER_INTERVAL_MS: f64 = 4.0;

#[derive(Clone)]
struct Timer {
    callback: HostValue,
    arguments: Vec<HostValue>,
    due: Instant,
    interval_ms: f64,
    repeating: bool,
    nesting_level: u32,
}
struct NetworkJob {
    success: JavaScriptFunction,
    failure: JavaScriptFunction,
    id: u64,
    operation: Option<Box<dyn XMLHttpRequestOperation>>,
    response: Option<XMLHttpResponseData>,
    error: String,
}
// Rust borrow adapter for the Page-owned runtime captured by C++ host tasks.
// The task receives the turn's active runtime after the Window borrow is released.
type RuntimeTask = Box<dyn FnOnce(&mut dyn JavaScriptRuntime, &JavaScriptRealm)>;
struct AbortTimeoutCancellation(Rc<RefCell<Option<RuntimeTask>>>);
impl interaction::default::core::dom::abort_signal::CancellableAbortTask
    for AbortTimeoutCancellation
{
    fn Cancel(&self) {
        self.0.borrow_mut().take();
    }
}
enum HostTaskCallback {
    Plain(Box<dyn FnOnce()>),
    WithRuntime(RuntimeTask),
}
type RuntimeTaskReady = Rc<dyn Fn(&mut dyn JavaScriptRuntime, &JavaScriptRealm) -> bool>;
#[derive(Clone)]
enum HostTaskReady {
    Plain(Rc<dyn Fn() -> bool>),
    WithRuntime(RuntimeTaskReady),
}
struct HostTask {
    identity: Option<u64>,
    ready_at: Option<Instant>,
    never_ready: bool,
    run: Option<HostTaskCallback>,
    ready: Option<HostTaskReady>,
}
/// Navigation is handed to the embedder; it must start after active script returns.
#[derive(Clone, Debug)]
pub struct LocationNavigation {
    pub url: String,
    pub referrer: String,
    pub replace_history: bool,
}
type LocationNavigator = Rc<dyn Fn(LocationNavigation)>;
type AnimationTick = Rc<RefCell<Box<dyn FnMut(f64)>>>;

// cpp: webapi/window_bindings.h:21-75
pub struct WindowJavaScriptBindings {
    dom: Rc<RefCell<dyn JavaScriptHostBindings>>,
    dom_namespace: WeakJavaScriptHostObjectNamespace,
    window_dom: Rc<RefCell<dyn JavaScriptHostBindings>>,
    window_namespace: WeakJavaScriptHostObjectNamespace,
    xhr_transport: Box<dyn XMLHttpRequestTransport>,
    user_agent: String,
    document_id: u64,
    globals: Vec<String>,
    location: HashMap<String, String>,
    location_navigator: Option<LocationNavigator>,
    document_domain: String,
    url_records: HashMap<u64, HashMap<String, HostValue>>,
    next_url_record: u64,
    width: f64,
    height: f64,
    scale: f64,
    preferred_color_scheme: style::PreferredColorScheme,
    prefers_reduced_motion: bool,
    forced_colors: style::media_queries::forced_colors::ForcedColors,
    performance_timing: Rc<WindowPerformanceTiming>,
    event_timing_callback: Option<JavaScriptFunction>,
    next_timer: u64,
    timers: HashMap<u64, Timer>,
    timer_deadlines: BTreeSet<(Instant, u64)>,
    timer_nesting_level: u32,
    animation_callbacks: BTreeMap<u64, JavaScriptFunction>,
    animation_fallback_due: Option<Instant>,
    animation_dispatching: bool,
    last_animation_frame: Option<Instant>,
    begin_frame_source: Option<Arc<dyn BeginFrameSource>>,
    next_request: u64,
    network_jobs: Vec<NetworkJob>,
    host_tasks: Vec<HostTask>,
    next_host_task: u64,
    // Exceptions raised by posted-message callbacks are reported at the end
    // of their task, just like timer and network callback exceptions.  The
    // callback itself runs after releasing Window's borrow, so retain the
    // task-owned exception queue separately from the host-task vector.
    posted_message_errors: Rc<RefCell<Vec<JavaScriptException>>>,
    next_task_source: usize,
    animation_tick: Option<AnimationTick>,
    bound_realm: Option<WeakJavaScriptRealm>,
    // Releases the Window host borrow before registering Page's deferred Rust
    // ownership adapters. Source EnqueueTask itself remains synchronous.
    task_registration_flusher: Option<Rc<dyn Fn()>>,
}

fn value(value: HostValue) -> HostResult {
    HostResult {
        value,
        ..Default::default()
    }
}
fn object(id: u64) -> HostResult {
    value(HostValue::Object(HostObjectRef { id }))
}
fn string(text: impl Into<String>) -> HostResult {
    value(HostValue::String(text.into()))
}
fn number(n: f64) -> HostResult {
    value(HostValue::Number(n))
}
fn method(receiver: u64, name: &str) -> HostResult {
    value(HostValue::Method(HostMethodRef {
        receiver,
        name: name.into(),
    }))
}
fn unhandled() -> HostResult {
    HostResult {
        handled: false,
        ..Default::default()
    }
}
fn error(text: &str) -> HostResult {
    HostResult::Failure(JavaScriptExceptionKind::kTypeError, text)
}
fn text_arg(args: &[HostValue], i: usize) -> Option<&str> {
    if let Some(HostValue::String(s)) = args.get(i) {
        Some(s)
    } else {
        None
    }
}
fn function_arg(args: &[HostValue], i: usize) -> Option<JavaScriptFunction> {
    if let Some(HostValue::JavaScriptFunction(f)) = args.get(i) {
        Some(f.clone())
    } else {
        None
    }
}
fn milliseconds(n: f64) -> Duration {
    Duration::from_millis(n.max(0.0) as u64)
}

impl WindowJavaScriptBindings {
    // cpp: webapi/window_bindings.cc:35-64
    pub fn new(
        dom: Rc<RefCell<dyn JavaScriptHostBindings>>,
        xhr_transport: Box<dyn XMLHttpRequestTransport>,
        user_agent: String,
        animation_tick: Option<Box<dyn FnMut(f64)>>,
    ) -> Self {
        Self::WithPerformanceTiming(
            dom,
            xhr_transport,
            user_agent,
            animation_tick,
            Rc::new(WindowPerformanceTiming::new()),
        )
    }
    pub fn WithPerformanceTiming(
        dom: Rc<RefCell<dyn JavaScriptHostBindings>>,
        xhr_transport: Box<dyn XMLHttpRequestTransport>,
        user_agent: String,
        animation_tick: Option<Box<dyn FnMut(f64)>>,
        performance_timing: Rc<WindowPerformanceTiming>,
    ) -> Self {
        let mut globals = dom.borrow().GlobalNames();
        for &name in WINDOW_GLOBAL_NAMES {
            globals.push(name.into());
        }
        let document = dom.borrow_mut().Invoke(&HostCall {
            receiver: 0,
            operation: HostOperation::kGet,
            member: "document",
            symbol: HostSymbol::kNone,
            arguments: &[],
        });
        let document_id = if let HostValue::Object(o) = document.value {
            o.id
        } else {
            0
        };
        let mut result = Self {
            window_dom: dom.clone(),
            window_namespace: Default::default(),
            dom,
            dom_namespace: Default::default(),
            xhr_transport,
            user_agent,
            document_id,
            globals,
            location: HashMap::new(),
            location_navigator: None,
            document_domain: String::new(),
            url_records: HashMap::new(),
            next_url_record: !4095,
            width: 0.0,
            height: 0.0,
            scale: 1.0,
            preferred_color_scheme: style::PreferredColorScheme::kLight,
            prefers_reduced_motion: false,
            forced_colors: style::media_queries::forced_colors::ForcedColors::kNone,
            performance_timing,
            event_timing_callback: None,
            next_timer: 1,
            timers: HashMap::new(),
            timer_deadlines: BTreeSet::new(),
            timer_nesting_level: 0,
            animation_callbacks: BTreeMap::new(),
            animation_fallback_due: None,
            animation_dispatching: false,
            last_animation_frame: None,
            begin_frame_source: None,
            next_request: 1,
            network_jobs: Vec::new(),
            host_tasks: Vec::new(),
            next_host_task: 0,
            posted_message_errors: Rc::new(RefCell::new(Vec::new())),
            next_task_source: 0,
            animation_tick: animation_tick.map(|f| Rc::new(RefCell::new(f))),
            bound_realm: None,
            task_registration_flusher: None,
        };
        result.SetURL("about:blank".into());
        result
    }

    // Rust lifetime adapter: during InvokeWithRuntime the engine reborrows its
    // active realm, instead of Window retaining an aliased runtime pointer.
    pub fn BindRuntime(&mut self, realm: &JavaScriptRealm) {
        self.bound_realm = Some(realm.Downgrade());
    }
    /// Freeze this Document's native wrapper owner before installing WebIDL.
    /// ClearForReuse replaces this association, never the original namespace.
    pub fn BindInitialDocumentNamespace(&mut self, namespace: &JavaScriptHostObjectNamespace) {
        assert!(self.dom_namespace.Upgrade().is_none());
        self.dom_namespace = namespace.Downgrade();
        self.window_namespace = namespace.Downgrade();
    }
    /// ClearForReuse keeps this Window's tasks, native objects and realm.
    /// The new DOM has a separate namespace and never replaces old wrappers.
    pub fn InstallDocumentForReuse(
        &mut self,
        dom: Rc<RefCell<dyn JavaScriptHostBindings>>,
        namespace: &JavaScriptHostObjectNamespace,
        url: String,
    ) {
        self.document_id = dom
            .borrow()
            .DocumentHostObjectId()
            .expect("Document native owner");
        self.dom = dom;
        self.dom_namespace = namespace.Downgrade();
        self.SetURL(url);
    }
    fn WindowMember<'a>(call: &'a HostCall<'_>) -> Option<&'a str> {
        if call.receiver != 0 {
            return None;
        }
        if call.member != "__domInvoke" {
            return Some(call.member);
        }
        let mut receiver = call.arguments.first()?;
        while let HostValue::Retained(_, value) = receiver {
            receiver = value;
        }
        if !matches!(receiver, HostValue::Object(object) if object.id == 0) {
            return None;
        }
        match call.arguments.get(1)? {
            HostValue::String(member) => Some(member),
            _ => None,
        }
    }
    fn IsWindowSupplementMember(member: &str) -> bool {
        matches!(
            member,
            "addEventListener" | "removeEventListener" | "__eventTargetCreate"
        ) || member.starts_with("on")
            || member.starts_with("__navigation")
            || member.starts_with("__abort")
            || member.starts_with("__closeWatcher")
    }
    fn InvocationDOM(&self, call: &HostCall<'_>) -> Rc<RefCell<dyn JavaScriptHostBindings>> {
        if Self::WindowMember(call).is_some_and(Self::IsWindowSupplementMember) {
            self.window_dom.clone()
        } else {
            self.dom.clone()
        }
    }
    pub fn SetAnimationTick(&mut self, callback: Box<dyn FnMut(f64)>) {
        self.animation_tick = Some(Rc::new(RefCell::new(callback)));
    }
    /// LocalDOMWindow::FrameDestroyed / ExecutionContext destruction stops
    /// this Window's task sources. Keepalive resources live in ResourceFetcher,
    /// independently of these non-keepalive XMLHttpRequest operations.
    pub fn ResetForNewWindow(&mut self) {
        self.timers.clear();
        self.timer_deadlines.clear();
        self.animation_callbacks.clear();
        self.animation_fallback_due = None;
        self.begin_frame_source = None;
        self.host_tasks.clear();
        self.network_jobs.clear();
        self.task_registration_flusher = None;
        self.location_navigator = None;
        self.animation_tick = None;
        self.event_timing_callback = None;
    }
    pub fn SetBeginFrameSource(&mut self, source: Option<Arc<dyn BeginFrameSource>>) {
        self.begin_frame_source = source;
        self.animation_fallback_due = None;
        self.ScheduleAnimationFrame();
    }
    pub fn HasPendingAnimationFrames(&self) -> bool {
        !self.animation_callbacks.is_empty()
    }
    /// Earliest owner-thread turn which can advance a Window task source.
    /// The embedding event loop supplies `now`; Window never blocks or owns a
    /// clock-driven message pump.
    pub fn NextTaskDeadline(&self, now: Instant) -> Option<Instant> {
        let mut wake = self.next_timer().map(|(_, due)| due);
        if let Some(due) = self.animation_fallback_due {
            wake = Some(wake.map_or(due, |current| current.min(due)));
        }
        if !self.network_jobs.is_empty() {
            // XMLHttpRequestOperation is a poll interface. Polling remains a
            // Window task until the transport grows a typed wake callback.
            let due = now + Duration::from_millis(1);
            wake = Some(wake.map_or(due, |current| current.min(due)));
        }
        for task in &self.host_tasks {
            if task.never_ready {
                continue;
            }
            let due = if let Some(due) = task.ready_at.filter(|due| *due > now) {
                due
            } else {
                let ready = match &task.ready {
                    None => true,
                    Some(HostTaskReady::Plain(ready)) => ready(),
                    Some(HostTaskReady::WithRuntime(_)) => false,
                };
                if ready {
                    now
                } else {
                    now + Duration::from_millis(1)
                }
            };
            wake = Some(wake.map_or(due, |current| current.min(due)));
        }
        wake
    }
    fn ScheduleAnimationFrame(&mut self) {
        if !self.HasPendingAnimationFrames() {
            return;
        }
        if let Some(source) = &self.begin_frame_source {
            source.request_begin_frame();
        } else if self.animation_fallback_due.is_none() {
            self.animation_fallback_due = Some(Instant::now() + Duration::from_millis(16));
        }
    }
    pub fn SetViewport(&mut self, width: f64, height: f64, scale: f64) {
        self.width = width;
        self.height = height;
        self.scale = scale;
    }
    /// Inject a host preference; this binding never queries platform APIs.
    pub fn SetPreferredColorScheme(&mut self, preference: style::PreferredColorScheme) {
        self.preferred_color_scheme = preference;
    }
    pub fn SetPrefersReducedMotion(&mut self, preference: bool) {
        self.prefers_reduced_motion = preference;
    }
    pub fn SetForcedColors(
        &mut self,
        forced_colors: style::media_queries::forced_colors::ForcedColors,
    ) {
        self.forced_colors = forced_colors;
    }
    pub fn BootstrapSource() -> String {
        crate::WindowBootstrapSource()
    }
    pub fn EnqueueTask(&mut self, task: Box<dyn FnOnce()>, ready: Option<Rc<dyn Fn() -> bool>>) {
        self.host_tasks.push(HostTask {
            identity: None,
            ready_at: None,
            never_ready: false,
            run: Some(HostTaskCallback::Plain(task)),
            ready: ready.map(HostTaskReady::Plain),
        });
    }

    /// Cancellable native task on the same Window task source. EventHandler
    /// timers are task-runner timers, so DOMTimer's nesting clamp does not apply.
    pub fn EnqueueDelayedNativeTask(&mut self, delay: Duration, task: Box<dyn FnOnce()>) -> u64 {
        let identity = self.next_host_task;
        self.next_host_task = identity
            .checked_add(1)
            .expect("native Window task identity");
        self.host_tasks.push(HostTask {
            identity: Some(identity),
            ready_at: Some(Instant::now() + delay),
            never_ready: false,
            run: Some(HostTaskCallback::Plain(task)),
            ready: None,
        });
        identity
    }
    /// AbortSignal::timeout uses the same Window task source with a
    /// cancellable callback payload, released synchronously on cancellation.
    pub(crate) fn EnqueueAbortTimeout(
        &mut self,
        delay: Duration,
        callback: RuntimeTask,
    ) -> Rc<dyn interaction::default::core::dom::abort_signal::CancellableAbortTask> {
        let callback = Rc::new(RefCell::new(Some(callback)));
        let captured = callback.clone();
        let ready_at = Instant::now().checked_add(delay);
        self.host_tasks.push(HostTask {
            identity: None,
            ready_at,
            never_ready: ready_at.is_none(),
            run: Some(HostTaskCallback::WithRuntime(Box::new(
                move |runtime, realm| {
                    let callback = captured.borrow_mut().take();
                    if let Some(callback) = callback {
                        callback(runtime, realm);
                    }
                },
            ))),
            ready: None,
        });
        Rc::new(AbortTimeoutCancellation(callback))
    }
    pub fn CancelNativeTask(&mut self, identity: u64) {
        self.host_tasks
            .retain(|task| task.identity != Some(identity));
    }

    // cpp: webapi/window_bindings.cc:354-360
    // Same host queue/readiness as EnqueueTask. Explicit runtime reborrowing
    // replaces C++ Page::Impl's retained JavaScriptRuntime pointer.
    pub fn EnqueueTaskWithRuntime(
        &mut self,
        task: RuntimeTask,
        ready: Option<Rc<dyn Fn() -> bool>>,
    ) {
        self.host_tasks.push(HostTask {
            identity: None,
            ready_at: None,
            never_ready: false,
            run: Some(HostTaskCallback::WithRuntime(task)),
            ready: ready.map(HostTaskReady::Plain),
        });
    }

    /// Prepare/poll runtime-owned resources before a task becomes runnable.
    /// Called without a Window borrow. Readiness must not execute page script.
    pub fn EnqueueTaskWithRuntimeReadiness(&mut self, task: RuntimeTask, ready: RuntimeTaskReady) {
        self.host_tasks.push(HostTask {
            identity: None,
            ready_at: None,
            never_ready: false,
            run: Some(HostTaskCallback::WithRuntime(task)),
            ready: Some(HostTaskReady::WithRuntime(ready)),
        });
    }

    // cpp: webapi/window_bindings.cc:66-96
    pub fn SetTaskRegistrationFlusher(&mut self, flusher: Rc<dyn Fn()>) {
        self.task_registration_flusher = Some(flusher);
    }

    pub fn SetLocationNavigator(&mut self, navigator: LocationNavigator) {
        self.location_navigator = Some(navigator);
    }

    pub fn SetURL(&mut self, input: String) {
        self.location = HashMap::from([
            ("host".into(), String::new()),
            ("hostname".into(), String::new()),
            ("port".into(), String::new()),
        ]);
        self.location.insert("href".into(), input.clone());
        let colon = input.find(':');
        let protocol = colon.map_or_else(String::new, |n| input[..n + 1].to_owned());
        self.location.insert("protocol".into(), protocol.clone());
        self.location.insert("origin".into(), "null".into());
        let mut url = URLRecord::new();
        if url.set(&input) {
            let hostname = url.part(HOST);
            let mut port = url.part(PORT);
            if (protocol == "https:" && port == "443") || (protocol == "http:" && port == "80") {
                port.clear();
            }
            self.document_domain = hostname.clone();
            self.location.insert("hostname".into(), hostname.clone());
            self.location.insert("port".into(), port.clone());
            let authority = hostname
                + &if port.is_empty() {
                    String::new()
                } else {
                    format!(":{port}")
                };
            self.location.insert("host".into(), authority.clone());
            self.location.insert("pathname".into(), url.part(PATH));
            if protocol == "https:" || protocol == "http:" {
                self.location
                    .insert("origin".into(), protocol + "//" + &authority);
            }
        } else {
            let begin = colon.map_or(0, |n| n + 1);
            let end = input[begin..]
                .find(['?', '#'])
                .map_or(input.len(), |n| begin + n);
            self.location
                .insert("pathname".into(), input[begin..end].into());
        }
        let hash = input.find('#');
        let query = input.find('?');
        self.location.insert(
            "hash".into(),
            hash.filter(|&n| n + 1 < input.len())
                .map_or_else(String::new, |n| input[n..].into()),
        );
        self.location.insert(
            "search".into(),
            query
                .filter(|&n| n < hash.unwrap_or(usize::MAX) && n + 1 < input.len())
                .map_or_else(String::new, |n| {
                    input[n..hash.unwrap_or(input.len())].into()
                }),
        );
    }

    // cpp: webapi/window_bindings.cc:102-359
    fn invoke(
        &mut self,
        call: &HostCall<'_>,
        mut runtime: Option<&mut dyn JavaScriptHostRuntime>,
    ) -> HostResult {
        let receiver = call.receiver;
        let name = call.member;
        let args = call.arguments;
        if receiver == 0 && name == "__mediaQueryMatches" {
            return match call.operation {
                HostOperation::kGet => method(0, name),
                HostOperation::kCall => {
                    use style::media_queries::media_query_backend::{
                        ForMediaValues, ParseMediaQuerySet,
                    };
                    use style::media_queries::{MediaValuesCached, MediaValuesCachedData};
                    let queries = ParseMediaQuerySet(text_arg(args, 0).unwrap_or(""));
                    let values = MediaValuesCached::new(&MediaValuesCachedData {
                        viewport_width: self.width,
                        viewport_height: self.height,
                        small_viewport_width: self.width,
                        small_viewport_height: self.height,
                        large_viewport_width: self.width,
                        large_viewport_height: self.height,
                        dynamic_viewport_width: self.width,
                        dynamic_viewport_height: self.height,
                        device_width: self.width as i32,
                        device_height: self.height as i32,
                        device_pixel_ratio: self.scale as f32,
                        media_type: foundation::String::from("screen"),
                        preferred_color_scheme: self.preferred_color_scheme,
                        prefers_reduced_motion: self.prefers_reduced_motion,
                        forced_colors: self.forced_colors,
                        scripting: style::media_queries::scripting::Scripting::kEnabled,
                        ..Default::default()
                    });
                    value(HostValue::Boolean(ForMediaValues(&values).Eval(&queries)))
                }
                _ => unhandled(),
            };
        }
        if let Some(record) = self.url_records.get(&receiver) {
            return if call.operation == HostOperation::kGet {
                record.get(name).cloned().map_or_else(unhandled, value)
            } else {
                unhandled()
            };
        }
        if receiver == 0 && name == "__bindEventTiming" {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall {
                let Some(callback) = function_arg(args, 0) else {
                    return error("Event Timing requires a callback");
                };
                self.event_timing_callback = Some(callback);
                return HostResult::default();
            }
        }
        if receiver == 0 && name == "__queueScrollCompletion" {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall {
                let Some(HostValue::JavaScriptFunction(resolve)) = args.first() else {
                    return error("Scroll completion requires a resolver");
                };
                let resolve = resolve.clone();
                // ScrollPromiseResolver posts instant completion after the
                // synchronous native offset write, using the host task source.
                self.EnqueueTaskWithRuntime(
                    Box::new(move |runtime, realm| {
                        let _ = runtime.Call(
                            realm,
                            &resolve,
                            &HostValue::default(),
                            &[HostValue::Record(vec![(
                                "interrupted".into(),
                                HostValue::Boolean(false),
                            )])],
                        );
                    }),
                    None,
                );
                return HostResult::default();
            }
        }
        // URLSearchParams::DecodeString (core/url/url_search_params.cc:136):
        // replace '+' before percent decoding, then UTF-8 decode without BOM
        // handling. The bootstrap supplies string slices, not author codecs.
        if receiver == 0 && name == "__urlDecodeForm" {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall {
                let Some(input) = text_arg(args, 0) else {
                    return error("Form decoding requires a string");
                };
                let input = if input.as_bytes().contains(&b'+') {
                    std::borrow::Cow::Owned(input.replace('+', " "))
                } else {
                    std::borrow::Cow::Borrowed(input)
                };
                return value(HostValue::String(
                    percent_encoding::percent_decode_str(&input)
                        .decode_utf8_lossy()
                        .into_owned(),
                ));
            }
        }
        if receiver == 0 && matches!(name, "__urlParts" | "__historyUpdate") {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall {
                let Some(input) = text_arg(args, 0) else {
                    return error("URL requires a string");
                };
                let base = text_arg(args, 1).unwrap_or(&self.location["href"]);
                let mut url = URLRecord::new();
                url.set(base);
                if !url.set(input) {
                    return error("Invalid URL");
                }
                let href = url.part(URL);
                let protocol = url.part(SCHEME) + ":";
                let hostname = url.part(HOST);
                let mut port = url.part(PORT);
                if (protocol == "https:" && port == "443") || (protocol == "http:" && port == "80")
                {
                    port.clear();
                }
                let authority = hostname.clone()
                    + &if port.is_empty() {
                        String::new()
                    } else {
                        format!(":{port}")
                    };
                let origin = url.origin();
                if name == "__historyUpdate" {
                    if origin != self.location["origin"] {
                        return error("History URL must have the same origin");
                    }
                    self.SetURL(href);
                    return HostResult::default();
                }
                let query = url.part(QUERY);
                let fragment = url.part(FRAGMENT);
                let id = self.next_url_record;
                self.next_url_record -= 1;
                self.url_records.insert(
                    id,
                    [
                        ("href", href),
                        ("protocol", protocol),
                        ("host", authority),
                        ("hostname", hostname),
                        ("port", port),
                        ("origin", origin),
                        ("pathname", url.part(PATH)),
                        ("username", url.part(USER)),
                        ("password", url.part(PASSWORD)),
                        (
                            "search",
                            if query.is_empty() {
                                String::new()
                            } else {
                                "?".to_owned() + &query
                            },
                        ),
                        (
                            "hash",
                            if fragment.is_empty() {
                                String::new()
                            } else {
                                "#".to_owned() + &fragment
                            },
                        ),
                    ]
                    .into_iter()
                    .map(|(k, v)| (k.into(), HostValue::String(v)))
                    .collect(),
                );
                return object(id);
            }
        }
        if receiver == 0 && name == "__randomHex" {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall && args.len() == 1 {
                let Some(HostValue::Number(count)) = args.first() else {
                    return error("invalid random byte count");
                };
                if !count.is_finite() || *count < 0.0 || *count > 65536.0 || count.floor() != *count
                {
                    return error("invalid random byte count");
                }
                let mut bytes = vec![0; *count as usize];
                if let Err(e) = getrandom::fill(&mut bytes) {
                    return HostResult::Failure(
                        JavaScriptExceptionKind::kRuntimeError,
                        e.to_string(),
                    );
                }
                let mut result = String::with_capacity(bytes.len() * 2);
                const HEX: &[u8] = b"0123456789abcdef";
                for byte in bytes {
                    result.push(HEX[(byte >> 4) as usize] as char);
                    result.push(HEX[(byte & 15) as usize] as char);
                }
                return string(result);
            }
            return error("random byte count is required");
        }
        if receiver == 0 && name == "__cancelRequest" {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall && !args.is_empty() {
                if let HostValue::Number(id) = args[0] {
                    self.network_jobs.retain(|job| job.id as f64 != id);
                }
                return HostResult::default();
            }
        }
        if receiver == 0 && matches!(name, "structuredClone" | "__request" | "__requestSync") {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall {
                if name == "structuredClone" {
                    if args.is_empty()
                        || self
                            .bound_realm
                            .as_ref()
                            .is_none_or(|realm| !realm.IsValid())
                        || runtime.is_none()
                    {
                        return error("structuredClone requires a value");
                    }
                    return runtime.as_mut().unwrap().Clone(&args[0]);
                }
                let synchronous = name == "__requestSync";
                let expected = if synchronous { 4 } else { 6 };
                if args.len() != expected {
                    return error("invalid request arguments");
                }
                let (Some(method), Some(url), Some(body), Some(headers)) = (
                    text_arg(args, 0),
                    text_arg(args, 1),
                    text_arg(args, 2),
                    text_arg(args, 3),
                ) else {
                    return error("invalid request arguments");
                };
                let mut resolved = url.to_owned();
                if url.find(':').is_none()
                    || url.find(['/', '?', '#']).unwrap_or(usize::MAX)
                        < url.find(':').unwrap_or(usize::MAX)
                {
                    let mut address = URLRecord::new();
                    if address.set(&self.location["href"]) && address.set(url) {
                        resolved = address.part(URL);
                    }
                }
                let id = self.next_request;
                self.next_request += 1;
                let request = XMLHttpRequestData {
                    method: method.into(),
                    url: resolved,
                    referrer: self.location["href"].clone(),
                    body: body.as_bytes().to_vec(),
                    headers: headers
                        .split_terminator('\n')
                        .filter_map(|line| line.split_once(':').map(|(k, v)| (k.into(), v.into())))
                        .collect(),
                };
                if synchronous {
                    // Source: core/xmlhttprequest/xml_http_request.cc:1097-1159.
                    // Blink selects kRequestSynchronously and returns from
                    // send() only after ThreadableLoader has completed.  Keep
                    // that blocking behavior at the existing typed transport
                    // boundary; no Window task or XHR progress event is
                    // dispatched while this operation is pending.
                    let mut operation = match self.xhr_transport.Start(&request) {
                        Ok(operation) => operation,
                        Err(e) => return error(&e.to_string()),
                    };
                    loop {
                        match operation.Poll() {
                            Ok(Some(response)) => {
                                return value(HostValue::Record(vec![
                                    ("status".into(), HostValue::Number(response.status as f64)),
                                    ("bytes".into(), HostValue::Bytes(response.body.into())),
                                    ("url".into(), HostValue::String(response.final_url)),
                                    ("mime".into(), HostValue::String(response.mime_type)),
                                ]));
                            }
                            Ok(None) => std::thread::sleep(Duration::from_millis(1)),
                            Err(e) => return error(&e.to_string()),
                        }
                    }
                }
                let (Some(success), Some(failure)) = (function_arg(args, 4), function_arg(args, 5))
                else {
                    return error("invalid request arguments");
                };
                let mut job = NetworkJob {
                    success,
                    failure,
                    id,
                    operation: None,
                    response: None,
                    error: String::new(),
                };
                match self.xhr_transport.Start(&request) {
                    Ok(mut operation) => {
                        match operation.Poll() {
                            Ok(response) => job.response = response,
                            Err(e) => job.error = e.to_string(),
                        };
                        job.operation = Some(operation);
                    }
                    Err(e) => job.error = e.to_string(),
                }
                self.network_jobs.push(job);
                return number(id as f64);
            }
        }
        if receiver == 0 && name == "queueMicrotask" {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall {
                let Some(callback) = function_arg(args, 0) else {
                    return error("queueMicrotask requires a function");
                };
                if self
                    .bound_realm
                    .as_ref()
                    .is_none_or(|realm| !realm.IsValid())
                    || runtime.is_none()
                {
                    return error("queueMicrotask requires a function");
                }
                return runtime.as_mut().unwrap().EnqueueMicrotask(&callback);
            }
        }
        if receiver == 0 && name == "reportError" {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall {
                let Some(reported) = args.first() else {
                    return error("reportError requires a value");
                };
                let Some(runtime) = runtime.as_mut() else {
                    return error("reportError requires an active script context");
                };
                return runtime.ReportException(reported);
            }
        }
        if receiver == 0 && name == "__queueTask" {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall {
                let Some(callback) = function_arg(args, 0) else {
                    return error("Task queue requires a function");
                };
                let callback = callback.clone();
                let errors = self.posted_message_errors.clone();
                // MessagePort dispatch belongs to the posted-message task
                // source. It is a macrotask, but unlike a nested DOM timer it
                // does not acquire timer nesting or the 4 ms timer clamp.
                self.EnqueueTaskWithRuntime(
                    Box::new(move |runtime, realm| {
                        if let Some(error) = runtime
                            .Call(realm, &callback, &HostValue::default(), &[])
                            .exception
                        {
                            errors.borrow_mut().push(error);
                        }
                    }),
                    None,
                );
                return HostResult::default();
            }
        }
        if receiver == 0
            && matches!(
                name,
                "setTimeout"
                    | "setInterval"
                    | "clearTimeout"
                    | "clearInterval"
                    | "requestAnimationFrame"
                    | "cancelAnimationFrame"
            )
        {
            if call.operation == HostOperation::kGet {
                return method(0, name);
            }
            if call.operation == HostOperation::kCall {
                if name.starts_with("clear") || name == "cancelAnimationFrame" {
                    if let Some(HostValue::Number(id)) = args.first() {
                        if *id > 0.0 {
                            let id = *id as u64;
                            if name == "cancelAnimationFrame" {
                                self.animation_callbacks.remove(&id);
                                if !self.HasPendingAnimationFrames() {
                                    self.animation_fallback_due = None;
                                }
                            } else if let Some(timer) = self.timers.remove(&id) {
                                self.timer_deadlines.remove(&(timer.due, id));
                            }
                        }
                    }
                    return HostResult::default();
                }
                if name == "requestAnimationFrame" {
                    let Some(callback) = function_arg(args, 0) else {
                        return error("requestAnimationFrame requires a function");
                    };
                    let id = self.next_timer;
                    self.next_timer += 1;
                    self.animation_callbacks.insert(id, callback);
                    self.ScheduleAnimationFrame();
                    return number(id as f64);
                }
                let Some(callback) = args.first().filter(|v| {
                    matches!(v, HostValue::JavaScriptFunction(_) | HostValue::String(_))
                }) else {
                    return error("Timer requires a function or source text");
                };
                let delay = match args.get(1) {
                    Some(HostValue::Number(n)) if n.is_finite() => n.clamp(0.0, 2147483647.0),
                    _ => 0.0,
                };
                // Chromium's DOMTimer inherits the nesting level of the
                // currently firing timer and clamps levels above six to 4ms.
                // Keep setInterval's historical 1ms floor at low nesting.
                let nesting_level = self.timer_nesting_level.saturating_add(1);
                let delay = if nesting_level > MAX_TIMER_NESTING_LEVEL {
                    delay.max(MINIMUM_NESTED_TIMER_INTERVAL_MS)
                } else {
                    delay
                };
                let id = self.next_timer;
                self.next_timer += 1;
                let due = Instant::now() + milliseconds(delay);
                browser_tracing::instant(
                    "timer",
                    "TimerInstalled",
                    &[
                        ("id", id as f64),
                        ("delay_ms", delay),
                        ("nesting", nesting_level as f64),
                        ("repeating", (name == "setInterval") as u8 as f64),
                    ],
                );
                let replaced = self.timers.insert(
                    id,
                    Timer {
                        callback: callback.clone(),
                        arguments: args.iter().skip(2).cloned().collect(),
                        due,
                        interval_ms: delay.max(1.0),
                        repeating: name == "setInterval",
                        nesting_level,
                    },
                );
                if let Some(timer) = replaced {
                    self.timer_deadlines.remove(&(timer.due, id));
                }
                self.timer_deadlines.insert((due, id));
                return number(id as f64);
            }
        }
        if call.operation == HostOperation::kGet {
            if receiver == 0 {
                match name {
                    "innerWidth" | "outerWidth" => return number(self.width),
                    "innerHeight" | "outerHeight" => return number(self.height),
                    "devicePixelRatio" => return number(self.scale),
                    "navigator" => return object(NAVIGATOR),
                    "screen" => return object(SCREEN),
                    "performance" => return object(PERFORMANCE),
                    _ => {}
                }
            }
            if receiver == NAVIGATOR {
                return match name {
                    "userAgent" => string(self.user_agent.clone()),
                    "appVersion" => string(
                        String::from_utf8_lossy(
                            self.user_agent
                                .as_bytes()
                                .get(8..)
                                .expect("user agent shorter than source substr offset"),
                        )
                        .into_owned(),
                    ),
                    "platform" => string("MacIntel"),
                    "language" => string("en-US"),
                    "vendor" => string("Google Inc."),
                    "cookieEnabled" => value(HostValue::Boolean(false)),
                    "onLine" => value(HostValue::Boolean(true)),
                    "maxTouchPoints" => number(0.0),
                    "mimeTypes" => object(MIME_TYPES),
                    "plugins" => object(PLUGINS),
                    _ => unhandled(),
                };
            }
            if receiver == MIME_TYPES || receiver == PLUGINS {
                return match name {
                    "length" => number(0.0),
                    "item" | "namedItem" => method(receiver, name),
                    _ => unhandled(),
                };
            }
            if receiver == SCREEN {
                return match name {
                    "width" | "availWidth" => number(self.width),
                    "height" | "availHeight" => number(self.height),
                    "availLeft" | "availTop" => number(0.0),
                    "colorDepth" | "pixelDepth" => number(24.0),
                    _ => unhandled(),
                };
            }
            if (receiver == 0 || receiver == self.document_id) && name == "location" {
                return object(LOCATION);
            }
            if receiver == self.document_id {
                match name {
                    "URL" | "documentURI" => return string(self.location["href"].clone()),
                    "cookie" => return string(""),
                    "domain" => {
                        return self.dom.borrow_mut().Invoke(&HostCall {
                            receiver: 0,
                            operation: HostOperation::kCall,
                            member: "__documentDomainGet",
                            symbol: HostSymbol::kNone,
                            arguments: &[],
                        });
                    }
                    _ => {}
                }
            }
            if receiver == PERFORMANCE {
                return match name {
                    "now" => method(receiver, name),
                    "timeOrigin" => number(self.performance_timing.TimeOrigin()),
                    "timing" => object(PERFORMANCE_TIMING),
                    _ => unhandled(),
                };
            }
            if receiver == PERFORMANCE_TIMING {
                return number(self.performance_timing.TimeOrigin());
            }
            if receiver == LOCATION {
                return if matches!(name, "toString" | "assign" | "replace") {
                    method(receiver, name)
                } else {
                    self.location
                        .get(name)
                        .cloned()
                        .map_or_else(unhandled, string)
                };
            }
        }
        if call.operation == HostOperation::kSet && receiver == self.document_id {
            if name == "cookie" {
                return HostResult::default();
            }
            if name == "domain" {
                let Some(domain) = text_arg(args, 0) else {
                    return error("Invalid document.domain");
                };
                if self.location["hostname"] != domain
                    && !self.location["hostname"].ends_with(&format!(".{domain}"))
                {
                    return error("Invalid document.domain");
                }
                return self.dom.borrow_mut().Invoke(&HostCall {
                    receiver: 0,
                    operation: HostOperation::kCall,
                    member: "__documentDomainSet",
                    symbol: HostSymbol::kNone,
                    arguments: args,
                });
            }
        }
        if call.operation == HostOperation::kCall {
            if receiver == PERFORMANCE && name == "now" {
                return number(self.performance_timing.Now());
            }
            if receiver == LOCATION && name == "toString" {
                return string(self.location["href"].clone());
            }
            if (receiver == MIME_TYPES || receiver == PLUGINS)
                && matches!(name, "item" | "namedItem")
            {
                return value(HostValue::Null(JavaScriptNull));
            }
        }
        if [
            LOCATION,
            PERFORMANCE,
            PERFORMANCE_TIMING,
            NAVIGATOR,
            MIME_TYPES,
            PLUGINS,
            SCREEN,
        ]
        .contains(&receiver)
        {
            return unhandled();
        }
        match runtime {
            Some(runtime) => self
                .InvocationDOM(call)
                .borrow_mut()
                .InvokeWithRuntime(call, runtime),
            None => self.InvocationDOM(call).borrow_mut().Invoke(call),
        }
    }
}

impl JavaScriptHostBindings for WindowJavaScriptBindings {
    fn DocumentHostObjectId(&self) -> Option<HostObjectId> {
        Some(self.document_id)
    }
    fn IsWindowHost(&self) -> bool {
        true
    }
    fn ArgumentNamespace(&self, call: &HostCall<'_>) -> Option<JavaScriptHostObjectNamespace> {
        if call.receiver == 0 && !WINDOW_GLOBAL_NAMES.contains(&call.member) {
            if Self::WindowMember(call).is_some_and(Self::IsWindowSupplementMember) {
                self.window_namespace.Upgrade()
            } else {
                self.dom_namespace.Upgrade()
            }
        } else {
            None
        }
    }
    fn BindTaskAttributionTracker(
        &mut self,
        tracker: Rc<foundation::task_attribution::TaskAttributionTracker>,
    ) {
        self.dom.borrow_mut().BindTaskAttributionTracker(tracker);
    }

    fn OwnsGlobalHostObject(&self, id: HostObjectId) -> bool {
        [
            LOCATION,
            PERFORMANCE,
            PERFORMANCE_TIMING,
            NAVIGATOR,
            MIME_TYPES,
            PLUGINS,
            SCREEN,
        ]
        .contains(&id)
            || self.url_records.contains_key(&id)
    }
    fn ArgumentPolicy(
        &self,
        receiver: HostObjectId,
        member: &str,
    ) -> javascript::javascript_runtime::HostArgumentPolicy {
        self.dom.borrow().ArgumentPolicy(receiver, member)
    }
    fn ObjectTraceSource(
        &self,
    ) -> Option<Rc<dyn javascript::javascript_runtime::HostObjectTraceSource>> {
        self.dom.borrow().ObjectTraceSource()
    }

    fn PrepareInvocation(&mut self, call: &HostCall<'_>) -> Option<HostContinuation> {
        let location_call = call.receiver == LOCATION
            && call.operation == HostOperation::kCall
            && matches!(call.member, "assign" | "replace");
        let href_set = call.operation == HostOperation::kSet
            && ((call.receiver == LOCATION && call.member == "href")
                || ((call.receiver == 0 || call.receiver == self.document_id)
                    && call.member == "location"));
        if call.symbol == HostSymbol::kNone && (location_call || href_set) {
            let argument = call.arguments.first().cloned();
            let missing = location_call && argument.is_none();
            let argument = argument.unwrap_or_default();
            let referrer = self.location["href"].clone();
            let navigator = self.location_navigator.clone();
            let replace_history = location_call && call.member == "replace";
            // Blink Location::SetLocation completes against the entered document,
            // then requests kStandard or kReplaceCurrentItem navigation. Do not
            // mutate current Location while the outgoing document still runs.
            // Coercion may execute user JS, so it must run outside Window's borrow.
            return Some(Box::new(move |runtime| {
                if missing {
                    return error("Location method requires a URL");
                }
                let converted = runtime.ToString(&argument);
                if converted.exception.is_some() || converted.thrown.is_some() {
                    return converted;
                }
                let text = match converted.value {
                    HostValue::String(text) => text,
                    // Location.href is USVString: this is its declared IDL
                    // replacement of unpaired surrogates, after VM ToString.
                    HostValue::String16(units) => String::from_utf16_lossy(&units),
                    _ => unreachable!(),
                };
                let resolved = url::Url::parse(&referrer)
                    .ok()
                    .and_then(|base| base.join(&text).ok());
                let Some(url) = resolved else {
                    return HostResult::Failure(
                        JavaScriptExceptionKind::kSyntaxError,
                        "Invalid navigation URL",
                    );
                };
                if let Some(navigator) = navigator {
                    navigator(LocationNavigation {
                        url: url.into(),
                        referrer,
                        replace_history,
                    });
                }
                HostResult::default()
            }));
        }
        self.InvocationDOM(call)
            .borrow_mut()
            .PrepareInvocation(call)
    }

    fn GlobalNames(&self) -> Vec<String> {
        self.globals.clone()
    }
    fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
        self.invoke(call, None)
    }
    fn InvokeWithRuntime(
        &mut self,
        call: &HostCall<'_>,
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        self.invoke(call, Some(runtime))
    }
    fn UsesOrdinaryProperties(&self, id: u64) -> bool {
        // Window still supplies Document's location/URL/cookie/domain access
        // through property interception. Its other DOM wrappers use the
        // already installed ordinary IDL descriptors supplied by DOM.
        id != self.document_id && self.dom.borrow().UsesOrdinaryProperties(id)
    }
    fn PrototypeFor(&mut self, id: u64) -> HostResult {
        self.dom.borrow_mut().PrototypeFor(id)
    }
}

enum Task {
    Network(NetworkJob),
    Host(HostTaskCallback),
    Timer(Timer),
    AnimationFrame(Instant),
}
impl WindowJavaScriptBindings {
    fn take_network(&mut self) -> Option<Task> {
        for index in 0..self.network_jobs.len() {
            let job = &mut self.network_jobs[index];
            if job.response.is_none() && job.error.is_empty() {
                match job
                    .operation
                    .as_mut()
                    .expect("pending request has operation")
                    .Poll()
                {
                    Ok(response) => job.response = response,
                    Err(e) => job.error = e.to_string(),
                }
            }
            if job.response.is_some() || !job.error.is_empty() {
                return Some(Task::Network(self.network_jobs.remove(index)));
            }
        }
        None
    }
    fn next_timer(&self) -> Option<(u64, Instant)> {
        // HashMap iteration scans capacity, including slots left by cancelled
        // timers. Keep exact deadline/id ordering in a live-entry-only index.
        self.timer_deadlines.first().map(|&(due, id)| (id, due))
    }
    fn take_timer(&mut self) -> Option<Task> {
        let now = Instant::now();
        // Headless users without an embedder source retain a natural timed
        // frame fallback. Attached sources exclusively drive animation frames.
        if let Some(due) = self.animation_fallback_due {
            if self.begin_frame_source.is_none()
                && !self.animation_dispatching
                && due <= now
                && self
                    .next_timer()
                    .is_none_or(|(_, timer_due)| due <= timer_due)
            {
                self.animation_fallback_due = None;
                return Some(Task::AnimationFrame(now));
            }
        }
        let (id, due) = self.next_timer()?;
        if due > now {
            return None;
        }
        let mut timer = self.timers[&id].clone();
        self.timer_deadlines.remove(&(due, id));
        if timer.repeating {
            timer.nesting_level = timer.nesting_level.saturating_add(1);
            if timer.nesting_level == MAX_TIMER_NESTING_LEVEL + 1 {
                timer.interval_ms = timer.interval_ms.max(MINIMUM_NESTED_TIMER_INTERVAL_MS);
            }
            let next_due = Instant::now() + milliseconds(timer.interval_ms);
            let stored = self.timers.get_mut(&id).unwrap();
            stored.due = next_due;
            stored.interval_ms = timer.interval_ms;
            stored.nesting_level = timer.nesting_level;
            self.timer_deadlines.insert((next_due, id));
        } else {
            self.timers.remove(&id);
        }
        Some(Task::Timer(timer))
    }

    /// Blink FrameRequestCallbackCollection snapshots registrations before
    /// invoking callbacks; cancellation remains live throughout that snapshot.
    pub fn RunAnimationFrameCallbacks(
        window: &Rc<RefCell<Self>>,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        frame_time: Instant,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) {
        let _trace = browser_tracing::span("task", "Window.AnimationFrameCallbacks");
        let (ids, timestamp, tick) = {
            let mut w = window.borrow_mut();
            if w.animation_dispatching
                || w.last_animation_frame
                    .is_some_and(|last| frame_time <= last)
            {
                return;
            }
            w.animation_dispatching = true;
            w.last_animation_frame = Some(frame_time);
            w.animation_fallback_due = None;
            (
                w.animation_callbacks.keys().copied().collect::<Vec<_>>(),
                w.performance_timing
                    .MonotonicTimeToDOMHighResTimeStamp(frame_time),
                w.animation_tick.clone(),
            )
        };
        if let Some(tick) = tick {
            tick.borrow_mut()(timestamp);
        }
        for id in ids {
            if !realm.IsValid() {
                break;
            }
            // Release Window's borrow before script/checkpoints reenter it.
            let callback = window.borrow_mut().animation_callbacks.remove(&id);
            let Some(callback) = callback else {
                continue;
            };
            let result = runtime.Call(
                realm,
                &callback,
                &HostValue::Object(HostObjectRef { id: 0 }),
                &[HostValue::Number(timestamp)],
            );
            if let Some(error) = result.exception {
                report_error(&error);
            }
            runtime.PerformMicrotaskCheckpoint();
            for error in runtime.TakePendingExceptions(realm) {
                report_error(&error);
            }
            let flusher = window.borrow().task_registration_flusher.clone();
            if let Some(flusher) = flusher {
                flusher();
            }
        }
        let mut w = window.borrow_mut();
        w.animation_dispatching = false;
        w.ScheduleAnimationFrame();
    }

    // cpp: webapi/window_bindings.cc:362-453
    // Borrow Window only to pick a task; callbacks can reenter its host API.
    pub fn RunTaskTurn(
        window: &Rc<RefCell<Self>>,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        duration_ms: f64,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) {
        let _trace = browser_tracing::span("task", "Window.RunTaskTurn");
        let deadline = Instant::now() + milliseconds(duration_ms);
        // A nonblocking host turn runs one macrotask and its complete microtask
        // checkpoint, then returns to input/render scheduling. Work posted by
        // that task belongs to a later turn, even when it is already ready.
        let task_limit = if duration_ms > 0.0 { 10000 } else { 1 };
        for _ in 0..task_limit {
            let flusher = window.borrow().task_registration_flusher.clone();
            if let Some(flusher) = flusher {
                flusher();
            }
            if duration_ms > 0.0 && Instant::now() >= deadline {
                break;
            }
            let next_source = window.borrow().next_task_source;
            let mut selected = None;
            for attempt in 0..3 {
                let source = (next_source + attempt) % 3;
                selected = match source {
                    0 => window.borrow_mut().take_network(),
                    1 => {
                        let mut index = 0;
                        let mut task = None;
                        loop {
                            let ready = {
                                let w = window.borrow();
                                w.host_tasks
                                    .get(index)
                                    .map(|t| (t.ready.clone(), t.ready_at, t.never_ready))
                            };
                            let Some((ready, ready_at, never_ready)) = ready else {
                                break;
                            };
                            let runnable = !never_ready
                                && ready_at.is_none_or(|due| due <= Instant::now())
                                && match ready {
                                    None => true,
                                    Some(HostTaskReady::Plain(ready)) => ready(),
                                    Some(HostTaskReady::WithRuntime(ready)) => {
                                        ready(runtime, realm)
                                    }
                                };
                            if runnable {
                                task = Some(Task::Host(
                                    window
                                        .borrow_mut()
                                        .host_tasks
                                        .remove(index)
                                        .run
                                        .take()
                                        .unwrap(),
                                ));
                                break;
                            }
                            index += 1;
                        }
                        task
                    }
                    _ => window.borrow_mut().take_timer(),
                };
                if selected.is_some() {
                    window.borrow_mut().next_task_source = (source + 1) % 3;
                    break;
                }
            }
            if let Some(task) = selected {
                // Profile the one selected macrotask, without interrupting its
                // callback or splitting the mandatory microtask checkpoint.
                // Callback time includes checkpoints performed by a host task.
                let task_started = std::env::var_os("BROWSER_PROFILE_INPUT")
                    .is_some()
                    .then(Instant::now);
                let task_kind = match &task {
                    Task::Network(_) => "network",
                    Task::Host(HostTaskCallback::Plain(_)) => "host-plain",
                    Task::Host(HostTaskCallback::WithRuntime(_)) => "host-runtime",
                    Task::AnimationFrame(_) => "animation-frame",
                    Task::Timer(timer) => match &timer.callback {
                        HostValue::String(_) => "timer-source",
                        _ => "timer-function",
                    },
                };
                let task_id = if browser_tracing::enabled() {
                    browser_tracing::next_id()
                } else {
                    0
                };
                let _task_context = browser_tracing::scope(browser_tracing::Context {
                    task_id,
                    ..Default::default()
                });
                let mut _task_trace = browser_tracing::span("task", task_kind);
                if let Task::Timer(timer) = &task {
                    _task_trace.set("nesting", timer.nesting_level as f64);
                    _task_trace.set("interval_ms", timer.interval_ms);
                    _task_trace.set("repeating", timer.repeating as u8 as f64);
                }
                let timer_task = matches!(&task, Task::Timer(_));
                let result = match task {
                    Task::Network(job) => {
                        if let Some(response) = job.response {
                            runtime.Call(
                                realm,
                                &job.success,
                                &HostValue::Object(HostObjectRef { id: 0 }),
                                &[
                                    HostValue::Number(response.status as f64),
                                    HostValue::Bytes(response.body.into()),
                                    HostValue::String(response.final_url),
                                    HostValue::String(response.mime_type),
                                ],
                            )
                        } else {
                            runtime.Call(
                                realm,
                                &job.failure,
                                &HostValue::Object(HostObjectRef { id: 0 }),
                                &[HostValue::String(job.error)],
                            )
                        }
                    }
                    Task::Host(run) => {
                        match run {
                            HostTaskCallback::Plain(run) => run(),
                            HostTaskCallback::WithRuntime(run) => run(runtime, realm),
                        }
                        JavaScriptResult::default()
                    }
                    Task::AnimationFrame(frame_time) => {
                        Self::RunAnimationFrameCallbacks(
                            window,
                            runtime,
                            realm,
                            frame_time,
                            report_error,
                        );
                        if let Some(started) = task_started {
                            let elapsed = started.elapsed();
                            if elapsed >= Duration::from_millis(16) {
                                // Animation dispatch performs its own checkpoints.
                                eprintln!(
                                    "javascript-selected-task-profile kind={task_kind} total_ms={:.3}",
                                    elapsed.as_secs_f64() * 1000.0
                                );
                            }
                        }
                        continue;
                    }
                    Task::Timer(timer) => {
                        // Timer callbacks install descendants at this timer's
                        // inherited level. Reset after the callback's required
                        // microtask checkpoint, matching DOMTimer::Fired.
                        window.borrow_mut().timer_nesting_level = timer.nesting_level;
                        match timer.callback {
                            HostValue::JavaScriptFunction(f) => runtime.Call(
                                realm,
                                &f,
                                &HostValue::Object(HostObjectRef { id: 0 }),
                                &timer.arguments,
                            ),
                            HostValue::String(s) => runtime.Evaluate(realm, &s, "browser:timer"),
                            _ => unreachable!("validated timer callback"),
                        }
                    }
                };
                let callback_elapsed = task_started.map(|started| started.elapsed());
                if let Some(e) = result.exception {
                    report_error(&e);
                }
                let checkpoint_started = task_started.map(|_| Instant::now());
                runtime.PerformMicrotaskCheckpoint();
                for e in runtime.TakePendingExceptions(realm) {
                    report_error(&e);
                }
                let posted_message_errors = {
                    let errors = window.borrow().posted_message_errors.clone();
                    let pending = std::mem::take(&mut *errors.borrow_mut());
                    pending
                };
                for error in posted_message_errors {
                    report_error(&error);
                }
                if timer_task {
                    window.borrow_mut().timer_nesting_level = 0;
                }
                if let Some(started) = task_started {
                    let elapsed = started.elapsed();
                    if elapsed >= Duration::from_millis(16) {
                        eprintln!(
                            "javascript-selected-task-profile kind={task_kind} callback_ms={:.3} checkpoint_ms={:.3} total_ms={:.3}",
                            callback_elapsed.unwrap().as_secs_f64() * 1000.0,
                            checkpoint_started.unwrap().elapsed().as_secs_f64() * 1000.0,
                            elapsed.as_secs_f64() * 1000.0
                        );
                    }
                }
                continue;
            }
            let now = Instant::now();
            let w = window.borrow();
            if now >= deadline
                || (w.timers.is_empty()
                    && w.animation_fallback_due.is_none()
                    && w.network_jobs.is_empty()
                    && w.host_tasks.is_empty())
            {
                break;
            }
            let mut wake = w
                .next_timer()
                .map_or(deadline, |(_, due)| due.min(deadline));
            if let Some(due) = w.animation_fallback_due {
                wake = wake.min(due);
            }
            if !w.network_jobs.is_empty() || !w.host_tasks.is_empty() {
                wake = wake.min(now + Duration::from_millis(1));
            }
            drop(w);
            std::thread::sleep(wake.saturating_duration_since(Instant::now()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom_bindings::DOMJavaScriptBindings;
    use javascript::quickjs_javascript_runtime::JsValue;
    use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
    use std::io;

    struct ImmediateOperation(Option<XMLHttpResponseData>);
    impl XMLHttpRequestOperation for ImmediateOperation {
        fn Poll(&mut self) -> io::Result<Option<XMLHttpResponseData>> {
            Ok(self.0.take())
        }
    }
    struct MockTransport(Rc<RefCell<Vec<XMLHttpRequestData>>>);
    impl XMLHttpRequestTransport for MockTransport {
        fn Start(
            &mut self,
            request: &XMLHttpRequestData,
        ) -> io::Result<Box<dyn XMLHttpRequestOperation>> {
            self.0.borrow_mut().push(request.clone());
            if request.url.ends_with("fail") {
                return Err(io::Error::other("network failure"));
            }
            Ok(Box::new(ImmediateOperation(Some(XMLHttpResponseData {
                status: 201,
                final_url: request.url.clone(),
                mime_type: "text/plain".into(),
                body: b"reply".to_vec(),
                ..Default::default()
            }))))
        }
    }
    fn assert_script(
        runtime: &mut QuickJsJavaScriptRuntime,
        realm: &JavaScriptRealm,
        script: &str,
    ) {
        let result = runtime.Evaluate(realm, script, "window-test.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
    }
    #[test]
    fn timer_deadline_index_preserves_ties_cancellation_and_interval_reentry() {
        let owner = Rc::new(RefCell::new(dom::DOM::new()));
        let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            owner,
            Box::new(|_| {}),
        )));
        let mut window = WindowJavaScriptBindings::new(
            dom,
            Box::new(MockTransport(Rc::new(RefCell::new(Vec::new())))),
            "test".into(),
            None,
        );
        fn register(window: &mut WindowJavaScriptBindings, member: &str, delay: f64) -> u64 {
            let args = [HostValue::String(member.into()), HostValue::Number(delay)];
            let result = window.Invoke(&HostCall {
                receiver: 0,
                operation: HostOperation::kCall,
                member,
                symbol: HostSymbol::kNone,
                arguments: &args,
            });
            if let HostValue::Number(id) = result.value {
                id as u64
            } else {
                panic!("timer id")
            }
        }
        fn cancel(window: &mut WindowJavaScriptBindings, member: &str, id: u64) {
            window.Invoke(&HostCall {
                receiver: 0,
                operation: HostOperation::kCall,
                member,
                symbol: HostSymbol::kNone,
                arguments: &[HostValue::Number(id as f64)],
            });
        }
        let first = register(&mut window, "setTimeout", 1000.0);
        let interval = register(&mut window, "setInterval", 1000.0);
        let later = register(&mut window, "setTimeout", 1000.0);
        // Equal due times are deterministic here; the public registration path
        // normally calls Instant::now separately for each timer.
        let due = Instant::now() - Duration::from_millis(10);
        for (id, deadline) in [
            (first, due),
            (interval, due),
            (later, due + Duration::from_millis(1)),
        ] {
            let timer = window.timers.get_mut(&id).unwrap();
            window.timer_deadlines.remove(&(timer.due, id));
            timer.due = deadline;
            window.timer_deadlines.insert((deadline, id));
        }
        assert_eq!(window.next_timer(), Some((first, due)));
        cancel(&mut window, "clearTimeout", first);
        cancel(&mut window, "cancelAnimationFrame", 999999);
        assert_eq!(window.next_timer(), Some((interval, due)));
        assert!(matches!(window.take_timer(), Some(Task::Timer(timer)) if timer.repeating));
        assert!(window.timers[&interval].due > due);
        assert_eq!(window.timer_deadlines.len(), window.timers.len());
        // A selected interval is already rescheduled before its callback can
        // cancel itself or register another timer.
        cancel(&mut window, "clearInterval", interval);
        let nested = register(&mut window, "setTimeout", 0.0);
        assert_eq!(window.next_timer().unwrap().0, later);
        assert!(matches!(window.take_timer(), Some(Task::Timer(timer)) if !timer.repeating));
        assert_eq!(window.next_timer().unwrap().0, nested);
        assert!(window.take_timer().is_some());
        assert!(window.timers.is_empty());
        assert!(window.timer_deadlines.is_empty());
        assert_eq!(window.next_timer(), None);

        // Blink DOMTimer clamps deeply nested one-shot timers, and raises a
        // fast interval to the same floor after repeated fires cross the
        // nesting threshold.
        window.timer_nesting_level = MAX_TIMER_NESTING_LEVEL;
        let clamped = register(&mut window, "setTimeout", 0.0);
        assert_eq!(window.timers[&clamped].nesting_level, 7);
        assert_eq!(
            window.timers[&clamped].interval_ms,
            MINIMUM_NESTED_TIMER_INTERVAL_MS
        );
        cancel(&mut window, "clearTimeout", clamped);

        window.timer_nesting_level = 0;
        let fast_interval = register(&mut window, "setInterval", 0.0);
        assert_eq!(window.timers[&fast_interval].interval_ms, 1.0);
        for _ in 0..6 {
            let old_due = window.timers[&fast_interval].due;
            window.timer_deadlines.remove(&(old_due, fast_interval));
            window.timers.get_mut(&fast_interval).unwrap().due = due;
            window.timer_deadlines.insert((due, fast_interval));
            assert!(window.take_timer().is_some());
        }
        assert_eq!(
            window.timers[&fast_interval].interval_ms,
            MINIMUM_NESTED_TIMER_INTERVAL_MS
        );
    }

    #[test]
    fn nonblocking_task_turn_yields_to_host_after_checkpoint() {
        let owner = Rc::new(RefCell::new(dom::DOM::new()));
        let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            owner,
            Box::new(|_| {}),
        )));
        let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
            dom,
            Box::new(MockTransport(Rc::new(RefCell::new(Vec::new())))),
            "test".into(),
            None,
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(window.clone());
        assert!(runtime
            .Evaluate(&realm, "var taskCount=0, microtaskCount=0", "test:setup")
            .Succeeded());
        fn enqueue(window: &Rc<RefCell<WindowJavaScriptBindings>>, remaining: usize) {
            let next = Rc::downgrade(window);
            window.borrow_mut().EnqueueTaskWithRuntime(
                Box::new(move |runtime, realm| {
                    let result = runtime.Evaluate(
                        realm,
                        "taskCount++;Promise.resolve().then(()=>microtaskCount++)",
                        "test:self-posting",
                    );
                    assert!(result.Succeeded());
                    if remaining > 1 {
                        enqueue(&next.upgrade().unwrap(), remaining - 1);
                    }
                }),
                None,
            );
        }
        // A finite reproduction of scripts posting more ready tasks. The host
        // must regain control between tasks; their Promise jobs finish first.
        enqueue(&window, 32);
        for expected in 1..=32 {
            WindowJavaScriptBindings::RunTaskTurn(
                &window,
                &mut runtime,
                &realm,
                0.0,
                &mut |error| panic!("unexpected script error: {error:?}"),
            );
            assert_script(
                &mut runtime,
                &realm,
                &format!("taskCount==={expected} && microtaskCount==={expected}"),
            );
        }
    }

    #[test]
    fn bound_window_does_not_own_the_runtime_realm() {
        let owner = Rc::new(RefCell::new(dom::DOM::new()));
        let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            owner,
            Box::new(|_| {}),
        )));
        let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
            dom,
            Box::new(MockTransport(Rc::new(RefCell::new(Vec::new())))),
            "test".into(),
            None,
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(window.clone());
        window.borrow_mut().BindRuntime(&realm);
        assert!(window.borrow().bound_realm.as_ref().unwrap().IsValid());
        assert_eq!(Rc::strong_count(&window), 2);
        drop(realm);
        assert!(
            window.borrow().bound_realm.as_ref().unwrap().IsValid(),
            "engine still owns its realm"
        );
        drop(runtime);
        assert!(!window.borrow().bound_realm.as_ref().unwrap().IsValid());
        assert_eq!(
            Rc::strong_count(&window),
            1,
            "runtime destruction must release its host and Page-owned state"
        );
    }
    #[test]
    fn composed_window_bootstrap_url_globals_tasks_and_network() {
        let owner = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<!doctype html><div></div>",
        )));
        // The loader's canonical origin admission precedes Window bindings;
        // SetURL only changes URL state and cannot manufacture an origin.
        owner
            .borrow_mut()
            .GetDocumentMut()
            .InitializeSecurityContextForNavigation(
                dom::persistent_document::SecurityOrigin::CreateForNavigation(
                    "https://www.example.test:443/a/b?old=1#fragment",
                    None,
                    Rc::new(dom::persistent_document::SecurityOriginSchemeRegistry::default()),
                ),
                false,
            );

        let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            owner,
            Box::new(|_| {}),
        )));
        let requests = Rc::new(RefCell::new(Vec::new()));
        let ticks = Rc::new(RefCell::new(Vec::new()));
        let tick_values = ticks.clone();
        let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
            dom,
            Box::new(MockTransport(requests.clone())),
            "Mozilla/5.0 TEST".into(),
            Some(Box::new(move |time| tick_values.borrow_mut().push(time))),
        )));
        window
            .borrow_mut()
            .SetURL("https://www.example.test:443/a/b?old=1#fragment".into());
        window.borrow_mut().SetViewport(1024.0, 768.0, 2.0);
        window.borrow_mut().SetPrefersReducedMotion(true);
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(window.clone());
        window.borrow_mut().BindRuntime(&realm);
        let result = runtime.Evaluate(&realm, crate::DOMBootstrapSource(), "dom-webidl");
        assert!(result.Succeeded(), "{:?}", result.exception);
        let result = runtime.Evaluate(
            &realm,
            &WindowJavaScriptBindings::BootstrapSource(),
            "window-webidl",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        realm
            .WithImplementation::<javascript::quickjs_javascript_runtime::QuickJsRealm, _>(
                |realm| {
                    let element = realm
                        .context
                        .eval("document.querySelector('div')")
                        .unwrap_or_else(|error| {
                            panic!("{}", realm.context.format_exception(&error))
                        });
                    assert!(
                        !realm.context.is_proxy(&element),
                        "composed host must preserve ordinary Node wrapping"
                    );
                    let collection = realm.context.eval("document.childNodes").unwrap();
                    assert!(
                        realm.context.is_proxy(&collection),
                        "indexed collection still needs interception"
                    );
                    let document = realm.context.eval("document").unwrap();
                    assert!(
                        realm.context.is_proxy(&document),
                        "Window owns additional Document accessors"
                    );
                },
            )
            .unwrap();
        assert_script(
            &mut runtime,
            &realm,
            r#"
            if (document.location !== location || location.href !== document.URL || document.documentURI !== location.href) throw Error('document location identity');
            if (location.origin !== 'https://www.example.test' || location.host !== 'www.example.test' || location.port !== '') throw Error('default port');
            if (navigator.userAgent !== 'Mozilla/5.0 TEST' || navigator.appVersion !== '5.0 TEST' || navigator.platform !== 'MacIntel' || navigator.language !== 'en-US' || navigator.vendor !== 'Google Inc.' || !navigator.onLine || navigator.maxTouchPoints !== 0) throw Error('navigator');
            if (navigator.cookieEnabled || document.cookie !== '' || navigator.plugins.length !== 0 || navigator.mimeTypes.item(0) !== null) throw Error('cookie-disabled source');
            if (innerWidth !== 1024 || outerHeight !== 768 || devicePixelRatio !== 2 || screen.width !== 1024 || screen.colorDepth !== 24) throw Error('viewport');
            if (!matchMedia('(1000px < width <= 1024px) and (resolution: 192dpi)').matches) throw Error('typed media range/resolution');
            if (!matchMedia('(min-width: 64em)').matches || matchMedia('(max-width: 63em)').matches) throw Error('typed media font lengths');
            if (!matchMedia('(prefers-color-scheme: light)').matches || matchMedia('not (unknown: x)').matches) throw Error('typed media preference/unknown');
            if (!matchMedia('(prefers-reduced-motion: reduce)').matches || matchMedia('(prefers-reduced-motion: no-preference)').matches) throw Error('typed reduced motion preference');
            if (!matchMedia('(scripting: enabled)').matches) throw Error('typed media scripting');

            document.cookie='ignored=1';
            document.domain='example.test';
            var rejected=false; try {document.domain='ample.test'} catch(e) {rejected=e instanceof TypeError}
            if (!rejected || document.domain !== 'example.test') throw Error('domain');
            var parts=__urlParts('../next?q=2#h');
            if (parts.href !== 'https://www.example.test/next?q=2#h' || parts.pathname !== '/next' || parts.search !== '?q=2' || parts.hash !== '#h') throw Error('relative URL');
            if(!URL.canParse('https://example.test/a')||!URL.canParse('../next','https://example.test/a/')||URL.canParse('../next')||URL.canParse('/next','not a base'))throw Error('URL.canParse');
            if(URL.parse('../next')!==null||URL.parse('/next','not a base')!==null||URL.parse('../next','https://example.test/a/').href!=='https://example.test/next')throw Error('URL.parse');
            const form=new URLSearchParams('plain=a+b&plus=%2B&eq=a=b&bad=%FF%C0%AF&bom=%EF%BB%BF&emoji=%F0%9F%98%80&bare=%zz&nil=%00&dup=first&dup=last');
            if(form.get('plain')!=='a b'||form.get('plus')!=='+'||form.get('eq')!=='a=b'||form.get('bad')!=='\ufffd\ufffd\ufffd'||form.get('bom')!=='\ufeff'||form.get('emoji')!=='😀'||form.get('bare')!=='%zz'||form.get('nil')!=='\0'||form.getAll('dup').join(',')!=='first,last')throw Error('form decoding');
            if(new URLSearchParams('q=\ud800').get('q')!=='\ufffd'||new URLSearchParams('q=%ED%A0%80').get('q')!=='\ufffd\ufffd\ufffd')throw Error('form scalar conversion');
            const encoder=globalThis.TextEncoder,decoder=globalThis.TextDecoder;
            globalThis.TextEncoder=globalThis.TextDecoder=function(){throw Error('author codec called')};
            try{if(new URLSearchParams('q=plain+%E4%B8%AD').get('q')!=='plain 中')throw Error('native form decoder')}
            finally{globalThis.TextEncoder=encoder;globalThis.TextDecoder=decoder;}
            __historyUpdate('/same?q=3');
            if(location.pathname !== '/same') throw Error('history update');
            rejected=false; try {__historyUpdate('https://other.test/')} catch(e) {rejected=e instanceof TypeError}
            if(!rejected) throw Error('cross-origin history');
            if(__randomHex(16).length !== 32 || !/^[0-9a-f]*$/.test(__randomHex(4))) throw Error('random');
            rejected=false; try {__randomHex(0.5)} catch(e) {rejected=e instanceof TypeError}
            if(!rejected) throw Error('random count');
            var log=[];
            var objectURL = URL.createObjectURL(new Blob(['hello',new Uint8Array([0,255])],{type:'application/octet-stream'}));
            if(!objectURL.startsWith('blob:https://www.example.test/'))throw Error('blob origin');
            var otherObjectURL = URL.createObjectURL(new Blob(['other']));
            if(objectURL===otherObjectURL)throw Error('blob URL identity');
            var blobPassed=false, revokePassed=false, blobMethodPassed=false;
            fetch(objectURL).then(response=>{if(response.status!==200||response.headers.get('content-length')!=='7'||response.headers.get('content-type')!=='application/octet-stream')throw Error('blob headers');return response.arrayBuffer()}).then(buffer=>{var bytes=new Uint8Array(buffer);blobPassed=bytes.length===7&&bytes[0]===104&&bytes[5]===0&&bytes[6]===255;});
            fetch(otherObjectURL,{method:'POST'}).catch(error=>{blobMethodPassed=error instanceof TypeError;});
            URL.revokeObjectURL(objectURL);
            fetch(objectURL).catch(error=>{revokePassed=error instanceof TypeError;});
            URL.revokeObjectURL(objectURL);
            rejected=false;try{URL.createObjectURL('invalid')}catch(error){rejected=error instanceof TypeError}
            if(!rejected)throw Error('blob validation');
            queueMicrotask(()=>log.push('queued'));
            Promise.resolve().then(()=>log.push('promise'));
            __request('POST','../endpoint','payload','X-One: one\nX-Two:two', (s,b,u,m)=>{if(s!==201||new TextDecoder().decode(b)!=='reply'||m!=='text/plain')throw Error('response');log.push('network');queueMicrotask(()=>log.push('network-job'));}, e=>{throw Error(e)});
            var canceled=__request('GET','cancel','','',()=>{throw Error('canceled callback ran')},()=>{throw Error('canceled failure ran')});
            __cancelRequest(canceled);
            __request('GET','fail','','',()=>{throw Error('unexpected success')},e=>{if(e!=='network failure')throw Error('failure');log.push('failure')});
            setTimeout((a,b)=>{if(a!==7||b!==8)throw Error('arguments');log.push('timer');queueMicrotask(()=>log.push('timer-job'));},0,7,8);
            var dropped=setTimeout(()=>{throw Error('cleared timer ran')},0);clearTimeout(dropped);
            var channelLog=[];
            var channel=new MessageChannel();
            channel.port1.onmessage=event=>channelLog.push(event.data.value);
            channel.port2.postMessage({value:9});
            var failingChannel=new MessageChannel();
            failingChannel.port1.onmessage=()=>{throw Error('reported message failure')};
            failingChannel.port2.postMessage(null);
            true
        "#,
        );
        let host_window = window.clone();
        let host_realm = realm.clone();
        // Host callbacks may enqueue another task without a Window borrow panic.
        window.borrow_mut().EnqueueTask(
            Box::new(move || {
                host_window.borrow_mut().EnqueueTask(
                    Box::new(move || {
                        assert!(host_realm.IsValid());
                    }),
                    None,
                );
            }),
            None,
        );
        runtime.PerformMicrotaskCheckpoint();
        assert_script(
            &mut runtime,
            &realm,
            "blobPassed && revokePassed && blobMethodPassed",
        );
        let mut errors = Vec::new();
        // Explicitly pump separate host turns to complete this finite fixture.
        for _ in 0..8 {
            WindowJavaScriptBindings::RunTaskTurn(&window, &mut runtime, &realm, 0.0, &mut |e| {
                errors.push(e.clone())
            });
        }
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message.contains("reported message failure"));
        errors.clear();
        assert_script(&mut runtime, &realm, "JSON.stringify(channelLog) === '[9]'");
        assert_script(
            &mut runtime,
            &realm,
            "JSON.stringify(log) === JSON.stringify(['queued','promise','network','network-job','timer','timer-job','failure'])",
        );
        assert_eq!(
            requests.borrow()[0].url,
            "https://www.example.test/endpoint"
        );
        assert_eq!(requests.borrow()[0].body, b"payload");
        assert_eq!(
            requests.borrow()[0].headers,
            vec![
                ("X-One".into(), " one".into()),
                ("X-Two".into(), "two".into())
            ]
        );
        assert_script(
            &mut runtime,
            &realm,
            r#"
            var intervalCount=0, animationCount=0;
            var interval=setInterval(()=>{intervalCount++;clearInterval(interval);throw new Error('reported timer failure')},0);
            requestAnimationFrame(t=>{if(!(t>=16))throw Error('animation timestamp');animationCount++});
            setTimeout('log.push("source-text")',0);
            true
        "#,
        );
        WindowJavaScriptBindings::RunTaskTurn(&window, &mut runtime, &realm, 30.0, &mut |e| {
            errors.push(e.clone())
        });
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message.contains("reported timer failure"));
        assert_script(
            &mut runtime,
            &realm,
            "intervalCount === 1 && animationCount === 1 && log.at(-1) === 'source-text'",
        );
        assert_eq!(ticks.borrow().len(), 1);
    }

    #[test]
    fn begin_frame_raf_snapshot_timestamp_cancellation_and_checkpoints() {
        struct Source(std::sync::atomic::AtomicUsize);
        impl BeginFrameSource for Source {
            fn request_begin_frame(&self) {
                self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        let owner = Rc::new(RefCell::new(dom::DOM::new()));
        let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            owner,
            Box::new(|_| {}),
        )));
        let ticks = Rc::new(RefCell::new(Vec::new()));
        let observed = ticks.clone();
        let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
            dom,
            Box::new(MockTransport(Rc::new(RefCell::new(Vec::new())))),
            "test".into(),
            Some(Box::new(move |timestamp| {
                observed.borrow_mut().push(timestamp)
            })),
        )));
        let source = Arc::new(Source(std::sync::atomic::AtomicUsize::new(0)));
        window
            .borrow_mut()
            .SetBeginFrameSource(Some(source.clone()));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(window.clone());
        window.borrow_mut().BindRuntime(&realm);
        assert_script(
            &mut runtime,
            &realm,
            r#"(()=>{
            globalThis.frameLog=[];globalThis.frameTimes=[];globalThis.jobFinished=false;
            const timer=setTimeout(()=>frameLog.push('timer'),0);
            cancelAnimationFrame(timer); // RAF cancellation never cancels a timer.
            const first=requestAnimationFrame(t=>{
                frameLog.push('A');frameTimes.push(t);
                queueMicrotask(()=>{jobFinished=true;frameLog.push('microtask');cancelAnimationFrame(second)});
                requestAnimationFrame(t=>{frameLog.push('D');frameTimes.push(t)});
                throw Error('frame callback failure');
            });
            clearTimeout(first); // Timer cancellation never cancels a RAF.
            const second=requestAnimationFrame(()=>{throw Error('cancelled callback ran')});
            requestAnimationFrame(t=>{
                if(!jobFinished)throw Error('missing callback checkpoint');
                frameLog.push('C');frameTimes.push(t);
            });
            let rejected=false;try{requestAnimationFrame('frameLog.push("text")')}catch(e){rejected=e instanceof TypeError}
            return rejected;
        })()"#,
        );
        assert!(source.0.load(std::sync::atomic::Ordering::Relaxed) > 0);
        let mut errors = Vec::new();
        WindowJavaScriptBindings::RunTaskTurn(&window, &mut runtime, &realm, 0.0, &mut |error| {
            errors.push(error.clone())
        });
        assert_script(
            &mut runtime,
            &realm,
            "JSON.stringify(frameLog)==='[\"timer\"]'",
        );
        assert!(ticks.borrow().is_empty());
        let first_time = window.borrow().start + Duration::from_millis(40);
        WindowJavaScriptBindings::RunAnimationFrameCallbacks(
            &window,
            &mut runtime,
            &realm,
            first_time,
            &mut |error| errors.push(error.clone()),
        );
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message.contains("frame callback failure"));
        assert_script(
            &mut runtime,
            &realm,
            "JSON.stringify(frameLog)===JSON.stringify(['timer','A','microtask','C']) && frameTimes.every(t=>t===40)",
        );
        assert!(window.borrow().HasPendingAnimationFrames());
        // An already serviced frame must not execute registrations made in it.
        WindowJavaScriptBindings::RunAnimationFrameCallbacks(
            &window,
            &mut runtime,
            &realm,
            first_time,
            &mut |error| errors.push(error.clone()),
        );
        WindowJavaScriptBindings::RunTaskTurn(&window, &mut runtime, &realm, 0.0, &mut |error| {
            errors.push(error.clone())
        });
        assert_script(&mut runtime, &realm, "frameLog.length===4");
        WindowJavaScriptBindings::RunAnimationFrameCallbacks(
            &window,
            &mut runtime,
            &realm,
            first_time + Duration::from_millis(40),
            &mut |error| errors.push(error.clone()),
        );
        assert_script(
            &mut runtime,
            &realm,
            "JSON.stringify(frameLog)===JSON.stringify(['timer','A','microtask','C','D']) && frameTimes.at(-1)===80",
        );
        assert_eq!(&*ticks.borrow(), &[40.0, 80.0]);
        assert!(!window.borrow().HasPendingAnimationFrames());
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn structured_clone_cache_preserves_public_freshness_globals_and_current_transfer_contract() {
        let owner = Rc::new(RefCell::new(dom::DOM::new()));
        let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            owner,
            Box::new(|_| {}),
        )));
        let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
            dom,
            Box::new(MockTransport(Rc::new(RefCell::new(Vec::new())))),
            "test".into(),
            None,
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(window.clone());
        window.borrow_mut().BindRuntime(&realm);
        assert_script(
            &mut runtime,
            &realm,
            r#"(()=>{
          function check(condition, label){if(!condition)throw Error(label)}
          const names=Object.getOwnPropertyNames(globalThis).join('|');
          const a={v:{x:7}};a.self=a;const first=structuredClone(a),second=structuredClone(a);
          first.v.x=9;check(first!==second&&second.v.x===7&&a.v.x===7,'fresh graphs');
          check(first.self===first&&second.self===second,'cycles');
          check(Object.getPrototypeOf(first)===Object.prototype,'prototype');
          let reads=0;const buffer=new ArrayBuffer(4);new Uint8Array(buffer)[2]=42;
          const copy=structuredClone(buffer,{get transfer(){reads++;throw Error('read transfer')}});
          check(reads===0&&buffer.byteLength===4&&copy.byteLength===4&&new Uint8Array(copy)[2]===42,'current ignored transfer');
          check(structuredClone(buffer,{transfer:[buffer,buffer]}).byteLength===4,'current ignored duplicate transfer');
          check(structuredClone(buffer,{transfer:[{}]}).byteLength===4,'current ignored invalid transfer');
          check(structuredClone(undefined,{transfer:[buffer]})===undefined,'undefined value');
          let missing=false;try{structuredClone()}catch(e){missing=true}check(missing,'missing value');
          let caught=false;try{structuredClone({bad:()=>{}})}catch(e){caught=true}
          check(caught&&structuredClone({ok:8}).ok===8,'failure recovery');
          const OriginalMap=Map;let calls=0;
          globalThis.Map=function(){calls++;return new OriginalMap()};
          try {check(structuredClone({ok:9}).ok===9&&calls===0,'native clone ignores author Map')}finally{globalThis.Map=OriginalMap}
          const token={};let originalThrown=false;
          try{structuredClone({get x(){throw token}})}catch(e){originalThrown=e===token}
          check(originalThrown,'original thrown identity');
          check(names===Object.getOwnPropertyNames(globalThis).join('|'),'no helper globals');
          return true;
        })()"#,
        );
    }
}
