//! Chromium `third_party/blink/renderer/core/frame/report_body.h`: report body
//! variants retain their source-owned payloads.

use super::{
    csp::csp_violation_report_body::CSPViolationReportBody,
    intervention_report_body::InterventionReportBody,
};

#[derive(Clone, Debug)]
pub enum ReportBody {
    Intervention(InterventionReportBody),
    CSPViolation(CSPViolationReportBody),
}

impl ReportBody {
    // ReportBody::IsExtensionSource virtual dispatch, with the inherited
    // LocationReportBody query reading each payload's original location.
    pub(super) fn IsExtensionSource(&self) -> bool {
        match self {
            Self::Intervention(body) => body.location.IsExtensionSource(),
            Self::CSPViolation(body) => {
                super::location_report_body::IsExtensionSource(body.data.sourceFile())
            }
        }
    }
}
