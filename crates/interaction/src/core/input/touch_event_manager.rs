// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/touch_event_manager.h:25-155
// Source: third_party/blink/renderer/core/input/touch_event_manager.cc:34-723
// Rust shared owners replace Member/GC handles. The call-scoped environment
// lends the original Document, Frame, registry and resident hit-test leaves.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use interaction_dispatch::core::{
    dom::events::{
        event::{EventRef, EventTargetRef},
        event_dispatch_result::DispatchEventResult,
    },
    events::{
        touch_event::{NativeTouchInputEvent, Touch, TouchEvent, TouchList},
        ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef},
    },
    input::event_handling_util::{MergeEventResult, ToWebInputEventResult, WebInputEventResult},
};
use interaction_input::public::common::input::{
    web_coalesced_input_event::WebCoalescedInputEvent,
    web_input_event::Type,
    web_pointer_event::WebPointerEvent,
    web_pointer_properties::PointF,
    web_touch_event::WebTouchEvent,
    web_touch_point::{TouchState, WebTouchPoint},
};

use super::pointer_event_manager::{PointerEventTarget, PointerTouchAction, PointerTouchManager};

/// Source virtual operations on the existing Frame/Document/registry owners.
/// Dispatch may synchronously reenter this manager; no state borrow crosses it.
pub trait TouchEventEnvironment {
    fn OwnerDocument(&self, node: &EventTargetRef) -> EventTargetRef;
    fn FrameHasView(&self, document: &EventTargetRef) -> bool;
    fn DocumentHasPage(&self, document: &EventTargetRef) -> bool;
    fn HasTouchHandlers(&self, document: &EventTargetRef) -> bool;
    fn ViewForDocument(&self, document: &EventTargetRef) -> Option<AbstractViewRef>;
    fn TouchCapabilitiesForDocument(
        &self,
        document: &EventTargetRef,
    ) -> Option<InputDeviceCapabilitiesRef>;
    fn LayoutZoomFactor(&self, document: &EventTargetRef) -> f32;
    fn RootFrameToDocument(&self, document: &EventTargetRef, point: PointF) -> PointF;
    fn LayoutViewportScrollOffset(&self, document: &EventTargetRef) -> PointF;
    fn DocumentToFrame(&self, document: &EventTargetRef, point: PointF) -> PointF;
    /// kTouchEvent|kReadOnly|kActive, after ConvertFromRootFrame and LayoutUnit
    /// rounding in that document's resident frame owner (cc:511-523).
    fn HitTestTouchNode(
        &self,
        document: &EventTargetRef,
        event: &WebPointerEvent,
    ) -> Option<EventTargetRef>;
    fn IsTextNode(&self, node: &EventTargetRef) -> bool;
    fn FlatTreeParent(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn IsElement(&self, node: &EventTargetRef) -> bool;
    fn IsPseudoElement(&self, node: &EventTargetRef) -> bool;
    fn ParentOrShadowHostElement(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn IsShadowIncludingInclusiveAncestorOf(
        &self,
        ancestor: &EventTargetRef,
        node: &EventTargetRef,
    ) -> bool;
    fn IsVerticalScrollEnforced(&self, document: &EventTargetRef) -> bool;
    /// cc:34-51: consult the canonical EventHandlerRegistry target membership
    /// for DOMWindow and NodeTraversal::InclusiveAncestorsOf(node).
    fn HasBlockingTouchHandlerInAncestorPath(&self, node: &EventTargetRef) -> bool;
    fn SetTouchAction(&self, action: PointerTouchAction);
    fn DispatchEvent(&self, target: &EventTargetRef, event: EventRef) -> DispatchEventResult;
}

// Source: touch_event_manager.h:77-97.
#[derive(Clone)]
struct TouchPointAttributes {
    event: WebPointerEvent,
    coalesced_events: Vec<WebPointerEvent>,
    target: Option<EventTargetRef>,
    stale: bool,
}
impl TouchPointAttributes {
    fn new(event: WebPointerEvent) -> Self {
        Self {
            event,
            coalesced_events: Vec::new(),
            target: None,
            stale: false,
        }
    }
}

// Source: touch_event_manager.h:114-152.
struct TouchStateOwner {
    touch_attribute_map: HashMap<i32, TouchPointAttributes>,
    touch_sequence_document: Option<EventTargetRef>,
    suppressing_touchmoves_within_slop: bool,
    last_coalesced_touch_event: WebTouchEvent,
    current_touch_action: PointerTouchAction,
    should_enforce_vertical_scroll: bool,
    delayed_effective_touch_action: Option<PointerTouchAction>,
}

pub struct TouchEventManagerState {
    data: RefCell<TouchStateOwner>,
}
impl TouchEventManagerState {
    pub fn IsAnyTouchActive(&self) -> bool {
        !self.data.borrow().touch_attribute_map.is_empty()
    }
    fn borrow(&self) -> std::cell::Ref<'_, TouchStateOwner> {
        self.data.borrow()
    }
    fn borrow_mut(&self) -> std::cell::RefMut<'_, TouchStateOwner> {
        self.data.borrow_mut()
    }
}
impl Default for TouchEventManagerState {
    fn default() -> Self {
        Self {
            data: RefCell::new(TouchStateOwner {
                touch_attribute_map: HashMap::new(),
                touch_sequence_document: None,
                suppressing_touchmoves_within_slop: false,
                last_coalesced_touch_event: WebTouchEvent::default(),
                current_touch_action: PointerTouchAction::kAuto,
                should_enforce_vertical_scroll: false,
                delayed_effective_touch_action: None,
            }),
        }
    }
}

