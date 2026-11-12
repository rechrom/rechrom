//! Chromium performance_event_timing.h:37-235; .cc:140-202,421-431.
use super::performance_timeline_entry_id_generator::PerformanceTimelineEntryIdInfo as Id;
use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetRef};
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FallbackReason {
    None,
    UnexpectedFrameSource,
    VisibilityChange,
    ModalDialog,
    SwapPromiseBroken,
    MacOSArtificialEvent,
    DoesNotNeedNextPaint,
    WindowDestroyed,
    InteractionInterruptedByContextMenu,
}
#[derive(Clone, Copy, Debug)]
pub struct EventTimingReportingInfo {
    pub fallback_reason: FallbackReason,
    pub frame_index: u64,
    pub creation_time: TimeTicks,
    pub enqueued_to_main_thread_time: TimeTicks,
    pub processing_start_time: TimeTicks,
    pub processing_end_time: TimeTicks,
    pub commit_finish_time: TimeTicks,
    pub presentation_time: TimeTicks,
    pub fallback_time: TimeTicks,
    pub render_start_time: TimeTicks,
    pub key_code: Option<i32>,
    pub pointer_id: Option<i32>,
    pub prevent_counting_as_interaction: bool,
    pub is_processing_fully_nested_in_another_event: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserInteractionType {
    TapOrClick,
    Keyboard,
}
pub struct PerformanceEventTiming {
    pub name: String,
    pub cancelable: bool,
    pub navigation_id: u64,
    pub reporting_info: RefCell<EventTimingReportingInfo>,
    interaction_id: Cell<Option<Id>>,
    target: RefCell<Option<Weak<dyn interaction_dispatch::core::dom::events::event::EventTarget>>>,
}
impl PerformanceEventTiming {
    pub fn Create(
        event: EventRef,
        reporting_info: EventTimingReportingInfo,
        navigation_id: u64,
    ) -> Rc<Self> {
        let (name, cancelable) = {
            let e = event.borrow();
            (
                e.BaseEvent().r#type().to_owned(),
                e.BaseEvent().cancelable(),
            )
        };
        Rc::new(Self {
            name,
            cancelable,
            navigation_id,
            reporting_info: RefCell::new(reporting_info),
            interaction_id: Cell::new(None),
            target: RefCell::new(None),
        })
    }
    pub fn GetInteractionIdInfo(&self) -> Option<Id> {
        if self.reporting_info.borrow().prevent_counting_as_interaction {
            Some(Id::NONE)
        } else {
            self.interaction_id.get()
        }
    }
    pub fn SetInteractionIdInfo(&self, id: Id) {
        self.interaction_id.set(Some(id));
    }
    pub fn HasInteractionId(&self) -> bool {
        self.interaction_id.get().is_some()
    }
    pub fn IsInteraction(&self) -> bool {
        self.GetInteractionIdInfo().is_some_and(|id| id != Id::NONE)
    }
    pub fn InteractionType(&self) -> UserInteractionType {
        if self
            .reporting_info
            .borrow()
            .pointer_id
            .is_some_and(|p| p != -1)
        {
            UserInteractionType::TapOrClick
        } else {
            UserInteractionType::Keyboard
        }
    }
    pub fn SetTarget(&self, target: &EventTargetRef) {
        *self.target.borrow_mut() = (target.Kind()
            == interaction_dispatch::core::dom::events::event::EventTargetKind::Node)
            .then(|| Rc::downgrade(target));
    }
    pub fn Target(&self) -> Option<EventTargetRef> {
        self.target.borrow().as_ref()?.upgrade()
    }
    pub fn HasKnownEndTime(&self) -> bool {
        let i = self.reporting_info.borrow();
        !i.presentation_time.is_null() || !i.fallback_time.is_null()
    }
    pub fn IsReadyForReporting(&self) -> bool {
        !self.reporting_info.borrow().processing_end_time.is_null()
            && self.HasKnownEndTime()
            && self.HasInteractionId()
    }
    pub fn GetStartTime(&self) -> TimeTicks {
        self.reporting_info.borrow().creation_time
    }
    pub fn GetEndTime(&self) -> TimeTicks {
        assert!(self.HasKnownEndTime());
        let i = self.reporting_info.borrow();
        if !i.fallback_time.is_null() {
            i.fallback_time
        } else {
            i.presentation_time
        }
    }
    /// WindowPerformance::FlushEventTiming rounds duration to 8 ms before
    /// notifying observers; bindings project this source value unchanged.
    pub fn DurationMilliseconds(&self) -> f64 {
        ((self.GetEndTime().ToInternalValue() - self.GetStartTime().ToInternalValue()) as f64
            / 8000.0)
            .round()
            * 8.0
    }
    pub fn UpdateFallbackTime(&self, time: TimeTicks, reason: FallbackReason) {
        let mut i = self.reporting_info.borrow_mut();
        if i.fallback_time.is_null() || time < i.fallback_time {
            i.fallback_time = time;
            i.fallback_reason = reason;
        }
    }
    pub fn NeedsNextPaintMeasurement(&self) -> bool {
        let i = self.reporting_info.borrow();
        i.fallback_time.is_null() && !i.processing_end_time.is_null()
    }
}
