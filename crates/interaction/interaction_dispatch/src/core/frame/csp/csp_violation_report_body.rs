//! Chromium
//! `third_party/blink/renderer/core/frame/csp/csp_violation_report_body.h`:
//! retained CSP violation payload.

#[derive(Clone, Debug)]
pub struct CSPViolationReportBody {
    pub data:
        crate::core::events::security_policy_violation_event::SecurityPolicyViolationEventData,
}
