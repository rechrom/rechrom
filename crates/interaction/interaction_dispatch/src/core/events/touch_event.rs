// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Source: third_party/blink/renderer/core/events/touch_event.{h,cc}
// Source: third_party/blink/renderer/core/input/touch_event_manager.cc
// Immutable list owners replace GC TouchList; all targets retain DOM identity.
use super::{
    ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef},
    ui_event_with_key_state::UIEventWithKeyState,
};
use crate::core::dom::events::event::{Bubbles, Cancelable, Event, EventInterface, EventTargetRef};
use crate::core::dom::events::{
    event_dispatch_result::DispatchEventResult,
    event_dispatcher::{EventDispatchHook, EventDispatcher},
    event_path as path,
};
use interaction_input::public::common::input::{
    web_coalesced_input_event::WebCoalescedInputEvent,
    web_input_event::{DispatchType, WebInputEvent},
    web_pointer_properties::PointF,
    web_touch_event::WebTouchEvent,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Touch {
    pub identifier: i32,
    pub target: EventTargetRef,
    pub screen_position: PointF,
    pub document_position: PointF,
    pub client_position: PointF,
    pub absolute_location: PointF,
    pub radius_x: f32,
    pub radius_y: f32,
    pub rotation_angle: f32,
    pub force: f32,
}
pub type TouchList = Rc<Vec<Rc<Touch>>>;

/// The same canonical input payload produced by TouchEventManager. Its
/// ui::LatencyInfo is empty in that source constructor and is not read here.
pub type NativeTouchInputEvent = WebCoalescedInputEvent<()>;
pub struct TouchEvent {
    key_state: UIEventWithKeyState,
    touches: path::TouchListRef,
    target_touches: path::TouchListRef,
    changed_touches: path::TouchListRef,
    pub native_event: NativeTouchInputEvent,
    pub current_touch_action: u32,
}
impl TouchEvent {
    pub fn new(
        native_event: NativeTouchInputEvent,
        touches: TouchList,
        target_touches: TouchList,
        changed_touches: TouchList,
        name: &str,
        view: Option<AbstractViewRef>,
        source_capabilities: Option<InputDeviceCapabilitiesRef>,
        current_touch_action: u32,
    ) -> Self {
        let attributes = native_event
            .Event()
            .AsAny()
            .downcast_ref::<WebTouchEvent>()
            .expect("TouchEvent owns canonical WebTouchEvent");
        debug_assert!(WebInputEvent::IsTouchEventType(attributes.GetType()));
        let mut event = Self {
            key_state: UIEventWithKeyState::new(
                name,
                Bubbles::kYes,
                if attributes.dispatch_type == DispatchType::kBlocking {
                    Cancelable::kYes
                } else {
                    Cancelable::kNo
                },
                view,
                0,
                attributes.GetModifiers(),
                attributes.TimeStamp(),
                source_capabilities,
            ),
            touches: list(touches),
            target_touches: list(target_touches),
            changed_touches: list(changed_touches),
            native_event,
            current_touch_action,
        };
        event
            .BaseEventMut()
            .SetEventPathClass(crate::core::dom::events::event::EventPathClass::Touch);
        event
    }
    pub fn touches(&self) -> &path::TouchListRef {
        &self.touches
    }
    pub fn targetTouches(&self) -> &path::TouchListRef {
        &self.target_touches
    }
    pub fn changedTouches(&self) -> &path::TouchListRef {
        &self.changed_touches
    }
    pub fn IsTouchStartOrFirstTouchMove(&self) -> bool {
        self.native_event
            .Event()
            .AsAny()
            .downcast_ref::<WebTouchEvent>()
            .expect("canonical WebTouchEvent")
            .touch_start_or_first_touch_move
    }
}
impl EventInterface for TouchEvent {
    fn AsTouchEvent(&self) -> Option<&TouchEvent> {
        Some(self)
    }
    fn PrepareDispatchHook<'a>(
        &self,
        _nested: &'a dyn super::mouse_event::MouseNestedDispatch,
    ) -> Box<dyn EventDispatchHook + 'a> {
        Box::new(TouchDispatchHook)
    }
    fn BaseEvent(&self) -> &Event {
        self.key_state.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.key_state.BaseEventMut()
    }
    fn InterfaceName(&self) -> &str {
        "TouchEvent"
    }
    fn IsTouchEvent(&self) -> bool {
        true
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    fn AsUIEventWithKeyState(&self) -> Option<&UIEventWithKeyState> {
        Some(&self.key_state)
    }
}

