/*
 * Copyright (C) 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc. All rights
 * reserved.
 * Copyright (C) 2006 Alexey Proskuryakov (ap@webkit.org)
 * Copyright (C) 2012 Digia Plc. and/or its subsidiary(-ies)
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/event_handler.h:77-101,111-124,131-140,155-169,172-177,207-214,222-233,237-245,249,253-261,268-276,282-288,301-305,316-321,326-328,349,351,355,375-402,406-415,424-461,467-489
// Source: third_party/blink/renderer/core/input/event_handler.cc:144-152,196-216,269-351,371-451,856-1356,1525-1553,1563-1652,1658-1668,1678-1687,1751-1775,2184-2223,2384-2412,2418-2493,2504-2698
// Selected: manager construction/identity and clearing, resident hit testing,
// mouse/pointer dispatch and capture, keyboard/focus forwarding and text input.
// Gesture root routing and targeting are translated in event_handler_gesture.rs
// (event_handler.cc:1689-1749,1782-2182).
// Nonlocated context-menu placement is translated in
// event_handler_nonlocated_context_menu.rs (event_handler.cc:153-165,2226-2382).
// Scroll-owned definitions remain in interaction_scroll's EventHandler
// slice. Drag/drop, cursor selection, and their helpers (event_handler.cc:167-194,
// 218-268,477-854,1358-1523,1555-1561,1654-1656,1778-1780,2224,
// 2495-2502) are explicitly outside this slice.
// Composition is owned by InputMethodController, not EventHandler: no second
// composition state or dispatch algorithm is introduced here.
// Ports project the existing manager, DOM and resident hit-result owners.
// They must not materialize another interaction tree or schedule deferred
// replacements for synchronous source calls. Interior mutability at the live
// owners allows listeners to synchronously reenter EventHandler.

#![allow(non_snake_case, non_camel_case_types)]

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetRef};
use interaction_dispatch::core::events::keyboard_event::{KeyboardEvent, KeyboardEventRef};
use interaction_dispatch::core::events::mouse_event::MouseEvent;
use interaction_dispatch::core::events::ui_event::InputDeviceCapabilitiesRef;
use interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult;
use interaction_hit_test::core::layout::hit_test_location::HitTestLocation;
use interaction_hit_test::core::layout::hit_test_request::RequestType;
use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::pointer_id::PointerId;
use interaction_input::public::common::input::web_input_event::{Type, WebInputEvent};
use interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent;
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use interaction_input::public::common::input::web_pointer_event::WebPointerEvent;
use interaction_input::public::common::input::web_pointer_properties::Button;

// Source: event_handler.cc:371-451,868-1356,2640-2673.
// Location, point and result remain the existing hit-test owner's types. Only
// the operations used by this source translation cross the package boundary.
pub trait EventHandlerHitResult: Clone {
    fn HitTestRequestType(&self) -> u32;
    fn InnerNode(&self) -> Option<NodeRef>;
    fn InnerElement(&self) -> Option<NodeRef>;
    fn InnerPossiblyPseudoElement(&self) -> Option<NodeRef>;
    fn GetScrollbar(&self) -> Option<Rc<dyn EventHandlerScrollbar>>;
    fn SetToShadowHostIfInUAShadowRoot(&mut self);
    fn SetNodeAndPosition(&mut self, node: NodeRef, point: &HitTestLocation);
    fn SetScrollbar(&mut self, scrollbar: Option<Rc<dyn EventHandlerScrollbar>>);
    fn SetURLElement(&mut self, node: Option<NodeRef>);
    fn SetIsOverEmbeddedContentView(&mut self, value: bool);
}

pub type NodeRef = Rc<dyn EventHandlerNode>;

// Source: event_handler.cc:901-926,962-967,1023-1028,1643,2655-2660.
pub trait EventHandlerNode {
    fn DomNodeId(&self) -> i32;
    fn StatePreservingAtomicMoveInProgress(&self) -> bool;
    fn AbsoluteToLocalPoint(&self, point: &HitTestLocation) -> HitTestLocation;
    fn EnclosingLinkEventParentOrSelf(&self) -> Option<NodeRef>;
    fn HasParent(&self) -> bool;
    fn IsShadowRootWithInputHost(&self) -> bool;
    fn IsElementWithTitle(&self) -> bool;
    fn IsSVGUseElement(&self) -> bool;
    fn ContainingShadowRootHasSVGUseHost(&self) -> bool;
}

// Source: event_handler.cc:1120-1122,1140-1142,1199-1202,1298-1301,2547-2577.
pub trait EventHandlerScrollbar {
    fn HandleGestureTapOrPress(
        &self,
        event: &interaction_input::public::common::input::web_gesture_event::WebGestureEvent,
    ) -> bool;
    fn Pressed(&self) -> bool;
    fn ThumbPressed(&self) -> bool;
    fn Enabled(&self) -> bool;
    fn MouseDown(&self, event: &WebMouseEvent);
    fn MouseUp(&self, event: &WebMouseEvent);
    fn MouseMoved(&self, event: &WebMouseEvent);
    fn MouseEntered(&self);
    fn MouseExited(&self);
}

/// The located context-menu transaction, shared by the full EventHandler and
/// its call-scoped production binder. Hit results retain the original target.
pub trait LocatedContextMenuSource {
    type Target;
    type Hit;
    fn HasView(&self) -> bool;
    fn ReleaseMousePress(&self);
    fn LastScrollbarMouseUp(&self, event: &WebMouseEvent);
    fn HitTestContextMenu(&self, event: &WebMouseEvent) -> Self::Hit;
    fn UpdateStyleAndLayoutForContextMenu(&self);
    fn InnerPossiblyPseudoElement(&self, hit: &Self::Hit) -> Option<Self::Target>;
    fn DispatchContextMenuEvent(
        &self,
        target: Option<Self::Target>,
        event: &WebMouseEvent,
    ) -> WebInputEventResult;
}
// Source: event_handler.cc:2184-2223.
pub fn SendLocatedContextMenu<S: LocatedContextMenuSource>(
    source: &S,
    event: &WebMouseEvent,
    override_target: Option<S::Target>,
) -> WebInputEventResult {
    if !source.HasView() {
        return WebInputEventResult::kNotHandled;
    }
    source.ReleaseMousePress();
    source.LastScrollbarMouseUp(event);
    let hit = source.HitTestContextMenu(event);
    source.UpdateStyleAndLayoutForContextMenu();
    source.DispatchContextMenuEvent(
        override_target.or_else(|| source.InnerPossiblyPseudoElement(&hit)),
        event,
    )
}

/// EventHandler::HandleMouseReleaseEvent, cc:1302-1317. The retained scrollbar
/// is the press/hover owner, even when the release hit is elsewhere. Both the
/// full handler and the production binder execute this source branch.
pub fn HandleLastScrollbarMouseUp(
    state: &EventHandlerCursorState,
    event: &WebMouseEvent,
    invalidate_click: impl FnOnce(),
    release_mouse_capture: impl FnOnce(),
) -> bool {
    let Some(scrollbar) = state.LastScrollbarUnderMouse() else {
        return false;
    };
    invalidate_click();
    scrollbar.MouseUp(event);
    if MouseEvent::WebInputEventModifiersToButtons(event.GetModifiers() as u32) == 0 {
        release_mouse_capture();
    }
    true
}

// Source: event_handler.cc:2541-2563. Keep the press owner's identity and
// widget-capture transition shared by the complete handler and native binder.
pub fn PassMousePressToScrollbar(
    state: &EventHandlerCursorState,
    scrollbar: Option<Rc<dyn EventHandlerScrollbar>>,
    event: &WebMouseEvent,
    capture_mouse_to_widget: impl FnOnce(bool),
) -> bool {
    if state
        .LastScrollbarUnderMouse()
        .is_some_and(|last| last.Pressed())
    {
        return false;
    }
    state.UpdateLastScrollbarUnderMouse(scrollbar.clone(), true);
    let Some(scrollbar) = scrollbar else {
        return false;
    };
    if !scrollbar.Enabled() {
        return false;
    }
    scrollbar.MouseDown(event);
    if scrollbar.ThumbPressed() {
        capture_mouse_to_widget(true);
    }
    true
}

// Source: event_handler.cc:1105-1123,1138-1143. A lost native release must
// clear drag/capture before the saved scrollbar receives another move.
pub fn ResetMouseMoveWithNoButton(
    state: &EventHandlerCursorState,
    event: &WebMouseEvent,
    clear_drag: impl FnOnce(),
    release_capture: impl FnOnce(),
) {
    if event.pointer_properties.button == Button::kNoButton
        && event.GetModifiers() & WebInputEvent::kRelativeMotionEvent == 0
    {
        clear_drag();
        state.ClearMouseCapture();
        release_capture();
        if let Some(scrollbar) = state.LastScrollbarUnderMouse() {
            if scrollbar.Pressed() {
                scrollbar.MouseUp(event);
            }
        }
    }
}
pub fn HandleLastScrollbarMouseMove(
    state: &EventHandlerCursorState,
    event: &WebMouseEvent,
    mouse_pressed: bool,
) -> bool {
    if let Some(scrollbar) = state.LastScrollbarUnderMouse() {
        if mouse_pressed {
            scrollbar.MouseMoved(event);
            return true;
        }
    }
    false
}

// Source: event_handler.cc:273-276,299-302,327-330,2384-2403.
// The embedding supplies task-runner timers. No clock or event pump is owned
// by the translated manager.
pub trait EventHandlerTimer {
    fn Stop(&self);
    fn IsActive(&self) -> bool;
    fn StartOneShotMilliseconds(&self, milliseconds: u32);
    fn StartOneShotMicroseconds(&self, microseconds: u64);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventHandlerTimerKind {
    Hover,
    CursorUpdate,
    ActiveInterval,
}
impl EventHandlerTimerKind {
    fn Index(self) -> usize {
        match self {
            Self::Hover => 0,
            Self::CursorUpdate => 1,
            Self::ActiveInterval => 2,
        }
    }
}
/// Task-runner delivery carries identity, never a borrowed handler or frame.
/// The generation belongs to the retained source owner, not the event pump.
#[derive(Clone)]
pub struct EventHandlerTimerTask {
    pub frame_identity: Rc<()>,
    pub kind: EventHandlerTimerKind,
    pub generation: u64,
}

// Source: event_handler.cc:269-302. This is construction, not an input
// processing callback: each returned Rc is the original manager owner.
pub trait EventHandlerFactory<'scope, R: EventHandlerHitResult> {
    fn SelectionController(&self) -> Rc<dyn EventHandlerSelection<R> + 'scope>;
    fn HoverTimer(&self) -> Rc<dyn EventHandlerTimer + 'scope>;
    fn CursorUpdateTimer(&self) -> Rc<dyn EventHandlerTimer + 'scope>;
    fn EventHandlerRegistry(&self, local_root: bool) -> Rc<dyn EventHandlerRegistry + 'scope>;
    fn ScrollManager(&self) -> Rc<dyn EventHandlerScrollPort<R> + 'scope>;
    fn MouseEventManager(
        &self,
        scroll: &Rc<dyn EventHandlerScrollPort<R> + 'scope>,
    ) -> Rc<dyn EventHandlerMouse<R> + 'scope>;
    fn MouseWheelEventManager(
        &self,
        scroll: &Rc<dyn EventHandlerScrollPort<R> + 'scope>,
    ) -> Rc<dyn EventHandlerWheel + 'scope>;
    fn KeyboardEventManager(
        &self,
        scroll: &Rc<dyn EventHandlerScrollPort<R> + 'scope>,
    ) -> Rc<dyn EventHandlerKeyboard + 'scope>;
    fn PointerEventManager(
        &self,
        mouse: &Rc<dyn EventHandlerMouse<R> + 'scope>,
    ) -> Rc<dyn EventHandlerPointer<'scope, R> + 'scope>;
    fn GestureManager(
        &self,
        scroll: &Rc<dyn EventHandlerScrollPort<R> + 'scope>,
        mouse: &Rc<dyn EventHandlerMouse<R> + 'scope>,
        pointer: &Rc<dyn EventHandlerPointer<'scope, R> + 'scope>,
        selection: &Rc<dyn EventHandlerSelection<R> + 'scope>,
    ) -> Rc<dyn EventHandlerGesture<R> + 'scope>;
    fn ActiveIntervalTimer(&self) -> Rc<dyn EventHandlerTimer + 'scope>;
    fn GestureTargeting(&self)
        -> Rc<dyn gesture::EventHandlerGestureTargeting<'scope, R> + 'scope>;
}

pub trait EventHandlerRegistry {}
pub trait EventHandlerGesture<R: EventHandlerHitResult> {
    fn HandleTouchDragEnd(&self, event: &WebMouseEvent, operation_is_none: bool);
    fn GetPointerIdFromWebGestureEvent(
        &self,
        event: &interaction_input::public::common::input::web_gesture_event::WebGestureEvent,
    ) -> i32;
    fn Clear(&self);
    fn GetHitTypeForGestureType(&self, event_type: Type) -> u32;
    fn HandleGestureEventInFrame(
        &self,
        event: &super::gesture_manager::GestureEventWithHitTestResults<R>,
    ) -> WebInputEventResult;
    fn GestureContextMenuDeferred(&self) -> bool;
    fn ResetLongTapContextMenuStates(&self);
}
pub trait EventHandlerWheel {
    fn Clear(&self);
    fn ElementRemoved(&self, target: &NodeRef);
}

// Source: event_handler.cc:339,935-940,985-986,1182-1183,1341.
// Every operation forwards to the same source ScrollManager; scroll policy is
// not duplicated in EventHandler.
pub trait EventHandlerScrollPort<R: EventHandlerHitResult> {
    fn Clear(&self);
    fn MiddleClickAutoscrollInProgress(&self) -> bool;
    fn StopMiddleClickAutoscroll(&self);
    fn InResizeMode(&self) -> bool;
    fn Resize(&self, event: &WebMouseEvent);
    fn TrySetResizeScrollableArea(&self, result: &R, event: &WebMouseEvent) -> bool;
    fn ClearResizeScrollableArea(&self, destroyed: bool);
}

// Source: event_handler.cc:1000,1338-1339,2583.
pub trait EventHandlerSelection<R: EventHandlerHitResult> {
    /// The retained SelectionController's source mouse-down state.
    fn MouseDownWasSingleClickInSelection(&self) -> bool;
    fn InitializeSelectionState(&self);
    fn NotifySelectionChanged(&self);
    fn HasExtendedSelection(&self) -> bool;
    fn IsSelectionOverLink(&self, result: &R, event: &WebMouseEvent) -> bool;
    fn PassMousePressEventToSubframe(&self, event: &MouseEventWithHitTestResults<R>);
}

// Source: event_handler.cc:349,856-1356,1525-1553,1684,2470,2599,2696.
pub trait EventHandlerMouse<R: EventHandlerHitResult> {
    fn DragSourceEndedAt(
        &self,
        event: &WebMouseEvent,
        operation: interaction_dispatch::core::clipboard::data_transfer::DragOperation,
    );
    fn Clear(&self);
    fn SetElementUnderMouse(&self, target: Option<NodeRef>, event: &WebMouseEvent);
    fn NodeChildrenWillBeRemoved(&self, container: &NodeRef);
    fn NodeWillBeRemoved(&self, node: &NodeRef);
    fn HandlePseudoElementRemoval(&self, pseudo: &NodeRef);
    fn UpdateSelectionForMouseDrag(&self);
    fn HandleMousePressEventUpdateStates(&self, event: &WebMouseEvent);
    fn SetMousePressNode(&self, node: NodeRef);
    fn MousePressNode(&self) -> Option<NodeRef>;
    fn MousePressed(&self) -> bool;
    fn SetClickCount(&self, count: i32);
    fn SetMouseDownElement(&self, node: Option<NodeRef>);
    fn InvalidateClick(&self);
    fn HandleMouseFocus(
        &self,
        result: &R,
        capabilities: &InputDeviceCapabilitiesRef,
    ) -> WebInputEventResult;
    fn HandleMousePressEvent(&self, event: &MouseEventWithHitTestResults<R>)
        -> WebInputEventResult;
    fn SetLastKnownMousePosition(&self, event: &WebMouseEvent);
    fn HandleSvgPanIfNeeded(&self, release: bool);
    fn ClearDragHeuristicState(&self);
    fn HandleMouseDraggedEvent(
        &self,
        event: &MouseEventWithHitTestResults<R>,
    ) -> WebInputEventResult;
    fn ReleaseMousePress(&self);
    fn HandleMouseReleaseEvent(
        &self,
        event: &MouseEventWithHitTestResults<R>,
    ) -> WebInputEventResult;
    fn HandleMouseReleaseEventUpdateStates(&self);
    fn SetElementUnderMouseAndDispatchMouseUp(
        &self,
        target: Option<NodeRef>,
        event: &WebMouseEvent,
    ) -> WebInputEventResult;
    fn GetElementUnderMouse(&self) -> Option<NodeRef>;
    fn MouseDownMayStartDrag(&self) -> bool;
    fn SetMouseDownMayStartAutoscroll(&self);
    fn RecomputeMouseHoverStateIfNeeded(&self);
    fn MarkHoverStateDirty(&self);
    fn IsMousePositionUnknown(&self) -> bool;
    fn SetLastMousePositionAsUnknown(&self);
    fn LastKnownMousePositionInViewport(&self) -> HitTestLocation;
    fn LastKnownMouseScreenPosition(
        &self,
    ) -> interaction_input::public::common::input::web_pointer_properties::PointF;
    fn DispatchContextMenuEvent(
        &self,
        target: Option<NodeRef>,
        event: &WebMouseEvent,
    ) -> WebInputEventResult;
}

// Source: event_handler.cc:856-866,1149-1151,1218-1220,1239-1241,1563-1687,2688.
pub trait EventHandlerPointer<'scope, R: EventHandlerHitResult> {
    fn Clear(&self);
    fn NodeChildrenWillBeRemoved(&self, container: &NodeRef);
    fn NodeWillBeRemoved(&self, node: &NodeRef);
    fn HandlePseudoElementRemoval(&self, pseudo: &NodeRef);
    fn CurrentTouchDownElement(&self) -> Option<NodeRef>;
    fn FlushEvents(&self) -> WebInputEventResult;
    fn HandlePointerEvent(
        &self,
        event: &WebPointerEvent,
        coalesced: &[WebPointerEvent],
        predicted: &[WebPointerEvent],
    ) -> WebInputEventResult;
    fn GetMouseCaptureTarget(&self) -> Option<NodeRef>;
    fn IsAnyTouchActive(&self) -> bool;
    fn SendMouseAndPointerBoundaryEvents(&self, target: Option<NodeRef>, event: &WebMouseEvent);
    fn SendEffectivePanActionAtPointer(&self, event: &WebMouseEvent, node: Option<NodeRef>);
    fn RemoveLastMousePosition(&self);
    fn IsPointerIdActiveOnFrame(
        &self,
        id: PointerId,
        frame: &Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
    ) -> bool;
    fn AppendTouchIdForCanceledPointerDown(&self, unique_touch_event_id: u32);
    fn IsActive(&self, id: PointerId) -> bool;
    fn SetPointerCapture(&self, id: PointerId, target: &NodeRef) -> bool;
    fn ReleasePointerCapture(&self, id: PointerId, target: &NodeRef) -> bool;
    fn HasPointerCapture(&self, id: PointerId, target: &NodeRef) -> bool;
    fn ElementRemoved(&self, target: &NodeRef);
    fn SendMousePointerEvent(
        &self,
        target: Option<NodeRef>,
        event_type: Type,
        event: &WebMouseEvent,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
        skip_click: bool,
    ) -> WebInputEventResult;
    fn DirectDispatchMousePointerEvent(
        &self,
        target: NodeRef,
        event: &WebMouseEvent,
        event_type: &str,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
    ) -> WebInputEventResult;
    fn ReleaseMousePointerCapture(&self);
}

// Source: event_handler.cc:2405-2407,2459-2475,2537-2539.
pub trait EventHandlerKeyboard {
    fn IsHandlingKeyEvent(&self) -> bool;
    fn HandleAccessKey(&self, event: &WebKeyboardEvent) -> bool;
    fn KeyEvent(&self, event: &WebKeyboardEvent) -> WebInputEventResult;
    fn DefaultKeyboardEventHandler(
        &self,
        event: &KeyboardEventRef,
        mouse_press_node: Option<NodeRef>,
    );
    fn DefaultTabEventHandler(&self, event: &KeyboardEventRef) -> bool;
    fn CapsLockStateMayHaveChanged(&self);
}

// Source: event_handler.cc:2504-2535. TextEvent's concrete source type and
// TextEventInputType stay with their event/editor owners. This port creates
// the original event and synchronously dispatches it, retaining EventRef's
// underlying-event identity across listener reentry.
pub trait EventHandlerTextInput {
    type InputType;
    fn EventTargetNodeForDocument(&self) -> Option<EventTargetRef>;
    fn CreateTextEvent(&self, text: &[u16], input_type: Self::InputType) -> EventRef;
    fn DispatchEvent(&self, target: &EventTargetRef, event: &EventRef);
    fn EditorHandleTextEvent(&self, event: &EventRef) -> bool;
}

// Source: event_handler.cc:2504-2530. Shared by the complete handler and the
// Editor's call-scoped text-input edge; the original EventRef is dispatched.
pub fn HandleTextInputEvent<T: EventHandlerTextInput>(
    text: &[u16],
    underlying_event: Option<EventRef>,
    input_type: T::InputType,
    text_input: &T,
) -> bool {
    if let Some(event) = underlying_event.as_ref() {
        let event = event.borrow();
        debug_assert!(!event.IsKeyboardEvent() || event.BaseEvent().r#type() == "keypress");
    }
    let target = if let Some(event) = underlying_event.as_ref() {
        event.borrow().BaseEvent().RawTarget()
    } else {
        text_input.EventTargetNodeForDocument()
    };
    let Some(target) = target else {
        return false;
    };
    let event = text_input.CreateTextEvent(text, input_type);
    event
        .borrow_mut()
        .BaseEventMut()
        .SetUnderlyingEvent(underlying_event);
    text_input.DispatchEvent(&target, &event);
    let event = event.borrow();
    event.BaseEvent().DefaultHandled() || event.BaseEvent().defaultPrevented()
}

// Source: event_handler.cc:2532-2535. The Page default-action edge and the
// complete EventHandler use the same source default-handled transition.
pub fn DefaultTextInputEventHandler<T: EventHandlerTextInput>(event: &EventRef, text_input: &T) {
    if text_input.EditorHandleTextEvent(event) {
        event.borrow_mut().BaseEventMut().SetDefaultHandled();
    }
}

// Source: event_handler.cc:371-451. The readiness port describes the live
// LayoutView, not a copied fragment tree. Lifecycle preparation is supplied
// synchronously by Page at the source HitTest call, before the borrowed query.
pub trait EventHandlerHitTest<R: EventHandlerHitResult> {
    fn HasContentLayoutObject(&self) -> bool;
    fn HasView(&self) -> bool;
    fn DidFirstLayout(&self) -> bool;
    fn LifecycleUpdatesActive(&self) -> bool;
    fn NewResult(&self, hit_type: u32, location: &HitTestLocation) -> R;
    fn HitTest(&self, location: &HitTestLocation, result: &mut R, no_lifecycle_update: bool);
    fn UpdateHoverActiveState(
        &self,
        active: bool,
        update_active_chain: bool,
        node: Option<NodeRef>,
    );
    fn ConvertLocationToRoot(
        &self,
        location: &HitTestLocation,
        visual_overflow: bool,
    ) -> HitTestLocation;
    fn ConvertLocationFromRoot(&self, location: &HitTestLocation) -> HitTestLocation;
    fn ViewportToFrame(&self, location: &HitTestLocation) -> HitTestLocation;
    fn ContentPointFromRootFrame(&self, event: &WebMouseEvent) -> HitTestLocation;
    // Source: event_handler.cc:888-891. Unlike ContentPointFromRootFrame's
    // fractional conversion followed by rounding, mouse press saves a point
    // floored in root coordinates before ConvertFromRootFrame.
    fn DocumentPointForMousePress(&self, event: &WebMouseEvent) -> HitTestLocation;
    // Source: event_handler.cc:1002-1003: this is the existing frame hit-test
    // helper, not a second Document::PerformMouseEventHitTest invocation.
    fn HitTestResultInFrame(&self, hit_type: u32, location: &HitTestLocation) -> R;
    fn PerformMouseEventHitTest(
        &self,
        hit_type: u32,
        location: &HitTestLocation,
        event: &WebMouseEvent,
    ) -> R;
}

// Source: event_handler.cc:899-930,960-967,1002-1028,1047-1060,1102-1133,
// 1208-1245,1280-1286,1319-1339,1563-1640,2615-2631,2680-2690.
// Frame identity and existing EventHandler handles are retained. These ports
// do not expose mutable DOM/Layout storage or perform hit traversal policy.
pub trait EventHandlerFrame<'scope, R: EventHandlerHitResult> {
    /// Stable LocalFrame identity shared across call-scoped projections.
    fn Identity(&self) -> Rc<()>;
    fn FrameForIdentity(
        &self,
        identity: &Rc<()>,
    ) -> Option<Rc<dyn EventHandlerFrame<'scope, R> + 'scope>>;
    fn IsLocalRoot(&self) -> bool;
    fn LocalFrameRoot(&self) -> Rc<dyn EventHandlerFrame<'scope, R> + 'scope>;
    fn Handler(&self) -> Rc<EventHandler<'scope, R>>;
    fn HitTesting(&self) -> &dyn EventHandlerHitTest<R>;
    fn HasPage(&self) -> bool;
    fn IsAttached(&self) -> bool;
    fn IsOutermostMainFrame(&self) -> bool;
    fn IsDescendantOf(&self, frame: &Rc<dyn EventHandlerFrame<'scope, R> + 'scope>) -> bool;
    fn HasClient(&self) -> bool;
    fn SetMouseCapture(&self, capture: bool);
    fn SetSequentialFocusNavigationStartingPoint(&self, node: NodeRef);
    fn ShouldDiscardEventTargetingFrame(&self, event: &WebMouseEvent) -> bool;
    fn ShouldDiscardGestureEventTargetingFrame(
        &self,
        event: &interaction_input::public::common::input::web_gesture_event::WebGestureEvent,
    ) -> bool;
    fn NotifyUserActivation(&self);
    fn MiddleClickAutoscrollEnabled(&self) -> bool;
    fn HandleMouseMoveForMiddleClickAutoscroll(&self, event: &WebMouseEvent);
    fn HandleMouseReleaseForMiddleClickAutoscroll(&self, middle_button: bool);
    fn SetCaretBlinkingSuspended(&self, suspended: bool);
    fn FiresTouchEvents(&self, from_touch: bool) -> InputDeviceCapabilitiesRef;
    fn WillDispatchPointerDown(&self, node: NodeRef);
    fn DidDispatchMouseDown(&self, node: NodeRef);
    fn TargetSubframe(
        &self,
        result: &R,
    ) -> (Option<Rc<dyn EventHandlerFrame<'scope, R> + 'scope>>, bool);
    fn SubframeForTargetNode(
        &self,
        node: Option<&NodeRef>,
    ) -> Option<Rc<dyn EventHandlerFrame<'scope, R> + 'scope>>;
    fn MouseDidMoveOverElement(&self, location: &HitTestLocation, result: &R);
    fn ClearToolTip(&self);
    fn AnchorTrackerOnMouseMove(&self, event: &WebMouseEvent);
    fn RecheckMenuSafeTriangle(&self);
    fn SelectAndSetCursor(&self, location: &HitTestLocation, result: &R);
    fn UpdateStyleAndLayoutForContextMenu(&self);
    fn CursorUpdateTimerFired(&self);
}

// Source: event_handler.cc:144-152.
fn ShouldRefetchEventTarget<R: EventHandlerHitResult>(result: &R) -> bool {
    let Some(node) = result.InnerNode() else {
        return true;
    };
    if !node.HasParent() {
        return true;
    }
    node.IsShadowRootWithInputHost()
}

// Source: event_handler.cc:196-216.
fn HasTitleAndNotSVGUseElement<R: EventHandlerHitResult>(result: &R) -> bool {
    let Some(node) = result.InnerNode() else {
        return false;
    };
    if !node.IsElementWithTitle() {
        return false;
    }
    if node.IsSVGUseElement() || node.ContainingShadowRootHasSVGUseHost() {
        return false;
    }
    true
}

// Source: event_handler.h:424-461,467-489; event_handler.cc:305-325.
// Rc owns exactly the traced source identities. RefCells cover only individual
// source fields and never remain borrowed while dispatching listeners.
//
// The cursor-visible slots are a distinct retained source owner so the Page
// facade and a fully assembled EventHandler can share the same mouse-capture
// and accessibility state. Pointer capture is intentionally not stored here.
pub struct EventHandlerCursorState {
    is_widget_capturing_mouse_events_: Cell<bool>,
    last_scrollbar_under_mouse_: RefCell<Option<Rc<dyn EventHandlerScrollbar>>>,
    capturing_mouse_events_element_: RefCell<Option<NodeRef>>,
    cursor_accessibility_scale_factor_: Cell<f32>,
    touch_adjustment_unique_event_id_: Cell<u32>,
    touch_adjustment_point_: Cell<foundation::gfx::PointF>,
    last_mouse_move_subframe_: RefCell<Option<Weak<()>>>,
}

impl Default for EventHandlerCursorState {
    fn default() -> Self {
        Self {
            is_widget_capturing_mouse_events_: Cell::new(false),
            last_scrollbar_under_mouse_: RefCell::new(None),
            capturing_mouse_events_element_: RefCell::new(None),
            cursor_accessibility_scale_factor_: Cell::new(1.0),
            touch_adjustment_unique_event_id_: Cell::new(0),
            touch_adjustment_point_: Cell::new(foundation::gfx::PointF::default()),
            last_mouse_move_subframe_: RefCell::new(None),
        }
    }
}

impl EventHandlerCursorState {
    /// EventHandler::last_mouse_move_event_subframe_ over the embedding's
    /// original LocalFrame identity. The state never retains a Frame tree.
    pub fn LastMouseMoveSubframe(&self) -> Option<Rc<()>> {
        self.last_mouse_move_subframe_.borrow().as_ref()?.upgrade()
    }
    pub fn SetLastMouseMoveSubframe(&self, frame: Option<&Rc<()>>) {
        *self.last_mouse_move_subframe_.borrow_mut() = frame.map(Rc::downgrade);
    }
    // Source: event_handler.cc:2625-2630. The client sends a typed host
    // notification; the same retained bit de-duplicates both assembly paths.
    pub fn CaptureMouseEventsToWidget(&self, capture: bool, set_mouse_capture: impl FnOnce(bool)) {
        if capture == self.is_widget_capturing_mouse_events_.get() {
            return;
        }
        set_mouse_capture(capture);
        self.is_widget_capturing_mouse_events_.set(capture);
    }
    pub fn LastScrollbarUnderMouse(&self) -> Option<Rc<dyn EventHandlerScrollbar>> {
        self.last_scrollbar_under_mouse_.borrow().clone()
    }
    pub fn ClearLastScrollbarUnderMouse(&self) {
        self.last_scrollbar_under_mouse_.replace(None);
    }
    // Source: event_handler.cc:2565-2578. Both production native pointer
    // boundaries and a full EventHandler update this same retained identity.
    pub fn UpdateLastScrollbarUnderMouse(
        &self,
        scrollbar: Option<Rc<dyn EventHandlerScrollbar>>,
        set_last: bool,
    ) {
        let last = self.LastScrollbarUnderMouse();
        let same = match (&last, &scrollbar) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if !same {
            if let Some(last) = last {
                last.MouseExited();
            }
            if set_last {
                if let Some(scrollbar) = scrollbar.as_ref() {
                    scrollbar.MouseEntered();
                }
            }
            self.last_scrollbar_under_mouse_
                .replace(if set_last { scrollbar } else { None });
        }
    }
    // Source event_handler.cc:1793-1797; shared with the retained EventHandler.
    pub fn CacheTouchAdjustmentResult(&self, id: u32, point: foundation::gfx::PointF) {
        self.touch_adjustment_unique_event_id_.set(id);
        self.touch_adjustment_point_.set(point);
    }
    pub fn TouchAdjustmentUniqueEventId(&self) -> u32 {
        self.touch_adjustment_unique_event_id_.get()
    }
    pub fn TouchAdjustmentPoint(&self) -> foundation::gfx::PointF {
        self.touch_adjustment_point_.get()
    }
    pub fn ClearMouseCapture(&self) {
        self.capturing_mouse_events_element_.replace(None);
    }

    pub fn SetMouseCapture(&self, element: Option<NodeRef>) {
        self.capturing_mouse_events_element_.replace(element);
    }

    pub fn HasCapturingMouseEventsElement(&self) -> bool {
        self.capturing_mouse_events_element_.borrow().is_some()
    }

    pub fn CursorAccessibilityScaleFactor(&self) -> f32 {
        self.cursor_accessibility_scale_factor_.get()
    }

    pub fn SetCursorAccessibilityScaleFactor(&self, scale: f32) {
        self.cursor_accessibility_scale_factor_.set(scale);
    }
}

/// Retained EventHandler identity and source fields. A transaction binds live
/// services to this owner; it never constructs another mouse/gesture state.
pub struct EventHandlerState {
    timer_generations_: [Cell<u64>; 3],
    delayed_navigation_task_handle_: RefCell<Option<Rc<dyn interaction_default::core::html::html_anchor_element::AnchorNavigationTaskHandle>>>,
    cursor_state_: Rc<EventHandlerCursorState>,
    capturing_subframe_element_: RefCell<Option<NodeRef>>,
    last_mouse_move_event_subframe_: RefCell<Option<Rc<()>>>,
    drag_target_: RefCell<Option<NodeRef>>,
    should_only_fire_drag_over_event_: Cell<bool>,
    frame_set_being_resized_: RefCell<Option<NodeRef>>,
    last_show_press_timestamp_: Cell<Option<TimeTicks>>,
    last_deferred_tap_element_: RefCell<Option<NodeRef>>,
    should_use_touch_event_adjusted_point_: Cell<bool>,
    discarded_mouse_down_: Cell<Option<(i32, TimeTicks)>>,
    discarded_tap_: Cell<Option<(i32, TimeTicks)>>,
}
impl EventHandlerState {
    pub fn SetDelayedNavigationTaskHandle(
        &self,
        handle: Rc<
            dyn interaction_default::core::html::html_anchor_element::AnchorNavigationTaskHandle,
        >,
    ) {
        let previous = self.delayed_navigation_task_handle_.replace(Some(handle));
        drop(previous);
    }
    pub fn NextTimerTask(
        &self,
        frame_identity: Rc<()>,
        kind: EventHandlerTimerKind,
    ) -> EventHandlerTimerTask {
        self.InvalidateTimer(kind);
        EventHandlerTimerTask {
            frame_identity,
            kind,
            generation: self.timer_generations_[kind.Index()].get(),
        }
    }
    pub fn InvalidateTimer(&self, kind: EventHandlerTimerKind) {
        let generation = &self.timer_generations_[kind.Index()];
        generation.set(generation.get().wrapping_add(1));
    }
    pub fn IsCurrentTimerTask(&self, task: &EventHandlerTimerTask) -> bool {
        self.timer_generations_[task.kind.Index()].get() == task.generation
    }

    fn ClearBeforeManagers(&self) {
        self.last_mouse_move_event_subframe_.replace(None);
        self.cursor_state_.last_scrollbar_under_mouse_.replace(None);
        self.frame_set_being_resized_.replace(None);
        self.drag_target_.replace(None);
        self.should_only_fire_drag_over_event_.set(false);
        self.cursor_state_.ClearMouseCapture();
        self.capturing_subframe_element_.replace(None);
    }
    fn ClearAfterManagers(&self) {
        self.last_show_press_timestamp_.set(None);
        self.last_deferred_tap_element_.replace(None);
        self.should_use_touch_event_adjusted_point_.set(false);
        self.cursor_state_.touch_adjustment_unique_event_id_.set(0);
    }
    /// Document detachment can release source fields without binding detached
    /// DOM/Layout services. The embedding clears managers at their source
    /// lifecycle positions and cancels the original task-runner tokens.
    pub fn ClearRetainedState(&self) {
        for timer in [
            EventHandlerTimerKind::Hover,
            EventHandlerTimerKind::CursorUpdate,
            EventHandlerTimerKind::ActiveInterval,
        ] {
            self.InvalidateTimer(timer);
        }
        self.ClearBeforeManagers();
        self.ClearAfterManagers();
        drop(self.delayed_navigation_task_handle_.take());
    }
    // Source: event_handler.cc:2452-2457. Document's activation notification
    // clears this same retained deferred target, including scoped bindings.
    pub fn NotifyElementActivated(&self, active_interval_timer: &dyn EventHandlerTimer) {
        active_interval_timer.Stop();
        self.last_deferred_tap_element_.replace(None);
    }
    pub fn WithCursorState(cursor_state: Rc<EventHandlerCursorState>) -> Rc<Self> {
        Rc::new(Self {
            timer_generations_: Default::default(),
            delayed_navigation_task_handle_: RefCell::new(None),
            cursor_state_: cursor_state,
            capturing_subframe_element_: RefCell::new(None),
            last_mouse_move_event_subframe_: RefCell::new(None),
            drag_target_: RefCell::new(None),
            should_only_fire_drag_over_event_: Cell::new(false),
            frame_set_being_resized_: RefCell::new(None),
            last_show_press_timestamp_: Cell::new(None),
            last_deferred_tap_element_: RefCell::new(None),
            should_use_touch_event_adjusted_point_: Cell::new(false),
            discarded_mouse_down_: Cell::new(None),
            discarded_tap_: Cell::new(None),
        })
    }
}
pub struct EventHandler<'scope, R: EventHandlerHitResult> {
    state_: Rc<EventHandlerState>,
    frame_: Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
    selection_controller_: Rc<dyn EventHandlerSelection<R> + 'scope>,
    hover_timer_: Rc<dyn EventHandlerTimer + 'scope>,
    cursor_update_timer_: Rc<dyn EventHandlerTimer + 'scope>,
    event_handler_registry_: Rc<dyn EventHandlerRegistry + 'scope>,
    scroll_manager_: Rc<dyn EventHandlerScrollPort<R> + 'scope>,
    mouse_event_manager_: Rc<dyn EventHandlerMouse<R> + 'scope>,
    mouse_wheel_event_manager_: Rc<dyn EventHandlerWheel + 'scope>,
    keyboard_event_manager_: Rc<dyn EventHandlerKeyboard + 'scope>,
    pointer_event_manager_: Rc<dyn EventHandlerPointer<'scope, R> + 'scope>,
    gesture_manager_: Rc<dyn EventHandlerGesture<R> + 'scope>,
    gesture_targeting_: Rc<dyn gesture::EventHandlerGestureTargeting<'scope, R> + 'scope>,
    active_interval_timer_: Rc<dyn EventHandlerTimer + 'scope>,
}
impl<R: EventHandlerHitResult> std::ops::Deref for EventHandler<'_, R> {
    type Target = EventHandlerState;
    fn deref(&self) -> &Self::Target {
        &self.state_
    }
}

// Source: event_handler.cc:868-1356 (MouseEventWithHitTestResults stack value).
// R is the canonical query result, not a retained projection of its tree.
pub struct MouseEventWithHitTestResults<R> {
    pub event: WebMouseEvent,
    pub location: HitTestLocation,
    pub result: R,
}

// Source: event_handler.h:87-99,119-124,255-257,268-276.
// Inline header definitions precede the out-of-line .cc definitions below.
impl<'scope, R: EventHandlerHitResult> EventHandler<'scope, R> {
    // Source: event_handler.h:87-90.
    pub fn NodeChildrenWillBeRemoved(&self, container: &NodeRef) {
        self.mouse_event_manager_
            .NodeChildrenWillBeRemoved(container);
        self.pointer_event_manager_
            .NodeChildrenWillBeRemoved(container);
    }

    // Source: event_handler.h:92-95.
    pub fn NodeWillBeRemoved(&self, node: &NodeRef) {
        self.mouse_event_manager_.NodeWillBeRemoved(node);
        self.pointer_event_manager_.NodeWillBeRemoved(node);
    }

    // Source: event_handler.h:96-99.
    pub fn HandlePseudoElementRemoval(&self, pseudo: &NodeRef) {
        self.mouse_event_manager_.HandlePseudoElementRemoval(pseudo);
        self.pointer_event_manager_
            .HandlePseudoElementRemoval(pseudo);
    }

    // Source: event_handler.cc:2477-2493. Release hit testing updates the
    // original hover/active owners and routes completion to its target frame.
    pub fn DragSourceEndedAt(
        &self,
        event: &WebMouseEvent,
        operation: interaction_dispatch::core::clipboard::data_transfer::DragOperation,
    ) {
        let hit = self.frame_.HitTesting();
        let location = hit.ContentPointFromRootFrame(event);
        let result = hit.PerformMouseEventHitTest(RequestType::kRelease as u32, &location, event);
        if let Some(frame) = self
            .frame_
            .SubframeForTargetNode(result.InnerNode().as_ref())
        {
            frame.Handler().DragSourceEndedAt(event, operation);
            return;
        }
        self.mouse_event_manager_
            .DragSourceEndedAt(event, operation);
        self.gesture_manager_.HandleTouchDragEnd(
            event,
            operation == interaction_dispatch::core::clipboard::data_transfer::DragOperation::None,
        );
    }

    // Source: event_handler.h:119.
    pub fn MousePressed(&self) -> bool {
        self.mouse_event_manager_.MousePressed()
    }
    // Source: event_handler.cc:468-471. At the local FrameRoot the visual
    // viewport-to-frame conversion is its viewport-to-root conversion.
    pub fn LastKnownMousePositionInRootFrame(&self) -> foundation::gfx::PointF {
        *self
            .frame_
            .HitTesting()
            .ViewportToFrame(&self.mouse_event_manager_.LastKnownMousePositionInViewport())
            .TransformedPoint()
    }
    // Source: event_handler.cc:353-369. Layout and controller are aliases of
    // their existing owners; the source frame keeps mouse position/click state.
    pub fn StartMiddleClickAutoscroll<
        F: interaction_default::core::page::autoscroll_controller::AutoscrollFrame + ?Sized,
    >(
        &self,
        layout: Rc<dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout>,
        controller: &interaction_default::core::page::autoscroll_controller::AutoscrollController<
            '_,
            F,
        >,
    ) {
        if !layout.IsBox() {
            return;
        }
        let scrollable = layout
            .FindAutoscrollable(true)
            .expect("source middle-click ancestor is autoscrollable");
        let position = self.LastKnownMousePositionInRootFrame();
        controller.StartMiddleClickAutoscroll(
            scrollable,
            interaction_input::public::common::input::web_pointer_properties::PointF::new(
                position.x(),
                position.y(),
            ),
            self.mouse_event_manager_.LastKnownMouseScreenPosition(),
        );
        self.mouse_event_manager_.InvalidateClick();
    }

    /// The cursor facade reads EventHandler's original capture slot. Native
    /// mouse capture and pointer capture are distinct source states.
    pub fn HasCapturingMouseEventsElement(&self) -> bool {
        self.cursor_state_.HasCapturingMouseEventsElement()
    }

    // Source: event_handler.h:292-299.
    pub fn CursorAccessibilityScaleFactor(&self) -> f32 {
        self.cursor_state_.CursorAccessibilityScaleFactor()
    }
    pub fn SetCursorAccessibilityScaleFactor(&self, scale: f32) {
        self.cursor_state_.SetCursorAccessibilityScaleFactor(scale);
    }
    pub fn CursorState(&self) -> &Rc<EventHandlerCursorState> {
        &self.cursor_state_
    }

    // Source: event_handler.h:120-122.
    pub fn IsMousePositionUnknown(&self) -> bool {
        self.mouse_event_manager_.IsMousePositionUnknown()
    }

    // Source: event_handler.h:124.
    // Source event_handler.cc:1778-1780; event_handler.h:235.
    pub fn SetMouseDownMayStartAutoscroll(&self) {
        self.mouse_event_manager_.SetMouseDownMayStartAutoscroll();
    }
    pub fn InResizeMode(&self) -> bool {
        self.scroll_manager_.InResizeMode()
    }
    pub fn ClearMouseEventManager(&self) {
        self.mouse_event_manager_.Clear();
    }

    // Source: event_handler.h:255-257.
    pub fn GetSelectionController(&self) -> &Rc<dyn EventHandlerSelection<R> + 'scope> {
        &self.selection_controller_
    }

    // Source: event_handler.h:268-270.
    pub fn GetEventHandlerRegistry(&self) -> &Rc<dyn EventHandlerRegistry + 'scope> {
        &self.event_handler_registry_
    }

    // Source: event_handler.h:272.
    pub fn GetGestureManager(&self) -> &Rc<dyn EventHandlerGesture<R> + 'scope> {
        &self.gesture_manager_
    }

    // Source: event_handler.h:274-276.
    pub fn GetKeyboardEventManager(&self) -> &Rc<dyn EventHandlerKeyboard + 'scope> {
        &self.keyboard_event_manager_
    }
}

impl<'scope, R: EventHandlerHitResult> EventHandler<'scope, R> {
    // Source: event_handler.cc:269-302.
    pub fn new(
        frame: Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
        factory: &dyn EventHandlerFactory<'scope, R>,
    ) -> Self {
        Self::WithCursorState(frame, factory, Rc::new(EventHandlerCursorState::default()))
    }

    /// Constructs EventHandler around the original cursor-visible state owner.
    /// Page-facing assembly uses the same Rc while the rest of EventHandler is
    /// introduced incrementally; no cursor-only state mirror is created.
    pub fn WithCursorState(
        frame: Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
        factory: &dyn EventHandlerFactory<'scope, R>,
        cursor_state: Rc<EventHandlerCursorState>,
    ) -> Self {
        Self::WithState(
            frame,
            factory,
            EventHandlerState::WithCursorState(cursor_state),
        )
    }

    /// Bind live source ports to the same retained EventHandler identity.
    pub fn WithState(
        frame: Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
        factory: &dyn EventHandlerFactory<'scope, R>,
        state: Rc<EventHandlerState>,
    ) -> Self {
        let selection_controller = factory.SelectionController();
        let hover_timer = factory.HoverTimer();
        let cursor_update_timer = factory.CursorUpdateTimer();
        let event_handler_registry = factory.EventHandlerRegistry(frame.IsLocalRoot());
        let scroll_manager = factory.ScrollManager();
        let mouse_event_manager = factory.MouseEventManager(&scroll_manager);
        let mouse_wheel_event_manager = factory.MouseWheelEventManager(&scroll_manager);
        let keyboard_event_manager = factory.KeyboardEventManager(&scroll_manager);
        let pointer_event_manager = factory.PointerEventManager(&mouse_event_manager);
        let gesture_manager = factory.GestureManager(
            &scroll_manager,
            &mouse_event_manager,
            &pointer_event_manager,
            &selection_controller,
        );
        let active_interval_timer = factory.ActiveIntervalTimer();
        Self {
            state_: state,
            frame_: frame,
            selection_controller_: selection_controller,
            hover_timer_: hover_timer,
            cursor_update_timer_: cursor_update_timer,
            event_handler_registry_: event_handler_registry,
            scroll_manager_: scroll_manager,
            mouse_event_manager_: mouse_event_manager,
            mouse_wheel_event_manager_: mouse_wheel_event_manager,
            keyboard_event_manager_: keyboard_event_manager,
            pointer_event_manager_: pointer_event_manager,
            gesture_manager_: gesture_manager,
            gesture_targeting_: factory.GestureTargeting(),
            active_interval_timer_: active_interval_timer,
        }
    }

    // Source: event_handler.cc:327-347.
    // Source event_handler.cc:1555-1561; replacing/dropping TaskHandle cancels
    // the same scheduled task. Native widget double-click cancellation uses it.
    pub fn SetDelayedNavigationTaskHandle(
        &self,
        handle: Rc<
            dyn interaction_default::core::html::html_anchor_element::AnchorNavigationTaskHandle,
        >,
    ) {
        self.state_.SetDelayedNavigationTaskHandle(handle);
    }
    pub fn CancelDelayedNavigationTask(&self) {
        let previous = self.delayed_navigation_task_handle_.take();
        drop(previous);
    }
    pub fn Clear(&self) {
        self.hover_timer_.Stop();
        self.cursor_update_timer_.Stop();
        self.active_interval_timer_.Stop();
        self.state_.ClearBeforeManagers();
        self.pointer_event_manager_.Clear();
        self.scroll_manager_.Clear();
        self.gesture_manager_.Clear();
        self.mouse_event_manager_.Clear();
        self.mouse_wheel_event_manager_.Clear();
        self.state_.ClearAfterManagers();
    }

    /// Deliver a one-shot task only to its original retained frame and current
    /// timer generation. Stale deliveries after restart/detach are discarded.
    pub fn RunTimerTask(&self, task: &EventHandlerTimerTask) -> bool {
        if !Rc::ptr_eq(&self.frame_.Identity(), &task.frame_identity)
            || !self.state_.IsCurrentTimerTask(task)
        {
            return false;
        }
        self.state_.InvalidateTimer(task.kind);
        match task.kind {
            EventHandlerTimerKind::Hover => self.HoverTimerFired(),
            EventHandlerTimerKind::CursorUpdate => self.frame_.CursorUpdateTimerFired(),
            EventHandlerTimerKind::ActiveInterval => self.ActiveIntervalTimerFired(),
        }
        true
    }

    // Source: event_handler.cc:349-351.
    pub fn UpdateSelectionForMouseDrag(&self) {
        self.mouse_event_manager_.UpdateSelectionForMouseDrag();
    }

    // Source: event_handler.cc:371-399.
    pub fn PerformHitTest(
        &self,
        location: &HitTestLocation,
        result: &mut R,
        no_lifecycle_update: bool,
    ) {
        let hit = self.frame_.HitTesting();
        if !hit.HasContentLayoutObject()
            || !hit.HasView()
            || !hit.DidFirstLayout()
            || !hit.LifecycleUpdatesActive()
        {
            return;
        }
        hit.HitTest(location, result, no_lifecycle_update);
        let hit_type = result.HitTestRequestType();
        if hit_type & RequestType::kReadOnly as u32 == 0 {
            hit.UpdateHoverActiveState(
                hit_type & RequestType::kActive as u32 != 0,
                hit_type & RequestType::kMove as u32 == 0,
                result.InnerPossiblyPseudoElement(),
            );
        }
    }

    // Source: event_handler.cc:401-451. The stop node/callback overload is not
    // selected yet: the caller here requests the source default null values.
    pub fn HitTestResultAtLocation(
        &self,
        location: &HitTestLocation,
        hit_type: u32,
        no_lifecycle_update: bool,
    ) -> R {
        if self.frame_.HasPage() {
            let root = self.frame_.LocalFrameRoot();
            if !Rc::ptr_eq(&self.frame_.Identity(), &root.Identity())
                && self.frame_.HitTesting().HasView()
                && root.HitTesting().HasView()
            {
                if location.IsRectBasedTest() {
                    debug_assert!(location.IsRectilinear());
                }
                let adjusted = self.frame_.HitTesting().ConvertLocationToRoot(
                    location,
                    hit_type & RequestType::kHitTestVisualOverflow as u32 != 0,
                );
                let adjusted = root.HitTesting().ConvertLocationFromRoot(&adjusted);
                return root.Handler().HitTestResultAtLocation(
                    &adjusted,
                    hit_type,
                    no_lifecycle_update,
                );
            }
        }
        let hit_type = hit_type | RequestType::kAllowChildFrameContent as u32;
        let mut result = self.frame_.HitTesting().NewResult(hit_type, location);
        self.PerformHitTest(location, &mut result, no_lifecycle_update);
        result
    }

    // Source: event_handler.cc:856-858.
    pub fn DispatchBufferedTouchEvents(&self) -> WebInputEventResult {
        self.pointer_event_manager_.FlushEvents()
    }

    // Source: event_handler.h:207-214; event_handler.cc:2184-2223.
    pub fn SendContextMenuEvent(
        &self,
        event: &WebMouseEvent,
        override_target_element: Option<NodeRef>,
    ) -> WebInputEventResult {
        SendLocatedContextMenu(self, event, override_target_element)
    }

    // Source: event_handler.cc:860-866.
    pub fn HandlePointerEvent(
        &self,
        event: &WebPointerEvent,
        coalesced: &[WebPointerEvent],
        predicted: &[WebPointerEvent],
    ) -> WebInputEventResult {
        self.pointer_event_manager_
            .HandlePointerEvent(event, coalesced, predicted)
    }

    // Source: event_handler.cc:868-1033.
    pub fn HandleMousePressEvent(&self, event: &WebMouseEvent) -> WebInputEventResult {
        if event.pointer_properties.button == Button::kNoButton {
            return WebInputEventResult::kHandledSuppressed;
        }
        self.cursor_state_.ClearMouseCapture();
        self.mouse_event_manager_
            .HandleMousePressEventUpdateStates(event);
        if !self.frame_.HitTesting().HasView() {
            return WebInputEventResult::kNotHandled;
        }
        let document_point = self.frame_.HitTesting().DocumentPointForMousePress(event);
        let mut mev = self.GetMouseEventTarget(RequestType::kActive as u32, event);
        let Some(node) = mev.result.InnerNode() else {
            if self.PassMousePressEventToScrollbar(&mev) {
                return WebInputEventResult::kHandledSystem;
            }
            self.mouse_event_manager_.InvalidateClick();
            return WebInputEventResult::kNotHandled;
        };
        self.mouse_event_manager_
            .SetMousePressNode(Rc::clone(&node));
        self.frame_
            .SetSequentialFocusNavigationStartingPoint(Rc::clone(&node));
        if let Some(subframe) = self.frame_.TargetSubframe(&mev.result).0 {
            let result = self.PassMousePressEventToSubframe(&mev, &subframe);
            if self.mouse_event_manager_.MousePressed() {
                self.cursor_state_
                    .SetMouseCapture(mev.result.InnerElement());
                self.capturing_subframe_element_
                    .replace(mev.result.InnerElement());
            }
            self.mouse_event_manager_.InvalidateClick();
            return result;
        }
        // Source: event_handler.cc:913-920. TODO(instrumentation): count
        // kInputEventToRecentlyMovedIframeMistakenlyDiscarded for the retained
        // discarded target/time pair at the source 5-second interval.
        if self.frame_.ShouldDiscardEventTargetingFrame(&mev.event) {
            self.discarded_mouse_down_
                .set(Some((node.DomNodeId(), event.TimeStamp())));
            return WebInputEventResult::kHandledSuppressed;
        }
        self.discarded_mouse_down_.set(None);
        self.frame_.NotifyUserActivation();
        if self.frame_.MiddleClickAutoscrollEnabled() {
            let in_progress = self.scroll_manager_.MiddleClickAutoscrollInProgress();
            self.scroll_manager_.StopMiddleClickAutoscroll();
            if in_progress {
                self.mouse_event_manager_.InvalidateClick();
                return WebInputEventResult::kHandledSuppressed;
            }
        }
        self.mouse_event_manager_.SetClickCount(event.click_count);
        self.mouse_event_manager_
            .SetMouseDownElement(mev.result.InnerPossiblyPseudoElement());
        if !event.FromTouch() {
            self.frame_.SetCaretBlinkingSuspended(true);
        }
        if event.pointer_properties.button == Button::kLeft && mev.result.InnerNode().is_some() {
            let mut result = mev.result.clone();
            result.SetToShadowHostIfInUAShadowRoot();
            if let Some(node) = result.InnerNode() {
                self.frame_.WillDispatchPointerDown(node);
            }
        }
        let mut result = self.DispatchMousePointerEvent(
            Type::kPointerDown,
            mev.result.InnerPossiblyPseudoElement(),
            &mev.event,
            &[],
            &[],
            false,
        );
        if matches!(
            result,
            WebInputEventResult::kNotHandled | WebInputEventResult::kHandledSuppressed
        ) && self.frame_.HitTesting().HasView()
            && self
                .scroll_manager_
                .TrySetResizeScrollableArea(&mev.result, event)
        {
            return WebInputEventResult::kHandledSystem;
        }
        self.selection_controller_.InitializeSelectionState();
        let hit = self
            .frame_
            .HitTesting()
            .HitTestResultInFrame(RequestType::kReadOnly as u32, &document_point);
        let capabilities = self.frame_.FiresTouchEvents(event.FromTouch());
        if result == WebInputEventResult::kNotHandled {
            result = self
                .mouse_event_manager_
                .HandleMouseFocus(&hit, &capabilities);
        }
        if (result == WebInputEventResult::kNotHandled || mev.result.GetScrollbar().is_some())
            && self.frame_.IsAttached()
            && !self.frame_.IsOutermostMainFrame()
        {
            self.CaptureMouseEventsToWidget(true);
        }
        if self.PassMousePressEventToScrollbar(&mev) {
            result = WebInputEventResult::kHandledSystem;
        }
        if result == WebInputEventResult::kNotHandled {
            if ShouldRefetchEventTarget(&mev.result) {
                mev.result = self.frame_.HitTesting().PerformMouseEventHitTest(
                    RequestType::kReadOnly as u32 | RequestType::kActive as u32,
                    &document_point,
                    event,
                );
                mev.location = document_point;
            }
            result = self.mouse_event_manager_.HandleMousePressEvent(&mev);
        }
        if event.pointer_properties.button == Button::kLeft && mev.result.InnerNode().is_some() {
            debug_assert_eq!(event.GetType(), Type::kMouseDown);
            let mut hit = mev.result.clone();
            hit.SetToShadowHostIfInUAShadowRoot();
            if let Some(node) = hit.InnerNode() {
                self.frame_.DidDispatchMouseDown(node);
            }
        }
        result
    }

    // Source: event_handler.cc:1035-1060.
    pub fn HandleMouseMoveEvent(
        &self,
        event: &WebMouseEvent,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
    ) -> WebInputEventResult {
        debug_assert_eq!(event.GetType(), Type::kMouseMove);
        let location = HitTestLocation::new();
        let mut hovered = (
            location,
            self.frame_.HitTesting().NewResult(
                RequestType::kReadOnly as u32 | RequestType::kActive as u32,
                &location,
            ),
        );
        let result =
            self.HandleMouseMoveOrLeaveEvent(event, coalesced, predicted, Some(&mut hovered));
        if !self.frame_.HasPage() {
            return result;
        }
        let (location, mut hit) = hovered;
        if !HasTitleAndNotSVGUseElement(&hit) {
            hit.SetToShadowHostIfInUAShadowRoot();
        }
        self.frame_.MouseDidMoveOverElement(&location, &hit);
        result
    }

    // Source: event_handler.cc:1062-1073.
    pub fn HandleMouseLeaveEvent(&self, event: &WebMouseEvent) {
        debug_assert_eq!(event.GetType(), Type::kMouseLeave);
        if self.frame_.HasPage() {
            self.frame_.ClearToolTip();
        }
        self.HandleMouseMoveOrLeaveEvent(event, &[], &[], None);
        self.pointer_event_manager_.RemoveLastMousePosition();
    }

    // Source: event_handler.cc:1075-1263.
    pub fn HandleMouseMoveOrLeaveEvent(
        &self,
        event: &WebMouseEvent,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
        mut hovered: Option<&mut (HitTestLocation, R)>,
    ) -> WebInputEventResult {
        debug_assert!(self.frame_.HitTesting().HasView());
        debug_assert!(matches!(
            event.GetType(),
            Type::kMouseMove | Type::kMouseLeave
        ));
        self.mouse_event_manager_.SetLastKnownMousePosition(event);
        self.hover_timer_.Stop();
        self.cursor_update_timer_.Stop();
        self.mouse_event_manager_.HandleSvgPanIfNeeded(false);
        if event.GetType() == Type::kMouseMove {
            self.frame_.AnchorTrackerOnMouseMove(event);
            self.frame_.RecheckMenuSafeTriangle();
        }
        ResetMouseMoveWithNoButton(
            &self.cursor_state_,
            event,
            || self.mouse_event_manager_.ClearDragHeuristicState(),
            || self.ReleaseMouseCaptureFromLocalRoot(),
        );
        if self.frame_.MiddleClickAutoscrollEnabled() && self.frame_.HasPage() {
            self.frame_.HandleMouseMoveForMiddleClickAutoscroll(event);
        }
        let frame_set = self.frame_set_being_resized_.borrow().clone();
        if let Some(frame_set) = frame_set {
            return self.DispatchMousePointerEvent(
                Type::kPointerMove,
                Some(frame_set),
                event,
                coalesced,
                predicted,
                false,
            );
        }
        if HandleLastScrollbarMouseMove(
            &self.cursor_state_,
            event,
            self.mouse_event_manager_.MousePressed(),
        ) {
            return WebInputEventResult::kHandledSystem;
        }
        let mut hit_type = RequestType::kMove as u32;
        if self.mouse_event_manager_.MousePressed() {
            hit_type |= RequestType::kActive as u32;
        }
        if self.pointer_event_manager_.IsAnyTouchActive() && event.GetType() == Type::kMouseMove {
            hit_type |= RequestType::kActive as u32 | RequestType::kReadOnly as u32;
        }
        let mev = if event.GetType() == Type::kMouseLeave {
            self.frame_.HitTesting().UpdateHoverActiveState(
                hit_type & RequestType::kActive as u32 != 0,
                false,
                None,
            );
            let location = HitTestLocation::new();
            let result = self.frame_.HitTesting().NewResult(hit_type, &location);
            MouseEventWithHitTestResults {
                event: event.clone(),
                location,
                result,
            }
        } else {
            self.GetMouseEventTarget(hit_type, event)
        };
        if let Some(hovered) = hovered.as_deref_mut() {
            *hovered = (mev.location, mev.result.clone());
        }
        let mut scrollbar = None;
        if self.scroll_manager_.InResizeMode() {
            self.scroll_manager_.Resize(&mev.event);
        } else {
            scrollbar = mev.result.GetScrollbar();
            self.UpdateLastScrollbarUnderMouse(
                scrollbar.clone(),
                !self.mouse_event_manager_.MousePressed(),
            );
        }
        let mut result = WebInputEventResult::kNotHandled;
        let (current_subframe, remote) = self.frame_.TargetSubframe(&mev.result);
        let last_subframe = self
            .last_mouse_move_event_subframe_
            .borrow()
            .as_ref()
            .and_then(|identity| self.frame_.FrameForIdentity(identity));
        if let Some(last) = last_subframe {
            if last.IsDescendantOf(&self.frame_)
                && !same_frame(Some(&last), current_subframe.as_ref())
            {
                let mut leave = mev.event.clone();
                leave.SetType(Type::kMouseLeave);
                last.Handler().HandleMouseLeaveEvent(&leave);
                last.Handler()
                    .mouse_event_manager_
                    .SetLastMousePositionAsUnknown();
            }
        }
        if let Some(subframe) = current_subframe.as_ref() {
            self.pointer_event_manager_
                .SendMouseAndPointerBoundaryEvents(
                    self.EffectiveMouseEventTargetElement(mev.result.InnerPossiblyPseudoElement()),
                    &mev.event,
                );
            if subframe.HitTesting().HasView() {
                result = self
                    .PassMouseMoveEventToSubframe(&mev, coalesced, predicted, subframe, hovered);
            }
        } else {
            if let Some(scrollbar) = scrollbar {
                if !self.mouse_event_manager_.MousePressed() {
                    scrollbar.MouseMoved(&mev.event);
                }
            }
            self.pointer_event_manager_
                .SendEffectivePanActionAtPointer(&mev.event, mev.result.InnerNode());
            if !remote && self.frame_.HitTesting().HasView() {
                self.frame_.SelectAndSetCursor(&mev.location, &mev.result);
            }
        }
        self.last_mouse_move_event_subframe_
            .replace(current_subframe.as_ref().map(|frame| frame.Identity()));
        if result != WebInputEventResult::kNotHandled {
            return result;
        }
        result = self.DispatchMousePointerEvent(
            Type::kPointerMove,
            mev.result.InnerPossiblyPseudoElement(),
            &mev.event,
            coalesced,
            predicted,
            false,
        );
        if matches!(
            result,
            WebInputEventResult::kNotHandled | WebInputEventResult::kHandledApplication
        ) {
            result = self.mouse_event_manager_.HandleMouseDraggedEvent(&mev);
        }
        result
    }

    // Source: event_handler.cc:1265-1356.
    pub fn HandleMouseReleaseEvent(&self, event: &WebMouseEvent) -> WebInputEventResult {
        if event.pointer_properties.button == Button::kNoButton {
            return WebInputEventResult::kHandledSuppressed;
        }
        if !event.FromTouch() {
            self.frame_.SetCaretBlinkingSuspended(false);
        }
        if self.frame_.MiddleClickAutoscrollEnabled() && self.frame_.HasPage() {
            self.frame_.HandleMouseReleaseForMiddleClickAutoscroll(
                event.pointer_properties.button == Button::kMiddle,
            );
        }
        self.mouse_event_manager_.ReleaseMousePress();
        self.mouse_event_manager_.SetLastKnownMousePosition(event);
        self.mouse_event_manager_.HandleSvgPanIfNeeded(true);
        let frame_set = self.frame_set_being_resized_.borrow().clone();
        if let Some(frame_set) = frame_set {
            let result = self
                .mouse_event_manager_
                .SetElementUnderMouseAndDispatchMouseUp(
                    self.EffectiveMouseEventTargetElement(Some(frame_set)),
                    event,
                );
            if MouseEvent::WebInputEventModifiersToButtons(event.GetModifiers() as u32) == 0 {
                self.ReleaseMouseCaptureFromLocalRoot();
            }
            return result;
        }
        if HandleLastScrollbarMouseUp(
            self.cursor_state_.as_ref(),
            event,
            || self.mouse_event_manager_.InvalidateClick(),
            || self.ReleaseMouseCaptureFromLocalRoot(),
        ) {
            return self.DispatchMousePointerEvent(
                Type::kPointerUp,
                self.mouse_event_manager_.GetElementUnderMouse(),
                event,
                &[],
                &[],
                false,
            );
        }
        debug_assert!(!event.FromTouch());
        let mev = self.GetMouseEventTarget(RequestType::kRelease as u32, event);
        let subframe = self.frame_.TargetSubframe(&mev.result).0;
        self.cursor_state_.ClearMouseCapture();
        if let Some(subframe) = subframe {
            return self.PassMouseReleaseEventToSubframe(&mev, &subframe);
        }
        let mut result = if self.frame_.ShouldDiscardEventTargetingFrame(&mev.event) {
            WebInputEventResult::kHandledSuppressed
        } else {
            self.DispatchMousePointerEvent(
                Type::kPointerUp,
                mev.result.InnerPossiblyPseudoElement(),
                &mev.event,
                &[],
                &[],
                self.selection_controller_.HasExtendedSelection()
                    && self
                        .selection_controller_
                        .IsSelectionOverLink(&mev.result, &mev.event),
            )
        };
        self.scroll_manager_.ClearResizeScrollableArea(false);
        if result == WebInputEventResult::kNotHandled {
            result = self.mouse_event_manager_.HandleMouseReleaseEvent(&mev);
        }
        self.mouse_event_manager_
            .HandleMouseReleaseEventUpdateStates();
        if MouseEvent::WebInputEventModifiersToButtons(event.GetModifiers() as u32) == 0 {
            self.ReleaseMouseCaptureFromLocalRoot();
        }
        result
    }

    // Source: event_handler.cc:1525-1527.
    pub fn RecomputeMouseHoverStateIfNeeded(&self) {
        self.mouse_event_manager_.RecomputeMouseHoverStateIfNeeded();
    }

    // Source: event_handler.cc:1529-1531.
    pub fn MarkHoverStateDirty(&self) {
        self.mouse_event_manager_.MarkHoverStateDirty();
    }

    // Source: event_handler.cc:1533-1539.
    pub fn EffectiveMouseEventTargetElement(&self, target: Option<NodeRef>) -> Option<NodeRef> {
        self.pointer_event_manager_
            .GetMouseCaptureTarget()
            .or(target)
    }

    // Source: event_handler.cc:1541-1545.
    pub fn OnScrollbarDestroyed(&self, scrollbar: &Rc<dyn EventHandlerScrollbar>) {
        let last = self
            .cursor_state_
            .last_scrollbar_under_mouse_
            .borrow()
            .clone();
        if last
            .as_ref()
            .is_some_and(|last| Rc::ptr_eq(last, scrollbar))
        {
            self.cursor_state_.last_scrollbar_under_mouse_.replace(None);
        }
    }

    // Source: event_handler.cc:1547-1549.
    pub fn GetElementUnderMouse(&self) -> Option<NodeRef> {
        self.mouse_event_manager_.GetElementUnderMouse()
    }

    // Source: event_handler.cc:1551-1553.
    pub fn CurrentTouchDownElement(&self) -> Option<NodeRef> {
        self.pointer_event_manager_.CurrentTouchDownElement()
    }

    // Source: event_handler.cc:1563-1567.
    pub fn IsPointerIdActiveOnFrame(
        &self,
        id: PointerId,
        frame: &Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
    ) -> bool {
        debug_assert!(
            self.frame_.IsLocalRoot() || Rc::ptr_eq(&self.frame_.Identity(), &frame.Identity())
        );
        self.pointer_event_manager_
            .IsPointerIdActiveOnFrame(id, frame)
    }

    // Source: event_handler.cc:1569-1574.
    fn RootFrameTrackedActivePointerInCurrentFrame(&self, id: PointerId) -> bool {
        !self.frame_.IsLocalRoot()
            && self
                .frame_
                .LocalFrameRoot()
                .Handler()
                .IsPointerIdActiveOnFrame(id, &self.frame_)
    }

    // Source: event_handler.cc:1576-1580.
    pub fn AppendTouchIdForCanceledPointerDown(&self, unique_touch_event_id: u32) {
        self.pointer_event_manager_
            .AppendTouchIdForCanceledPointerDown(unique_touch_event_id);
    }

    // Source: event_handler.cc:1582-1585.
    pub fn IsPointerEventActive(&self, id: PointerId) -> bool {
        self.pointer_event_manager_.IsActive(id)
            || self.RootFrameTrackedActivePointerInCurrentFrame(id)
    }

    // Source: event_handler.cc:1587-1597.
    pub fn DetermineActivePointerTrackerFrame(
        &self,
        id: PointerId,
    ) -> Option<Rc<dyn EventHandlerFrame<'scope, R> + 'scope>> {
        if self.IsPointerIdActiveOnFrame(id, &self.frame_) {
            return Some(Rc::clone(&self.frame_));
        }
        if self.RootFrameTrackedActivePointerInCurrentFrame(id) {
            return Some(self.frame_.LocalFrameRoot());
        }
        None
    }

    // Source: event_handler.cc:1599-1612.
    pub fn SetPointerCapture(&self, id: PointerId, target: &NodeRef) {
        let captured = self
            .DetermineActivePointerTrackerFrame(id)
            .is_some_and(|frame| {
                frame
                    .Handler()
                    .pointer_event_manager_
                    .SetPointerCapture(id, target)
            });
        // Source: core/events/pointer_event_factory.cc:235 (kMouseId).
        if captured && id == 1 {
            self.CaptureMouseEventsToWidget(true);
        }
    }

    // Source: event_handler.cc:1614-1626.
    pub fn ReleasePointerCapture(&self, id: PointerId, target: &NodeRef) {
        let released = self
            .DetermineActivePointerTrackerFrame(id)
            .is_some_and(|frame| {
                frame
                    .Handler()
                    .pointer_event_manager_
                    .ReleasePointerCapture(id, target)
            });
        // Source: core/events/pointer_event_factory.cc:235 (kMouseId).
        if released && id == 1 {
            self.CaptureMouseEventsToWidget(false);
        }
    }

    // Source: event_handler.cc:1628-1630.
    pub fn ReleaseMousePointerCapture(&self) {
        self.ReleaseMouseCaptureFromLocalRoot();
    }

    // Source: event_handler.cc:1632-1640.
    pub fn HasPointerCapture(&self, id: PointerId, target: &NodeRef) -> bool {
        self.DetermineActivePointerTrackerFrame(id)
            .is_some_and(|frame| {
                frame
                    .Handler()
                    .pointer_event_manager_
                    .HasPointerCapture(id, target)
            })
    }

    // Source: event_handler.cc:1642-1648.
    pub fn ElementRemoved(&self, target: &NodeRef) {
        if !target.StatePreservingAtomicMoveInProgress() {
            self.pointer_event_manager_.ElementRemoved(target);
        }
        self.mouse_wheel_event_manager_.ElementRemoved(target);
    }

    // Source: event_handler.cc:1650-1652.
    pub fn ResetMousePositionForPointerUnlock(&self) {
        self.pointer_event_manager_.RemoveLastMousePosition();
    }

    // Source: event_handler.cc:1658-1668.
    pub fn DispatchMousePointerEvent(
        &self,
        event_type: Type,
        target: Option<NodeRef>,
        event: &WebMouseEvent,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
        skip_click_dispatch: bool,
    ) -> WebInputEventResult {
        self.pointer_event_manager_.SendMousePointerEvent(
            self.EffectiveMouseEventTargetElement(target),
            event_type,
            event,
            coalesced,
            predicted,
            skip_click_dispatch,
        )
    }

    // Source: event_handler.cc:1678-1687.
    pub fn HandleTargetedMouseEvent(
        &self,
        target: NodeRef,
        event: &WebMouseEvent,
        event_type: &str,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
    ) -> WebInputEventResult {
        self.mouse_event_manager_.SetClickCount(event.click_count);
        self.pointer_event_manager_
            .DirectDispatchMousePointerEvent(target, event, event_type, coalesced, predicted)
    }

    // Source: event_handler.cc:1751-1775. Targeting and cross-frame routing
    // remain the caller's preceding source stage; this entry retains the
    // original GestureManager and discard state for the selected frame.
    pub fn HandleGestureEventInFrame(
        &self,
        event: &super::gesture_manager::GestureEventWithHitTestResults<R>,
    ) -> WebInputEventResult {
        let is_tap = event.event.GetType() == Type::kGestureTap;
        // TODO(instrumentation): count the same recently-moved iframe feature
        // as the mouse branch for the retained discarded target/time pair.
        if self
            .frame_
            .ShouldDiscardGestureEventTargetingFrame(&event.event)
        {
            if is_tap {
                self.discarded_tap_.set(Some((
                    event
                        .result
                        .InnerNode()
                        .expect("targeted tap has an inner node")
                        .DomNodeId(),
                    event.event.TimeStamp(),
                )));
            }
            return WebInputEventResult::kHandledSuppressed;
        }
        if is_tap {
            self.discarded_tap_.set(None);
        }
        self.gesture_manager_.HandleGestureEventInFrame(event)
    }

    // Source: event_handler.cc:2384-2389.
    pub fn ScheduleHoverStateUpdate(&self) {
        if !self.hover_timer_.IsActive() && !self.mouse_event_manager_.IsMousePositionUnknown() {
            self.hover_timer_.StartOneShotMilliseconds(0);
        }
    }

    // Source: event_handler.cc:2391-2399; event_handler.cc:257 (20ms).
    pub fn ScheduleCursorUpdate(&self) {
        debug_assert!(self.frame_.IsLocalRoot());
        if !self.cursor_update_timer_.IsActive() {
            self.cursor_update_timer_.StartOneShotMilliseconds(20);
        }
    }

    // Source: event_handler.cc:2401-2403.
    pub fn CursorUpdatePending(&self) -> bool {
        self.cursor_update_timer_.IsActive()
    }

    // Source: event_handler.cc:2405-2407.
    pub fn IsHandlingKeyEvent(&self) -> bool {
        self.keyboard_event_manager_.IsHandlingKeyEvent()
    }

    // Source: event_handler.cc:2409-2412.
    pub fn SetResizingFrameSet(&self, frame_set: Option<NodeRef>) {
        self.CaptureMouseEventsToWidget(true);
        self.frame_set_being_resized_.replace(frame_set);
    }

    // Source: event_handler.cc:2418-2436.
    pub fn HoverTimerFired(&self) {
        let hit = self.frame_.HitTesting();
        if hit.HasContentLayoutObject() && hit.HasView() {
            let hit_type = RequestType::kMove as u32;
            let location =
                hit.ViewportToFrame(&self.mouse_event_manager_.LastKnownMousePositionInViewport());
            let mut result = hit.NewResult(hit_type, &location);
            hit.HitTest(&location, &mut result, false);
            hit.UpdateHoverActiveState(false, false, result.InnerPossiblyPseudoElement());
        }
    }

    // Source: event_handler.cc:2438-2450.
    pub fn ActiveIntervalTimerFired(&self) {
        let target = self.last_deferred_tap_element_.borrow().clone();
        if let Some(target) = target {
            self.frame_
                .HitTesting()
                .UpdateHoverActiveState(false, true, Some(target));
        }
        self.last_deferred_tap_element_.replace(None);
    }

    // Source: event_handler.cc:2452-2457.
    pub fn NotifyElementActivated(&self) {
        self.state_
            .NotifyElementActivated(self.active_interval_timer_.as_ref());
    }

    // Source: event_handler.cc:2459-2461.
    pub fn HandleAccessKey(&self, event: &WebKeyboardEvent) -> bool {
        self.keyboard_event_manager_.HandleAccessKey(event)
    }

    // Source: event_handler.cc:2463-2466.
    pub fn KeyEvent(&self, event: &WebKeyboardEvent) -> WebInputEventResult {
        self.keyboard_event_manager_.KeyEvent(event)
    }

    // Source: event_handler.cc:2468-2471.
    pub fn DefaultKeyboardEventHandler(&self, event: &KeyboardEventRef) {
        self.keyboard_event_manager_
            .DefaultKeyboardEventHandler(event, self.mouse_event_manager_.MousePressNode());
    }

    // Source: event_handler.cc:2473-2475.
    pub fn DefaultTabEventHandler(&self, event: &KeyboardEventRef) -> bool {
        self.keyboard_event_manager_.DefaultTabEventHandler(event)
    }

    // Source: event_handler.cc:2504-2530.
    pub fn HandleTextInputEvent<T: EventHandlerTextInput>(
        &self,
        text: &[u16],
        underlying_event: Option<EventRef>,
        input_type: T::InputType,
        text_input: &T,
    ) -> bool {
        HandleTextInputEvent(text, underlying_event, input_type, text_input)
    }

    // Source: event_handler.cc:2532-2535.
    pub fn DefaultTextInputEventHandler<T: EventHandlerTextInput>(
        &self,
        event: &EventRef,
        text_input: &T,
    ) {
        DefaultTextInputEventHandler(event, text_input)
    }

    // Source: event_handler.cc:2537-2539.
    pub fn CapsLockStateMayHaveChanged(&self) {
        self.keyboard_event_manager_.CapsLockStateMayHaveChanged();
    }

    // Source: event_handler.cc:2541-2563.
    pub fn PassMousePressEventToScrollbar(&self, event: &MouseEventWithHitTestResults<R>) -> bool {
        PassMousePressToScrollbar(
            &self.cursor_state_,
            event.result.GetScrollbar(),
            &event.event,
            |capture| self.CaptureMouseEventsToWidget(capture),
        )
    }

    // Source: event_handler.cc:2565-2578.
    pub fn UpdateLastScrollbarUnderMouse(
        &self,
        scrollbar: Option<Rc<dyn EventHandlerScrollbar>>,
        set_last: bool,
    ) {
        self.cursor_state_
            .UpdateLastScrollbarUnderMouse(scrollbar, set_last);
    }

    // Source: event_handler.cc:2580-2589.
    pub fn PassMousePressEventToSubframe(
        &self,
        event: &MouseEventWithHitTestResults<R>,
        subframe: &Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
    ) -> WebInputEventResult {
        self.selection_controller_
            .PassMousePressEventToSubframe(event);
        let result = subframe.Handler().HandleMousePressEvent(&event.event);
        if result != WebInputEventResult::kNotHandled {
            result
        } else {
            WebInputEventResult::kHandledSystem
        }
    }

    // Source: event_handler.cc:2591-2607.
    pub fn PassMouseMoveEventToSubframe(
        &self,
        event: &MouseEventWithHitTestResults<R>,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
        subframe: &Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
        hovered: Option<&mut (HitTestLocation, R)>,
    ) -> WebInputEventResult {
        if self.mouse_event_manager_.MouseDownMayStartDrag() {
            return WebInputEventResult::kNotHandled;
        }
        let result = subframe.Handler().HandleMouseMoveOrLeaveEvent(
            &event.event,
            coalesced,
            predicted,
            hovered,
        );
        if result != WebInputEventResult::kNotHandled {
            result
        } else {
            WebInputEventResult::kHandledSystem
        }
    }

    // Source: event_handler.cc:2609-2613.
    pub fn PassMouseReleaseEventToSubframe(
        &self,
        event: &MouseEventWithHitTestResults<R>,
        subframe: &Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
    ) -> WebInputEventResult {
        subframe.Handler().HandleMouseReleaseEvent(&event.event)
    }

    // Source: event_handler.cc:2615-2631.
    pub fn CaptureMouseEventsToWidget(&self, capture: bool) {
        if !self.frame_.IsLocalRoot() {
            self.frame_
                .LocalFrameRoot()
                .Handler()
                .CaptureMouseEventsToWidget(capture);
            return;
        }
        if !self.frame_.HasClient() {
            return;
        }
        self.cursor_state_
            .CaptureMouseEventsToWidget(capture, |capture| self.frame_.SetMouseCapture(capture));
    }

    // Source: event_handler.cc:2633-2674.
    pub fn GetMouseEventTarget(
        &self,
        hit_type: u32,
        event: &WebMouseEvent,
    ) -> MouseEventWithHitTestResults<R> {
        let hit = self.frame_.HitTesting();
        let location = hit.ContentPointFromRootFrame(event);
        if event.GetType() != Type::kMouseDown {
            let mut result = hit.NewResult(hit_type, &location);
            let capturing = self.capturing_subframe_element_.borrow().clone();
            let capture_target = if self
                .frame_
                .SubframeForTargetNode(capturing.as_ref())
                .is_some()
            {
                result.SetIsOverEmbeddedContentView(true);
                capturing
            } else {
                self.pointer_event_manager_.GetMouseCaptureTarget()
            };
            if let Some(target) = capture_target {
                let local_point = target.AbsoluteToLocalPoint(&location);
                result.SetNodeAndPosition(Rc::clone(&target), &local_point);
                result.SetScrollbar(
                    self.cursor_state_
                        .last_scrollbar_under_mouse_
                        .borrow()
                        .clone(),
                );
                result.SetURLElement(target.EnclosingLinkEventParentOrSelf());
                if hit_type & RequestType::kReadOnly as u32 == 0 {
                    hit.UpdateHoverActiveState(
                        hit_type & RequestType::kActive as u32 != 0,
                        hit_type & RequestType::kMove as u32 == 0,
                        result.InnerPossiblyPseudoElement(),
                    );
                }
                return MouseEventWithHitTestResults {
                    event: event.clone(),
                    location,
                    result,
                };
            }
        }
        let result = hit.PerformMouseEventHitTest(hit_type, &location, event);
        MouseEventWithHitTestResults {
            event: event.clone(),
            location,
            result,
        }
    }

    // Source: event_handler.cc:2676-2682.
    pub fn ReleaseMouseCaptureFromLocalRoot(&self) {
        self.CaptureMouseEventsToWidget(false);
        self.frame_
            .LocalFrameRoot()
            .Handler()
            .ReleaseMouseCaptureFromCurrentFrame();
    }

    // Source: event_handler.cc:2684-2690.
    pub fn ReleaseMouseCaptureFromCurrentFrame(&self) {
        let capturing = self.capturing_subframe_element_.borrow().clone();
        if let Some(subframe) = self.frame_.SubframeForTargetNode(capturing.as_ref()) {
            subframe.Handler().ReleaseMouseCaptureFromCurrentFrame();
        }
        self.pointer_event_manager_.ReleaseMousePointerCapture();
        self.capturing_subframe_element_.replace(None);
    }

    // Source: event_handler.cc:2692-2698.
    pub fn ResetLastMousePositionForWebTest(&self) {
        self.mouse_event_manager_.SetLastMousePositionAsUnknown();
    }
}

fn same_frame<'scope, R: EventHandlerHitResult>(
    a: Option<&Rc<dyn EventHandlerFrame<'scope, R> + 'scope>>,
    b: Option<&Rc<dyn EventHandlerFrame<'scope, R> + 'scope>>,
) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(&a.Identity(), &b.Identity()),
        (None, None) => true,
        _ => false,
    }
}

impl<'scope, R: EventHandlerHitResult> LocatedContextMenuSource for EventHandler<'scope, R> {
    type Target = NodeRef;
    type Hit = R;
    fn HasView(&self) -> bool {
        self.frame_.HitTesting().HasView()
    }
    fn ReleaseMousePress(&self) {
        self.mouse_event_manager_.ReleaseMousePress();
    }
    fn LastScrollbarMouseUp(&self, event: &WebMouseEvent) {
        if let Some(scrollbar) = self.cursor_state_.LastScrollbarUnderMouse() {
            scrollbar.MouseUp(event);
        }
    }
    fn HitTestContextMenu(&self, event: &WebMouseEvent) -> R {
        let location = self.frame_.HitTesting().DocumentPointForMousePress(event);
        self.frame_.HitTesting().PerformMouseEventHitTest(
            RequestType::kActive as u32,
            &location,
            event,
        )
    }
    fn UpdateStyleAndLayoutForContextMenu(&self) {
        self.frame_.UpdateStyleAndLayoutForContextMenu();
    }
    fn InnerPossiblyPseudoElement(&self, hit: &R) -> Option<NodeRef> {
        hit.InnerPossiblyPseudoElement()
    }
    fn DispatchContextMenuEvent(
        &self,
        target: Option<NodeRef>,
        event: &WebMouseEvent,
    ) -> WebInputEventResult {
        self.mouse_event_manager_
            .DispatchContextMenuEvent(target, event)
    }
}

#[path = "event_handler_gesture.rs"]
pub mod gesture;

#[path = "event_handler_nonlocated_context_menu.rs"]
pub mod nonlocated_context_menu;