pub struct TouchEventManager<'a> {
    environment: Rc<dyn TouchEventEnvironment + 'a>,
    state: Rc<TouchEventManagerState>,
}

// Source: touch_event_manager.cc:71-132.
fn TouchEventNameForPointerEventType(event_type: Type) -> &'static str {
    match event_type {
        Type::kPointerUp => "touchend",
        Type::kPointerCancel => "touchcancel",
        Type::kPointerDown => "touchstart",
        Type::kPointerMove => "touchmove",
        _ => unreachable!("touch point pointer event type"),
    }
}
fn TouchPointStateFromPointerEventType(event_type: Type, stale: bool) -> TouchState {
    if stale {
        return TouchState::kStateStationary;
    }
    match event_type {
        Type::kPointerUp => TouchState::kStateReleased,
        Type::kPointerCancel => TouchState::kStateCancelled,
        Type::kPointerDown => TouchState::kStatePressed,
        Type::kPointerMove => TouchState::kStateMoved,
        _ => unreachable!("touch point pointer event type"),
    }
}
fn CreateWebTouchPointFromWebPointerEvent(event: &WebPointerEvent, stale: bool) -> WebTouchPoint {
    let mut point = WebTouchPoint::new(event.pointer_properties.clone());
    point.state = TouchPointStateFromPointerEventType(event.GetType(), stale);
    point.radius_x = event.width / 2.0;
    point.radius_y = event.height / 2.0;
    point.rotation_angle = event.rotation_angle;
    point
}
fn SetWebTouchEventAttributesFromWebPointerEvent(
    touch: &mut WebTouchEvent,
    pointer: &WebPointerEvent,
) {
    touch.dispatch_type = pointer.dispatch_type;
    touch.touch_start_or_first_touch_move = pointer.touch_start_or_first_touch_move;
    touch.moved_beyond_slop_region = pointer.moved_beyond_slop_region;
    touch.SetFrameScale(pointer.FrameScale());
    touch.SetFrameTranslate(pointer.FrameTranslate());
    touch.SetTimeStamp(pointer.TimeStamp());
    touch.SetModifiers(pointer.GetModifiers());
}

