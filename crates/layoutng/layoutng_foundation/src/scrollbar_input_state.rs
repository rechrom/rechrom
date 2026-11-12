//! Scrollbar's retained input fields shared with its translated input methods.
//! Source core/scroll/scrollbar.h:271-276,296,309-314; scrollbar.cc:177-201,306-429,655-737.
//! The native Scrollbar owns one value; adapters borrow it for each call.
use crate::gfx;
use std::cell::Cell;
#[derive(Default)]
pub struct ScrollbarInputState {
    pub drag_origin: Cell<f32>,
    pub manipulation_on_compositor: Cell<bool>,
    pub pressed_pos: Cell<i32>,
    pub scroll_pos: Cell<f32>,
    pub dragging_document: Cell<bool>,
    pub document_drag_pos: Cell<i32>,
    pub injected_gesture_scroll_begin: Cell<bool>,
    pub pending_injected_delta: Cell<gfx::Vector2dF>,
    pub autoscroll_timer_generation: Cell<u64>,
    pub autoscroll_timer_active: Cell<bool>,
}
