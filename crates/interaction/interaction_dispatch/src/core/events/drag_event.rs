// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/events/drag_event.h:18-67; drag_event.cc:14-46.
use super::{
    mouse_event::{
        MouseCoordinateContext, MouseEvent, MouseEventInit, MouseEventInterface,
        MouseNestedDispatch, SyntheticEventType,
    },
    ui_event::{UIEvent, UIEventInterface},
    ui_event_with_key_state::{UIEventWithKeyState, UIEventWithKeyStateInterface},
};
use crate::core::{
    clipboard::data_transfer::DataTransfer,
    dom::events::{
        event::{
            Event, EventDispatcher as EventDispatcherInterface, EventInterface, EventTargetRef,
        },
        event_dispatch_result::DispatchEventResult,
        event_dispatcher::{EventDispatchHook, EventDispatcher},
    },
};
use interaction_input::{
    base::time::time::TimeTicks, ui::base::mojom::menu_source_type::MenuSourceType,
};
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// MouseEvent is a base subobject; this owner remains a DragEvent throughout
/// dispatch and retains the exact DataTransfer supplied by DragState.
pub struct DragEvent {
    mouse_event: MouseEvent,
    data_transfer: Option<Rc<DataTransfer>>,
}
impl DragEvent {
    pub fn new(
        event_type: impl Into<String>,
        initializer: &MouseEventInit,
        data_transfer: Option<Rc<DataTransfer>>,
        timestamp: TimeTicks,
        synthetic: SyntheticEventType,
        coordinates: MouseCoordinateContext,
    ) -> Self {
        Self {
            mouse_event: MouseEvent::new(
                event_type,
                initializer,
                timestamp,
                synthetic,
                MenuSourceType::kNone,
                None,
                coordinates,
            ),
            data_transfer,
        }
    }
    pub fn dataTransfer(&self) -> Option<Rc<DataTransfer>> {
        self.data_transfer.clone()
    }
}
impl Deref for DragEvent {
    type Target = MouseEvent;
    fn deref(&self) -> &MouseEvent {
        &self.mouse_event
    }
}
impl DerefMut for DragEvent {
    fn deref_mut(&mut self) -> &mut MouseEvent {
        &mut self.mouse_event
    }
}
impl EventInterface for DragEvent {
    fn PrepareDispatchHook<'a>(
        &self,
        _: &'a dyn MouseNestedDispatch,
    ) -> Box<dyn EventDispatchHook + 'a> {
        Box::new(DragEventDispatchHook)
    }
    fn AsDragEvent(&self) -> Option<&DragEvent> {
        Some(self)
    }
    fn AsMouseEvent(&self) -> Option<&MouseEvent> {
        Some(&self.mouse_event)
    }
    fn AsUIEventWithKeyState(&self) -> Option<&UIEventWithKeyState> {
        Some(self.mouse_event.BaseUIEventWithKeyState())
    }
    fn BaseEvent(&self) -> &Event {
        self.mouse_event.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.mouse_event.BaseEventMut()
    }
    fn InterfaceName(&self) -> &str {
        "DragEvent"
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    fn IsMouseEvent(&self) -> bool {
        false
    }
    fn IsDragEvent(&self) -> bool {
        true
    }
    fn relatedTarget(&self) -> Option<EventTargetRef> {
        self.mouse_event.relatedTarget()
    }
    fn SetRelatedTarget(&mut self, target: Option<EventTargetRef>) {
        self.mouse_event.SetRelatedTarget(target);
    }
    fn ReceivedTarget(&mut self) {
        self.mouse_event.ReceivedTarget();
    }
    fn DispatchEvent(
        &mut self,
        dispatcher: &mut dyn EventDispatcherInterface,
    ) -> DispatchEventResult {
        dispatcher.Dispatch()
    }
}
impl UIEventInterface for DragEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        self.mouse_event.BaseUIEvent()
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        self.mouse_event.BaseUIEventMut()
    }
    fn which(&self) -> u32 {
        self.mouse_event.which()
    }
}
impl UIEventWithKeyStateInterface for DragEvent {
    fn BaseUIEventWithKeyState(&self) -> &UIEventWithKeyState {
        self.mouse_event.BaseUIEventWithKeyState()
    }
    fn BaseUIEventWithKeyStateMut(&mut self) -> &mut UIEventWithKeyState {
        self.mouse_event.BaseUIEventWithKeyStateMut()
    }
}
impl MouseEventInterface for DragEvent {
    fn BaseMouseEvent(&self) -> &MouseEvent {
        &self.mouse_event
    }
    fn BaseMouseEventMut(&mut self) -> &mut MouseEvent {
        &mut self.mouse_event
    }
}
/// drag_event.cc:41-45. This deliberately does not run the MouseEvent virtual
/// hook (which synthesizes click/dblclick and adjusts disabled controls).
pub struct DragEventDispatchHook;
impl EventDispatchHook for DragEventDispatchHook {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult {
        let related = dispatcher.GetEvent().borrow().relatedTarget();
        dispatcher
            .GetEventPath()
            .borrow_mut()
            .AdjustForRelatedTarget(&dispatcher.GetNode(), related);
        dispatcher.Dispatch()
    }
}
