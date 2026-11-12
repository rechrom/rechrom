//! Wheel dispatch acknowledgement at the Page/compositor boundary.
//!
//! Chromium does not apply a second Blink pixel scroll after wheel dispatch.
//! MouseWheelEventQueue generates gesture scrolling from the original wheel
//! only after its DOM acknowledgement; cc::InputHandler owns target latching,
//! scroll consumption and overscroll. Page routes these small values back to
//! the original compositor request, which retains the wheel and its phase.
//!
//! Source: components/input/mouse_wheel_event_queue.cc:95-124,126-271
//! Source: third_party/blink/renderer/core/input/mouse_wheel_event_manager.cc:41-125
//! Source: third_party/blink/renderer/platform/widget/input/input_handler_proxy.cc:1257-1302
//! Source: cc/input/input_handler.cc:2175-2314,2321-2387

use crate::input_event::WheelEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WheelDOMAcknowledgement {
    pub request: u64,
    pub default_prevented: bool,
}

/// A coalesced DOM event acknowledges every original impl-owned wheel with the
/// same cancellation result. No replacement scroll state or geometry is read.
pub fn WheelDOMAcknowledgements(
    wheel: &WheelEvent,
    default_prevented: bool,
) -> impl Iterator<Item = WheelDOMAcknowledgement> + '_ {
    wheel
        .compositor_ack
        .iter()
        .copied()
        .map(move |request| WheelDOMAcknowledgement {
            request,
            default_prevented,
        })
}
