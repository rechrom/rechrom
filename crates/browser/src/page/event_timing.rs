//! LocalDOMWindow::performance and FrameWidget presentation-promise assembly.
//! Source WindowPerformance owns entries/responsiveness; the widget owns only
//! its original weak callbacks, including callbacks from retired Documents.
use super::*;
use interaction::{
    default::core::timing::{
        widget_presentation_timing::WidgetPresentationTiming,
        window_performance::{
            EventTimingEnvironment, PresentationTimeRequest, PresentationTimingDetails,
            WindowPerformance,
        },
    },
    input::base::time::time::TimeTicks,
};

pub(super) struct PageEventTimingWidget {
    timing: RefCell<WidgetPresentationTiming>,
    requests: RefCell<Vec<u64>>,
}
impl Default for PageEventTimingWidget {
    fn default() -> Self {
        Self {
            timing: RefCell::new(WidgetPresentationTiming::new()),
            requests: Default::default(),
        }
    }
}
struct WindowEventTimingEnvironment {
    page: Weak<PageState>,
    document: Weak<DocumentEngine>,
    identity: dom::persistent_document::DOMOwnerHandle,
    frame: u64,
    clock: Arc<PageCompositorExecutor>,
    widget: Weak<PageEventTimingWidget>,
}
impl WindowEventTimingEnvironment {
    fn Document(&self) -> Option<Rc<DocumentEngine>> {
        self.document
            .upgrade()
            .filter(|document| document.borrow().GetDocument().RootHandle() == self.identity)
    }
    fn ResourceWallTime(&self, time: std::time::Instant) -> Option<i64> {
        let document = self.Document()?;
        let owner = document.borrow();
        let timing = owner.GetDocument().DocumentLoaderLoadTiming()?;
        timing.ReferenceMonotonicTime()?;
        Some(timing.MonotonicTimeToPseudoWallMicroseconds(Some(time)))
    }
}
impl EventTimingEnvironment for WindowEventTimingEnvironment {
    fn MonotonicTimeToPseudoWallMicroseconds(&self, time: TimeTicks) -> Option<i64> {
        if time.is_null() {
            return Some(0);
        }
        // Convert through the same injected Page clock, never a new epoch.
        let delta = time.ToInternalValue().checked_sub(60 * 60 * 1_000_000)?;
        let instant = if delta >= 0 {
            self.clock
                .origin
                .checked_add(std::time::Duration::from_micros(delta as u64))?
        } else {
            self.clock
                .origin
                .checked_sub(std::time::Duration::from_micros(delta.unsigned_abs()))?
        };
        self.ResourceWallTime(instant)
    }
    fn ResourceTimeToPseudoWallMicroseconds(&self, time: std::time::Instant) -> Option<i64> {
        self.ResourceWallTime(time)
    }
    fn HasFrame(&self) -> bool {
        let (Some(page), Some(document)) = (self.page.upgrade(), self.Document()) else {
            return false;
        };
        let Some(domain) = page.local_frames.get() else {
            return false;
        };
        domain
            .Document(self.frame)
            .is_some_and(|current| Rc::ptr_eq(&current, &document))
            && document.borrow().GetDocument().IsActive()
    }
    fn IsContextDestroyed(&self) -> bool {
        self.Document()
            .is_none_or(|document| document.borrow().GetDocument().IsContextDestroyed())
    }
    fn IsContextPaused(&self) -> bool {
        self.Document()
            .is_some_and(|document| document.borrow().GetDocument().IsContextPaused())
    }
    fn NotifyPresentationTime(
        &self,
        performance: &Rc<WindowPerformance>,
        request: PresentationTimeRequest,
    ) {
        if let Some(widget) = self.widget.upgrade() {
            let id = widget.timing.borrow_mut().Register(performance, request);
            widget.requests.borrow_mut().push(id);
        }
    }
}
impl PageState {
    pub(super) fn BindWindowEventTiming(
        self: &Rc<Self>,
        frame: u64,
        document: &Rc<DocumentEngine>,
        timing: &webapi::window_bindings::WindowPerformanceTiming,
    ) {
        let environment = Rc::new(WindowEventTimingEnvironment {
            page: Rc::downgrade(self),
            document: Rc::downgrade(document),
            frame,
            identity: document.borrow().GetDocument().RootHandle(),
            clock: self.rendering.borrow().executor.clone(),
            widget: Rc::downgrade(&self.event_timing_widget),
        });
        if let Some(performance) = timing.ExistingEventTimingSource() {
            performance.RebindEnvironmentForInitialWindowReuse(environment);
        } else {
            use interaction::adapter::host_event_clock::HostInputClock;
            let origin = self
                .rendering
                .borrow()
                .executor
                .TimeForInstant(timing.MonotonicOrigin());
            timing.BindEventTimingEnvironment(
                self.InteractionEventClock(),
                environment,
                origin,
                document
                    .borrow()
                    .GetDocument()
                    .ExecutionContextIsolation()
                    .CrossOriginIsolatedCapability(),
            );
        }
    }
    pub(super) fn ForEachWindowPerformance(
        &self,
        mut callback: impl FnMut(&Rc<WindowPerformance>),
    ) {
        if let Some(performance) = self.window_performance_timing.ExistingEventTimingSource() {
            callback(&performance);
        }
        if let Some(domain) = self.local_frames.get() {
            for frame in domain.frames.borrow().values() {
                if let Some(performance) = frame
                    .window_performance_timing
                    .borrow()
                    .ExistingEventTimingSource()
                {
                    callback(&performance);
                }
            }
        }
    }
}
impl Page {
    pub fn EventTimingDidCommit(&self, time: std::time::Instant) {
        let time = self.state.rendering.borrow().InputTimeForInstant(time);
        self.state
            .window_performance_timing
            .EventTimingSource()
            .SetCommitFinishTimeStampForPendingEvents(time);
    }
    /// Move only promise identities across the document/impl sequence boundary.
    pub fn TakeEventTimingPresentationRequests(&self) -> Vec<u64> {
        std::mem::take(&mut *self.state.event_timing_widget.requests.borrow_mut())
    }
    pub fn EventTimingDidSwap(
        &self,
        requests: Vec<u64>,
        frame_token: u32,
        swap_time: std::time::Instant,
    ) {
        let time = self.state.rendering.borrow().InputTimeForInstant(swap_time);
        let mut widget = self.state.event_timing_widget.timing.borrow_mut();
        for request in requests {
            widget.DidSwap(request, frame_token, time);
        }
    }
    pub fn EventTimingDidNotSwap(&self, requests: Vec<u64>, time: std::time::Instant) {
        let time = self.state.rendering.borrow().InputTimeForInstant(time);
        let callbacks = {
            let mut widget = self.state.event_timing_widget.timing.borrow_mut();
            requests
                .into_iter()
                .filter_map(|request| widget.TakeDidNotSwapCallback(request, time))
                .collect::<Vec<_>>()
        };
        for callback in callbacks {
            callback.Run();
        }
    }
    pub fn EventTimingDidPresent(&self, feedback: PageFramePresentationFeedback) {
        let PagePresentationOutcome::Presented { timestamp, .. } = feedback.outcome else {
            return;
        };
        let details = PresentationTimingDetails {
            timestamp: self.state.rendering.borrow().InputTimeForInstant(timestamp),
            actual_frame_source_id: feedback.frame_id.source_id,
            manual_begin_frame: false,
            actual_source_is_manual: false,
            ignore_unexpected_frame_source: false,
        };
        let callbacks = self
            .state
            .event_timing_widget
            .timing
            .borrow_mut()
            .TakePresentedCallbacks(feedback.frame_token, details);
        for callback in callbacks {
            callback.Run();
        }
    }
}
