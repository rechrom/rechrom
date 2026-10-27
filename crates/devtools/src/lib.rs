//! Local Performance UI and a small JSON control endpoint for browser tracing.
use base64::Engine;
use browser_tracing::{Snapshot, StartOptions};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

mod smoothness;

const UI: &str = include_str!("performance.html");
const MAX_HEADER: usize = 16 * 1024;
const MAX_BODY: usize = 64 * 1024;
const MAX_CLIENTS: usize = 8;

type ReloadHandler = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;
fn reload_handler() -> &'static std::sync::Mutex<Option<ReloadHandler>> {
    static HANDLER: std::sync::OnceLock<std::sync::Mutex<Option<ReloadHandler>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| std::sync::Mutex::new(None))
}
pub fn set_reload_handler(handler: ReloadHandler) {
    *reload_handler().lock().unwrap() = Some(handler);
}

pub type NavigateHandler = Arc<dyn Fn(String) -> Result<(), String> + Send + Sync>;
fn navigate_handler() -> &'static std::sync::Mutex<Option<NavigateHandler>> {
    static HANDLER: std::sync::OnceLock<std::sync::Mutex<Option<NavigateHandler>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| std::sync::Mutex::new(None))
}
pub fn set_navigate_handler(handler: NavigateHandler) {
    *navigate_handler().lock().unwrap() = Some(handler);
}

/// Evaluate JavaScript in the active page. This is intentionally a control
/// operation: callers that need a value can serialize it into a thrown error
/// until the protocol grows remote-object handles.
pub type EvaluateHandler = Arc<dyn Fn(String) -> Result<(), String> + Send + Sync>;
fn evaluate_handler() -> &'static std::sync::Mutex<Option<EvaluateHandler>> {
    static HANDLER: std::sync::OnceLock<std::sync::Mutex<Option<EvaluateHandler>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| std::sync::Mutex::new(None))
}
pub fn set_evaluate_handler(handler: EvaluateHandler) {
    *evaluate_handler().lock().unwrap() = Some(handler);
}

/// Encoded PNG capture of an immutable target frame. Target 1 is the page and
/// target 2 is browser chrome, matching the tracing target registry.
pub type ScreenshotHandler = Arc<dyn Fn(u64) -> Result<Vec<u8>, String> + Send + Sync>;
fn screenshot_handler() -> &'static std::sync::Mutex<Option<ScreenshotHandler>> {
    static HANDLER: std::sync::OnceLock<std::sync::Mutex<Option<ScreenshotHandler>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| std::sync::Mutex::new(None))
}
pub fn set_screenshot_handler(handler: ScreenshotHandler) {
    *screenshot_handler().lock().unwrap() = Some(handler);
}

/// CDP device metrics in CSS pixels. The browser embedding remains responsible
/// for converting them into its device-pixel viewport and chrome placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeviceMetrics {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f64,
}
pub type DeviceMetricsHandler = Arc<dyn Fn(DeviceMetrics) -> Result<(), String> + Send + Sync>;
fn device_metrics_handler() -> &'static std::sync::Mutex<Option<DeviceMetricsHandler>> {
    static HANDLER: std::sync::OnceLock<std::sync::Mutex<Option<DeviceMetricsHandler>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| std::sync::Mutex::new(None))
}
pub fn set_device_metrics_handler(handler: DeviceMetricsHandler) {
    *device_metrics_handler().lock().unwrap() = Some(handler);
}

/// Synthetic wheel input in window CSS coordinates, including the toolbar.
/// Deltas are CSS pixels with the same sign as the dispatched DOM wheel event.
#[derive(Clone, Debug, PartialEq)]
pub struct WheelInjection {
    pub x: f64,
    pub y: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub phase: String,
    pub precise: bool,
    pub native_timestamp_seconds: Option<f64>,
    pub native_phase: Option<u64>,
    pub momentum_phase: Option<u64>,
    pub native_scale_factor: Option<f64>,
}
pub type WheelHandler = Arc<dyn Fn(WheelInjection) -> Result<(), String> + Send + Sync>;
fn wheel_handler() -> &'static std::sync::Mutex<Option<WheelHandler>> {
    static HANDLER: std::sync::OnceLock<std::sync::Mutex<Option<WheelHandler>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| std::sync::Mutex::new(None))
}
pub fn set_wheel_handler(handler: WheelHandler) {
    *wheel_handler().lock().unwrap() = Some(handler);
}

/// CDP-shaped page-coordinate mouse input. The embedding maps it onto the
/// same typed input queue used by the native window.
#[derive(Clone, Debug, PartialEq)]
pub struct MouseInjection {
    pub target_id: u64,
    pub event_type: String,
    pub x: f64,
    pub y: f64,
    pub button: String,
    pub click_count: u32,
}
pub type MouseHandler = Arc<dyn Fn(MouseInjection) -> Result<(), String> + Send + Sync>;
fn mouse_handler() -> &'static std::sync::Mutex<Option<MouseHandler>> {
    static HANDLER: std::sync::OnceLock<std::sync::Mutex<Option<MouseHandler>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| std::sync::Mutex::new(None))
}
pub fn set_mouse_handler(handler: MouseHandler) {
    *mouse_handler().lock().unwrap() = Some(handler);
}

