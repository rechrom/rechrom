//! Local Performance UI and a small JSON control endpoint for browser tracing.
use base64::Engine;
use browser_tracing::{Snapshot, StartOptions};
use serde_json::{json, Value};
use std::collections::BTreeMap;
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
        Ok(Self {
            x: number("x", 0.0)?,
            y: number("y", 0.0)?,
            delta_x: number("delta_x", -1_000_000.0)?,
            delta_y: number("delta_y", -1_000_000.0)?,
            phase: phase.into(),
            precise,
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
    let response = match serde_json::from_slice::<Value>(&request.body) {
        Ok(value) => match dispatch(value) {
            Ok(result) => json!({"result":result}),
            Err(error) => json!({"error":error}),
        },
        Err(error) => json!({"error":format!("Invalid JSON: {error}")}),
    };
    let _ = respond_json(stream, "200 OK", response);
}

fn respond_json(stream: &mut TcpStream, status: &str, value: Value) -> io::Result<()> {
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

fn dispatch(request: Value) -> Result<Value, String> {
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .ok_or("method is required")?;
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
    if !params.is_object() {
        return Err("params must be an object".into());
    }
    match method {
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
        "Page.reload" => {
            let reload = reload_handler()
                .lock()
                .unwrap()
                .clone()
                .ok_or("No browser target")?;
            reload()?;
            Ok(json!({}))
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
        "Tracing.start" => {
            let options: StartOptions =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            Ok(json!({"recording_id":browser_tracing::start(options)?}))
        }
        "Tracing.end" => Ok(json!({"recording_id":browser_tracing::stop()?})),
        "Performance.status" => Ok(browser_tracing::status()),
        "Performance.snapshot" => {
            let collect_started = std::time::Instant::now();
            let snapshot = browser_tracing::snapshot(recording_id(&params)?)?;
            let collection_ms = collect_started.elapsed().as_secs_f64() * 1000.;
            let analysis_started = std::time::Instant::now();
            let mut summary = analyze_snapshot(&snapshot);
            let analysis_ms = analysis_started.elapsed().as_secs_f64() * 1000.;
            summary["measurement_overhead"] = json!({"collection_ms":collection_ms,"analysis_ms":analysis_ms,"producer_overhead":"not calibrated","serialization_ms":null});
            let mut value = serde_json::to_value(snapshot).map_err(|e| e.to_string())?;
            value["summary"] = summary;
            Ok(value)
        }
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
    }
}

/// Stage durations are inclusive; nested categories must not be added together.
/// Active/incomplete spans are reported separately and excluded from percentiles.
pub fn analyze_snapshot(snapshot: &Snapshot) -> Value {
    let mut groups: BTreeMap<(String, String), Vec<u64>> = BTreeMap::new();
    let mut incomplete = 0usize;
    let mut frames: BTreeMap<(u64, u64, u64), (bool, bool, bool)> = BTreeMap::new();
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
                    | "LayoutEngine.LayoutWithoutExport"
                    | "LayoutEngine.ExportFragments"
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
        groups
            .entry((span.category.to_string(), span.name.to_string()))
            .or_default()
            .push(span.duration_ns);
    }
    let mut stages: Vec<Value> = groups
        .into_iter()
        .map(|((category, name), durations)| {
            let mut stage = duration_statistics(durations);
            stage["category"] = json!(category);
            stage["name"] = json!(name);
            stage
        })
        .collect();
    stages.sort_by(|a, b| {
        b["total_ms"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["total_ms"].as_f64().unwrap_or(0.0))
    });
    json!({"recording_id":snapshot.recording_id,"active":snapshot.active,
        "duration_ms":snapshot.duration_ns as f64 / 1_000_000.0,
        "dropped_events":snapshot.dropped_events,"incomplete_spans":incomplete,
        "cache_hit_frames":frames.values().filter(|frame| frame.0).count(),
        "pure_cached_scroll_frames":frames.values().filter(|frame| frame.0 && !frame.1 && frame.2).count(),
        "duration_kind":"inclusive","percentile_method":"nearest_rank","stages":stages,
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
