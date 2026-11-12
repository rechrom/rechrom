// Copyright 2014 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: ui/events/gesture_detection/scale_gesture_detector.cc:111-278.
// Span and pinch transitions use the same retained provider state.
use super::gesture_provider::GestureProvider;
use crate::public::common::input::web_gesture_event::WebGestureEvent;
use crate::public::common::input::web_input_event::Type;
use crate::public::common::input::web_touch_event::WebTouchEvent;
use crate::public::common::input::web_touch_point::TouchState;
use foundation::gfx_geometry::PointF;

impl GestureProvider {
    pub(super) fn scale(&mut self, e: &WebTouchEvent, events: &mut Vec<WebGestureEvent>) {
        let (x, y, n) = Self::focus(e, true);
        if e.GetType() == Type::kTouchCancel || e.IsTouchSequenceEnd() {
            if self.pinching {
                let index = events
                    .iter()
                    .position(|g| {
                        matches!(
                            g.GetType(),
                            Type::kGestureScrollEnd | Type::kGestureFlingStart
                        )
                    })
                    .unwrap_or(events.len());
                events.insert(index, self.make(Type::kGesturePinchEnd, e));
                self.pinching = false;
            }
            return;
        }
        let changed = matches!(
            e.GetType(),
            Type::kTouchStart | Type::kTouchEnd | Type::kTouchCancel
        );
        if self.pinching && (changed || n < 2) {
            events.push(self.make(Type::kGesturePinchEnd, e));
            self.pinching = false;
        }
        if n < 2 {
            self.initial_span = 0.;
            self.previous_span = 0.;
            return;
        }
        let mut sx = 0.;
        let mut sy = 0.;
        for p in &e.touches[..e.touches_length] {
            if matches!(
                p.state,
                TouchState::kStateReleased | TouchState::kStateCancelled
            ) {
                continue;
            }
            let pos = p.pointer_properties.PositionInWidget();
            sx += (pos.x() - x).abs();
            sy += (pos.y() - y).abs();
        }
        let span = (sx * sx + sy * sy).sqrt() * 2. / n as f32;
        if changed {
            self.initial_span = span;
            self.previous_span = span;
        }
        if !self.pinching
            && span.max(self.initial_span) >= self.config.min_scaling_span + 0.05
            && (span - self.initial_span).abs() > self.config.span_slop + 0.05
        {
            self.previous_span = (self.initial_span
                + (span - self.initial_span).signum() * self.config.span_slop)
                .max(self.config.min_scaling_span);
            self.pinching = true;
            if !self.scrolling {
                let mut begin = self.make(Type::kGestureScrollBegin, e);
                begin.data.scroll_begin.pointer_count = n as i32;
                begin.SetPositionInWidget(PointF::new(x, y));
                events.push(begin);
                self.scrolling = true;
            }
            let mut g = self.make(Type::kGesturePinchBegin, e);
            g.SetPositionInWidget(PointF::new(x, y));
            events.push(g);
        }
        if self.pinching
            && e.GetType() == Type::kTouchMove
            && (span - self.previous_span).abs() >= self.config.min_pinch_update_span_delta
            && self.previous_span != 0.
        {
            let mut g = self.make(Type::kGesturePinchUpdate, e);
            g.SetPositionInWidget(PointF::new(x, y));
            g.data.pinch_update.scale = span / self.previous_span;
            events.push(g);
            self.previous_span = span;
        }
        if self.pinching && span < self.config.min_scaling_span + 0.05 {
            events.push(self.make(Type::kGesturePinchEnd, e));
            self.pinching = false;
            self.initial_span = span;
            self.previous_span = span;
        }
    }
    // Source: scale_gesture_detector.cc:88-103.
    pub(super) fn CancelPinchDetection(
        &self,
        e: &WebTouchEvent,
        events: &mut Vec<WebGestureEvent>,
    ) {
        if self.pinching {
            events.push(self.make(Type::kGesturePinchEnd, e));
        }
    }
    pub(super) fn ResetScaleGestureDetector(&mut self) {
        self.pinching = false;
    }
}