impl<'a> TouchEventManager<'a> {
    // Source: touch_event_manager.cc:151-167.
    pub fn new(environment: Rc<dyn TouchEventEnvironment + 'a>) -> Self {
        Self {
            environment,
            state: Rc::new(TouchEventManagerState::default()),
        }
    }
    /// Bind the current synchronous dispatcher to the same retained sequence
    /// owner. Rebinding does not clear active touches or coalesced history.
    pub fn WithState(
        state: Rc<TouchEventManagerState>,
        environment: Rc<dyn TouchEventEnvironment + 'a>,
    ) -> Self {
        Self { environment, state }
    }
    pub fn State(&self) -> Rc<TouchEventManagerState> {
        self.state.clone()
    }
    pub fn Clear(&self) {
        let mut state = self.state.borrow_mut();
        state.touch_sequence_document = None;
        state.touch_attribute_map.clear();
        state.last_coalesced_touch_event = WebTouchEvent::default();
        state.suppressing_touchmoves_within_slop = false;
        state.current_touch_action = PointerTouchAction::kAuto;
    }

    // Source: touch_event_manager.cc:169-221; touch.cc:46-87.
    fn CreateDomTouch(
        &self,
        attributes: &TouchPointAttributes,
        document: &EventTargetRef,
    ) -> (Rc<Touch>, bool) {
        let known_target = attributes
            .target
            .as_ref()
            .is_some_and(|target| Rc::ptr_eq(&self.environment.OwnerDocument(target), document));
        let target = if known_target {
            attributes.target.as_ref().unwrap().clone()
        } else {
            document.clone()
        };
        debug_assert!(self.environment.FrameHasView(document));
        let transformed = attributes.event.WebPointerEventInRootFrame();
        let scale = 1.0 / self.environment.LayoutZoomFactor(document);
        let point = self
            .environment
            .RootFrameToDocument(document, *transformed.pointer_properties.PositionInWidget());
        let document_position = PointF::new(point.x() * scale, point.y() * scale);
        let scroll = self.environment.LayoutViewportScrollOffset(document);
        let client_position = PointF::new(
            document_position.x() - scroll.x() * scale,
            document_position.y() - scroll.y() * scale,
        );
        let zoom = self.environment.LayoutZoomFactor(document);
        let absolute = self.environment.DocumentToFrame(
            document,
            PointF::new(document_position.x() * zoom, document_position.y() * zoom),
        );
        (
            Rc::new(Touch {
                identifier: attributes.event.pointer_properties.id,
                target,
                screen_position: *transformed.pointer_properties.PositionInScreen(),
                document_position,
                client_position,
                absolute_location: PointF::new(
                    foundation::LayoutUnit::FromFloatFloor(absolute.x()).ToFloat(),
                    foundation::LayoutUnit::FromFloatFloor(absolute.y()).ToFloat(),
                ),
                radius_x: transformed.width / 2.0 * scale,
                radius_y: transformed.height / 2.0 * scale,
                rotation_angle: transformed.rotation_angle,
                force: transformed.pointer_properties.force,
            }),
            known_target,
        )
    }