// Source: touch.cc CloneWithNewTarget and touch_list.cc.
impl crate::core::dom::events::event_path::Touch for Touch {
    fn ClientX(&self) -> f64 {
        self.client_position.x() as f64
    }
    fn AbsoluteLocation(&self) -> PointF {
        self.absolute_location
    }
    fn target(&self) -> Option<EventTargetRef> {
        Some(self.target.clone())
    }
    fn CloneWithNewTarget(
        &self,
        target: EventTargetRef,
    ) -> crate::core::dom::events::event_path::TouchRef {
        Rc::new(Self {
            target,
            ..self.clone()
        })
    }
}

struct TouchValues(Vec<path::TouchRef>);
impl path::TouchList for TouchValues {
    fn length(&self) -> usize {
        self.0.len()
    }
    fn item(&self, index: usize) -> path::TouchRef {
        self.0[index].clone()
    }
    fn Append(&mut self, touch: path::TouchRef) {
        self.0.push(touch);
    }
}
fn list(values: TouchList) -> path::TouchListRef {
    Rc::new(RefCell::new(TouchValues(
        values
            .iter()
            .map(|touch| touch.clone() as path::TouchRef)
            .collect(),
    )))
}
fn empty_list() -> path::TouchListRef {
    Rc::new(RefCell::new(TouchValues(Vec::new())))
}
impl path::TouchEvent for TouchEvent {
    fn touches(&self) -> Option<path::TouchListRef> {
        Some(self.touches.clone())
    }
    fn targetTouches(&self) -> Option<path::TouchListRef> {
        Some(self.target_touches.clone())
    }
    fn changedTouches(&self) -> Option<path::TouchListRef> {
        Some(self.changed_touches.clone())
    }
}
// Source: touch_event_context.cc. Scope-specific lists replace the same event's
// list views immediately before that scope's listeners are called.
struct TouchContext {
    touches: path::TouchListRef,
    target: path::TouchListRef,
    changed: path::TouchListRef,
}
impl path::TouchEventContext for TouchContext {
    fn Touches(&self) -> path::TouchListRef {
        self.touches.clone()
    }
    fn TargetTouches(&self) -> path::TouchListRef {
        self.target.clone()
    }
    fn ChangedTouches(&self) -> path::TouchListRef {
        self.changed.clone()
    }
    fn HandleLocalEvents(&self, event: &crate::core::dom::events::event::EventRef) {
        let owner = event
            .Typed::<TouchEvent>()
            .expect("canonical TouchEvent owner");
        let mut touch = owner.borrow_mut();
        touch.touches = self.touches.clone();
        touch.target_touches = self.target.clone();
        touch.changed_touches = self.changed.clone();
    }
}
struct TouchContextFactory;
impl path::TouchEventContextFactory for TouchContextFactory {
    fn Create(&self) -> path::TouchEventContextRef {
        Rc::new(TouchContext {
            touches: empty_list(),
            target: empty_list(),
            changed: empty_list(),
        })
    }
}
struct TouchDispatchHook;
impl EventDispatchHook for TouchDispatchHook {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult {
        let event = dispatcher.GetEvent();
        if event.borrow().BaseEvent().isTrusted() {
            let touch = event
                .Typed::<TouchEvent>()
                .expect("canonical TouchEvent owner");
            dispatcher
                .GetEventPath()
                .borrow()
                .AdjustForTouchEvent(&*touch.borrow(), &TouchContextFactory);
        }
        dispatcher.Dispatch()
    }
}
