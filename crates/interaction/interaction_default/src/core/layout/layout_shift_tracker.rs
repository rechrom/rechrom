// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/layout_shift_tracker.h:237-268
// Source: third_party/blink/renderer/core/layout/layout_shift_tracker.cc:47,708-837,900-909
//! The retained input-notification portion of LocalFrameView's tracker.
//! Layout-shift geometry, scoring and PerformanceEntry publication remain with
//! the unimplemented prepaint observer; no shift values are synthesized here.

use interaction_dispatch::core::dom::events::event::EventClock;
use interaction_input::{
    base::time::time::TimeTicks,
    public::common::input::{
        web_input_event::{Type, WebInputEventPayload},
        web_pointer_event::WebPointerEvent,
    },
};
use std::{cell::Cell, rc::Rc, time::Duration};

const kTimerDelay: Duration = Duration::from_millis(500);

/// The document's kInternalDefault task runner owns this cancellable one-shot.
/// Its source TimerFired callback is empty: IsActive becomes false when the
/// ordinary timer task is taken, rather than when its wall-clock deadline passes.
pub trait LayoutShiftInputTimer {
    fn StartOneShot(&self, delay: Duration);
    fn IsActive(&self) -> bool;
    fn Stop(&self);
}

pub struct LayoutShiftInputTracker {
    clock: Rc<dyn EventClock>,
    timer: Rc<dyn LayoutShiftInputTimer>,
    timestamp_based_tracking: bool,
    num_pointerdowns: Cell<i32>,
    num_pressed_mouse_buttons: Cell<i32>,
    observed_input_or_scroll: Cell<bool>,
    most_recent_input_timestamp: Cell<TimeTicks>,
    most_recent_input_timestamp_initialized: Cell<bool>,
    most_recent_input_processing_timestamp: Cell<TimeTicks>,
}

impl LayoutShiftInputTracker {
    pub fn new(
        clock: Rc<dyn EventClock>,
        timer: Rc<dyn LayoutShiftInputTimer>,
        timestamp_based_tracking: bool,
    ) -> Self {
        Self {
            clock,
            timer,
            timestamp_based_tracking,
            num_pointerdowns: Cell::new(0),
            num_pressed_mouse_buttons: Cell::new(0),
            observed_input_or_scroll: Cell::new(false),
            most_recent_input_timestamp: Cell::new(TimeTicks::new()),
            most_recent_input_timestamp_initialized: Cell::new(false),
            most_recent_input_processing_timestamp: Cell::new(TimeTicks::new()),
        }
    }

    // Source: layout_shift_tracker.cc:708-763. Preserve release-before-exclusion
    // and reset-before-increment order, including unmatched up notifications.
    pub fn NotifyInput(&self, event: &dyn WebInputEventPayload) {
        let kind = event.Event().GetType();
        let mut release_all_mouse_buttons = false;
        if kind == Type::kMouseUp {
            if self.num_pressed_mouse_buttons.get() > 0 {
                self.num_pressed_mouse_buttons
                    .set(self.num_pressed_mouse_buttons.get() - 1);
            }
            release_all_mouse_buttons = self.num_pressed_mouse_buttons.get() == 0;
        }
        let mut release_all_pointers = false;
        if kind == Type::kPointerUp {
            if self.num_pointerdowns.get() > 0 {
                self.num_pointerdowns.set(self.num_pointerdowns.get() - 1);
            }
            release_all_pointers = self.num_pointerdowns.get() == 0;
        }
        let stops_pointerdown_buffering =
            matches!(kind, Type::kPointerCausedUaAction | Type::kPointerCancel);
        let hovering_pointerdown = kind == Type::kPointerDown
            && event
                .AsAny()
                .downcast_ref::<WebPointerEvent>()
                .expect("pointerdown requires its WebPointerEvent payload")
                .hovering;
        let should_trigger_shift_exclusion = matches!(
            kind,
            Type::kMouseDown | Type::kKeyDown | Type::kRawKeyDown | Type::kGestureTap
        ) || hovering_pointerdown
            || release_all_pointers
            || release_all_mouse_buttons;
        if should_trigger_shift_exclusion {
            self.observed_input_or_scroll.set(true);
            if !self.timestamp_based_tracking {
                self.timer.StartOneShot(kTimerDelay);
            }
            self.UpdateInputTimestamps(event.Event().TimeStamp());
        }
        if stops_pointerdown_buffering || release_all_mouse_buttons || release_all_pointers {
            // Chromium also reports accumulated prepaint score deltas here.
            // This input-only owner has no geometry observer or score buffer.
            self.num_pointerdowns.set(0);
            self.num_pressed_mouse_buttons.set(0);
        }
        if kind == Type::kPointerDown && !hovering_pointerdown {
            self.num_pointerdowns.set(self.num_pointerdowns.get() + 1);
        }
        if kind == Type::kMouseDown {
            self.num_pressed_mouse_buttons
                .set(self.num_pressed_mouse_buttons.get() + 1);
        }
    }

    // Source: layout_shift_tracker.cc:765-770. Original event time never moves
    // backwards, while the exclusion interval restarts at processing time.
    fn UpdateInputTimestamps(&self, timestamp: TimeTicks) {
        self.most_recent_input_timestamp_initialized.set(true);
        self.most_recent_input_timestamp
            .set(timestamp.max(self.most_recent_input_timestamp.get()));
        self.most_recent_input_processing_timestamp
            .set(self.clock.Now());
    }

    // Source: layout_shift_tracker.cc:772-788. The experimental branch uses
    // the current task's start, not the event timestamp or a fresh clock read.
    pub fn HasRecentInput(&self, current_task_start: TimeTicks) -> bool {
        if !self.timestamp_based_tracking {
            return self.timer.IsActive();
        }
        let processed = self.most_recent_input_processing_timestamp.get();
        if processed.is_null() {
            return false;
        }
        let elapsed =
            current_task_start.ToInternalValue() as i128 - processed.ToInternalValue() as i128;
        let recent = elapsed <= kTimerDelay.as_micros() as i128;
        if !recent {
            self.most_recent_input_processing_timestamp
                .set(TimeTicks::new());
        }
        recent
    }

    pub fn ObservedInputOrScroll(&self) -> bool {
        self.observed_input_or_scroll.get()
    }

    pub fn MostRecentInputTimestamp(&self) -> TimeTicks {
        self.most_recent_input_timestamp.get()
    }

    /// The initialized bit distinguishes an actual input at the null origin
    /// from the pre-input state when converting to a DOMHighResTimeStamp.
    pub fn InputTimestampInitialized(&self) -> bool {
        self.most_recent_input_timestamp_initialized.get()
    }

    pub fn IsBufferingPointerdown(&self) -> bool {
        self.num_pointerdowns.get() > 0 || self.num_pressed_mouse_buttons.get() > 0
    }

    pub fn Dispose(&self) {
        if !self.timestamp_based_tracking {
            self.timer.Stop();
        }
    }

    pub fn ResetTimerForTesting(&self) {
        self.most_recent_input_processing_timestamp
            .set(TimeTicks::new());
        self.timer.Stop();
    }
}

impl Drop for LayoutShiftInputTracker {
    fn drop(&mut self) {
        self.Dispose();
    }
}