    // Source: touch_event_manager.cc:223-346.
    fn GenerateWebCoalescedInputEvent(&self) -> NativeTouchInputEvent {
        let mut state = self.state.borrow_mut();
        debug_assert!(!state.touch_attribute_map.is_empty());
        let first = state
            .touch_attribute_map
            .values()
            .next()
            .unwrap()
            .event
            .clone();
        let mut event = WebTouchEvent::default();
        SetWebTouchEventAttributesFromWebPointerEvent(&mut event, &first);
        SetWebTouchEventAttributesFromWebPointerEvent(
            &mut state.last_coalesced_touch_event,
            &first,
        );
        let mut event_type = Type::kTouchMove;
        let mut all_coalesced = Vec::new();
        let mut ids: Vec<_> = state.touch_attribute_map.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            let attributes = &state.touch_attribute_map[&id];
            event.touches[event.touches_length] =
                CreateWebTouchPointFromWebPointerEvent(&attributes.event, attributes.stale);
            event.touches_length += 1;
            if !attributes.stale {
                let timestamp = event.TimeStamp().max(attributes.event.TimeStamp());
                event.SetTimeStamp(timestamp);
            }
            if event_type == Type::kTouchMove {
                event_type = match attributes.event.GetType() {
                    Type::kPointerDown => Type::kTouchStart,
                    Type::kPointerCancel => Type::kTouchCancel,
                    Type::kPointerUp => Type::kTouchEnd,
                    _ => Type::kTouchMove,
                };
            }
            all_coalesced.extend(attributes.coalesced_events.iter().cloned());
        }
        event.SetType(event_type);
        state.last_coalesced_touch_event.SetType(event_type);
        all_coalesced.sort_unstable_by_key(|event| event.TimeStamp());
        let mut result =
            WebCoalescedInputEvent::WithEvents(Box::new(event), Vec::new(), Vec::new(), &());
        for pointer in all_coalesced {
            let last = &mut state.last_coalesced_touch_event;
            let existing = last.touches[..last.touches_length]
                .iter()
                .position(|point| point.pointer_properties.id == pointer.pointer_properties.id);
            if pointer.GetType() == Type::kPointerDown {
                if let Some(index) = existing {
                    last.touches[index] = CreateWebTouchPointFromWebPointerEvent(&pointer, false);
                    last.SetTimeStamp(pointer.TimeStamp());
                } else {
                    last.touches[last.touches_length] =
                        CreateWebTouchPointFromWebPointerEvent(&pointer, false);
                    last.touches_length += 1;
                }
                last.touches[..last.touches_length]
                    .sort_unstable_by_key(|point| point.pointer_properties.id);
                result.AddCoalescedEvent(last);
            } else if let Some(index) = existing {
                last.touches[index] = CreateWebTouchPointFromWebPointerEvent(&pointer, false);
                last.SetTimeStamp(pointer.TimeStamp());
                result.AddCoalescedEvent(last);
                let mut size = 0;
                for index in 0..last.touches_length {
                    if !matches!(
                        last.touches[index].state,
                        TouchState::kStateCancelled | TouchState::kStateReleased
                    ) {
                        last.touches[size] = last.touches[index].clone();
                        size += 1;
                    }
                }
                last.touches_length = size;
            }
        }
        result
    }

    // Source: touch_event_manager.cc:348-500.
    fn DispatchTouchEventFromAccumulatdTouchPoints(&self) -> WebInputEventResult {
        let (attributes, document) = {
            let mut state = self.state.borrow_mut();
            if !state
                .touch_attribute_map
                .values()
                .any(|attributes| !attributes.stale)
            {
                return WebInputEventResult::kNotHandled;
            }
            let ended = state.touch_attribute_map.values().any(|attributes| {
                matches!(
                    attributes.event.GetType(),
                    Type::kPointerUp | Type::kPointerCancel
                )
            });
            if ended || state.touch_attribute_map.len() > 1 {
                state.suppressing_touchmoves_within_slop = false;
            }
            if state.suppressing_touchmoves_within_slop {
                debug_assert_eq!(state.touch_attribute_map.len(), 1);
                let attributes = state.touch_attribute_map.values().next().unwrap();
                if attributes.event.GetType() == Type::kPointerMove {
                    if !attributes.event.moved_beyond_slop_region {
                        return WebInputEventResult::kHandledSuppressed;
                    }
                    state.suppressing_touchmoves_within_slop = false;
                }
            }
            let mut ids: Vec<_> = state.touch_attribute_map.keys().copied().collect();
            ids.sort_unstable();
            (
                ids.into_iter()
                    .map(|id| state.touch_attribute_map[&id].clone())
                    .collect::<Vec<_>>(),
                state.touch_sequence_document.clone().unwrap(),
            )
        };
        let all_pressed = attributes
            .iter()
            .all(|attributes| attributes.event.GetType() == Type::kPointerDown);
        let mut touches = Vec::new();
        let mut by_target: HashMap<usize, Vec<Rc<Touch>>> = HashMap::new();
        let mut changed: Vec<(Vec<Rc<Touch>>, Vec<EventTargetRef>)> =
            (Type::kPointerTypeFirst as i32..=Type::kPointerTypeLast as i32)
                .map(|_| (Vec::new(), Vec::new()))
                .collect();
        for attributes in &attributes {
            let (touch, known) = self.CreateDomTouch(attributes, &document);
            let target = touch.target.clone();
            let target_touches = by_target.entry(TargetKey(&target)).or_default();
            let event_type = attributes.event.GetType();
            if !matches!(event_type, Type::kPointerUp | Type::kPointerCancel) {
                touches.push(touch.clone());
                target_touches.push(touch.clone());
            }
            if !attributes.stale && known {
                let entry =
                    &mut changed[(event_type as i32 - Type::kPointerTypeFirst as i32) as usize];
                entry.0.push(touch);
                if !entry.1.iter().any(|other| Rc::ptr_eq(other, &target)) {
                    entry.1.push(target);
                }
            }
        }
        let touches: TouchList = Rc::new(touches);
        let coalesced = self.GenerateWebCoalescedInputEvent();
        let mut result = WebInputEventResult::kNotHandled;
        for (index, (changed_touches, targets)) in changed.into_iter().enumerate() {
            if changed_touches.is_empty() {
                continue;
            }
            let pointer_type = match index as i32 + Type::kPointerTypeFirst as i32 {
                value if value == Type::kPointerDown as i32 => Type::kPointerDown,
                value if value == Type::kPointerUp as i32 => Type::kPointerUp,
                value if value == Type::kPointerMove as i32 => Type::kPointerMove,
                value if value == Type::kPointerCancel as i32 => Type::kPointerCancel,
                _ => unreachable!("non-touch pointer action"),
            };
            let changed_touches = Rc::new(changed_touches);
            for target in targets {
                let document = self.environment.OwnerDocument(&target);
                let action = self.state.borrow().current_touch_action;
                let event = TouchEvent::new(
                    coalesced.clone(),
                    touches.clone(),
                    Rc::new(by_target[&TargetKey(&target)].clone()),
                    changed_touches.clone(),
                    TouchEventNameForPointerEventType(pointer_type),
                    self.environment.ViewForDocument(&document),
                    self.environment.TouchCapabilitiesForDocument(&document),
                    action.0,
                );
                let event = EventRef::new(event);
                result = MergeEventResult(
                    result,
                    ToWebInputEventResult(self.environment.DispatchEvent(&target, event)),
                );
            }
        }
        if self.state.borrow().should_enforce_vertical_scroll {
            result = self.EnsureVerticalScrollIsPossible(result);
        }
        if all_pressed && result == WebInputEventResult::kNotHandled {
            self.state.borrow_mut().suppressing_touchmoves_within_slop = true;
        }
        result
    }

    // Source: touch_event_manager.cc:502-533.
    pub fn GetTouchPointerNode(
        &self,
        event: &WebPointerEvent,
        target: &PointerEventTarget,
    ) -> Option<EventTargetRef> {
        debug_assert_eq!(event.GetType(), Type::kPointerDown);
        let mut node = target.target_element.clone();
        let document = self.state.borrow().touch_sequence_document.clone();
        if let Some(document) = document {
            if node
                .as_ref()
                .is_none_or(|node| !Rc::ptr_eq(&self.environment.OwnerDocument(node), &document))
            {
                if !self.environment.FrameHasView(&document) {
                    return None;
                }
                node = self.environment.HitTestTouchNode(&document, event);
                if node
                    .as_ref()
                    .is_some_and(|node| self.environment.IsTextNode(node))
                {
                    node = self.environment.FlatTreeParent(node.as_ref().unwrap());
                }
            }
        }
        node
    }

    // Source: touch_event_manager.cc:535-578.
    pub fn UpdateTouchAttributeMapsForPointerDown(
        &self,
        event: &WebPointerEvent,
        node: &EventTargetRef,
        action: PointerTouchAction,
    ) {
        debug_assert_eq!(event.GetType(), Type::kPointerDown);
        let document = self.environment.OwnerDocument(node);
        {
            let mut state = self.state.borrow_mut();
            state.touch_attribute_map.insert(
                event.pointer_properties.id,
                TouchPointAttributes::new(event.clone()),
            );
            if state.touch_sequence_document.is_none() {
                state.touch_sequence_document = Some(document);
            }
            state
                .touch_attribute_map
                .get_mut(&event.pointer_properties.id)
                .unwrap()
                .target = Some(node.clone());
        }
        let document = self.state.borrow().touch_sequence_document.clone().unwrap();
        let enforce = self.environment.IsVerticalScrollEnforced(&document);
        let blocking = enforce && self.environment.HasBlockingTouchHandlerInAncestorPath(node);
        let delayed = {
            let mut state = self.state.borrow_mut();
            state.should_enforce_vertical_scroll = enforce;
            if blocking {
                state.delayed_effective_touch_action = Some(PointerTouchAction(
                    state
                        .delayed_effective_touch_action
                        .unwrap_or(PointerTouchAction::kAuto)
                        .0
                        & action.0,
                ));
            }
            state.delayed_effective_touch_action.is_some()
        };
        if !delayed {
            self.environment.SetTouchAction(action);
        }
        self.state.borrow_mut().current_touch_action.0 &= action.0;
    }

    // Source: touch_event_manager.cc:580-627.
    pub fn HandleTouchPoint(
        &self,
        event: &WebPointerEvent,
        coalesced: &[WebPointerEvent],
        _target: &PointerEventTarget,
    ) {
        debug_assert!(
            event.GetType() >= Type::kPointerTypeFirst && event.GetType() <= Type::kPointerTypeLast
        );
        debug_assert_ne!(event.GetType(), Type::kPointerCausedUaAction);
        if self.state.borrow().touch_attribute_map.is_empty() {
            self.AllTouchesReleasedCleanup();
        }
        let document = self.state.borrow().touch_sequence_document.clone();
        if document
            .as_ref()
            .is_some_and(|document| !self.environment.FrameHasView(document))
        {
            if let Some(attributes) = self
                .state
                .borrow_mut()
                .touch_attribute_map
                .get_mut(&event.pointer_properties.id)
            {
                attributes.event = event.clone();
            }
            return;
        }
        let mut state = self.state.borrow_mut();
        let attributes = state
            .touch_attribute_map
            .entry(event.pointer_properties.id)
            .or_insert_with(|| TouchPointAttributes::new(event.clone()));
        attributes.event = event.clone();
        attributes.coalesced_events = coalesced.to_vec();
        attributes.stale = false;
    }

    // Source: touch_event_manager.cc:628-662.
    pub fn FlushEvents(&self) -> WebInputEventResult {
        let mut result = WebInputEventResult::kNotHandled;
        let document = self.state.borrow().touch_sequence_document.clone();
        if document.as_ref().is_some_and(|document| {
            self.environment.DocumentHasPage(document)
                && self.environment.HasTouchHandlers(document)
                && self.environment.FrameHasView(document)
        }) {
            result = self.DispatchTouchEventFromAccumulatdTouchPoints();
        }
        self.state
            .borrow_mut()
            .touch_attribute_map
            .retain(|_, attributes| {
                if matches!(
                    attributes.event.GetType(),
                    Type::kPointerUp | Type::kPointerCancel
                ) {
                    return false;
                }
                attributes.stale = true;
                attributes.event.pointer_properties.movement_x = 0;
                attributes.event.pointer_properties.movement_y = 0;
                attributes.coalesced_events.clear();
                true
            });
        if self.state.borrow().touch_attribute_map.is_empty() {
            self.AllTouchesReleasedCleanup();
        }
        result
    }

    // Source: touch_event_manager.cc:664-676.
    fn AllTouchesReleasedCleanup(&self) {
        let mut state = self.state.borrow_mut();
        state.touch_sequence_document = None;
        state.current_touch_action = PointerTouchAction::kAuto;
        state.last_coalesced_touch_event = WebTouchEvent::default();
        state.delayed_effective_touch_action = None;
        state.should_enforce_vertical_scroll = false;
    }

    // Source: touch_event_manager.cc:678-686.
    pub fn HandlePseudoElementRemoval(&self, pseudo: &EventTargetRef) {
        let parent = self.environment.ParentOrShadowHostElement(pseudo);
        let targets: Vec<_> = self
            .state
            .borrow()
            .touch_attribute_map
            .iter()
            .filter_map(|(id, attributes)| attributes.target.clone().map(|target| (*id, target)))
            .collect();
        for (id, target) in targets {
            if self.environment.IsPseudoElement(&target)
                && self
                    .environment
                    .IsShadowIncludingInclusiveAncestorOf(pseudo, &target)
            {
                if let Some(attributes) = self.state.borrow_mut().touch_attribute_map.get_mut(&id) {
                    attributes.target = parent.clone();
                }
            }
        }
    }
    // Source: touch_event_manager.cc:688-697.
    pub fn IsAnyTouchActive(&self) -> bool {
        !self.state.borrow().touch_attribute_map.is_empty()
    }
    pub fn CurrentTouchDownElement(&self) -> Option<EventTargetRef> {
        let target = {
            let state = self.state.borrow();
            if state.touch_attribute_map.len() != 1 {
                return None;
            }
            state.touch_attribute_map.values().next()?.target.clone()?
        };
        self.environment.IsElement(&target).then_some(target)
    }
    // Source: touch_event_manager.cc:699-723.
    fn EnsureVerticalScrollIsPossible(&self, result: WebInputEventResult) -> WebInputEventResult {
        let prevent_defaulted = result == WebInputEventResult::kHandledApplication;
        let delayed = {
            let mut state = self.state.borrow_mut();
            if prevent_defaulted {
                if let Some(action) = &mut state.delayed_effective_touch_action {
                    action.0 &= 0x0c;
                }
            }
            state.delayed_effective_touch_action
        };
        if let Some(action) = delayed {
            self.environment.SetTouchAction(action);
            self.state.borrow_mut().delayed_effective_touch_action = None;
        }
        if prevent_defaulted {
            WebInputEventResult::kNotHandled
        } else {
            result
        }
    }
}

