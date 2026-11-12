//! Chromium `third_party/blink/renderer/core/inspector/console_message.h`:
//! retained console message and node identities. Source and level values project
//! `third_party/blink/public/mojom/devtools/console_message.mojom`.

use crate::core::frame::location_report_body::ReportLocation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleMessageSource {
    Rendering,
    Other,
    JavaScript,
    Intervention,
    Security,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleMessageLevel {
    Verbose,
    Warning,
    Error,
    Info,
}
#[derive(Clone, Debug)]
pub struct ConsoleMessage {
    pub source: ConsoleMessageSource,
    pub level: ConsoleMessageLevel,
    pub message: String,
    pub location: ReportLocation,
    /// ConsoleMessage::SetNodes, original node identities in this Page domain.
    pub nodes: Vec<u64>,
}
