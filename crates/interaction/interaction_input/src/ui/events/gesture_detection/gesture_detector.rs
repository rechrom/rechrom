// Copyright 2014 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: ui/events/gesture_detection/gesture_detector.h:37-120;
// gesture_detector.cc:165-463,501-600,635-690.
// Methods operate on the pre-existing retained recognizer identity.
use super::gesture_provider::GestureProvider;
use crate::base::time::time::TimeTicks;
use crate::public::common::input::web_gesture_event::InertialPhaseState;
use crate::public::common::input::web_gesture_event::WebGestureEvent;
use crate::public::common::input::web_input_event::Type;
use crate::public::common::input::web_touch_event::WebTouchEvent;
use crate::public::common::input::web_touch_point::TouchState;

impl GestureProvider {
    pub fn NextDeadline(&self) -> Option<TimeTicks> {
        self.deadlines
            .iter()
            .flatten()
            .min()
            .copied()
            .map(TimeTicks::FromInternalValue)
    }
    pub(super) fn focus(e: &WebTouchEvent, excluding_released: bool) -> (f32, f32, usize) {
        let mut sum = (0., 0., 0);
        for p in &e.touches[..e.touches_length] {
            if excluding_released
                && matches!(
                    p.state,
                    TouchState::kStateReleased | TouchState::kStateCancelled
                )
            {
                continue;
            }
            let xy = p.pointer_properties.PositionInWidget();
            sum.0 += xy.x();
            sum.1 += xy.y();
            sum.2 += 1;
        }
        if sum.2 != 0 {
            sum.0 /= sum.2 as f32;
            sum.1 /= sum.2 as f32;
        }
        sum
    }
    pub(super) fn DetectTouchEvent(&mut self, e: &WebTouchEvent) -> Option<Vec<WebGestureEvent>> {
        if e.touches_length == 0 || e.touches_length > WebTouchEvent::kTouchesLengthCap {
            return None;
        }
        let start = e.IsTouchSequenceStart();
        if !start
            && e.GetType() != Type::kTouchCancel
            && (self.down.is_none() || self.max_pointers == 0)
        {
            return None;
        }
        let mut events = Vec::new();
        self.velocity.AddMovement(e);
        let focus = Self::focus(e, e.GetType() == Type::kTouchEnd && !e.IsTouchSequenceEnd());
        match e.GetType() {
            Type::kTouchStart if start => {
                let repeated = self
                    .down
                    .as_ref()
                    .zip(self.previous_up.as_ref())
                    .is_some_and(|(down, up)| {
                        let elapsed =
                            e.TimeStamp().ToInternalValue() - up.TimeStamp().ToInternalValue();
                        let p = down.touches[0].pointer_properties.PositionInWidget();
                        let q = e.touches[0].pointer_properties.PositionInWidget();
                        self.in_bigger_tap_region
                            && elapsed <= self.config.double_tap_timeout_us
                            && (!self.config.double_tap_enabled
                                || elapsed >= self.config.double_tap_min_time_us)
                            && (p.x() - q.x()).powi(2) + (p.y() - q.y()).powi(2)
                                < (self.config.double_tap_slop + 0.05).powi(2)
                    });
                self.double_tapping =
                    self.config.double_tap_enabled && repeated && self.deadlines[3].is_some();
                if self.double_tapping {
                    self.deadlines[3] = None;
                }
                self.tap_count = if repeated && !self.config.double_tap_enabled {
                    self.tap_count % self.config.single_tap_repeat_interval.max(1) + 1
                } else {
                    1
                };
                self.down = Some(e.clone());
                self.previous_up = None;
                self.origins.clear();
                self.last_focus = (focus.0, focus.1);
                self.max_pointers = 1;
                self.within_slop = true;
                self.in_bigger_tap_region = true;
                self.scrolling = false;
                self.longpress = false;
                let t = e.TimeStamp().ToInternalValue();
                self.deadlines[0] = Some(t + self.config.showpress_timeout_us);
                self.deadlines[1] = (!self.double_tapping).then_some(
                    t + self.config.showpress_timeout_us + self.config.shortpress_timeout_us,
                );
                self.deadlines[2] = (!self.double_tapping).then_some(
                    t + self.config.showpress_timeout_us + self.config.longpress_timeout_us,
                );
                if self.config.double_tap_enabled && !self.double_tapping {
                    self.deadlines[3] = Some(t + self.config.double_tap_timeout_us);
                }
                let mut g = self.make(Type::kGestureTapDown, e);
                g.data.tap_down.tap_down_count = self.tap_count;
                events.push(g);
            }
            Type::kTouchStart => {
                self.deadlines = [None; 4];
                self.double_tapping = false;
                self.last_focus = (focus.0, focus.1);
                self.max_pointers = self.max_pointers.max(focus.2);
                self.in_bigger_tap_region = false;
                events.push(self.make(Type::kGestureTapCancel, e));
            }
            Type::kTouchMove => {
                let (mut dx, mut dy) = (focus.0 - self.last_focus.0, focus.1 - self.last_focus.1);
                let outside = !self.IsWithinSlop(e);
                if !self.double_tapping
                    && ((self.within_slop && outside)
                        || (!self.within_slop && (dx.abs() > 0.1 || dy.abs() > 0.1)))
                {
                    if !self.scrolling {
                        // GestureProvider::ComputeFirstScrollDelta subtracts slop
                        // independently before averaging the active pointers.
                        (dx, dy) = self.ComputeFirstScrollDelta(e, focus.2, dx, dy);
                        if dx != 0. || dy != 0. {
                            let mut begin = self.make(Type::kGestureScrollBegin, e);
                            let down = self.down.as_ref().unwrap();
                            begin.SetPositionInWidget(
                                *down.touches[0].pointer_properties.PositionInWidget(),
                            );
                            begin.SetPositionInScreen(
                                *down.touches[0].pointer_properties.PositionInScreen(),
                            );
                            begin.data.scroll_begin.delta_x_hint = dx;
                            begin.data.scroll_begin.delta_y_hint = dy;
                            begin.data.scroll_begin.pointer_count = focus.2 as i32;
                            begin.data.scroll_begin.inertial_phase =
                                InertialPhaseState::kNonMomentum;
                            events.push(begin);
                            self.scrolling = true;
                        }
                    }
                    if self.scrolling {
                        let mut update = self.make(Type::kGestureScrollUpdate, e);
                        update.data.scroll_update.delta_x = dx;
                        update.data.scroll_update.delta_y = dy;
                        update.data.scroll_update.delta_x_unconstrained = dx;
                        update.data.scroll_update.delta_y_unconstrained = dy;
                        update.data.scroll_update.inertial_phase = InertialPhaseState::kNonMomentum;
                        events.push(update);
                    }
                    self.last_focus = (focus.0, focus.1);
                    self.within_slop = false;
                    self.deadlines = [None; 4];
                }
                if let Some(down) = &self.down {
                    let p = down.touches[0].pointer_properties.PositionInWidget();
                    if (focus.0 - p.x()).powi(2) + (focus.1 - p.y()).powi(2)
                        > (self.config.touch_slop + 0.05).powi(2)
                    {
                        self.in_bigger_tap_region = false;
                    }
                }
            }
            Type::kTouchEnd if !e.IsTouchSequenceEnd() => {
                self.last_focus = (focus.0, focus.1);
                // Opposing velocities on release invalidate the fling estimate.
                for released in e.touches[..e.touches_length]
                    .iter()
                    .filter(|p| p.state == TouchState::kStateReleased)
                {
                    let a = self.velocity.Velocity(
                        released.pointer_properties.id,
                        self.config.maximum_fling_velocity,
                    );
                    if e.touches[..e.touches_length]
                        .iter()
                        .filter(|p| p.state != TouchState::kStateReleased)
                        .any(|p| {
                            let b = self.velocity.Velocity(
                                p.pointer_properties.id,
                                self.config.maximum_fling_velocity,
                            );
                            a.0 * b.0 + a.1 * b.1 < 0.
                        })
                    {
                        self.velocity.Clear();
                    }
                }
            }
            Type::kTouchEnd => {
                if self.double_tapping && self.within_slop {
                    let mut g = self.make(Type::kGestureDoubleTap, e);
                    g.data.tap.tap_count = 1;
                    events.push(g);
                } else if self.within_slop && self.max_pointers == 1 {
                    let kind = if self.longpress {
                        Type::kGestureLongTap
                    } else if self.config.double_tap_enabled && self.deadlines[3].is_some() {
                        Type::kGestureTapUnconfirmed
                    } else {
                        Type::kGestureTap
                    };
                    let mut g = self.make(kind, e);
                    g.data.tap.tap_count = self.tap_count;
                    events.push(g);
                } else if self.scrolling {
                    let (vx, vy) = self.velocity.Velocity(
                        e.touches[0].pointer_properties.id,
                        self.config.maximum_fling_velocity,
                    );
                    if vx.abs() > self.config.minimum_fling_velocity
                        || vy.abs() > self.config.minimum_fling_velocity
                    {
                        let mut g = self.make(Type::kGestureFlingStart, e);
                        g.data.fling_start.velocity_x = vx;
                        g.data.fling_start.velocity_y = vy;
                        events.push(g);
                    } else {
                        events.push(self.make(Type::kGestureScrollEnd, e));
                    }
                }
                self.previous_up = Some(e.clone());
                self.max_pointers = 0;
                self.deadlines[..3].fill(None);
                self.velocity.Clear();
                self.double_tapping = false;
            }
            Type::kTouchCancel => self.CancelDetection(e, &mut events),
            _ => return None,
        }
        Some(events)
    }
    pub fn OnTimeout(&mut self, now: TimeTicks) -> Vec<WebGestureEvent> {
        let mut result = Vec::new();
        for i in 0..4 {
            let Some(deadline) = self.deadlines[i] else {
                continue;
            };
            if deadline > now.ToInternalValue() {
                continue;
            }
            self.deadlines[i] = None;
            let e = if i == 3 {
                self.previous_up.as_ref()
            } else {
                self.down.as_ref()
            };
            let Some(e) = e else {
                continue;
            };
            let kind = match i {
                0 => Type::kGestureShowPress,
                1 => Type::kGestureShortPress,
                2 => Type::kGestureLongPress,
                _ => Type::kGestureTap,
            };
            let mut g = self.make(kind, e);
            if i == 3 {
                g.data.tap.tap_count = 1;
            }
            result.push(g);
            if i == 2 {
                self.longpress = true;
                self.deadlines = [None; 4];
            }
        }
        result
    }
    // Source: gesture_detector.cc:561-578.
    pub(super) fn CancelTapDetection(&self, events: &mut Vec<WebGestureEvent>) {
        if self.deadlines[3].is_some() {
            if let Some(down) = self.down.as_ref() {
                events.push(self.make(Type::kGestureTapCancel, down));
            }
        }
    }
    pub(super) fn ResetGestureDetector(&mut self) {
        self.deadlines = [None; 4];
        self.velocity.Clear();
        self.max_pointers = 0;
        self.within_slop = false;
        self.in_bigger_tap_region = false;
        self.down = None;
        self.double_tapping = false;
        self.tap_count = 0;
    }
}