fn TargetKey(target: &EventTargetRef) -> usize {
    Rc::as_ptr(target) as *const () as usize
}

impl PointerTouchManager for TouchEventManager<'_> {
    fn Clear(&self) {
        Self::Clear(self);
    }
    fn HandlePseudoElementRemoval(&self, pseudo: &EventTargetRef) {
        Self::HandlePseudoElementRemoval(self, pseudo);
    }
    fn FlushEvents(&self) -> WebInputEventResult {
        Self::FlushEvents(self)
    }
    fn IsAnyTouchActive(&self) -> bool {
        Self::IsAnyTouchActive(self)
    }
    fn CurrentTouchDownElement(&self) -> Option<EventTargetRef> {
        Self::CurrentTouchDownElement(self)
    }
    fn GetTouchPointerNode(
        &self,
        event: &WebPointerEvent,
        target: &PointerEventTarget,
    ) -> Option<EventTargetRef> {
        Self::GetTouchPointerNode(self, event, target)
    }
    fn UpdateTouchAttributeMapsForPointerDown(
        &self,
        event: &WebPointerEvent,
        node: &EventTargetRef,
        action: PointerTouchAction,
    ) {
        Self::UpdateTouchAttributeMapsForPointerDown(self, event, node, action);
    }
    fn HandleTouchPoint(
        &self,
        event: &WebPointerEvent,
        coalesced: &[WebPointerEvent],
        target: &PointerEventTarget,
    ) {
        Self::HandleTouchPoint(self, event, coalesced, target);
    }
}