pub type InsertTextHandler = Arc<dyn Fn(String) -> Result<(), String> + Send + Sync>;
fn insert_text_handler() -> &'static std::sync::Mutex<Option<InsertTextHandler>> {
    static HANDLER: std::sync::OnceLock<std::sync::Mutex<Option<InsertTextHandler>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| std::sync::Mutex::new(None))
}
pub fn set_insert_text_handler(handler: InsertTextHandler) {
    *insert_text_handler().lock().unwrap() = Some(handler);
}

/// CDP-shaped key input, dispatched to the currently focused host Page.
/// This local transport supports rawKeyDown/keyDown/keyUp, not CDP char events.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyInjection {
    pub event_type: String,
    pub key: String,
    pub code: String,
    pub text: String,
    pub windows_virtual_key_code: u32,
    pub location: u32,
    pub modifiers: u8,
    pub repeat: bool,
}
pub type KeyHandler = Arc<dyn Fn(KeyInjection) -> Result<(), String> + Send + Sync>;
fn key_handler() -> &'static std::sync::Mutex<Option<KeyHandler>> {
    static HANDLER: std::sync::OnceLock<std::sync::Mutex<Option<KeyHandler>>> =
        std::sync::OnceLock::new();
    HANDLER.get_or_init(|| std::sync::Mutex::new(None))
}
pub fn set_key_handler(handler: KeyHandler) {
    *key_handler().lock().unwrap() = Some(handler);
}
impl KeyInjection {
    fn from_params(params: &Value) -> Result<Self, String> {
        let event_type = params
            .get("type")
            .and_then(Value::as_str)
            .filter(|t| matches!(*t, "rawKeyDown" | "keyDown" | "keyUp"))
            .ok_or("type must be rawKeyDown, keyDown or keyUp")?;
        let key = params
            .get("key")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty() && s.len() <= 256)
            .ok_or("key must be a nonempty string of at most 256 bytes")?;
        let text = params
            .get("text")
            .map(|v| v.as_str().ok_or("text must be a string"))
            .transpose()?
            .unwrap_or("");
        let code = params
            .get("code")
            .map(|v| v.as_str().ok_or("code must be a string"))
            .transpose()?
            .unwrap_or("");
        if code.len() > 256 {
            return Err("code exceeds 256 bytes".into());
        }
        let windows_virtual_key_code = params
            .get("windowsVirtualKeyCode")
            .map(|v| {
                v.as_u64()
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or("windowsVirtualKeyCode must be an unsigned 32-bit integer")
            })
            .transpose()?
            .unwrap_or(0);
        let location = if params
            .get("isKeypad")
            .map(|v| v.as_bool().ok_or("isKeypad must be a boolean"))
            .transpose()?
            .unwrap_or(false)
        {
            3
        } else if code.ends_with("Left") {
            1
        } else if code.ends_with("Right") {
            2
        } else {
            0
        };
        if text.len() > MAX_BODY {
            return Err("text exceeds 64 KiB".into());
        }
        let modifiers = params
            .get("modifiers")
            .map(|v| {
                v.as_u64()
                    .filter(|n| *n <= 15)
                    .ok_or("modifiers must be an integer in [0, 15]")
            })
            .transpose()?
            .unwrap_or(0);
        let repeat = params
            .get("autoRepeat")
            .map(|v| v.as_bool().ok_or("autoRepeat must be a boolean"))
            .transpose()?
            .unwrap_or(false);
        Ok(Self {
            event_type: event_type.into(),
            key: key.into(),
            code: code.into(),
            text: text.into(),
            windows_virtual_key_code,
            location,
            modifiers: modifiers as u8,
            repeat,
        })
    }
}

impl WheelInjection {
    fn from_params(params: &Value) -> Result<Self, String> {
        // Bound the local control input independently of any browser types.
        // The embedding performs normal viewport hit testing after queueing.
        let number = |name: &str, minimum: f64| -> Result<f64, String> {
            params
                .get(name)
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite() && (minimum..=1_000_000.0).contains(value))
                .ok_or_else(|| format!("{name} must be finite and in [{minimum}, 1000000]"))
        };
        let phase = match params.get("phase") {
            None => "changed",
            Some(value) => value.as_str().ok_or("phase must be a string")?,
        };
        if !matches!(phase, "began" | "changed" | "ended" | "cancelled" | "none") {
            return Err("phase must be began, changed, ended, cancelled or none".into());
        }
        let precise = match params.get("precise") {
            None => true,
            Some(value) => value.as_bool().ok_or("precise must be a boolean")?,
        };
        let optional_number = |name: &str| -> Result<Option<f64>, String> {
            params
                .get(name)
                .map(|value| {
                    value
                        .as_f64()
                        .filter(|value| value.is_finite())
                        .ok_or_else(|| format!("{name} must be finite"))
                })
                .transpose()
        };
        let optional_u64 = |name: &str| -> Result<Option<u64>, String> {
            params
                .get(name)
                .map(|value| {
                    value
                        .as_u64()
                        .ok_or_else(|| format!("{name} must be an unsigned integer"))
                })
                .transpose()
        };
        Ok(Self {
            x: number("x", 0.0)?,
            y: number("y", 0.0)?,
            delta_x: number("delta_x", -1_000_000.0)?,
            delta_y: number("delta_y", -1_000_000.0)?,
            phase: phase.into(),
            precise,
            native_timestamp_seconds: optional_number("native_timestamp_seconds")?,
            native_phase: optional_u64("native_phase")?,
            momentum_phase: optional_u64("momentum_phase")?,
            native_scale_factor: optional_number("native_scale_factor")?,
        })
    }
}

