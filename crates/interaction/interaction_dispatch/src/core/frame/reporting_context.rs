//! Chromium `third_party/blink/renderer/core/frame/reporting_context.h` and
//! `reporting_context.cc`: retained ExecutionContext reports and observer buffers.
//! Report identity is preserved for buffers and observers; UseCounter counts
//! each feature once. Service requests remain queued for the browser-side service.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

// Preserve the established public paths while each source module owns its types.
pub use super::{
    csp::csp_violation_report_body::CSPViolationReportBody,
    intervention_report_body::InterventionReportBody,
    location_report_body::ReportLocation,
    report::Report,
    report_body::ReportBody,
    reporting_observer::{
        ReportingObserver, ReportingObserverCallback, ReportingObserverOwner,
        ReportingObserverTaskRunner,
    },
};
pub use crate::core::inspector::{
    console_message::{ConsoleMessage, ConsoleMessageLevel, ConsoleMessageSource},
    console_message_storage::ConsoleMessageStorage,
};

/// The original ReportingServiceProxy request, retained until its browser-side
/// Reporting API service takes it. It is not an invented successful delivery.
#[derive(Clone, Debug)]
pub struct InterventionServiceReport {
    pub report: Rc<Report>,
}
#[derive(Clone, Debug)]
pub struct CSPServiceReport {
    pub report: Rc<Report>,
    pub endpoint_group: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReportingFeature {
    InterventionReport,
    ReportingObserver,
}
#[derive(Default)]
pub struct ReportingContext {
    observers: RefCell<Vec<Rc<dyn ReportingObserver>>>,
    report_buffer: RefCell<HashMap<&'static str, VecDeque<Rc<Report>>>>,
    csp_service_reports: RefCell<VecDeque<CSPServiceReport>>,
    counted_features: RefCell<HashSet<ReportingFeature>>,
    service_reports: RefCell<VecDeque<InterventionServiceReport>>,
}
impl ReportingContext {
    pub fn CountFeature(&self, feature: ReportingFeature) -> bool {
        self.counted_features.borrow_mut().insert(feature)
    }
    pub fn FeatureCounted(&self, feature: ReportingFeature) -> bool {
        self.counted_features.borrow().contains(&feature)
    }
    // reporting_context.cc:83-115. Eligibility remains Report-owned.
    pub fn QueueInterventionReport(&self, report: Rc<Report>) {
        self.QueueReport(report);
    }
    pub fn QueueReport(&self, report: Rc<Report>) {
        self.QueueReportWithEndpoints(report, vec!["default".into()]);
    }
    pub fn QueueReportWithEndpoints(&self, report: Rc<Report>, endpoints: Vec<String>) {
        if !report.ShouldSendReport() {
            return;
        }
        if matches!(&report.body, ReportBody::Intervention(_)) {
            self.CountFeature(ReportingFeature::InterventionReport);
        }
        self.NotifyInternal(report.clone());
        for endpoint in endpoints {
            match &report.body {
                ReportBody::Intervention(_) => {
                    self.service_reports
                        .borrow_mut()
                        .push_back(InterventionServiceReport {
                            report: report.clone(),
                        })
                }
                ReportBody::CSPViolation(_) => {
                    self.csp_service_reports
                        .borrow_mut()
                        .push_back(CSPServiceReport {
                            report: report.clone(),
                            endpoint_group: endpoint,
                        })
                }
            }
        }
    }
    fn NotifyInternal(&self, report: Rc<Report>) {
        let mut buffers = self.report_buffer.borrow_mut();
        let buffer = buffers.entry(report.Type()).or_default();
        if !buffer.iter().any(|old| Rc::ptr_eq(old, &report)) {
            buffer.push_back(report.clone());
        }
        if buffer.len() > 100 {
            buffer.pop_front();
        }
        drop(buffers);
        let observers = self.observers.borrow().clone();
        for observer in observers {
            observer.QueueReport(report.clone());
        }
    }
    pub fn RegisterObserver(&self, observer: Rc<dyn ReportingObserver>) {
        self.CountFeature(ReportingFeature::ReportingObserver);
        let mut observers = self.observers.borrow_mut();
        if !observers.iter().any(|old| Rc::ptr_eq(old, &observer)) {
            observers.push(observer.clone());
        }
        drop(observers);
        if !observer.Buffered() {
            return;
        }
        observer.ClearBuffered();
        let reports: Vec<_> = self
            .report_buffer
            .borrow()
            .values()
            .flat_map(|buffer| buffer.iter().cloned())
            .collect();
        for report in reports {
            observer.QueueReport(report);
        }
    }
    pub fn UnregisterObserver(&self, observer: &Rc<dyn ReportingObserver>) {
        self.observers
            .borrow_mut()
            .retain(|old| !Rc::ptr_eq(old, observer));
    }
    pub fn BufferedInterventions(&self) -> Vec<Rc<Report>> {
        self.BufferedReports("intervention")
    }
    pub fn BufferedReports(&self, kind: &str) -> Vec<Rc<Report>> {
        self.report_buffer
            .borrow()
            .get(kind)
            .map_or_else(Vec::new, |buffer| buffer.iter().cloned().collect())
    }
    pub fn TakeCSPServiceReports(&self) -> Vec<CSPServiceReport> {
        self.csp_service_reports.borrow_mut().drain(..).collect()
    }
    pub fn TakeServiceReports(&self) -> Vec<InterventionServiceReport> {
        self.service_reports.borrow_mut().drain(..).collect()
    }
}
