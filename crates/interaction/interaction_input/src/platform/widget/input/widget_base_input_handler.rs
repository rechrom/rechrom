// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/widget/input/widget_base_input_handler.cc:48-88

use crate::public::common::input::{
    web_coalesced_input_event::WebCoalescedInputEvent,
    web_input_event::WebInputEventPayload,
    web_pointer_event::{TouchState, WebPointerEvent},
    web_touch_event::WebTouchEvent,
};

// Convert each matching nonstationary touch sample for this pointer only.
// The caller dispatches each point before preparing the next one, retaining
// WidgetBaseInputHandler::HandleTouchEvent's synchronous listener ordering.
pub fn GetCoalescedWebPointerEventForTouch<L: Clone>(
    pointer: &WebPointerEvent,
    coalesced: &[Box<dyn WebInputEventPayload>],
    predicted: &[Box<dyn WebInputEventPayload>],
    latency: &L,
) -> WebCoalescedInputEvent<L> {
    GetCoalescedWebPointerEventForTouchValues(
        pointer,
        coalesced.iter().map(|event| event.as_ref()),
        predicted.iter().map(|event| event.as_ref()),
        latency,
    )
}

pub fn GetCoalescedWebPointerEventForTouchValues<'a, L: Clone>(
    pointer: &WebPointerEvent,
    coalesced: impl IntoIterator<Item = &'a dyn WebInputEventPayload>,
    predicted: impl IntoIterator<Item = &'a dyn WebInputEventPayload>,
    latency: &L,
) -> WebCoalescedInputEvent<L> {
    fn related<'a>(
        pointer: &WebPointerEvent,
        events: impl IntoIterator<Item = &'a dyn WebInputEventPayload>,
    ) -> Vec<Box<dyn WebInputEventPayload>> {
        let mut pointers: Vec<Box<dyn WebInputEventPayload>> = Vec::new();
        for event in events {
            let touch = event
                .AsAny()
                .downcast_ref::<WebTouchEvent>()
                .expect("touch sample");
            for point in &touch.touches[..touch.touches_length] {
                if point.pointer_properties.id == pointer.pointer_properties.id
                    && point.state != TouchState::kStateStationary
                {
                    pointers.push(Box::new(WebPointerEvent::FromTouch(touch, point)));
                }
            }
        }
        pointers
    }
    WebCoalescedInputEvent::WithEvents(
        Box::new(pointer.clone()),
        related(pointer, coalesced),
        related(pointer, predicted),
        latency,
    )
}