impl MouseInjection {
    fn from_params(params: &Value) -> Result<Self, String> {
        let target_id = params
            .get("target_id")
            .map(|v| {
                v.as_u64()
                    .filter(|id| matches!(*id, 1 | 2))
                    .ok_or("target_id must be 1 (page) or 2 (browser toolbar)")
            })
            .transpose()?
            .unwrap_or(1);
        let number = |name: &str| -> Result<f64, String> {
            params
                .get(name)
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite() && (0.0..=1_000_000.0).contains(value))
                .ok_or_else(|| format!("{name} must be finite and in [0, 1000000]"))
        };
        let event_type = params
            .get("type")
            .and_then(Value::as_str)
            .ok_or("type must be a string")?;
        if !matches!(event_type, "mouseMoved" | "mousePressed" | "mouseReleased") {
            return Err("type must be mouseMoved, mousePressed or mouseReleased".into());
        }
        let button =
            params
                .get("button")
                .and_then(Value::as_str)
                .unwrap_or(if event_type == "mouseMoved" {
                    "none"
                } else {
                    "left"
                });
        if !matches!(button, "none" | "left" | "right" | "middle") {
            return Err("button must be none, left, right or middle".into());
        }
        let click_count = params
            .get("clickCount")
            .map(|value| {
                value
                    .as_u64()
                    .ok_or("clickCount must be an unsigned integer")
            })
            .transpose()?
            .unwrap_or(1);
        if click_count > 2 {
            return Err("clickCount must be 0, 1 or 2".into());
        }
        Ok(Self {
            target_id,
            event_type: event_type.into(),
            x: number("x")?,
            y: number("y")?,
            button: button.into(),
            click_count: click_count as u32,
        })
    }
}

/// Start a detached server on IPv4 localhost. Port zero chooses a free port.
pub fn serve(port: u16) -> io::Result<String> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let port = listener.local_addr()?.port();
    let clients = Arc::new(AtomicUsize::new(0));
    std::thread::Builder::new()
        .name("devtools-http".into())
        .spawn(move || {
            for connection in listener.incoming() {
                let Ok(mut stream) = connection else {
                    continue;
                };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
                if clients.fetch_add(1, Ordering::Relaxed) >= MAX_CLIENTS {
                    clients.fetch_sub(1, Ordering::Relaxed);
                    let _ = respond(
                        &mut stream,
                        "503 Service Unavailable",
                        "application/json",
                        br#"{"error":"Too many connections"}"#,
                    );
                    continue;
                }
                let guard = ClientGuard(clients.clone());
                let _ = std::thread::Builder::new()
                    .name("devtools-request".into())
                    .spawn(move || {
                        let _guard = guard;
                        handle_connection(&mut stream, port);
                    });
            }
        })?;
    Ok(format!("http://127.0.0.1:{port}/"))
}

struct ClientGuard(Arc<AtomicUsize>);
impl Drop for ClientGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

struct Request {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

fn read_request(reader: &mut impl Read) -> Result<Request, String> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 1024];
    let header_end = loop {
        let count = reader.read(&mut chunk).map_err(|e| e.to_string())?;
        if count == 0 {
            return Err("Incomplete request headers".into());
        }
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(index) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            if index + 4 > MAX_HEADER {
                return Err("Request headers too large".into());
            }
            break index + 4;
        }
        if bytes.len() >= MAX_HEADER {
            return Err("Request headers too large".into());
        }
    };
    let text = std::str::from_utf8(&bytes[..header_end]).map_err(|_| "Invalid request headers")?;
    let mut lines = text.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split_ascii_whitespace();
    let method = first.next().ok_or("Missing HTTP method")?.to_owned();
    let path = first.next().ok_or("Missing request path")?.to_owned();
    let version = first.next().ok_or("Missing HTTP version")?;
    if first.next().is_some() || !matches!(version, "HTTP/1.0" | "HTTP/1.1") {
        return Err("Invalid request line".into());
    }
    let mut headers = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').ok_or("Invalid HTTP header")?;
        let name = name.to_ascii_lowercase();
        if name.is_empty()
            || name
                .bytes()
                .any(|b| !b.is_ascii_alphanumeric() && b != b'-')
        {
            return Err("Invalid HTTP header name".into());
        }
        if headers.insert(name, value.trim().to_owned()).is_some() {
            return Err("Duplicate HTTP header".into());
        }
    }
    if headers.contains_key("transfer-encoding") {
        return Err("Transfer-Encoding is unsupported; use Content-Length".into());
    }
    let length = match headers.get("content-length") {
        Some(length) => length
            .parse::<usize>()
            .map_err(|_| "Invalid Content-Length")?,
        None if method == "POST" => return Err("Content-Length required".into()),
        None => 0,
    };
    if length > MAX_BODY {
        return Err("Request body exceeds 64 KiB".into());
    }
    let available = bytes.len() - header_end;
    let mut body = bytes[header_end..header_end + available.min(length)].to_vec();
    if body.len() < length {
        let have = body.len();
        body.resize(length, 0);
        reader
            .read_exact(&mut body[have..])
            .map_err(|e| e.to_string())?;
    }
    Ok(Request {
        method,
        path,
        headers,
        body,
    })
}

fn local_host(host: &str, port: u16) -> bool {
    host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
}

