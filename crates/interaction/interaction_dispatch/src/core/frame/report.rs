//! Chromium `third_party/blink/renderer/core/frame/report.h` and `report.cc`:
//! report identity, type and delivery eligibility.

use super::report_body::ReportBody;

#[derive(Debug)]
pub struct Report {
    pub url: String,
    pub body: ReportBody,
}
impl Report {
    pub fn ShouldSendReport(&self) -> bool {
        !self.body.IsExtensionSource()
    }
    pub fn Type(&self) -> &'static str {
        match &self.body {
            ReportBody::Intervention(_) => "intervention",
            ReportBody::CSPViolation(_) => "csp-violation",
        }
    }
}
