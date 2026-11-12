//! Chromium 6c1d401f soft_navigation_context.h:39-240; .cc:31-356.
//! Contexts retain the original event, records and LCP calculator. URL changes,
//! paints and presentation feedback update this same interaction-owned object.
use super::{
    interaction_contentful_paint::InteractionContentfulPaint,
    largest_contentful_paint::LargestContentfulPaint,
    largest_contentful_paint_calculator::{
        LargestContentfulPaintCalculator, LargestContentfulPaintDetails, LcpCandidates,
        LcpDelegate, LcpEnvironment, LcpFeatures,
    },
    paint_timing_record::{DOMPaintTimingInfo, PaintTimingRecord},
    performance_event_timing::PerformanceEventTiming,
    performance_timeline_entry_id_generator::PerformanceTimelineEntryIdInfo as Id,
    soft_navigation_heuristics::SoftNavigationHeuristics,
};
use crate::core::navigation_api::navigation_api::NavigationType;
use foundation::AtomicString;
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};
static LAST_CONTEXT_ID: AtomicU64 = AtomicU64::new(0);
pub struct SoftNavigationContext {
    self_: Weak<Self>,
    context_id: u64,
    owner: RefCell<Option<Weak<SoftNavigationHeuristics>>>,
    navigation_id: Cell<Id>,
    was_emitted: Cell<bool>,
    first_input_or_scroll_time: Cell<TimeTicks>,
    url_change_time: Cell<TimeTicks>,
    slicing_time: Cell<TimeTicks>,
    initial_url: RefCell<String>,
    metrics_token: Cell<[u8; 16]>,
    navigation_type: Cell<NavigationType>,
    lcp: RefCell<Option<Rc<LargestContentfulPaintCalculator>>>,
    initial_event: RefCell<Option<Rc<PerformanceEventTiming>>>,
    first_image_or_text: RefCell<Option<Rc<PaintTimingRecord>>>,
    largest_icp: RefCell<Option<Rc<InteractionContentfulPaint>>>,
    current_lcp: RefCell<Option<Rc<LargestContentfulPaint>>>,
    modified_nodes: Cell<u64>,
    painted_area: Cell<u64>,
    last_modified_nodes: Cell<u64>,
    last_painted_area: Cell<u64>,
}
impl SoftNavigationContext {
    pub fn NextContextId() -> u64 {
        LAST_CONTEXT_ID.load(Ordering::Relaxed).wrapping_add(1)
    }
    pub fn new(
        owner: &Rc<SoftNavigationHeuristics>,
        entry: Rc<PerformanceEventTiming>,
        environment: Rc<dyn LcpEnvironment>,
        features: LcpFeatures,
    ) -> Rc<Self> {
        assert!(entry.IsInteraction());
        let context = Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            context_id: LAST_CONTEXT_ID
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1),
            owner: RefCell::new(Some(Rc::downgrade(owner))),
            navigation_id: Cell::new(Id::NONE),
            was_emitted: Cell::new(false),
            first_input_or_scroll_time: Cell::new(TimeTicks::new()),
            url_change_time: Cell::new(TimeTicks::new()),
            slicing_time: Cell::new(TimeTicks::new()),
            initial_url: RefCell::new(String::new()),
            metrics_token: Cell::new([0; 16]),
            navigation_type: Cell::new(NavigationType::Push),
            lcp: RefCell::new(None),
            initial_event: RefCell::new(Some(entry)),
            first_image_or_text: RefCell::new(None),
            largest_icp: RefCell::new(None),
            current_lcp: RefCell::new(None),
            modified_nodes: Cell::new(0),
            painted_area: Cell::new(0),
            last_modified_nodes: Cell::new(0),
            last_painted_area: Cell::new(0),
        });
        let delegate: Rc<dyn LcpDelegate> = context.clone();
        *context.lcp.borrow_mut() = Some(LargestContentfulPaintCalculator::new(
            owner.Performance(),
            Rc::downgrade(&delegate),
            environment,
            features,
        ));
        owner.ForEachInteractionEffectsMonitor(|monitor| monitor.OnSoftNavigationContextCreated());
        context
    }
    pub fn GetSoftNavigationHeuristics(&self) -> Option<Rc<SoftNavigationHeuristics>> {
        self.owner.borrow().as_ref()?.upgrade()
    }
    fn Owner(&self) -> Rc<SoftNavigationHeuristics> {
        self.GetSoftNavigationHeuristics()
            .expect("context belongs to its live Window")
    }
    pub fn ContextId(&self) -> u64 {
        self.context_id
    }
    pub fn IsMostRecentlyCreatedContext(&self) -> bool {
        self.context_id == LAST_CONTEXT_ID.load(Ordering::Relaxed)
    }
    pub fn HasNavigationId(&self) -> bool {
        self.navigation_id.get() != Id::NONE
    }
    pub fn NavigationId(&self) -> Id {
        self.navigation_id.get()
    }
    pub fn GetInteractionIdInfo(&self) -> Id {
        self.initial_event
            .borrow()
            .as_ref()
            .expect("live context event")
            .GetInteractionIdInfo()
            .expect("interaction ID")
    }
    pub fn InteractionId(&self) -> u64 {
        self.initial_event
            .borrow()
            .as_ref()
            .and_then(|entry| entry.GetInteractionIdInfo())
            .map_or(0, |id| id.web_exposed_id)
    }
    pub fn TimeOrigin(&self) -> TimeTicks {
        self.initial_event
            .borrow()
            .as_ref()
            .expect("live context event")
            .GetStartTime()
    }
    pub fn ProcessingEnd(&self) -> TimeTicks {
        self.initial_event
            .borrow()
            .as_ref()
            .expect("live context event")
            .reporting_info
            .borrow()
            .processing_end_time
    }
    pub fn SoftNavigationSlicingTime(&self) -> TimeTicks {
        self.slicing_time.get()
    }
    pub fn OnSoftNavigationCommit(&self, navigation_id: Id, slicing_time: TimeTicks) {
        self.navigation_id.set(navigation_id);
        self.slicing_time.set(slicing_time);
    }
    pub fn AddUrl(&self, url: String, navigation_type: NavigationType, metrics: [u8; 16]) {
        assert!(!url.is_empty());
        if !self.initial_url.borrow().is_empty() {
            return;
        }
        *self.initial_url.borrow_mut() = url;
        self.navigation_type.set(navigation_type);
        self.metrics_token.set(metrics);
        self.url_change_time.set(self.Owner().Now());
    }
    pub fn NavigationType(&self) -> NavigationType {
        assert!(self.HasUrl());
        self.navigation_type.get()
    }
    pub fn SameDocumentMetricsToken(&self) -> [u8; 16] {
        self.metrics_token.get()
    }
    pub fn AttributionUrl(&self) -> String {
        self.initial_url.borrow().clone()
    }
    pub fn HasUrl(&self) -> bool {
        !self.initial_url.borrow().is_empty()
    }
    pub fn UrlChangeTime(&self) -> TimeTicks {
        self.url_change_time.get()
    }
    pub fn HasFirstContentfulPaint(&self) -> bool {
        self.first_image_or_text
            .borrow()
            .as_ref()
            .is_some_and(|record| record.HasPaintTime())
    }
    pub fn FirstContentfulPaint(&self) -> TimeTicks {
        self.first_image_or_text
            .borrow()
            .as_ref()
            .map_or(TimeTicks::new(), |r| r.PaintTime())
    }
    pub fn FirstContentfulPaintTimingInfo(&self) -> DOMPaintTimingInfo {
        assert!(self.HasFirstContentfulPaint());
        self.first_image_or_text
            .borrow()
            .as_ref()
            .unwrap()
            .PaintTimingInfo()
    }
    pub fn AddModifiedNode(&self) {
        self.modified_nodes
            .set(self.modified_nodes.get().wrapping_add(1));
    }
    pub fn HasDomModification(&self) -> bool {
        self.modified_nodes.get() > 0
    }
    pub fn PaintedArea(&self) -> u64 {
        self.painted_area.get()
    }
    pub fn LargestIcpEntry(&self) -> Option<Rc<InteractionContentfulPaint>> {
        self.largest_icp.borrow().clone()
    }
    pub fn AddPaintedArea(&self, record: &Rc<PaintTimingRecord>) -> bool {
        if !self.first_input_or_scroll_time.get().is_null() {
            return false;
        }
        if record.GetNode().is_none() {
            return false;
        }
        let rect = record.RootVisualRect();
        let area = (rect.width() * rect.height()) as u64;
        self.painted_area
            .set(self.painted_area.get().wrapping_add(area));
        if self.first_image_or_text.borrow().is_none() {
            *self.first_image_or_text.borrow_mut() = Some(record.clone());
        }
        true
    }
    pub fn SatisfiesSoftNavNonPaintCriteria(&self) -> bool {
        if self.TimeOrigin().is_null() || !self.HasDomModification() || !self.HasUrl() {
            return false;
        }
        assert!(!self.UrlChangeTime().is_null());
        true
    }
    pub fn SatisfiesSoftNavPaintCriteria(&self, required_area: u64) -> bool {
        self.painted_area.get() >= required_area
    }
    pub fn OnPaintFinished(&self) -> bool {
        let new_area = self.painted_area.get() - self.last_painted_area.get();
        if new_area > 0 {
            self.Owner().ForEachInteractionEffectsMonitor(|monitor| {
                monitor.OnContentfulPaint(self, new_area)
            });
        }
        self.last_modified_nodes.set(self.modified_nodes.get());
        self.last_painted_area.set(self.painted_area.get());
        new_area > 0
    }
    pub fn OnInputOrScroll(&self) {
        if !self.first_input_or_scroll_time.get().is_null() || self.painted_area.get() == 0 {
            return;
        }
        self.first_input_or_scroll_time.set(self.Owner().Now());
    }
    pub fn IsRecordingLargestContentfulPaint(&self) -> bool {
        self.first_input_or_scroll_time.get().is_null()
    }
    pub fn ShouldTrackForPaintTiming(&self, record: &PaintTimingRecord) -> bool {
        self.lcp
            .borrow()
            .as_ref()
            .expect("live calculator")
            .IsEligibleForLcp(record)
    }
    pub fn OnFramePresented(&self, candidates: &LcpCandidates) {
        assert!(self.IsRecordingLargestContentfulPaint());
        let calculator = self.lcp.borrow().as_ref().expect("live calculator").clone();
        calculator.OnFramePresented(candidates);
    }
    pub fn LatestLcpDetailsForUkm(&self) -> LargestContentfulPaintDetails {
        self.lcp
            .borrow()
            .as_ref()
            .expect("live calculator")
            .LatestLcpDetails()
            .clone()
    }
    pub fn WasEmitted(&self) -> bool {
        self.was_emitted.get()
    }
    pub fn EmitSoftNavigation(&self) {
        assert!(
            !self.HasBeenShutdown()
                && !self.WasEmitted()
                && self.HasFirstContentfulPaint()
                && self.SatisfiesSoftNavNonPaintCriteria()
        );
        self.was_emitted.set(true);
        let owner = self.Owner();
        if !owner.Features().expose_heuristics {
            return;
        }
        owner
            .Performance()
            .AddSoftNavigation(self.self_.upgrade().expect("live context"));
    }
    pub fn HasBeenShutdown(&self) -> bool {
        self.owner.borrow().is_none()
    }
    pub fn Shutdown(&self) {
        self.lcp.borrow_mut().take();
        self.first_image_or_text.borrow_mut().take();
        self.owner.borrow_mut().take();
        self.largest_icp.borrow_mut().take();
        self.current_lcp.borrow_mut().take();
        self.initial_event.borrow_mut().take();
    }
    pub fn Dispose(&self) {
        if let Some(owner) = self.GetSoftNavigationHeuristics() {
            owner.OnContextDisposed(self);
        }
    }
}
impl LcpDelegate for SoftNavigationContext {
    fn EmitLcpPerformanceEntry(
        &self,
        info: DOMPaintTimingInfo,
        paint_size: u64,
        load_time: TimeTicks,
        id: AtomicString,
        url: String,
        element: Option<EventTargetRef>,
    ) {
        let owner = self.Owner();
        if !owner.Features().expose_heuristics {
            return;
        }
        assert!(!self.HasBeenShutdown());
        let performance = owner.Performance();
        let lcp = LargestContentfulPaint::new(
            info.presentation_time,
            info.presentation_time,
            paint_size,
            performance.MonotonicTimeToDOMHighResTimeStamp(load_time),
            id,
            url,
            element,
            performance.NavigationId().web_exposed_id,
            info,
        );
        *self.current_lcp.borrow_mut() = Some(lcp.clone());
        let entry = InteractionContentfulPaint::new(
            performance.MonotonicTimeToDOMHighResTimeStamp(self.TimeOrigin()),
            info.presentation_time,
            lcp,
            performance.NavigationId().web_exposed_id,
            self.InteractionId(),
            info,
        );
        performance.OnInteractionContentfulPaintUpdated(entry.clone());
        *self.largest_icp.borrow_mut() = Some(entry);
    }
    fn OnLcpMetricsForReportingChanged(&self) {
        self.Owner().UpdateSoftLcpMetricsForContext(self);
    }
    fn IsHardNavigation(&self) -> bool {
        false
    }
}
impl Drop for SoftNavigationContext {
    fn drop(&mut self) {
        self.Dispose();
    }
}