fn handle_connection(stream: &mut TcpStream, port: u16) {
    let request = match read_request(stream) {
        Ok(request) => request,
        Err(error) => {
            let _ = respond_json(stream, "400 Bad Request", json!({"error":error}));
            return;
        }
    };
    if !request
        .headers
        .get("host")
        .is_some_and(|host| local_host(host, port))
    {
        let _ = respond_json(
            stream,
            "403 Forbidden",
            json!({"error":"Local Host header required"}),
        );
        return;
    }
    if request.method == "GET" && request.path == "/" {
        let _ = respond(stream, "200 OK", "text/html; charset=utf-8", UI.as_bytes());
        return;
    }
    if request.path != "/api" {
        let _ = respond_json(stream, "404 Not Found", json!({"error":"Not found"}));
        return;
    }
    if request.method != "POST" {
        let _ = respond_json(
            stream,
            "405 Method Not Allowed",
            json!({"error":"POST required"}),
        );
        return;
    }
    // JSON content type and same-origin checks prevent unrelated websites from
    // controlling a local recording. CLI clients may omit Origin.
    if !request
        .headers
        .get("content-type")
        .is_some_and(|content_type| {
            content_type
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .eq_ignore_ascii_case("application/json")
        })
        || request.headers.get("origin").is_some_and(|origin| {
            origin
                .strip_prefix("http://")
                .is_none_or(|host| !local_host(host, port))
        })
    {
        let _ = respond_json(
            stream,
            "403 Forbidden",
            json!({"error":"Same-origin application/json required"}),
        );
        return;
    }
    let response = serde_json::from_slice::<Value>(&request.body)
        .map_err(|error| format!("Invalid JSON: {error}"))
        .and_then(dispatch);
    match response {
        Ok(result) => {
            let _ = respond_json(stream, "200 OK", SuccessResponse { result });
        }
        Err(error) => {
            let _ = respond_json(stream, "200 OK", json!({"error":error}));
        }
    }
}

#[derive(Serialize)]
struct SuccessResponse {
    result: ApiResult,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ApiResult {
    Json(Value),
    Performance(PerformanceSnapshot),
}

// Serialize the recorded typed output directly. Materializing a second JSON
// object tree for every span multiplies memory while the inspected Page runs.
// Flattening preserves the existing Performance.snapshot wire format.
#[derive(Serialize)]
struct PerformanceSnapshot {
    #[serde(flatten)]
    snapshot: Snapshot,
    summary: Value,
}

fn respond_json(stream: &mut TcpStream, status: &str, value: impl Serialize) -> io::Result<()> {
    let bytes = serde_json::to_vec(&value).map_err(io::Error::other)?;
    respond(stream, status, "application/json; charset=utf-8", &bytes)
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> io::Result<()> {
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; img-src 'self' blob:; frame-ancestors 'none'\r\n\r\n", body.len())?;
    stream.write_all(body)
}

fn recording_id(params: &Value) -> Result<Option<u64>, String> {
    match params.get("recording_id").or_else(|| params.get("id")) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| "recording_id must be an unsigned integer".into()),
    }
}

fn dispatch(request: Value) -> Result<ApiResult, String> {
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .ok_or("method is required")?;
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
    if !params.is_object() {
        return Err("params must be an object".into());
    }
    if method == "Performance.snapshot" {
        let collect_started = std::time::Instant::now();
        let snapshot = browser_tracing::snapshot(recording_id(&params)?)?;
        let collection_ms = collect_started.elapsed().as_secs_f64() * 1000.;
        let analysis_started = std::time::Instant::now();
        let mut summary = analyze_snapshot(&snapshot);
        let analysis_ms = analysis_started.elapsed().as_secs_f64() * 1000.;
        summary["measurement_overhead"] = json!({"collection_ms":collection_ms,"analysis_ms":analysis_ms,"producer_overhead":"not calibrated","serialization_ms":null});
        return Ok(ApiResult::Performance(PerformanceSnapshot {
            snapshot,
            summary,
        }));
    }
    let result = match method {
        "Input.dispatchWheel" => {
            let input = WheelInjection::from_params(&params)?;
            let wheel = wheel_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            wheel(input)?;
            // This acknowledges queueing, not completion or presentation.
            Ok(json!({}))
        }
        "Input.dispatchMouseEvent" => {
            let input = MouseInjection::from_params(&params)?;
            let mouse = mouse_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            mouse(input)?;
            // This acknowledges queueing, not completion or presentation.
            Ok(json!({}))
        }
        "Input.insertText" => {
            let text = params
                .get("text")
                .and_then(Value::as_str)
                .ok_or("text must be a string")?;
            if text.len() > MAX_BODY {
                return Err("text exceeds 64 KiB".into());
            }
            let insert = insert_text_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            insert(text.to_owned())?;
            // This acknowledges queueing, not completion or presentation.
            Ok(json!({}))
        }
        "Input.dispatchKeyEvent" => {
            let input = KeyInjection::from_params(&params)?;
            let key = key_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            key(input)?;
            // This acknowledges queueing, not dispatch or navigation completion.
            Ok(json!({}))
        }
        "Page.reload" => {
            let reload = reload_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            reload()?;
            Ok(json!({}))
        }
        "Page.navigate" => {
            let url = params
                .get("url")
                .and_then(Value::as_str)
                .filter(|url| !url.is_empty() && url.len() <= MAX_BODY)
                .ok_or("url must be a non-empty string no larger than 64 KiB")?;
            let navigate = navigate_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            navigate(url.to_owned())?;
            // Navigation is asynchronous. The target registry and tracing
            // events expose commit/progress; this acknowledges queueing only.
            Ok(json!({"frameId":"1"}))
        }
        "Runtime.evaluate" => {
            let source = params
                .get("expression")
                .and_then(Value::as_str)
                .ok_or("expression must be a string")?
                .to_owned();
            let evaluate = evaluate_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            evaluate(source)?;
            Ok(json!({}))
        }
        "Page.captureScreenshot" => {
            let target_id = params
                .get("target_id")
                .map(|value| {
                    value
                        .as_u64()
                        .ok_or("target_id must be an unsigned integer")
                })
                .transpose()?
                .unwrap_or(1);
            let capture = screenshot_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            let png = capture(target_id)?;
            Ok(json!({"data":base64::engine::general_purpose::STANDARD.encode(png)}))
        }
        "Emulation.setDeviceMetricsOverride" => {
            let width = params
                .get("width")
                .and_then(Value::as_u64)
                .filter(|value| *value > 0 && *value <= u32::MAX as u64)
                .ok_or("width must be a positive integer")? as u32;
            let height = params
                .get("height")
                .and_then(Value::as_u64)
                .filter(|value| *value > 0 && *value <= u32::MAX as u64)
                .ok_or("height must be a positive integer")? as u32;
            let device_scale_factor = params
                .get("deviceScaleFactor")
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite() && *value > 0.0 && *value <= 4.0)
                .ok_or("deviceScaleFactor must be finite and in (0, 4]")?;
            let resize = device_metrics_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            resize(DeviceMetrics {
                width,
                height,
                device_scale_factor,
            })?;
            Ok(json!({}))
        }
        "Tracing.start" => {
            let options: StartOptions =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            Ok(json!({"recording_id":browser_tracing::start(options)?}))
        }
        "Tracing.end" => Ok(json!({"recording_id":browser_tracing::stop()?})),
        "Performance.status" => Ok(browser_tracing::status()),
        "Performance.smoothness" => Ok(smoothness::analyze(&browser_tracing::snapshot(
            recording_id(&params)?,
        )?)),
        "Performance.summary" => Ok(analyze_snapshot(&browser_tracing::snapshot(recording_id(
            &params,
        )?)?)),
        "Performance.export" => Ok(browser_tracing::chrome_trace(&browser_tracing::snapshot(
            recording_id(&params)?,
        )?)),
        _ => Err(format!("Unknown method: {method}")),
    };
    result.map(ApiResult::Json)
}

