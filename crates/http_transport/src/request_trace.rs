//! Diagnostic transport events. No context guard crosses an async suspension:
//! Tokio can resume a request on another thread, so context is scoped only
//! around each synchronous event emission. The recorder owns the event bound.
use crate::http_request::{HTTPRequestData, HTTPRequestPriority, TransferError};
use std::time::Instant;

#[derive(Clone, Copy, Default)]
pub(crate) struct RequestTrace {
    id: u64,
    recording_id: u64,
    context: browser_tracing::Context,
    queued: Option<Instant>,
}

impl RequestTrace {
    pub(crate) fn queued(request: &HTTPRequestData, streamed: bool) -> Self {
        let recording_id = browser_tracing::active_recording_id();
        if recording_id == 0 {
            return Self::default();
        }
        let trace = Self {
            id: browser_tracing::next_id(),
            recording_id,
            context: browser_tracing::context(),
            queued: Some(Instant::now()),
        };
        trace.url_event(
            "HTTPRequestQueued",
            &request.url,
            &[
                ("streamed", u8::from(streamed) as f64),
                (
                    "priority",
                    u8::from(request.priority == HTTPRequestPriority::High) as f64,
                ),
            ],
        );
        trace
    }

    pub(crate) fn admitted(self) {
        self.event(
            "HTTPRequestAdmitted",
            &[(
                "queue_ms",
                self.queued
                    .map_or(0.0, |at| at.elapsed().as_secs_f64() * 1000.0),
            )],
        );
    }

    pub(crate) fn failed(self, attempt: usize, error: &TransferError, retry: bool) {
        self.event(
            "HTTPRequestFailed",
            &[
                ("attempt", attempt as f64),
                ("retryable", u8::from(error.retryable) as f64),
                ("will_retry", u8::from(retry) as f64),
            ],
        );
    }

    pub(crate) fn event(self, name: &'static str, fields: &[(&'static str, f64)]) {
        self.emit(name, None, fields);
    }

    fn url_event(self, name: &'static str, url: &str, fields: &[(&'static str, f64)]) {
        if !self.active() {
            return;
        }
        // Trace the resource identity, never credentials embedded in URL userinfo.
        let text = match url::Url::parse(url) {
            Ok(mut url) => {
                let _ = url.set_username("");
                let _ = url.set_password(None);
                url.set_fragment(None);
                url.to_string()
            }
            Err(_) => "(invalid URL)".to_owned(),
        };
        self.emit(name, Some(&text), fields);
    }

    fn emit(self, name: &'static str, text: Option<&str>, fields: &[(&'static str, f64)]) {
        if !self.active() {
            return;
        }
        debug_assert!(fields.len() < 8, "request_id occupies one trace field");
        let mut values = [("request_id", self.id as f64); 8];
        let count = fields.len().min(7);
        values[1..1 + count].copy_from_slice(&fields[..count]);
        let _scope = browser_tracing::scope(self.context);
        match text {
            Some(text) => browser_tracing::instant_with_text("http", name, text, &values[..=count]),
            None => browser_tracing::instant("http", name, &values[..=count]),
        }
    }

    pub(crate) fn guard(self) -> RequestTraceGuard {
        RequestTraceGuard(Some(self))
    }

    fn active(self) -> bool {
        self.recording_id != 0 && self.recording_id == browser_tracing::active_recording_id()
    }
}

/// Records cancellation when a pending task is aborted or its future is dropped.
/// Successful and failed terminal paths disarm it after publishing their result.
pub(crate) struct RequestTraceGuard(Option<RequestTrace>);
impl RequestTraceGuard {
    pub(crate) fn finish(&mut self) {
        self.0 = None;
    }
}
impl Drop for RequestTraceGuard {
    fn drop(&mut self) {
        if let Some(trace) = self.0 {
            trace.event("HTTPRequestCancelled", &[]);
        }
    }
}
