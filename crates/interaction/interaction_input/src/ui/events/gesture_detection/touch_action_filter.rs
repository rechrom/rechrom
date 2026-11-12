// Copyright 2013 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: components/input/touch_action_filter.cc:22-53,97-279,381-448.
use crate::public::common::input::web_gesture_event::WebGestureEvent;
use crate::public::common::input::web_input_event::Type;

pub use crate::cc::input::touch_action::TouchAction;

#[derive(Default)]
pub(super) struct TouchActionFilter {
    action: TouchAction,
    drop_scroll: bool,
    drop_pinch: bool,
    drop_tap_end: bool,
    allow_double_tap: bool,
}
impl TouchActionFilter {
    pub fn SetTouchAction(&mut self, action: TouchAction) {
        self.action = action;
    }
    pub fn Filter(&mut self, event: &mut WebGestureEvent) -> bool {
        use Type::*;
        let action = self.action;
        match event.GetType() {
            kGestureScrollBegin => {
                let data = &mut event.data.scroll_begin;
                self.drop_scroll = if data.pointer_count >= 2 {
                    !action.has(TouchAction::kPinchZoom)
                } else if data.delta_x_hint == 0. && data.delta_y_hint == 0. {
                    false
                } else if data.delta_x_hint.abs() > data.delta_y_hint.abs() {
                    !action.has(if data.delta_x_hint > 0. {
                        TouchAction::kPanLeft
                    } else {
                        TouchAction::kPanRight
                    })
                } else {
                    !action.has(if data.delta_y_hint > 0. {
                        TouchAction::kPanUp
                    } else {
                        TouchAction::kPanDown
                    })
                };
                if !self.drop_scroll
                    && data.pointer_count == 1
                    && data.delta_x_hint.abs() > data.delta_y_hint.abs()
                    && !action.has(TouchAction::kInternalPanXScrolls)
                {
                    data.cursor_control = true;
                }
                return self.drop_scroll;
            }
            kGestureScrollUpdate => {
                if self.drop_scroll {
                    return true;
                }
                if action.has(TouchAction::kPanX) && !action.has(TouchAction::kPanY) {
                    event.data.scroll_update.delta_y = 0.;
                } else if action.has(TouchAction::kPanY) && !action.has(TouchAction::kPanX) {
                    event.data.scroll_update.delta_x = 0.;
                }
            }
            kGestureScrollEnd => {
                let drop = self.drop_scroll;
                self.drop_scroll = false;
                return drop;
            }
            kGesturePinchBegin => {
                self.drop_pinch = !action.has(TouchAction::kPinchZoom);
                return self.drop_pinch;
            }
            kGesturePinchUpdate => return self.drop_pinch,
            kGesturePinchEnd => {
                let drop = self.drop_pinch;
                self.drop_pinch = false;
                return drop;
            }
            kGestureDoubleTap => {
                if !self.allow_double_tap {
                    event.SetType(kGestureTap);
                    event.data.tap.tap_count = 2;
                }
                self.allow_double_tap = true;
            }
            kGestureTapUnconfirmed => {
                self.allow_double_tap = action.has(TouchAction::kDoubleTapZoom);
                if !self.allow_double_tap {
                    event.SetType(kGestureTap);
                    self.drop_tap_end = true;
                }
            }
            kGestureTap | kGestureTapCancel if self.drop_tap_end => {
                self.drop_tap_end = false;
                return true;
            }
            _ => {}
        }
        false
    }
}