/// Stage durations are inclusive; nested categories must not be added together.
/// Active/incomplete spans are reported separately and excluded from percentiles.
fn performance_owner<'a>(name: &'a str, category: &'a str) -> &'a str {
    let Some((prefix, _)) = name.split_once('.') else {
        return category;
    };
    if matches!(
        prefix,
        "layout_invalidation" | "layout_reuse" | "layout_fragment" | "layout_commit"
    ) || prefix.strip_prefix("layout_algo_").is_some_and(|suffix| {
        !suffix.is_empty()
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
    }) {
        prefix
    } else {
        category
    }
}

pub fn analyze_snapshot(snapshot: &Snapshot) -> Value {
    let mut groups: BTreeMap<(String, String), Vec<u64>> = BTreeMap::new();
    let mut category_self_times: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut owner_self_times: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut incomplete = 0usize;
    let mut frames: BTreeMap<(u64, u64, u64), (bool, bool, bool)> = BTreeMap::new();
    let mut resize_transactions = BTreeMap::<u64, (u64, usize)>::new();
    for span in &snapshot.spans {
        if span.context.frame_id != 0 {
            let frame = frames
                .entry((
                    span.context.target_id,
                    span.context.source_id,
                    span.context.frame_id,
                ))
                .or_default();
            frame.0 |= span.fields.get("raster_tasks") == Some(&0.0)
                && span
                    .fields
                    .get("reused_tiles")
                    .is_some_and(|&count| count > 0.0);
            frame.1 |= matches!(
                span.name.as_str(),
                "LayoutEngine.Layout"
                    | "LayoutEngine.RunLayout"
                    | "LayoutEngine.NativeLayout"
                    | "LayoutEngine.RunNativeLayoutWithoutPublication"
                    | "LayoutEngine.PopulatePaintData"
                    | "Paint.RecordPaint"
                    | "PaintLayerPainter.Paint"
                    | "PaintEngine.Paint"
            );
            frame.2 |= (span.name == "Page.UpdateFrameIfNeeded"
                && span.fields.get("scroll_records_reused") == Some(&1.0))
                || (matches!(
                    span.name.as_str(),
                    "Page.ApplyPendingScrollUpdates" | "PaintEngine.TryUpdateScrollProperties"
                ) && span.fields.get("records_reused") == Some(&1.0));
        }
        if span.incomplete {
            incomplete += 1;
            continue;
        }
        if span.name == "ResizeToPresentationComplete" {
            let id = if span.context.task_id == 0 {
                span.id
            } else {
                span.context.task_id
            };
            let samples = span.fields.get("samples").copied().unwrap_or(1.0).max(0.0) as usize;
            resize_transactions
                .entry(id)
                .and_modify(|transaction| {
                    if span.duration_ns < transaction.0 {
                        *transaction = (span.duration_ns, samples);
                    }
                })
                .or_insert((span.duration_ns, samples));
        }
        groups
            .entry((span.category.to_string(), span.name.to_string()))
            .or_default()
            .push(span.duration_ns);
    }
    // Attribute synchronous wall time to the innermost recorded package span.
    // Subtract only direct children: their inclusive intervals already contain
    // every deeper descendant, and each child contributes its own self time to
    // the category that owns it. This makes category totals additive without
    // double-counting nested spans from the same package.
    let complete = snapshot
        .spans
        .iter()
        .filter(|span| !span.incomplete)
        .map(|span| (span.id, span))
        .collect::<HashMap<_, _>>();
    let mut child_intervals = HashMap::<u64, Vec<(u64, u64)>>::new();
    for child in complete.values() {
        let Some(parent) = complete.get(&child.parent_id) else {
            continue;
        };
        if child.thread_id != parent.thread_id {
            continue;
        }
        let parent_end = parent.start_ns.saturating_add(parent.duration_ns);
        let child_start = child.start_ns.max(parent.start_ns);
        let child_end = child
            .start_ns
            .saturating_add(child.duration_ns)
            .min(parent_end);
        if child_start < child_end {
            child_intervals
                .entry(parent.id)
                .or_default()
                .push((child_start, child_end));
        }
    }
    for span in complete.values() {
        let mut intervals = child_intervals.remove(&span.id).unwrap_or_default();
        intervals.sort_unstable();
        let mut covered_ns = 0u64;
        let mut covered_until = 0u64;
        for (start, end) in intervals {
            let uncovered_start = start.max(covered_until);
            if uncovered_start < end {
                covered_ns = covered_ns.saturating_add(end - uncovered_start);
                covered_until = end;
            }
        }
        category_self_times
            .entry(span.category.clone())
            .or_default()
            .push(span.duration_ns.saturating_sub(covered_ns));
        owner_self_times
            .entry(performance_owner(&span.name, &span.category).to_string())
            .or_default()
            .push(span.duration_ns.saturating_sub(covered_ns));
    }
    let mut stages: Vec<Value> = groups
        .into_iter()
        .map(|((category, name), durations)| {
            let mut stage = duration_statistics(durations);
            let owner = performance_owner(&name, &category).to_string();
            stage["category"] = json!(category);
            stage["name"] = json!(name);
            stage["owner"] = json!(owner);
            stage
        })
        .collect();
    stages.sort_by(|a, b| {
        b["total_ms"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["total_ms"].as_f64().unwrap_or(0.0))
    });
    let mut categories = category_self_times
        .into_iter()
        .map(|(category, durations)| {
            let mut stats = duration_statistics(durations);
            stats["category"] = json!(category);
            stats
        })
        .collect::<Vec<_>>();
    categories.sort_by(|a, b| {
        b["total_ms"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["total_ms"].as_f64().unwrap_or(0.0))
    });
    let mut owners = owner_self_times
        .into_iter()
        .map(|(owner, durations)| {
            let mut stats = duration_statistics(durations);
            stats["owner"] = json!(owner);
            stats
        })
        .collect::<Vec<_>>();
    owners.sort_by(|a, b| {
        b["total_ms"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["total_ms"].as_f64().unwrap_or(0.0))
    });
    let resize_durations = resize_transactions
        .values()
        .map(|transaction| transaction.0)
        .collect::<Vec<_>>();
    let resize_samples = resize_transactions
        .values()
        .map(|transaction| transaction.1)
        .sum::<usize>();
    let resize_transaction_count = resize_transactions.len();
    let mut resize = duration_statistics(resize_durations);
    resize["measurement"] = json!("ingress_to_presentation_complete");
    resize["samples"] = json!(resize_samples);
    resize["coalesced_samples"] = json!(resize_samples.saturating_sub(resize_transaction_count));
    json!({"recording_id":snapshot.recording_id,"active":snapshot.active,
        "duration_ms":snapshot.duration_ns as f64 / 1_000_000.0,
        "dropped_events":snapshot.dropped_events,"incomplete_spans":incomplete,
        "cache_hit_frames":frames.values().filter(|frame| frame.0).count(),
        "pure_cached_scroll_frames":frames.values().filter(|frame| frame.0 && !frame.1 && frame.2).count(),
        "duration_kind":"inclusive","category_duration_kind":"exclusive",
        "owner_duration_kind":"exclusive","percentile_method":"nearest_rank",
        "resize":resize,
        "stages":stages,"categories":categories,"owners":owners,
        "smoothness":smoothness::analyze(snapshot)})
}

