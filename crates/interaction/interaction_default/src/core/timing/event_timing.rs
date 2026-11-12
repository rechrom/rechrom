//! Chromium event_timing.h:30-103, .cc:29-167.
use super::{
    performance_event_timing::PerformanceEventTiming,
    performance_timeline_entry_id_generator::PerformanceTimelineEntryIdInfo,
    window_performance::WindowPerformance,
};
use interaction_dispatch::core::dom::events::event::EventRef;
use std::rc::Rc;
/// Stack scope retains its original WindowPerformance even if dispatch removes
/// the frame or installs another Document/Window.
pub struct EventTiming {
    performance: Rc<WindowPerformance>,
    event: EventRef,
    entry: Rc<PerformanceEventTiming>,
    task_scope: Option<foundation::task_attribution::TaskScope>,
}
impl EventTiming {
    pub fn new(performance: Rc<WindowPerformance>, event: EventRef) -> Option<Self> {
        let e = performance.Environment();
        if !e.HasFrame()
            || e.IsContextDestroyed()
            || e.IsContextPaused()
            || performance.GetCurrentEventTimingEvent() == Some(event.Identity())
        {
            return None;
        }
        let entry = performance.EventTimingProcessingStart(&event);
        let task_scope = performance
            .SoftNavigationHeuristics()
            .and_then(|heuristics| heuristics.MaybeCreateTaskScopeForEvent(entry.clone()));
        Some(Self {
            task_scope,
            performance,
            event,
            entry,
        })
    }
    pub fn GetInteractionIdInfo(&self) -> Option<PerformanceTimelineEntryIdInfo> {
        self.entry.GetInteractionIdInfo()
    }
    pub fn GetEntry(&self) -> Rc<PerformanceEventTiming> {
        self.entry.clone()
    }
}
impl Drop for EventTiming {
    fn drop(&mut self) {
        self.performance
            .EventTimingProcessingEnd(&self.entry, &self.event);
    }
}
pub struct NavigationEventTiming {
    timing: Option<EventTiming>,
}
impl NavigationEventTiming {
    /// Source EventTiming.cc:113-115: an executing context without a frame
    /// has no performance entry, even for a user-initiated navigation event.
    pub fn ForAbsentFrame() -> Self {
        Self { timing: None }
    }

    /// The caller projects actual NavigateEvent/PopStateEvent/HashChangeEvent
    /// userInitiated from its canonical subclass; trust is intentionally not a
    /// gate here (event_timing.cc:64-87, before dispatch marks it trusted).
    pub fn new(performance: Rc<WindowPerformance>, event: EventRef, user_initiated: bool) -> Self {
        Self {
            timing: if user_initiated {
                EventTiming::new(performance, event)
            } else {
                None
            },
        }
    }
    pub fn GetInteractionIdInfo(&self) -> Option<PerformanceTimelineEntryIdInfo> {
        self.timing.as_ref()?.GetInteractionIdInfo()
    }
}
pub struct UIEventTiming {
    timing: Option<EventTiming>,
}
impl UIEventTiming {
    /// Source EventTiming.cc:113-115: a dispatcher without a LocalFrame does
    /// not obtain a WindowPerformance or register an entry.
    pub fn ForAbsentFrame() -> Self {
        Self { timing: None }
    }

    pub fn new(performance: Rc<WindowPerformance>, event: EventRef) -> Self {
        let measured = {
            let e = event.borrow();
            let name = e.BaseEvent().r#type();
            name == "compositionend"
                || (e.BaseEvent().IsFullyTrusted()
                    && !matches!(
                        name,
                        "mousemove"
                            | "pointermove"
                            | "touchmove"
                            | "pointerrawupdate"
                            | "wheel"
                            | "drag"
                    )
                    && (e.IsMouseEvent()
                        || e.IsPointerEvent()
                        || e.IsTouchEvent()
                        || e.IsKeyboardEvent()
                        || e.IsWheelEvent()
                        || e.IsInputEvent()
                        || e.IsCompositionEvent()
                        || e.IsDragEvent()))
        };
        Self {
            timing: if measured {
                EventTiming::new(performance, event)
            } else {
                None
            },
        }
    }
    pub fn GetInteractionIdInfo(&self) -> Option<PerformanceTimelineEntryIdInfo> {
        self.timing.as_ref()?.GetInteractionIdInfo()
    }
}
