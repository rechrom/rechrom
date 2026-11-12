//! Source: cursor_control_handler.h:20-44; cursor_control_handler.cc:11-60.
//! The impl proxy forwards this exceptional scroll sequence to the widget's
//! main-thread cursor handler and drops inertial updates.
use crate::public::common::input::{
    web_gesture_event::{InertialPhaseState, WebGestureEvent},
    web_input_event::Type,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorControlDisposition {
    MainThread,
    Drop,
}
#[derive(Default)]
pub struct CursorControlHandler {
    cursor_control_in_progress: bool,
}
impl CursorControlHandler {
    pub fn ObserveInputEvent(
        &mut self,
        event: &WebGestureEvent,
    ) -> Option<CursorControlDisposition> {
        match event.GetType() {
            Type::kGestureScrollBegin if event.data.scroll_begin.cursor_control => {
                self.cursor_control_in_progress = true;
                Some(CursorControlDisposition::MainThread)
            }
            Type::kGestureScrollUpdate if self.cursor_control_in_progress => Some(
                if event.data.scroll_update.inertial_phase == InertialPhaseState::kMomentum {
                    CursorControlDisposition::Drop
                } else {
                    CursorControlDisposition::MainThread
                },
            ),
            Type::kGestureScrollEnd if self.cursor_control_in_progress => {
                self.cursor_control_in_progress = false;
                Some(CursorControlDisposition::MainThread)
            }
            _ => None,
        }
    }
}
/// WebFrameWidgetImpl::WillHandleGestureEvent, cc:3261-3300. This state
/// belongs to the main widget, separately from the impl forwarding owner.
#[derive(Default)]
pub struct WidgetCursorControl {
    swipe_to_move_cursor_activated: bool,
}
impl WidgetCursorControl {
    pub fn WillHandleGestureEvent(&mut self, event: &WebGestureEvent) -> bool {
        match event.GetType() {
            Type::kGestureScrollBegin if event.data.scroll_begin.cursor_control => {
                self.swipe_to_move_cursor_activated = true;
                true
            }
            Type::kGestureScrollUpdate if self.swipe_to_move_cursor_activated => true,
            Type::kGestureScrollEnd if self.swipe_to_move_cursor_activated => {
                self.swipe_to_move_cursor_activated = false;
                true
            }
            _ => false,
        }
    }
}