fn duration_statistics(mut durations: Vec<u64>) -> Value {
    durations.sort_unstable();
    let count = durations.len();
    let percentile = |numerator: usize| -> f64 {
        if count == 0 {
            return 0.0;
        }
        let rank = (count * numerator).div_ceil(100);
        durations[rank.saturating_sub(1)] as f64 / 1_000_000.0
    };
    let total_ms = durations
        .iter()
        .map(|&ns| ns as f64 / 1_000_000.0)
        .sum::<f64>();
    json!({"count":count,"p50_ms":percentile(50),"p95_ms":percentile(95),
        "p99_ms":percentile(99),"max_ms":durations.last().copied().unwrap_or(0) as f64 / 1_000_000.0,
        "over_16ms":durations.iter().filter(|&&ns| ns > 16_000_000).count(),"total_ms":total_ms})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wheel_control_validates_coordinates_units_and_gesture_phase() {
        let params = json!({"x":640,"y":350,"delta_x":-0.625,"delta_y":12.25});
        let wheel = WheelInjection::from_params(&params).unwrap();
        assert_eq!(wheel.delta_x, -0.625);
        assert_eq!(wheel.delta_y, 12.25);
        assert_eq!(wheel.phase, "changed");
        assert!(wheel.precise);
        for (field, invalid) in [
            ("x", json!(-1)),
            ("y", json!(1_000_001)),
            ("delta_x", json!("12")),
            ("delta_y", Value::Null),
            ("phase", json!("momentum")),
            ("precise", json!(1)),
        ] {
            let mut bad = params.clone();
            bad[field] = invalid;
            assert!(WheelInjection::from_params(&bad).is_err(), "{field}");
        }
        let mut boundary = params;
        boundary["phase"] = json!("ended");
        boundary["precise"] = json!(false);
        assert!(!WheelInjection::from_params(&boundary).unwrap().precise);
    }
    #[test]
    fn percentiles_use_nearest_rank_and_strict_frame_threshold() {
        let result = duration_statistics(vec![1_000_000, 16_000_000, 17_000_000, 100_000_000]);
        assert_eq!(result["count"], 4);
        assert_eq!(result["p50_ms"], 16.0);
        assert_eq!(result["p95_ms"], 100.0);
        assert_eq!(result["p99_ms"], 100.0);
        assert_eq!(result["over_16ms"], 2);
        assert_eq!(result["total_ms"], 134.0);
        assert_eq!(duration_statistics(vec![])["max_ms"], 0.0);
    }

    #[test]
    fn resize_summary_counts_the_first_presented_frame_per_transaction() {
        let resize_span = |id, task_id, duration_ns, samples| {
            let mut fields = BTreeMap::new();
            fields.insert("samples".into(), samples);
            browser_tracing::RecordedSpan {
                id,
                parent_id: 0,
                thread_id: 1,
                category: "resize".into(),
                name: "ResizeToPresentationComplete".into(),
                start_ns: 0,
                duration_ns,
                incomplete: false,
                context: browser_tracing::Context {
                    task_id,
                    ..Default::default()
                },
                fields,
                text: None,
            }
        };
        let summary = analyze_snapshot(&Snapshot {
            version: 1,
            recording_id: 1,
            active: false,
            duration_ns: 100_000_000,
            dropped_events: 0,
            spans: vec![
                resize_span(1, 10, 12_000_000, 3.0),
                resize_span(2, 10, 25_000_000, 3.0),
                resize_span(3, 20, 20_000_000, 1.0),
            ],
            instants: vec![],
        });
        assert_eq!(summary["resize"]["count"], 2);
        assert_eq!(summary["resize"]["samples"], 4);
        assert_eq!(summary["resize"]["coalesced_samples"], 2);
        assert_eq!(summary["resize"]["p50_ms"], 12.0);
        assert_eq!(summary["resize"]["p95_ms"], 20.0);
    }

    #[test]
    fn category_totals_use_exclusive_nested_time() {
        let span =
            |id, parent_id, category: &str, start_ns, duration_ns| browser_tracing::RecordedSpan {
                id,
                parent_id,
                thread_id: 1,
                category: category.into(),
                name: format!("{category}.{id}"),
                start_ns,
                duration_ns,
                incomplete: false,
                context: Default::default(),
                fields: Default::default(),
                text: None,
            };
        let summary = analyze_snapshot(&Snapshot {
            version: 1,
            recording_id: 1,
            active: false,
            duration_ns: 100_000_000,
            dropped_events: 0,
            spans: vec![
                span(1, 0, "style", 0, 100_000_000),
                span(2, 1, "style_resolve", 20_000_000, 60_000_000),
                span(3, 2, "style_match", 30_000_000, 20_000_000),
            ],
            instants: vec![],
        });
        let categories = summary["categories"].as_array().unwrap();
        let total = |category: &str| {
            categories
                .iter()
                .find(|value| value["category"] == category)
                .unwrap()["total_ms"]
                .as_f64()
                .unwrap()
        };
        assert_eq!(total("style"), 40.0);
        assert_eq!(total("style_resolve"), 40.0);
        assert_eq!(total("style_match"), 20.0);
        assert_eq!(summary["category_duration_kind"], "exclusive");
    }

    #[test]
    fn layout_package_prefix_owns_stage_and_self_time_statistics() {
        assert_eq!(
            performance_owner("layout_algo_grid.Layout", "layout"),
            "layout_algo_grid"
        );
        assert_eq!(performance_owner("LayoutEngine.Layout", "layout"), "layout");
        let summary = analyze_snapshot(&Snapshot {
            version: 1,
            recording_id: 1,
            active: false,
            duration_ns: 5_000_000,
            dropped_events: 0,
            spans: vec![browser_tracing::RecordedSpan {
                id: 1,
                parent_id: 0,
                thread_id: 1,
                category: "layout".into(),
                name: "layout_reuse.PassSummary".into(),
                start_ns: 0,
                duration_ns: 5_000_000,
                incomplete: false,
                context: Default::default(),
                fields: Default::default(),
                text: None,
            }],
            instants: vec![],
        });
        assert_eq!(summary["stages"][0]["owner"], "layout_reuse");
        assert_eq!(summary["owners"][0]["owner"], "layout_reuse");
        assert_eq!(summary["owners"][0]["total_ms"], 5.0);
        assert_eq!(summary["owner_duration_kind"], "exclusive");
    }
    #[test]
    fn http_request_limits_and_partial_reads() {
        let mut request =
            &b"POST /api HTTP/1.1\r\nHost: localhost:9000\r\nContent-Length: 2\r\n\r\n{}"[..];
        assert_eq!(read_request(&mut request).unwrap().body, b"{}");
        let mut oversized = &b"POST /api HTTP/1.1\r\nContent-Length: 65537\r\n\r\n"[..];
        assert!(read_request(&mut oversized).is_err());
        let mut ambiguous =
            &b"POST /api HTTP/1.1\r\nContent-Length: 2\r\ncontent-length: 3\r\n\r\n{}"[..];
        assert!(read_request(&mut ambiguous).is_err());
        let mut chunked = &b"POST /api HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n"[..];
        assert!(read_request(&mut chunked).is_err());
        let mut missing_length = &b"POST /api HTTP/1.1\r\nHost: localhost:9000\r\n\r\n"[..];
        assert!(read_request(&mut missing_length).is_err());
    }
    #[test]
    fn local_control_and_recording_ids_are_exact() {
        assert!(local_host("127.0.0.1:9000", 9000));
        assert!(!local_host("localhost.evil:9000", 9000));
        assert!(!local_host("localhost:9001", 9000));
        assert_eq!(recording_id(&json!({"recording_id":7})).unwrap(), Some(7));
        assert!(recording_id(&json!({"recording_id":-1})).is_err());
        assert!(dispatch(json!({"method":"Missing"})).is_err());
        let snapshot = Snapshot {
            version: 1,
            recording_id: 7,
            active: false,
            duration_ns: 100,
            dropped_events: 0,
            spans: vec![],
            instants: vec![],
        };
        let summary = json!({"duration_ms": 0.0001, "nested": {"count": 0}});
        let mut expected = serde_json::to_value(&snapshot).unwrap();
        expected["summary"] = summary.clone();
        let response = SuccessResponse {
            result: ApiResult::Performance(PerformanceSnapshot { snapshot, summary }),
        };
        let wire: Value = serde_json::from_slice(&serde_json::to_vec(&response).unwrap()).unwrap();
        assert_eq!(wire, json!({"result": expected}));
        let key = KeyInjection::from_params(&json!({"type":"rawKeyDown", "key":"Enter"})).unwrap();
        assert_eq!(key.key, "Enter");
        assert_eq!(key.modifiers, 0);
        assert!(KeyInjection::from_params(&json!({"type":"char", "key":"a"})).is_err());
        assert!(
            KeyInjection::from_params(&json!({"type":"keyDown", "key":"a", "modifiers":16}))
                .is_err()
        );
        assert_eq!(
            MouseInjection::from_params(
                &json!({"type":"mousePressed", "x":200, "y":60, "target_id":2})
            )
            .unwrap()
            .target_id,
            2
        );
        assert!(MouseInjection::from_params(
            &json!({"type":"mouseMoved", "x":0, "y":0, "target_id":3})
        )
        .is_err());
    }
    #[test]
    fn cached_frames_require_proven_scroll_and_exclude_actual_layout_paint() {
        fn event(
            category: &str,
            name: &str,
            frame: u64,
            source: u64,
            fields: BTreeMap<String, f64>,
        ) -> browser_tracing::RecordedSpan {
            browser_tracing::RecordedSpan {
                text: None,
                id: frame,
                parent_id: 0,
                thread_id: 1,
                category: category.into(),
                name: name.into(),
                start_ns: 0,
                duration_ns: 1_000_000,
                incomplete: false,
                context: browser_tracing::Context {
                    target_id: 1,
                    frame_id: frame,
                    source_id: source,
                    input_id: 5,
                    ..Default::default()
                },
                fields,
            }
        }
        let cache =
            || BTreeMap::from([("raster_tasks".into(), 0.0), ("reused_tiles".into(), 12.0)]);
        let scroll = || BTreeMap::from([("scroll_records_reused".into(), 1.0)]);
        let snapshot = Snapshot {
            version: 1,
            recording_id: 1,
            active: false,
            duration_ns: 5_000_000,
            dropped_events: 0,
            instants: vec![],
            spans: vec![
                event("render", "LayerTilePaint", 1, 10, cache()),
                event("lifecycle", "Page.UpdateFrameIfNeeded", 1, 10, scroll()),
                event(
                    "layout",
                    "LayoutEngine.UpdateScrollLayout",
                    1,
                    10,
                    BTreeMap::new(),
                ),
                event(
                    "paint",
                    "PaintEngine.TryUpdateScrollProperties",
                    1,
                    10,
                    BTreeMap::from([("records_reused".into(), 1.0)]),
                ),
                event(
                    "layout",
                    "LayoutEngine.NativeLayout",
                    1,
                    11,
                    BTreeMap::new(),
                ),
                event("render", "LayerTilePaint", 2, 10, cache()),
                event("lifecycle", "Page.UpdateFrameIfNeeded", 2, 10, scroll()),
                event("paint", "Paint.RecordPaint", 2, 10, BTreeMap::new()),
                event(
                    "render",
                    "LayerTilePaint",
                    3,
                    10,
                    BTreeMap::from([("raster_tasks".into(), 0.0), ("reused_tiles".into(), 0.0)]),
                ),
                event("render", "LayerTilePaint", 4, 10, cache()), // Mouse input alone proves no scroll.
                event("render", "LayerTilePaint", 5, 10, cache()),
                event("lifecycle", "Page.UpdateFrameIfNeeded", 5, 10, scroll()),
                event(
                    "layout",
                    "LayoutEngine.NativeLayout",
                    5,
                    10,
                    BTreeMap::new(),
                ),
            ],
        };
        let summary = analyze_snapshot(&snapshot);
        assert_eq!(summary["cache_hit_frames"], 4);
        assert_eq!(summary["pure_cached_scroll_frames"], 1);
    }
}
