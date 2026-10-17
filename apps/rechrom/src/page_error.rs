//! Observable Page failure values. The host decides where to deliver/log them;
//! these values neither own engines nor terminate the window event loop.
use std::{any::Any, fmt, io};

#[derive(Clone, Debug)]
pub struct PageError {
    pub document: (u64, u64),
    pub url: String,
    pub stage: &'static str,
    pub kind: &'static str,
    pub terminal: bool,
    pub message: String,
    pub resource_url: Option<String>,
    pub stack: Option<String>,
}
impl PageError {
    pub(crate) fn emit_stderr(&self) {
        eprintln!("rechrom-page-error\t{}", self.json());
    }
    pub(crate) fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": 1,
            "event": "PageError",
            "timestamp_unix_ms": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(),
            "pid": std::process::id(),
            "view_id": self.document.0,
            "document_generation": self.document.1,
            "url": self.url,
            "stage": self.stage,
            "kind": self.kind,
            "terminal": self.terminal,
            "message": self.message,
            "resource_url": self.resource_url,
            "stack": self.stack,
        })
    }
    pub(crate) fn unavailable(&self) -> String {
        format!(
            "Page {}/{} stopped ({}): {}",
            self.document.0, self.document.1, self.stage, self.message
        )
    }
}
#[derive(Debug)]
struct PagePanic(String);
impl fmt::Display for PagePanic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for PagePanic {}

pub(crate) fn panic_error(payload: Box<dyn Any + Send>) -> io::Error {
    // Translated engines use typed panic_any boundaries as well as strings.
    // Preserve the actual capability/DOM error instead of "Box<dyn Any>".
    let message = if let Some(error) = payload.downcast_ref::<foundation::UnsupportedLayout>() {
        format!("unsupported layout: {error}")
    } else if let Some(error) = payload.downcast_ref::<dom::error::DOMException>() {
        format!("{:?}: {}", error.kind, error.message)
    } else if let Some(error) = payload.downcast_ref::<String>() {
        error.clone()
    } else if let Some(error) = payload.downcast_ref::<&'static str>() {
        (*error).to_owned()
    } else {
        "panic with an unknown payload".into()
    };
    io::Error::other(PagePanic(message))
}
pub(crate) fn error_kind(error: &io::Error) -> &'static str {
    if let Some(error) = error
        .get_ref()
        .and_then(|e| e.downcast_ref::<RasterFailure>())
    {
        return error_kind(&error.0);
    }
    if error.get_ref().is_some_and(|e| e.is::<PagePanic>()) {
        "panic"
    } else {
        "error"
    }
}

// Preserve the raster origin when asynchronous CPU tile completion is polled
// during compositor Prepare. The owning engine still returns its original IO
// error; only this embedding adds executor context for the host notification.
#[derive(Debug)]
pub(crate) struct RasterFailure(io::Error);
impl fmt::Display for RasterFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for RasterFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
pub(crate) fn raster_error(error: io::Error) -> io::Error {
    io::Error::new(error.kind(), RasterFailure(error))
}
pub(crate) fn is_raster_error(error: &io::Error) -> bool {
    error.get_ref().is_some_and(|e| e.is::<RasterFailure>())
}
