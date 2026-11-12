//! Chromium window_performance.h:127-145,239-250,294-325;
//! .cc:638-812,815-989,1000-1062,1354-1374,1777-1802.
use super::{
    performance_event_timing::{
        EventTimingReportingInfo, FallbackReason, PerformanceEventTiming as Entry,
    },
    performance_timeline_entry_id_generator::{
        PerformanceRandom, PerformanceTimelineEntryIdGenerator,
        PerformanceTimelineEntryIdInfo as Id,
    },
    responsiveness_metrics::{ResponsivenessMetrics, UserInteractionMetric},
};
use interaction_dispatch::core::{
    dom::events::event::{EventClock, EventRef},
    events::{keyboard_event::KeyboardEvent, pointer_event::PointerEvent},
};
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    rc::{Rc, Weak},
};
pub trait EventTimingEnvironment {
    fn MonotonicTimeToPseudoWallMicroseconds(&self, time: TimeTicks) -> Option<i64>;
    fn ResourceTimeToPseudoWallMicroseconds(&self, time: std::time::Instant) -> Option<i64>;
    fn HasFrame(&self) -> bool;
    fn IsContextDestroyed(&self) -> bool;
    fn IsContextPaused(&self) -> bool;
    /// ChromeClientImpl.cc:1103-1108. Enqueue the original widget promise
    /// synchronously at processingEnd, before a subsequent paint/commit.
    fn NotifyPresentationTime(
        &self,
        performance: &Rc<WindowPerformance>,
        request: PresentationTimeRequest,
    );
}
#[derive(Clone, Copy, Debug)]
pub struct PresentationTimeRequest {
    pub frame_index: u64,
    pub expected_frame_source_id: u64,
}
pub enum WindowPerformanceEffect {
    /// InteractiveDetector.cc's existing optional supplement consumes this
    /// exact input-delay sample; it is not a fabricated next-paint timestamp.
    InputDelay {
        event: EventRef,
        creation_time: TimeTicks,
        processing_start: TimeTicks,
    },
    EventTimingReady {
        entry: Rc<Entry>,
        primary_entry: Rc<Entry>,
    },
    UserInteraction(UserInteractionMetric),
    ArtificialPointerupOrClick,
}
pub type WindowPerformanceEffectSink = Rc<dyn Fn(WindowPerformanceEffect)>;
struct PublishPerformanceEffects<'a>(&'a WindowPerformance);
impl Drop for PublishPerformanceEffects<'_> {
    fn drop(&mut self) {
        self.0.PublishEffects();
    }
}
#[derive(Clone, Copy)]
pub struct PresentationTimingDetails {
    pub timestamp: TimeTicks,
    pub actual_frame_source_id: u64,
    pub manual_begin_frame: bool,
    pub actual_source_is_manual: bool,
    pub ignore_unexpected_frame_source: bool,
}
pub struct WindowPerformance {
    paint_timing_for_reporting:
        RefCell<Option<Rc<super::performance_timing_for_reporting::PerformanceTimingForReporting>>>,
    soft_navigation:
        RefCell<Option<Weak<super::soft_navigation_heuristics::SoftNavigationHeuristics>>>,
    time_origin: TimeTicks,
    cross_origin_isolated: bool,
    time_clamper: super::time_clamper::TimeClamper,
    soft_navigation_entries:
        RefCell<Vec<Rc<super::performance_soft_navigation::PerformanceSoftNavigation>>>,
    icp_entries: RefCell<Vec<Rc<super::interaction_contentful_paint::InteractionContentfulPaint>>>,
    lcp_entries: RefCell<Vec<Rc<super::largest_contentful_paint::LargestContentfulPaint>>>,
    dropped_soft_navigation_entries: Cell<u64>,
    dropped_icp_entries: Cell<u64>,
    paint_observers: RefCell<Vec<Weak<dyn PaintPerformanceObserver>>>,
    self_: Weak<Self>,
    pending_navigation_destination_url: RefCell<Option<String>>,
    navigation_destination_url: RefCell<Option<String>>,
    clock: Rc<dyn EventClock>,
    environment: RefCell<Rc<dyn EventTimingEnvironment>>,
    responsiveness: RefCell<ResponsivenessMetrics>,
    navigation_id: PerformanceTimelineEntryIdGenerator,
    current_event: Cell<Option<usize>>,
    active_entries: RefCell<Vec<Rc<Entry>>>,
    entries: RefCell<VecDeque<Rc<Entry>>>,
    counts: RefCell<HashMap<String, u64>>,
    current_frame_index: Cell<u64>,
    begin_main_frame_source_id: Cell<u64>,
    last_presentation_requested_for_frame_index: Cell<u64>,
    pending_pointer_down_start_time: Cell<TimeTicks>,
    autoscroll_active: Cell<bool>,
    last_hidden_timestamp: Cell<TimeTicks>,
    modal_dialog_timestamps: RefCell<VecDeque<TimeTicks>>,
    effects: RefCell<Vec<WindowPerformanceEffect>>,
    effect_sink: RefCell<Option<WindowPerformanceEffectSink>>,
}
impl WindowPerformance {
    pub fn new(
        clock: Rc<dyn EventClock>,
        environment: Rc<dyn EventTimingEnvironment>,
        random: Rc<dyn PerformanceRandom>,
        time_origin: TimeTicks,
        cross_origin_isolated: bool,
    ) -> Rc<Self> {
        let secret = (random.RandIntInclusive(0, u64::from(u32::MAX)) << 32)
            | random.RandIntInclusive(0, u64::from(u32::MAX));
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            paint_timing_for_reporting: RefCell::new(None),
            soft_navigation: RefCell::new(None),
            time_origin,
            cross_origin_isolated,
            time_clamper: super::time_clamper::TimeClamper::new(secret),
            soft_navigation_entries: RefCell::new(Vec::new()),
            icp_entries: RefCell::new(Vec::new()),
            lcp_entries: RefCell::new(Vec::new()),
            dropped_soft_navigation_entries: Cell::new(0),
            dropped_icp_entries: Cell::new(0),
            paint_observers: RefCell::new(Vec::new()),
            pending_navigation_destination_url: RefCell::new(None),
            navigation_destination_url: RefCell::new(None),
            clock,
            environment: RefCell::new(environment),
            responsiveness: RefCell::new(ResponsivenessMetrics::new(random.clone())),
            navigation_id: PerformanceTimelineEntryIdGenerator::new(random),
            current_event: Cell::new(None),
            active_entries: RefCell::new(Vec::new()),
            entries: RefCell::new(VecDeque::new()),
            counts: RefCell::new(HashMap::new()),
            current_frame_index: Cell::new(1),
            begin_main_frame_source_id: Cell::new(0),
            last_presentation_requested_for_frame_index: Cell::new(0),
            pending_pointer_down_start_time: Cell::new(TimeTicks::new()),
            autoscroll_active: Cell::new(false),
            last_hidden_timestamp: Cell::new(TimeTicks::new()),
            modal_dialog_timestamps: RefCell::new(VecDeque::new()),
            effects: RefCell::new(Vec::new()),
            effect_sink: RefCell::new(None),
        })
    }
    /// window_performance.h:101-113: pending is not exposed before pagehide.
    pub fn SetPendingNavigationDestinationURL(&self, url: &str) {
        *self.pending_navigation_destination_url.borrow_mut() = Some(url.to_owned());
    }
    pub fn PromoteNavigationDestinationURL(&self) {
        *self.navigation_destination_url.borrow_mut() =
            self.pending_navigation_destination_url.borrow().clone();
    }
    pub fn NavigationDestinationURL(&self) -> Option<String> {
        self.navigation_destination_url.borrow().clone()
    }
    pub fn Environment(&self) -> Rc<dyn EventTimingEnvironment> {
        self.environment.borrow().clone()
    }
    /// LocalDOMWindow's initial-empty secure transition retains performance
    /// state but installs a new Document on that same Window.
    pub fn RebindEnvironmentForInitialWindowReuse(
        &self,
        environment: Rc<dyn EventTimingEnvironment>,
    ) {
        *self.environment.borrow_mut() = environment;
        self.paint_timing_for_reporting.borrow_mut().take();
    }
    pub fn GetCurrentEventTimingEvent(&self) -> Option<usize> {
        self.current_event.get()
    }
    pub fn GetTopMostEventTimingEntry(&self) -> Option<Rc<Entry>> {
        self.active_entries.borrow().first().cloned()
    }
    pub fn EventCount(&self, name: &str) -> u64 {
        self.counts.borrow().get(name).copied().unwrap_or(0)
    }
    pub fn ResponsivenessMetrics(&self) -> &RefCell<ResponsivenessMetrics> {
        &self.responsiveness
    }
    pub fn TakeEffects(&self) -> Vec<WindowPerformanceEffect> {
        std::mem::take(&mut *self.effects.borrow_mut())
    }
    /// The Window's WebAPI projection consumes original ordered outputs. An
    /// unbound source retains TakeEffects for standalone engines and tests.
    pub fn BindEffectSink(&self, sink: WindowPerformanceEffectSink) {
        *self.effect_sink.borrow_mut() = Some(sink);
        self.PublishEffects();
    }
    fn PublishEffects(&self) {
        let Some(sink) = self.effect_sink.borrow().clone() else {
            return;
        };
        for effect in self.TakeEffects() {
            sink(effect);
        }
    }
    pub fn EventTimingProcessingStart(&self, event: &EventRef) -> Rc<Entry> {
        let _publish = PublishPerformanceEffects(self);
        assert!(self.Environment().HasFrame());
        let processing_start = self.clock.Now();
        let (name, creation, cancel_trusted) = {
            let e = event.borrow();
            (
                e.BaseEvent().r#type().to_owned(),
                e.BaseEvent().PlatformTimeStamp(),
                e.BaseEvent().isTrusted(),
            )
        };
        let pointer = event.Typed::<PointerEvent>();
        let keyboard = event.Typed::<KeyboardEvent>();
        let (creation, pointer_id, prevent) = if let Some(pointer) = pointer {
            let p = pointer.borrow();
            (
                p.OldestPlatformTimeStamp(),
                Some(p.pointerId()),
                p.GetPreventCountingAsInteraction(),
            )
        } else {
            (creation, None, false)
        };
        if cancel_trusted
            && matches!(
                name.as_str(),
                "pointerdown" | "pointerup" | "click" | "keydown" | "mousedown" | "mouseup"
            )
        {
            self.effects
                .borrow_mut()
                .push(WindowPerformanceEffect::InputDelay {
                    event: event.clone(),
                    creation_time: creation,
                    processing_start,
                });
        }
        *self.counts.borrow_mut().entry(name.clone()).or_default() += 1;
        if pointer_id.is_some() && name == "pointerdown" {
            self.pending_pointer_down_start_time.set(creation);
        }
        let info = EventTimingReportingInfo {
            fallback_reason: FallbackReason::None,
            frame_index: self.current_frame_index.get(),
            creation_time: creation,
            enqueued_to_main_thread_time: self
                .responsiveness
                .borrow()
                .CurrentInteractionEventQueuedTimestamp(),
            processing_start_time: processing_start,
            processing_end_time: TimeTicks::new(),
            commit_finish_time: TimeTicks::new(),
            presentation_time: TimeTicks::new(),
            fallback_time: TimeTicks::new(),
            render_start_time: TimeTicks::new(),
            key_code: keyboard.map(|k| k.borrow().keyCode()),
            pointer_id,
            prevent_counting_as_interaction: prevent || self.autoscroll_active.get(),
            is_processing_fully_nested_in_another_event: !self.active_entries.borrow().is_empty(),
        };
        let entry = Entry::Create(
            event.clone(),
            info,
            self.navigation_id.GetValue().web_exposed_id,
        );
        self.active_entries.borrow_mut().push(entry.clone());
        self.entries.borrow_mut().push_back(entry.clone());
        self.current_event.set(Some(event.Identity()));
        let top = self.GetTopMostEventTimingEntry();
        self.responsiveness
            .borrow_mut()
            .TryAssignInteractionId(&entry, top.as_ref());
        entry
    }
    pub fn EventTimingProcessingEnd(&self, entry: &Rc<Entry>, event: &EventRef) {
        let _publish = PublishPerformanceEffects(self);
        self.current_event.set(None);
        let end = self.clock.Now();
        let popped = self
            .active_entries
            .borrow_mut()
            .pop()
            .expect("active timing scope");
        assert!(Rc::ptr_eq(&popped, entry));
        entry.reporting_info.borrow_mut().processing_end_time = end;
        if !self.Environment().HasFrame() {
            entry.UpdateFallbackTime(end, FallbackReason::WindowDestroyed);
            return;
        }
        if matches!(entry.name.as_str(), "pointerup" | "click")
            && entry.reporting_info.borrow().creation_time
                == self.pending_pointer_down_start_time.get()
        {
            self.effects
                .borrow_mut()
                .push(WindowPerformanceEffect::ArtificialPointerupOrClick);
            if cfg!(target_os = "macos") {
                entry.UpdateFallbackTime(end, FallbackReason::MacOSArtificialEvent);
            }
        }
        let target = event.borrow().BaseEvent().RawTarget();
        if let Some(target) = target {
            entry.SetTarget(&target);
        }
        if entry.name == "contextmenu" {
            for pending in
                self.entries.borrow().iter().filter(|p| {
                    p.reporting_info.borrow().frame_index == self.current_frame_index.get()
                })
            {
                if !pending.HasKnownEndTime() {
                    pending.UpdateFallbackTime(
                        end,
                        FallbackReason::InteractionInterruptedByContextMenu,
                    );
                }
            }
        }
        if self.last_presentation_requested_for_frame_index.get() < self.current_frame_index.get() {
            let request = PresentationTimeRequest {
                frame_index: self.current_frame_index.get(),
                expected_frame_source_id: self.begin_main_frame_source_id.get(),
            };
            self.Environment().NotifyPresentationTime(
                &self
                    .self_
                    .upgrade()
                    .expect("active WindowPerformance scope"),
                request,
            );
            self.last_presentation_requested_for_frame_index
                .set(request.frame_index);
        }
    }
    pub fn SetCommitFinishTimeStampForPendingEvents(&self, time: TimeTicks) {
        for e in self.entries.borrow().iter() {
            if e.reporting_info.borrow().commit_finish_time.is_null()
                && e.NeedsNextPaintMeasurement()
            {
                e.reporting_info.borrow_mut().commit_finish_time = time;
            }
        }
    }
    pub fn SetRenderStartTimeForPendingEvents(&self, time: TimeTicks) {
        for e in self.entries.borrow().iter() {
            if e.NeedsNextPaintMeasurement() {
                e.reporting_info.borrow_mut().render_start_time = time;
            }
        }
    }
    pub fn OnPaintFinished(&self) {
        self.current_frame_index
            .set(self.current_frame_index.get() + 1);
    }
    pub fn OnBeginMainFrame(&self, source_id: u64) {
        if source_id != 0 {
            self.begin_main_frame_source_id.set(source_id);
        }
    }
    pub fn OnPageScroll(&self, selection_autoscroll_in_progress: bool) {
        self.autoscroll_active.set(selection_autoscroll_in_progress);
    }
    pub fn ResetAutoscroll(&self) {
        self.autoscroll_active.set(false);
    }
    pub fn WillNavigateEventCommitNow(&self, id: Id) {
        self.responsiveness
            .borrow_mut()
            .WillNavigateEventCommitNow(id);
    }
    pub fn OnPresentationPromiseResolved(
        &self,
        request: PresentationTimeRequest,
        details: PresentationTimingDetails,
    ) {
        assert!(!details.timestamp.is_null());
        if request.frame_index == self.current_frame_index.get() {
            for entry in
                self.entries.borrow().iter().filter(|e| {
                    e.reporting_info.borrow().frame_index == self.current_frame_index.get()
                })
            {
                entry.UpdateFallbackTime(details.timestamp, FallbackReason::DoesNotNeedNextPaint);
            }
            self.current_frame_index
                .set(self.current_frame_index.get() + 1);
            self.TryFlushEventTimingQueue();
            return;
        }
        let expected = request.expected_frame_source_id;
        let actual = details.actual_frame_source_id;
        let mut matches = expected == 0 || actual == 0 || expected == actual;
        if details.manual_begin_frame && ((expected >> 32) == 0 || details.actual_source_is_manual)
        {
            matches = true;
        }
        for entry in self
            .entries
            .borrow()
            .iter()
            .filter(|e| e.reporting_info.borrow().frame_index == request.frame_index)
        {
            if !entry.NeedsNextPaintMeasurement() {
                continue;
            }
            entry.reporting_info.borrow_mut().presentation_time = details.timestamp;
            let info = *entry.reporting_info.borrow();
            if !matches && details.ignore_unexpected_frame_source {
                assert!(!info.commit_finish_time.is_null());
                entry.UpdateFallbackTime(
                    info.commit_finish_time,
                    FallbackReason::UnexpectedFrameSource,
                );
            }
            let hidden = self.last_hidden_timestamp.get();
            if hidden > info.creation_time && hidden < details.timestamp {
                entry.UpdateFallbackTime(
                    if !info.commit_finish_time.is_null() && hidden > info.commit_finish_time {
                        info.commit_finish_time
                    } else {
                        info.processing_end_time
                    },
                    FallbackReason::VisibilityChange,
                );
            }
            let mut dialogs = self.modal_dialog_timestamps.borrow_mut();
            while dialogs
                .front()
                .is_some_and(|time| *time < info.creation_time)
            {
                dialogs.pop_front();
            }
            if let Some(time) = dialogs.front().filter(|time| **time < details.timestamp) {
                entry.UpdateFallbackTime(*time, FallbackReason::ModalDialog);
            }
        }
        self.TryFlushEventTimingQueue();
    }
    pub fn DidShowModalDialog(&self, time: TimeTicks) {
        self.modal_dialog_timestamps.borrow_mut().push_back(time);
    }
    pub fn PageHidden(&self, time: TimeTicks) {
        self.last_hidden_timestamp.set(time);
        self.responsiveness.borrow_mut().FlushAllEvents();
        for entry in self.entries.borrow().iter() {
            let end = entry.reporting_info.borrow().processing_end_time;
            if !entry.HasKnownEndTime() && !end.is_null() {
                entry.UpdateFallbackTime(end, FallbackReason::VisibilityChange);
            }
        }
        self.TryFlushEventTimingQueue();
    }
    pub fn TryFlushEventTimingQueue(&self) {
        let _publish = PublishPerformanceEffects(self);
        if !self.Environment().HasFrame() {
            self.entries.borrow_mut().clear();
            return;
        }
        loop {
            let frame = {
                let entries = self.entries.borrow();
                let Some(first) = entries.front() else {
                    return;
                };
                let frame = first.reporting_info.borrow().frame_index;
                let group: Vec<_> = entries
                    .iter()
                    .take_while(|e| e.reporting_info.borrow().frame_index == frame)
                    .cloned()
                    .collect();
                if !group.iter().all(|e| e.IsReadyForReporting()) {
                    return;
                }
                group
            };
            let mut primary = HashMap::<i64, Rc<Entry>>::new();
            for entry in &frame {
                let creation = entry.GetStartTime().ToInternalValue();
                let primary = primary
                    .entry(creation)
                    .or_insert_with(|| entry.clone())
                    .clone();
                if let Some(metric) = self.responsiveness.borrow_mut().ReportToMetrics(entry) {
                    self.effects
                        .borrow_mut()
                        .push(WindowPerformanceEffect::UserInteraction(metric));
                }
                self.effects
                    .borrow_mut()
                    .push(WindowPerformanceEffect::EventTimingReady {
                        entry: entry.clone(),
                        primary_entry: primary,
                    });
            }
            self.entries.borrow_mut().drain(..frame.len());
        }
    }
}

