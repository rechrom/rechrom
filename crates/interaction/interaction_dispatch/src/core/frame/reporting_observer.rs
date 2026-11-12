//! Chromium `third_party/blink/renderer/core/frame/reporting_observer.h` and
//! `reporting_observer.cc`: retained registration, report selection and delivery.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{report::Report, reporting_context::ReportingContext};

pub trait ReportingObserver {
    fn Buffered(&self) -> bool;
    fn ClearBuffered(&self);
    fn QueueReport(&self, report: Rc<Report>);
}

/// Source reporting_observer.cc:27-91. The script binding supplies its original
/// callback and MiscPlatformAPI task runner, never a report-selection algorithm.
pub trait ReportingObserverCallback {
    fn InvokeAndReportException(
        &self,
        reports: Vec<Rc<Report>>,
        observer: Rc<ReportingObserverOwner>,
    );
}
pub trait ReportingObserverTaskRunner {
    fn PostMiscPlatformTask(&self, task: Box<dyn FnOnce()>);
}
pub struct ReportingObserverOwner {
    self_: std::rc::Weak<Self>,
    context: std::rc::Weak<ReportingContext>,
    callback: Rc<dyn ReportingObserverCallback>,
    tasks: Rc<dyn ReportingObserverTaskRunner>,
    types: Option<Vec<String>>,
    buffered: Cell<bool>,
    registered: Cell<bool>,
    queue: RefCell<Vec<Rc<Report>>>,
}
impl ReportingObserverOwner {
    pub fn new(
        context: &Rc<ReportingContext>,
        callback: Rc<dyn ReportingObserverCallback>,
        tasks: Rc<dyn ReportingObserverTaskRunner>,
        types: Option<Vec<String>>,
        buffered: bool,
    ) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            context: Rc::downgrade(context),
            callback,
            tasks,
            types,
            buffered: Cell::new(buffered),
            registered: Cell::new(false),
            queue: RefCell::new(Vec::new()),
        })
    }
    pub fn HasPendingActivity(&self) -> bool {
        self.registered.get()
    }
    pub fn Observe(self: &Rc<Self>) {
        self.registered.set(true);
        if let Some(context) = self.context.upgrade() {
            context.RegisterObserver(self.clone());
        }
    }
    pub fn Disconnect(self: &Rc<Self>) {
        self.registered.set(false);
        if let Some(context) = self.context.upgrade() {
            let observer: Rc<dyn ReportingObserver> = self.clone();
            context.UnregisterObserver(&observer);
        }
    }
    pub fn TakeRecords(&self) -> Vec<Rc<Report>> {
        std::mem::take(&mut *self.queue.borrow_mut())
    }
    pub fn ReportToCallback(self: &Rc<Self>) {
        let reports = self.TakeRecords();
        self.callback
            .InvokeAndReportException(reports, self.clone());
    }
}
impl ReportingObserver for ReportingObserverOwner {
    fn Buffered(&self) -> bool {
        self.buffered.get()
    }
    fn ClearBuffered(&self) {
        self.buffered.set(false);
    }
    fn QueueReport(&self, report: Rc<Report>) {
        if self.types.as_ref().is_some_and(|types| {
            !types.is_empty() && !types.iter().any(|kind| kind == report.Type())
        }) {
            return;
        }
        let first = {
            let mut queue = self.queue.borrow_mut();
            queue.push(report);
            queue.len() == 1
        };
        if first {
            let owner = self.self_.clone();
            self.tasks.PostMiscPlatformTask(Box::new(move || {
                if let Some(owner) = owner.upgrade() {
                    owner.ReportToCallback();
                }
            }));
        }
    }
}
