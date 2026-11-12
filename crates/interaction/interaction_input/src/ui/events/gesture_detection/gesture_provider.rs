// Copyright 2014 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: ui/events/gesture_detection/gesture_provider.h;
// gesture_provider.cc:138-179,387-503,546-640,743-840,890-916,972-1060.
// The existing recognizer state and event order are retained. Source-named
// detector impls borrow this same identity; no detector state is copied.
pub use super::gesture_configuration::GestureProviderConfig;
use super::velocity_tracker::VelocityTracker;
use crate::base::time::time::TimeTicks;
use crate::public::common::input::web_gesture_event::GestureDevice;
use crate::public::common::input::web_gesture_event::WebGestureEvent;
use crate::public::common::input::web_input_event::DispatchType;
use crate::public::common::input::web_input_event::Type;
use crate::public::common::input::web_pointer_properties::PointerType;
use crate::public::common::input::web_touch_event::WebTouchEvent;
pub(super) use crate::public::common::input::web_touch_point::TouchState;
use std::collections::HashMap;

pub(super) struct GestureProvider {
    pub config: GestureProviderConfig,
    pub(super) down: Option<WebTouchEvent>,
    pub(super) previous_up: Option<WebTouchEvent>,
    pub(super) origins: HashMap<i32, (f32, f32, f32)>,
    pub(super) last_focus: (f32, f32),
    pub(super) max_pointers: usize,
    pub(super) within_slop: bool,
    pub(super) in_bigger_tap_region: bool,
    pub(super) scrolling: bool,
    pub(super) longpress: bool,
    pub(super) double_tapping: bool,
    pub(super) tap_count: i32,
    pub(super) deadlines: [Option<i64>; 4],
    pub(super) velocity: VelocityTracker,
    pub(super) initial_span: f32,
    pub(super) previous_span: f32,
    pub(super) pinching: bool,
}
impl GestureProvider {
    pub fn new(config: GestureProviderConfig) -> Self {
        Self {
            config,
            down: None,
            previous_up: None,
            origins: HashMap::new(),
            last_focus: (0., 0.),
            max_pointers: 0,
            within_slop: false,
            in_bigger_tap_region: false,
            scrolling: false,
            longpress: false,
            double_tapping: false,
            tap_count: 0,
            deadlines: [None; 4],
            velocity: VelocityTracker::default(),
            initial_span: 0.,
            previous_span: 0.,
            pinching: false,
        }
    }
    pub fn PrimaryTouchId(&self) -> u32 {
        self.down.as_ref().map_or(0, |e| e.unique_touch_event_id)
    }
    pub fn ResetDetection(&mut self, now: TimeTicks) -> Vec<WebGestureEvent> {
        let mut cancel = WebTouchEvent::new(Type::kTouchCancel, 0, now);
        cancel.touches_length = 1;
        cancel.touches[0].state = TouchState::kStateCancelled;
        self.OnTouchEvent(&cancel).unwrap_or_default()
    }
    pub(super) fn make(&self, kind: Type, e: &WebTouchEvent) -> WebGestureEvent {
        let mut g = WebGestureEvent::new(
            kind,
            e.GetModifiers(),
            e.TimeStamp(),
            GestureDevice::kTouchscreen,
        );
        g.unique_touch_event_id = e.unique_touch_event_id;
        g.primary_unique_touch_event_id = self
            .down
            .as_ref()
            .map_or(e.unique_touch_event_id, |d| d.unique_touch_event_id);
        g.is_source_touch_event_set_blocking = e.dispatch_type == DispatchType::kBlocking;
        *g.GetModifiableEventLatencyMetadata() = *e.GetEventLatencyMetadata();
        g.SetQueuedTimeStamp(e.QueuedTimeStamp());
        if let Some(p) = e.touches.first().filter(|_| e.touches_length != 0) {
            g.primary_pointer_type = p.pointer_properties.pointer_type;
            g.SetPositionInWidget(*p.pointer_properties.PositionInWidget());
            g.SetPositionInScreen(*p.pointer_properties.PositionInScreen());
            let (width, height) = (p.radius_x * 2., p.radius_y * 2.);
            g.data.tap.width = width;
            g.data.tap.height = height;
            g.data.tap_down.width = width;
            g.data.tap_down.height = height;
            g.data.show_press.width = width;
            g.data.show_press.height = height;
            g.data.long_press.width = width;
            g.data.long_press.height = height;
        }
        g
    }
    pub(super) fn IsWithinSlop(&self, e: &WebTouchEvent) -> bool {
        self.max_pointers <= 2
            && e.touches[..e.touches_length].iter().all(|p| {
                let Some((x, y, expand)) = self.origins.get(&p.pointer_properties.id) else {
                    return false;
                };
                let xy = p.pointer_properties.PositionInWidget();
                let slop = if p.pointer_properties.pointer_type == PointerType::kPen {
                    self.config.stylus_slop
                } else {
                    self.config.touch_slop
                } + 0.05;
                (xy.x() - x).powi(2) + (xy.y() - y).powi(2) <= slop * slop * expand
            })
    }
    // Source: gesture_provider.cc:766-840.
    pub(super) fn ComputeFirstScrollDelta(
        &self,
        e: &WebTouchEvent,
        pointer_count: usize,
        mut dx: f32,
        mut dy: f32,
    ) -> (f32, f32) {
        if self.max_pointers < 3 {
            dx = 0.;
            dy = 0.;
            for p in &e.touches[..e.touches_length] {
                let q = p.pointer_properties.PositionInWidget();
                let (x, y, _) = self.origins[&p.pointer_properties.id];
                let (px, py) = (q.x() - x, q.y() - y);
                let length = (px * px + py * py).sqrt();
                let ratio = if length > 1e-3 {
                    (length - self.config.touch_slop).max(0.) / length
                } else {
                    1.
                };
                dx += px * ratio;
                dy += py * ratio;
            }
            dx /= pointer_count as f32;
            dy /= pointer_count as f32;
        }
        (dx, dy)
    }
    // Source: gesture_provider.cc:138-179,890-908,972-1060.
    pub fn OnTouchEvent(&mut self, e: &WebTouchEvent) -> Option<Vec<WebGestureEvent>> {
        let mut events = self.DetectTouchEvent(e)?;
        for p in &e.touches[..e.touches_length] {
            if p.state == TouchState::kStatePressed {
                let xy = p.pointer_properties.PositionInWidget();
                self.origins.insert(
                    p.pointer_properties.id,
                    (
                        xy.x(),
                        xy.y(),
                        if p.radius_x == 0. && p.radius_y == 0. {
                            4.
                        } else {
                            1.
                        },
                    ),
                );
            }
        }
        self.scale(e, &mut events);
        Some(events)
    }
    // Source: gesture_provider.cc:145-179. Preserve cancellation output and
    // state-reset order across the same retained source detector methods.
    pub(super) fn CancelDetection(&mut self, e: &WebTouchEvent, events: &mut Vec<WebGestureEvent>) {
        self.CancelTapDetection(events);
        self.CancelPinchDetection(e, events);
        if self.scrolling {
            events.push(self.make(Type::kGestureScrollEnd, e));
        }
        self.ResetGestureDetector();
        self.scrolling = false;
        self.ResetScaleGestureDetector();
    }
}
