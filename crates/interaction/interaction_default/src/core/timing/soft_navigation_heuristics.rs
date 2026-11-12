//! Chromium 6c1d401f soft_navigation_heuristics.h:45-237; .cc:55-782.
//! Window-owned heuristics retain weak interaction contexts, the current URL
//! context, pending presentation contexts and the original paint attribution.
use super::{
    interaction_effects_monitor::InteractionEffectsMonitor,
    largest_contentful_paint_calculator::{
        LargestContentfulPaintDetails, LcpCandidates, LcpEnvironment, LcpFeatures,
    },
    paint_timing_record::PaintTimingRecord,
    performance_event_timing::PerformanceEventTiming,
    performance_timeline_entry_id_generator::PerformanceTimelineEntryIdInfo as Id,
    soft_navigation_context::SoftNavigationContext,
    soft_navigation_paint_attribution_tracker::SoftNavigationPaintAttributionTracker,
    window_performance::WindowPerformance,
};
use crate::core::navigation_api::navigation_api::NavigationType;
use foundation::task_attribution::{
    TaskAttributionInfo, TaskAttributionTracker, TaskScope, TaskScopeType,
};
use interaction_dispatch::core::dom::events::event::EventClock;
use interaction_input::base::time::time::TimeTicks;
use navigation_history::FrameLoadType;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};
#[derive(Clone, Copy, Debug)]
pub struct SoftNavigationFeatures {
    pub detection: bool,
    pub include_replace_state: bool,
    pub expose_heuristics: bool,
}
/// Facts from LocalDOMWindow/LocalFrame/Page's original owners. Gate order is
/// source CreateIfNeeded, cc:199-228 (including the detached-window CHECK).
pub struct SoftNavigationCreationFacts {
    pub has_window: bool,
    pub has_frame: bool,
    pub has_page: bool,
    pub is_outermost_main_frame: bool,
    pub is_ordinary_page: bool,
    pub document_is_devtools: Option<bool>,
}
pub fn ShouldCreateSoftNavigationHeuristics(
    features: SoftNavigationFeatures,
    facts: &SoftNavigationCreationFacts,
) -> bool {
    if !features.detection {
        return false;
    }
    assert!(
        facts.has_window && facts.has_frame && facts.has_page,
        "attached Window/Frame/Page"
    );
    facts.is_outermost_main_frame
        && facts.is_ordinary_page
        && !facts.document_is_devtools.unwrap_or(false)
}
pub trait SoftNavigationFrameFacts {
    fn HasFrame(&self) -> bool;
    fn LayoutViewportArea(&self) -> u64;
    fn ReferenceMonotonicTime(&self) -> TimeTicks;
    fn MonotonicTimeToPseudoWallMicroseconds(&self, time: TimeTicks) -> i64;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SoftNavigationOutcome {
    Detected = 0,
    NoContextDuringUrlChange = 1,
    InsufficientPaints = 2,
    NoDomModification = 4,
    NoContextMergedIntoPrevious = 8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoftNavigationPaintAreaMetric {
    Emitted,
    NotEmittedUrlEmpty,
    NotEmittedInsufficientPaint,
}
#[derive(Clone, Debug)]
pub struct SoftNavigationMetrics {
    pub performance_timeline_navigation_id: u64,
    pub start_time: i64,
    pub slicing_time: TimeTicks,
    pub navigation_type: NavigationType,
    pub same_document_metrics_token: [u8; 16],
}
#[derive(Clone, Debug)]
pub enum SoftNavigationEffect {
    Outcome(SoftNavigationOutcome),
    PaintArea {
        kind: SoftNavigationPaintAreaMetric,
        total: u64,
        points: u64,
    },
    DidObserveSoftNavigation(SoftNavigationMetrics),
    DidObserveSoftNavigationFirstContentfulPaint {
        navigation_id: u64,
        time: i64,
    },
    DidObserveSoftLargestContentfulPaint {
        navigation_id: u64,
        details: super::performance_timing_for_reporting::LargestContentfulPaintDetailsForReporting,
    },
}
pub struct SoftNavigationDOMModification {
    pub attributed: bool,
    pub mark_layout_context_changed:
        Option<interaction_dispatch::core::dom::events::event::EventTargetRef>,
}

pub struct SoftNavigationHeuristics {
    self_: Weak<Self>,
    performance: Rc<WindowPerformance>,
    clock: Rc<dyn EventClock>,
    frame: Rc<dyn SoftNavigationFrameFacts>,
    tracker: RefCell<Option<Weak<TaskAttributionTracker>>>,
    lcp_environment: Rc<dyn LcpEnvironment>,
    lcp_features: LcpFeatures,
    features: SoftNavigationFeatures,
    interaction_contexts: RefCell<HashMap<u64, Weak<SoftNavigationContext>>>,
    current_url_context: RefCell<Option<Rc<SoftNavigationContext>>>,
    waiting_for_timestamp: RefCell<Vec<Rc<SoftNavigationContext>>>,
    paint_attribution: Rc<SoftNavigationPaintAttributionTracker>,
    monitors: RefCell<HashMap<usize, Rc<InteractionEffectsMonitor>>>,
    count: Cell<u64>,
    effects: RefCell<Vec<SoftNavigationEffect>>,
}
impl SoftNavigationHeuristics {
    pub fn CreateIfNeeded(
        features: SoftNavigationFeatures,
        facts: SoftNavigationCreationFacts,
        performance: Rc<WindowPerformance>,
        clock: Rc<dyn EventClock>,
        frame: Rc<dyn SoftNavigationFrameFacts>,
        tracker: Option<&Rc<TaskAttributionTracker>>,
        lcp_environment: Rc<dyn LcpEnvironment>,
        lcp_features: LcpFeatures,
    ) -> Option<Rc<Self>> {
        if !ShouldCreateSoftNavigationHeuristics(features, &facts) {
            return None;
        }
        Some(Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            performance,
            clock,
            frame,
            tracker: RefCell::new(tracker.map(Rc::downgrade)),
            lcp_environment,
            lcp_features,
            features,
            interaction_contexts: RefCell::new(HashMap::new()),
            current_url_context: RefCell::new(None),
            waiting_for_timestamp: RefCell::new(Vec::new()),
            paint_attribution: Rc::new(Default::default()),
            monitors: RefCell::new(HashMap::new()),
            count: Cell::new(0),
            effects: RefCell::new(Vec::new()),
        }))
    }
    pub fn Features(&self) -> SoftNavigationFeatures {
        self.features
    }
    pub fn Performance(&self) -> Rc<WindowPerformance> {
        self.performance.clone()
    }
    pub fn Now(&self) -> TimeTicks {
        self.clock.Now()
    }
    pub fn GetPaintAttributionTracker(&self) -> Rc<SoftNavigationPaintAttributionTracker> {
        self.paint_attribution.clone()
    }
    pub fn SoftNavigationCount(&self) -> u64 {
        self.count.get()
    }
    pub fn TakeEffects(&self) -> Vec<SoftNavigationEffect> {
        std::mem::take(&mut *self.effects.borrow_mut())
    }
    fn Emit(&self, effect: SoftNavigationEffect) {
        self.effects.borrow_mut().push(effect);
    }
    fn Contexts(&self) -> Vec<Rc<SoftNavigationContext>> {
        self.interaction_contexts
            .borrow()
            .values()
            .filter_map(Weak::upgrade)
            .collect()
    }
    fn GetSoftNavigationContextForInteractionId(
        &self,
        id: Id,
    ) -> Option<Rc<SoftNavigationContext>> {
        if id == Id::NONE {
            return None;
        }
        self.interaction_contexts
            .borrow()
            .get(&id.non_web_exposed_id)
            .and_then(Weak::upgrade)
    }
    fn GetSoftNavigationContextForCurrentTask(&self) -> Option<Rc<SoftNavigationContext>> {
        if self.interaction_contexts.borrow().is_empty() {
            return None;
        }
        let tracker = self
            .tracker
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .expect("live tracker when contexts exist");
        let state = tracker.CurrentTaskState()?;
        let context = state
            .soft_navigation_context
            .as_ref()?
            .clone()
            .downcast::<SoftNavigationContext>()
            .ok()?;
        let owner = context.GetSoftNavigationHeuristics()?;
        std::ptr::eq(self, owner.as_ref()).then_some(context)
    }
    fn GetRelevantContextForNavigation(&self, id: Option<Id>) -> Option<Rc<SoftNavigationContext>> {
        let task = self.GetSoftNavigationContextForCurrentTask();
        let by_id = id.and_then(|id| self.GetSoftNavigationContextForInteractionId(id));
        assert!(
            task.as_ref()
                .zip(by_id.as_ref())
                .is_none_or(|(a, b)| Rc::ptr_eq(a, b)),
            "interaction and continuation name same context"
        );
        by_id.or(task)
    }
    pub fn MaybeCreateTaskScopeForEvent(
        &self,
        entry: Rc<PerformanceEventTiming>,
    ) -> Option<TaskScope> {
        if !entry.IsInteraction() {
            return None;
        }
        let id = entry.GetInteractionIdInfo().expect("interaction ID");
        let context = self
            .GetSoftNavigationContextForInteractionId(id)
            .unwrap_or_else(|| {
                let context = SoftNavigationContext::new(
                    &self.self_.upgrade().expect("Window retained owner"),
                    entry,
                    self.lcp_environment.clone(),
                    self.lcp_features,
                );
                self.interaction_contexts
                    .borrow_mut()
                    .insert(id.non_web_exposed_id, Rc::downgrade(&context));
                context
            });
        let tracker = self.tracker.borrow().as_ref().and_then(Weak::upgrade)?;
        let value: Rc<dyn std::any::Any> = context;
        let state = if let Some(state) = tracker.CurrentTaskState() {
            state.ForkAndSetSoftNavigation(Some(value))
        } else {
            TaskAttributionInfo::new(Some(value), None, None, 0)
        };
        Some(tracker.SetCurrentTaskState(state, TaskScopeType::SoftNavigation))
    }
    pub fn SameDocumentNavigationCommitted(
        &self,
        old_url: &str,
        new_url: &str,
        load: FrameLoadType,
        metrics: [u8; 16],
        interaction: Option<Id>,
    ) {
        if load == FrameLoadType::ReplaceCurrentItem && !self.features.include_replace_state {
            return;
        }
        if old_url == new_url {
            return;
        }
        let context = self.GetRelevantContextForNavigation(interaction);
        let navigation_type = match load {
            FrameLoadType::Standard => NavigationType::Push,
            FrameLoadType::ReplaceCurrentItem => NavigationType::Replace,
            FrameLoadType::Reload | FrameLoadType::ReloadBypassingCache => NavigationType::Reload,
            FrameLoadType::BackForward | FrameLoadType::Restore => NavigationType::Traverse,
        };
        let Some(context) = context else {
            let previous = self.current_url_context.borrow().clone();
            if let Some(previous) = previous {
                previous.AddUrl(new_url.to_owned(), navigation_type, metrics);
                self.Emit(SoftNavigationEffect::Outcome(
                    SoftNavigationOutcome::NoContextMergedIntoPrevious,
                ));
            } else {
                self.Emit(SoftNavigationEffect::Outcome(
                    SoftNavigationOutcome::NoContextDuringUrlChange,
                ));
            }
            return;
        };
        context.AddUrl(new_url.to_owned(), navigation_type, metrics);
        *self.current_url_context.borrow_mut() = Some(context.clone());
        self.MaybeCommitNavigationOrEmitSoftNavigation(&context);
    }
    /// Native DOM producer first checks InActiveDocument and projects the
    /// actual text/UA-shadow facts; return is exact Layout invalidation node.
    pub fn ModifiedDOM(
        &self,
        node: interaction_dispatch::core::dom::events::event::EventTargetRef,
        text_type: bool,
        ua_shadow_host: Option<interaction_dispatch::core::dom::events::event::EventTargetRef>,
    ) -> SoftNavigationDOMModification {
        assert!(self.frame.HasFrame());
        let Some(context) = self.GetSoftNavigationContextForCurrentTask() else {
            return SoftNavigationDOMModification {
                attributed: false,
                mark_layout_context_changed: None,
            };
        };
        let changed = self.paint_attribution.MarkNodeAsDirectlyModified(
            node,
            text_type,
            ua_shadow_host,
            context.clone(),
        );
        self.MaybeCommitNavigationOrEmitSoftNavigation(&context);
        SoftNavigationDOMModification {
            attributed: true,
            mark_layout_context_changed: changed,
        }
    }
    fn MaybeCommitNavigationOrEmitSoftNavigation(&self, context: &Rc<SoftNavigationContext>) {
        if context.WasEmitted() {
            return;
        }
        if context.HasNavigationId() {
            if context.HasFirstContentfulPaint() {
                self.EmitSoftNavigation(context);
            }
            return;
        }
        if !self
            .current_url_context
            .borrow()
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, context))
            || !context.SatisfiesSoftNavNonPaintCriteria()
            || !context.SatisfiesSoftNavPaintCriteria(self.CalculateRequiredPaintArea())
        {
            return;
        }
        self.performance.IncrementNavigationId();
        self.count.set(self.count.get().wrapping_add(1));
        context.OnSoftNavigationCommit(self.performance.NavigationId(), self.Now());
        self.ReportSoftNavigationToMetrics(context);
        if !context.HasFirstContentfulPaint() {
            let mut waiting = self.waiting_for_timestamp.borrow_mut();
            if !waiting.iter().any(|c| Rc::ptr_eq(c, context)) {
                waiting.push(context.clone());
            }
            return;
        }
        self.EmitSoftNavigation(context);
    }
    fn EmitSoftNavigation(&self, context: &Rc<SoftNavigationContext>) {
        context.EmitSoftNavigation();
        self.UpdateSoftFcpMetricsForContext(context);
        self.UpdateSoftLcpMetricsForContext(context);
    }
    pub fn InitializePaintTracking(&self, record: &Rc<PaintTimingRecord>) {
        let node = record.GetNode().expect("paint record Node");
        if let Some(context) = self
            .paint_attribution
            .GetSoftNavigationContextForNode(&node)
        {
            if context.IsRecordingLargestContentfulPaint()
                && context.ShouldTrackForPaintTiming(record)
            {
                record.SetSoftNavigationContext(Some(context));
            }
        }
    }
    pub fn OnPaintFinished(&self) {
        for context in self.Contexts() {
            if context.OnPaintFinished() {
                self.MaybeCommitNavigationOrEmitSoftNavigation(&context);
            }
        }
    }
    pub fn OnInputOrScroll(&self) {
        for context in self.Contexts() {
            context.OnInputOrScroll();
        }
    }
    pub fn OnFramePresented(
        &self,
        images: &[Rc<PaintTimingRecord>],
        texts: &[Rc<PaintTimingRecord>],
    ) {
        let mut groups: HashMap<usize, (Rc<SoftNavigationContext>, LcpCandidates)> = HashMap::new();
        for record in images.iter().chain(texts) {
            let Some(context) = record.GetSoftNavigationContext() else {
                continue;
            };
            if !context.IsRecordingLargestContentfulPaint() {
                continue;
            }
            let group = groups
                .entry(Rc::as_ptr(&context) as usize)
                .or_insert_with(|| (context, LcpCandidates::new()));
            if record.IsImageRecord() {
                group.1.MaybeUpdateImage(record.clone());
            } else {
                group.1.MaybeUpdateText(record.clone());
            }
        }
        for (context, candidates) in groups.values() {
            context.OnFramePresented(candidates);
        }
        let waiting = self.waiting_for_timestamp.borrow().clone();
        for context in waiting {
            assert!(!context.WasEmitted());
            self.MaybeCommitNavigationOrEmitSoftNavigation(&context);
        }
        self.waiting_for_timestamp
            .borrow_mut()
            .retain(|context| !context.WasEmitted());
    }
    pub fn UpdateSoftLcpMetricsForContext(&self, context: &SoftNavigationContext) {
        if !self
            .current_url_context
            .borrow()
            .as_ref()
            .is_some_and(|current| std::ptr::eq(current.as_ref(), context))
            || !context.WasEmitted()
        {
            return;
        }
        assert!(self.frame.HasFrame());
        self.Emit(SoftNavigationEffect::DidObserveSoftLargestContentfulPaint {
            navigation_id: context.NavigationId().non_web_exposed_id,
            details: {
                let mut details = self
                    .performance
                    .GetPerformanceTimingForReporting()
                    .PopulateLargestContentfulPaintDetailsForReporting(
                        &context.LatestLcpDetailsForUkm(),
                    );
                details.performance_timeline_navigation_id =
                    context.NavigationId().non_web_exposed_id;
                details
            },
        });
    }
    fn UpdateSoftFcpMetricsForContext(&self, context: &SoftNavigationContext) {
        assert!(context.HasFirstContentfulPaint() && self.frame.HasFrame());
        self.Emit(
            SoftNavigationEffect::DidObserveSoftNavigationFirstContentfulPaint {
                navigation_id: context.NavigationId().non_web_exposed_id,
                time: self
                    .frame
                    .MonotonicTimeToPseudoWallMicroseconds(context.FirstContentfulPaint()),
            },
        );
    }
    fn ReportSoftNavigationToMetrics(&self, context: &SoftNavigationContext) {
        assert!(self.frame.HasFrame());
        assert!(context.TimeOrigin() > self.frame.ReferenceMonotonicTime());
        self.Emit(SoftNavigationEffect::DidObserveSoftNavigation(
            SoftNavigationMetrics {
                performance_timeline_navigation_id: context.NavigationId().non_web_exposed_id,
                start_time: self
                    .frame
                    .MonotonicTimeToPseudoWallMicroseconds(context.TimeOrigin()),
                slicing_time: context.SoftNavigationSlicingTime(),
                navigation_type: context.NavigationType(),
                same_document_metrics_token: context.SameDocumentMetricsToken(),
            },
        ));
        self.Emit(SoftNavigationEffect::Outcome(
            SoftNavigationOutcome::Detected,
        ));
    }
    fn CalculateViewportArea(&self) -> u64 {
        self.frame.LayoutViewportArea().max(1)
    }
    fn CalculateRequiredPaintArea(&self) -> u64 {
        (self.CalculateViewportArea() / 10000).max(1)
    }
    fn Exhausted(&self, context: &SoftNavigationContext, viewport: u64, required: u64) {
        let kind = if !context.HasUrl() {
            Some(SoftNavigationPaintAreaMetric::NotEmittedUrlEmpty)
        } else if context.HasNavigationId() {
            Some(SoftNavigationPaintAreaMetric::Emitted)
        } else if !context.HasDomModification() {
            self.Emit(SoftNavigationEffect::Outcome(
                SoftNavigationOutcome::NoDomModification,
            ));
            None
        } else if !context.SatisfiesSoftNavPaintCriteria(required) {
            self.Emit(SoftNavigationEffect::Outcome(
                SoftNavigationOutcome::InsufficientPaints,
            ));
            Some(SoftNavigationPaintAreaMetric::NotEmittedInsufficientPaint)
        } else {
            None
        };
        if let Some(kind) = kind {
            self.Emit(SoftNavigationEffect::PaintArea {
                kind,
                total: context.PaintedArea(),
                points: context.PaintedArea().wrapping_mul(10000) / viewport,
            });
        }
    }
    pub fn OnContextDisposed(&self, context: &SoftNavigationContext) {
        let id = context.GetInteractionIdInfo().non_web_exposed_id;
        let mut contexts = self.interaction_contexts.borrow_mut();
        if contexts
            .get(&id)
            .is_some_and(|weak| std::ptr::eq(weak.as_ptr(), context))
        {
            contexts.remove(&id);
        }
        drop(contexts);
        self.Exhausted(
            context,
            self.CalculateViewportArea(),
            self.CalculateRequiredPaintArea(),
        );
    }
    pub fn ForEachInteractionEffectsMonitor(
        &self,
        mut callback: impl FnMut(&InteractionEffectsMonitor),
    ) {
        let monitors = self.monitors.borrow().values().cloned().collect::<Vec<_>>();
        for monitor in monitors {
            callback(&monitor);
        }
    }
    pub fn RegisterInteractionEffectsMonitor(&self, monitor: Rc<InteractionEffectsMonitor>) {
        assert!(self.frame.HasFrame());
        assert!(self
            .monitors
            .borrow_mut()
            .insert(Rc::as_ptr(&monitor) as usize, monitor)
            .is_none());
    }
    pub fn UnregisterInteractionEffectsMonitor(&self, monitor: &InteractionEffectsMonitor) {
        if !self.frame.HasFrame() {
            return;
        }
        assert!(self
            .monitors
            .borrow_mut()
            .remove(&(monitor as *const _ as usize))
            .is_some());
    }
    pub fn Shutdown(&self) {
        self.tracker.borrow_mut().take();
        let viewport = self.CalculateViewportArea();
        let required = self.CalculateRequiredPaintArea();
        for context in self.Contexts() {
            self.Exhausted(&context, viewport, required);
            context.Shutdown();
        }
        self.ForEachInteractionEffectsMonitor(|monitor| monitor.Shutdown());
        self.monitors.borrow_mut().clear();
        self.interaction_contexts.borrow_mut().clear();
    }
}