/// Actual source performance entries, sharing the original retained objects.
#[derive(Clone)]
pub enum PaintPerformanceEntry {
    SoftNavigation(Rc<super::performance_soft_navigation::PerformanceSoftNavigation>),
    InteractionContentfulPaint(Rc<super::interaction_contentful_paint::InteractionContentfulPaint>),
    LargestContentfulPaint(Rc<super::largest_contentful_paint::LargestContentfulPaint>),
}
pub trait PaintPerformanceObserver {
    fn Observes(&self, entry: &PaintPerformanceEntry) -> bool;
    fn Enqueue(&self, entry: PaintPerformanceEntry);
}
impl WindowPerformance {
    // window_performance.h:211-218.
    pub fn NavigationId(&self) -> Id {
        self.navigation_id.GetValue()
    }
    pub fn IncrementNavigationId(&self) {
        self.navigation_id.IncrementId();
    }
    pub fn BindSoftNavigationHeuristics(
        &self,
        owner: Option<&Rc<super::soft_navigation_heuristics::SoftNavigationHeuristics>>,
    ) {
        *self.soft_navigation.borrow_mut() = owner.map(Rc::downgrade);
    }
    pub fn SoftNavigationHeuristics(
        &self,
    ) -> Option<Rc<super::soft_navigation_heuristics::SoftNavigationHeuristics>> {
        self.soft_navigation.borrow().as_ref()?.upgrade()
    }
    // performance.cc:1328-1351. Clamp each absolute monotonic timestamp with
    // the same immutable random secret, before subtracting the time origin.
    pub fn CrossOriginIsolatedCapability(&self) -> bool {
        self.cross_origin_isolated
    }
    pub fn GetPerformanceTimingForReporting(
        &self,
    ) -> Rc<super::performance_timing_for_reporting::PerformanceTimingForReporting> {
        self.paint_timing_for_reporting
            .borrow_mut()
            .get_or_insert_with(|| {
                super::performance_timing_for_reporting::PerformanceTimingForReporting::new(
                    self.self_.clone(),
                )
            })
            .clone()
    }
    pub(crate) fn ClampTimeResolutionMicroseconds(&self, time: i64) -> i64 {
        self.time_clamper
            .ClampTimeResolution(time, self.cross_origin_isolated)
    }
    pub fn GetTimeOriginInternal(&self) -> TimeTicks {
        self.time_origin
    }
    pub fn MonotonicTimeToDOMHighResTimeStamp(&self, time: TimeTicks) -> f64 {
        if time.is_null() || self.time_origin.is_null() {
            return 0.0;
        }
        let clamp = |value: TimeTicks| {
            self.time_clamper
                .ClampTimeResolution(value.ToInternalValue(), self.cross_origin_isolated)
        };
        ((clamp(time) as f64 - clamp(self.time_origin) as f64) / 1000.0).max(0.0)
    }
    pub fn RegisterPaintPerformanceObserver(&self, observer: &Rc<dyn PaintPerformanceObserver>) {
        self.paint_observers
            .borrow_mut()
            .push(Rc::downgrade(observer));
    }
    fn NotifyPaintObservers(&self, entry: PaintPerformanceEntry) {
        let observers = self
            .paint_observers
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        for observer in observers {
            if observer.Observes(&entry) {
                observer.Enqueue(entry.clone());
            }
        }
    }
    // Performance::AddSoftNavigationToPerformanceTimeline, cc:805-814;
    // WindowPerformance::AddSoftNavigation (base performance producer).
    pub fn AddSoftNavigation(
        &self,
        context: Rc<super::soft_navigation_context::SoftNavigationContext>,
    ) {
        let entry = super::performance_soft_navigation::PerformanceSoftNavigation::new(
            self.MonotonicTimeToDOMHighResTimeStamp(context.TimeOrigin()),
            context.FirstContentfulPaintTimingInfo(),
            context,
        );
        self.NotifyPaintObservers(PaintPerformanceEntry::SoftNavigation(entry.clone()));
        let mut entries = self.soft_navigation_entries.borrow_mut();
        if entries.len() < 50 {
            let index = entries.partition_point(|old| old.start_time <= entry.start_time);
            entries.insert(index, entry);
        } else {
            self.dropped_soft_navigation_entries
                .set(self.dropped_soft_navigation_entries.get().wrapping_add(1));
        }
    }
    // window_performance.cc:1770-1778; performance.cc:792-803.
    pub fn OnInteractionContentfulPaintUpdated(
        &self,
        entry: Rc<super::interaction_contentful_paint::InteractionContentfulPaint>,
    ) {
        self.NotifyPaintObservers(PaintPerformanceEntry::InteractionContentfulPaint(
            entry.clone(),
        ));
        let mut entries = self.icp_entries.borrow_mut();
        if entries.len() < 150 {
            let index = entries.partition_point(|old| old.start_time <= entry.start_time);
            entries.insert(index, entry);
        } else {
            self.dropped_icp_entries
                .set(self.dropped_icp_entries.get().wrapping_add(1));
        }
    }
    pub fn OnLargestContentfulPaintUpdated(
        &self,
        info: super::paint_timing_record::DOMPaintTimingInfo,
        size: u64,
        load_time: TimeTicks,
        id: foundation::AtomicString,
        url: String,
        element: Option<interaction_dispatch::core::dom::events::event::EventTargetRef>,
    ) {
        let start_time = info.presentation_time;
        let entry = super::largest_contentful_paint::LargestContentfulPaint::new(
            start_time,
            info.presentation_time,
            size,
            self.MonotonicTimeToDOMHighResTimeStamp(load_time),
            id,
            url,
            element,
            self.NavigationId().web_exposed_id,
            info,
        );
        self.NotifyPaintObservers(PaintPerformanceEntry::LargestContentfulPaint(entry.clone()));
        let mut entries = self.lcp_entries.borrow_mut();
        if entries.len() < 150 {
            let index = entries.partition_point(|old| old.start_time <= entry.start_time);
            entries.insert(index, entry);
        }
    }
    pub fn SoftNavigationEntries(
        &self,
    ) -> std::cell::Ref<'_, Vec<Rc<super::performance_soft_navigation::PerformanceSoftNavigation>>>
    {
        self.soft_navigation_entries.borrow()
    }
    pub fn InteractionContentfulPaintEntries(
        &self,
    ) -> std::cell::Ref<'_, Vec<Rc<super::interaction_contentful_paint::InteractionContentfulPaint>>>
    {
        self.icp_entries.borrow()
    }
    pub fn LargestContentfulPaintEntries(
        &self,
    ) -> std::cell::Ref<'_, Vec<Rc<super::largest_contentful_paint::LargestContentfulPaint>>> {
        self.lcp_entries.borrow()
    }
}

// Event::timeStamp, event.cc:376-394, uses this same Window's performance
// time origin and TimeClamper; bindings never manufacture a second clock.
impl interaction_dispatch::core::dom::events::event::EventTimeStampContext for WindowPerformance {
    fn WindowTimeStamp(&self, timestamp: TimeTicks) -> Option<f64> {
        Some(self.MonotonicTimeToDOMHighResTimeStamp(timestamp))
    }
    fn WorkerTimeStamp(&self, _: TimeTicks) -> Option<f64> {
        None
    }
}
