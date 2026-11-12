//! Chromium `third_party/blink/renderer/core/frame/intervention_report_body.h`:
//! intervention payload and inherited location data.

use super::location_report_body::ReportLocation;

#[derive(Clone, Debug)]
pub struct InterventionReportBody {
    pub id: String,
    pub message: String,
    pub location: ReportLocation,
}
