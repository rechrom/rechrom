// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/gesture_manager.{h,cc}
// Source owners remain shared with EventHandler. Ports are synchronous DOM,
// frame, selection, and ChromeClient leaves; no port handles a whole gesture.
use super::{
    event_handler::{EventHandlerGesture, EventHandlerHitResult},
    mouse_event_manager::DragHandlingResult,
    pointer_event_manager::PointerEventManager,
};
use interaction_dispatch::core::input::event_handling_util::{
    MergeEventResult, ToWebInputEventResult, WebInputEventResult,
};
use interaction_dispatch::core::{
    dom::events::{
        event::{EventRef, EventTargetRef},
        event_dispatch_result::DispatchEventResult,
    },
    events::{
        gesture_event::GestureEvent,
        pointer_event_factory::{PointerTarget, PointerTargetRef},
        ui_event::AbstractViewRef,
    },
};
use interaction_hit_test::core::layout::{
    hit_test_location::HitTestLocation, hit_test_request::RequestType,
};
use interaction_input::public::common::input::{
    web_gesture_event::WebGestureEvent,
    web_input_event::{Type, WebInputEvent},
    web_mouse_event::WebMouseEvent,
    web_pointer_properties::{Button, PointF},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub trait GestureHitResult: Clone {
    fn InnerNode(&self) -> Option<EventTargetRef>;
    fn InnerElement(&self) -> Option<EventTargetRef>;
    fn InnerPossiblyPseudoElement(&self) -> Option<EventTargetRef>;
    fn InnerPossiblyPseudoNode(&self) -> Option<EventTargetRef>;
    fn HasScrollbar(&self) -> bool;
    fn HandleScrollbarGestureTapOrPress(&self, event: &WebGestureEvent) -> bool;
    fn SetToShadowHostIfInUAShadowRoot(&mut self);
    fn HasURLElement(&self) -> bool;
    fn HasImageURL(&self) -> bool;
    fn HasMediaURL(&self) -> bool;
    fn HasInnerNodeFrame(&self) -> bool;
}
#[derive(Clone)]
pub struct GestureEventWithHitTestResults<R> {
    pub event: WebGestureEvent,
    pub location: HitTestLocation,
    pub result: R,
}
pub trait GesturePointer {
    fn PrimaryPointerdownCanceled(&self, id: u32) -> bool;
    fn GetPointerIdForTouchGesture(&self, id: u32) -> i32;
    fn GetPointerDownTarget(&self, id: i32) -> Option<PointerTargetRef>;
    fn GetPointerUpTarget(&self, id: i32) -> Option<PointerTargetRef>;
    fn RemovePointerTargets(&self, id: i32);
}
impl GesturePointer for PointerEventManager<'_> {
    fn PrimaryPointerdownCanceled(&self, id: u32) -> bool {
        self.PrimaryPointerdownCanceled(id)
    }
    fn GetPointerIdForTouchGesture(&self, id: u32) -> i32 {
        self.GetPointerIdForTouchGesture(id)
    }
    fn GetPointerDownTarget(&self, id: i32) -> Option<PointerTargetRef> {
        self.GetPointerDownTarget(id)
    }
    fn GetPointerUpTarget(&self, id: i32) -> Option<PointerTargetRef> {
        self.GetPointerUpTarget(id)
    }
    fn RemovePointerTargets(&self, id: i32) {
        self.RemovePointerTargets(id)
    }
}
pub trait GestureSelection<R> {
    fn InitializeSelectionState(&self);
    fn HandleMousePressEvent(&self, event: &WebMouseEvent, location: &HitTestLocation, result: &R);
    fn HandleGestureLongPress(&self, result: &R) -> bool;
    fn HandleGestureTwoFingerTap(&self, event: &GestureEventWithHitTestResults<R>);
}
pub trait GestureMouse<R> {
    fn SetElementUnderMouseAndDispatchMouseEvent(
        &self,
        node: Option<EventTargetRef>,
        kind: &str,
        event: &WebMouseEvent,
        down: Option<PointerTargetRef>,
        up: Option<PointerTargetRef>,
    ) -> WebInputEventResult;
    fn SetMouseDownElement(&self, node: Option<EventTargetRef>);
    fn SetClickCount(&self, count: i32);
    fn HandleMouseFocusWithTouchCapabilities(&self, result: &R) -> WebInputEventResult;
    fn HandleMousePressEvent(
        &self,
        event: &WebMouseEvent,
        location: &HitTestLocation,
        result: &R,
    ) -> WebInputEventResult;
    fn HandleMouseReleaseEvent(
        &self,
        event: &WebMouseEvent,
        location: &HitTestLocation,
        result: &R,
    ) -> WebInputEventResult;
    fn ClearDragHeuristicState(&self);
    fn FocusDocumentView(&self);
    fn HandleDragDropIfPossible(
        &self,
        event: &GestureEventWithHitTestResults<R>,
        pointer_id: i32,
    ) -> DragHandlingResult;
}
// Values reflect original Settings and RuntimeEnabledFeatures, not a second
// policy owner. The embedding supplies current values on each source query.
#[derive(Clone, Copy, Default)]
pub struct GestureSettings {
    pub touch_drag_drop_enabled: bool,
    pub touch_drag_end_context_menu: bool,
    pub modal_context_menu: bool,
    pub show_context_menu_on_mouse_up: bool,
}
#[derive(Clone, Copy, Default)]
pub struct GestureFeatures {
    pub touch_text_editing_redesign: bool,
    pub touch_drag_on_short_press: bool,
    pub touch_drag_and_context_menu: bool,
    pub text_fragment_tap_opens_context_menu: bool,
    pub enable_unhandled_tap: bool,
}
pub trait GestureScrollableArea {
    fn ScrollsOverflow(&self) -> bool;
    fn CancelScrollAnimation(&self);
}
pub trait GestureFrame<R> {
    fn ActiveElementExists(&self) -> bool;
    fn Settings(&self) -> Option<GestureSettings>;
    fn Features(&self) -> GestureFeatures;
    fn HasView(&self) -> bool;
    fn HasPage(&self) -> bool;
    fn IsLocalRoot(&self) -> bool;
    fn LocalRootGestureManager(&self) -> Rc<dyn GesturePointerIdLookup>;
    fn ConvertFromRootFrame(&self, point: PointF) -> HitTestLocation;
    fn HitTestResultInFrame(&self, location: &HitTestLocation, request: u32) -> R;
    fn HitTestResultAtLocation(&self, location: &HitTestLocation) -> R;
    fn UpdateRootLifecycleExceptPaint(&self) -> bool;
    fn RootHasView(&self) -> bool;
    fn UpdateInnerNodeFrameLifecycleExceptPaint(&self, result: &R);
    fn IsOverSharedHighlight(&self, result: &R) -> bool;
    fn DomTreeVersion(&self) -> u64;
    fn StyleVersion(&self) -> u64;
    fn ViewForNode(&self, target: &EventTargetRef) -> Option<AbstractViewRef>;
    fn DispatchEvent(&self, target: &EventTargetRef, event: &EventRef) -> DispatchEventResult;
    fn NotifyUserActivation(&self, node: Option<&EventTargetRef>);
    fn HasLayoutObject(&self, node: &EventTargetRef) -> bool;
    fn DidDispatchMouseDown(&self, node: &EventTargetRef);
    fn CommonAncestorForClick(
        &self,
        node: &EventTargetRef,
        other: &EventTargetRef,
    ) -> Option<EventTargetRef>;
    fn AsElement(&self, node: Option<EventTargetRef>) -> Option<EventTargetRef>;
    fn ClientCoordinates(&self, event: &WebMouseEvent) -> (f64, f64);
    fn RegisterTapEvent(&self, node: &EventTargetRef);
    fn SendContextMenuEvent(&self, event: &WebMouseEvent) -> WebInputEventResult;
    fn SendContextMenuEventInAllowedScope(&self, event: &WebMouseEvent);
    fn ScrollableAreas(&self) -> Vec<Rc<dyn GestureScrollableArea>>;
    fn IsTextNode(&self, node: &EventTargetRef) -> bool;
    fn IsContentEditable(&self, node: &EventTargetRef) -> bool;
    fn IsInsideFocusableElementOrARIAWidget(&self, node: &EventTargetRef) -> bool;
    fn RootFrameToViewport(&self, point: PointF) -> Option<PointF>;
    fn ShowUnhandledTapUI(&self, position: PointF);
}
pub trait GesturePointerIdLookup {
    fn GetPointerIdFromWebGestureEvent(&self, event: &WebGestureEvent) -> i32;
}
// S is the original ScrollManager owner supplied during EventHandler assembly.
// Chromium retains it even though current gesture_manager.cc invokes no methods.
pub struct GestureManager<'a, R, S: ?Sized> {
    frame_: Rc<dyn GestureFrame<R> + 'a>,
    _scroll_manager_: Rc<S>,
    mouse_event_manager_: Rc<dyn GestureMouse<R> + 'a>,
    pointer_event_manager_: Rc<dyn GesturePointer + 'a>,
    selection_controller_: Rc<dyn GestureSelection<R> + 'a>,
    state: Rc<GestureManagerState>,
}
/// The original per-frame GestureManager members survive synchronous binding
/// scopes; DOM and geometry ports are borrowed only for the current dispatch.
pub struct GestureManagerState {
    suppress_mouse_events_from_gestures_: Cell<bool>,
    suppress_selection_on_repeated_tap_down_: Cell<bool>,
    gesture_context_menu_deferred_: Cell<bool>,
    long_press_position_in_root_frame_: Cell<PointF>,
    drag_in_progress_: Cell<bool>,
    lost_focus_during_drag_: Cell<bool>,
}
impl Default for GestureManagerState {
    fn default() -> Self {
        Self {
            suppress_mouse_events_from_gestures_: Cell::new(false),
            suppress_selection_on_repeated_tap_down_: Cell::new(false),
            gesture_context_menu_deferred_: Cell::new(false),
            long_press_position_in_root_frame_: Cell::new(PointF::default()),
            drag_in_progress_: Cell::new(false),
            lost_focus_during_drag_: Cell::new(false),
        }
    }
}
impl GestureManagerState {
    pub fn Clear(&self) {
        self.suppress_mouse_events_from_gestures_.set(false);
        self.suppress_selection_on_repeated_tap_down_.set(false);
        self.lost_focus_during_drag_.set(false);
        self.gesture_context_menu_deferred_.set(false);
        self.long_press_position_in_root_frame_
            .set(PointF::default());
        self.drag_in_progress_.set(false);
    }
}
impl<R, S: ?Sized> std::ops::Deref for GestureManager<'_, R, S> {
    type Target = GestureManagerState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl<R: GestureHitResult + EventHandlerHitResult + 'static, S: ?Sized + 'static>
    EventHandlerGesture<R> for GestureManager<'_, R, S>
{
    fn HandleTouchDragEnd(&self, event: &WebMouseEvent, operation_is_none: bool) {
        self.HandleTouchDragEnd(event, operation_is_none);
    }
    fn GetPointerIdFromWebGestureEvent(&self, event: &WebGestureEvent) -> i32 {
        self.GetPointerIdFromWebGestureEvent(event)
    }
    fn Clear(&self) {
        self.Clear();
    }
    fn GetHitTypeForGestureType(&self, t: Type) -> u32 {
        self.GetHitTypeForGestureType(t)
    }
    fn HandleGestureEventInFrame(
        &self,
        e: &GestureEventWithHitTestResults<R>,
    ) -> WebInputEventResult {
        self.HandleGestureEventInFrame(e)
    }
    fn GestureContextMenuDeferred(&self) -> bool {
        self.GestureContextMenuDeferred()
    }
    fn ResetLongTapContextMenuStates(&self) {
        self.ResetLongTapContextMenuStates();
    }
}
impl<R: GestureHitResult + 'static, S: ?Sized + 'static> GesturePointerIdLookup
    for GestureManager<'_, R, S>
{
    fn GetPointerIdFromWebGestureEvent(&self, e: &WebGestureEvent) -> i32 {
        self.GetPointerIdFromWebGestureEvent(e)
    }
}
impl<'a, R: GestureHitResult + 'static, S: ?Sized + 'static> GestureManager<'a, R, S> {
    pub fn new(
        frame: Rc<dyn GestureFrame<R> + 'a>,
        scroll: Rc<S>,
        mouse: Rc<dyn GestureMouse<R> + 'a>,
        pointer: Rc<dyn GesturePointer + 'a>,
        selection: Rc<dyn GestureSelection<R> + 'a>,
    ) -> Self {
        let manager = Self {
            frame_: frame,
            _scroll_manager_: scroll,
            mouse_event_manager_: mouse,
            pointer_event_manager_: pointer,
            selection_controller_: selection,
            state: Rc::new(GestureManagerState::default()),
        };
        manager.Clear();
        manager
    }
    pub fn WithState(
        state: Rc<GestureManagerState>,
        frame: Rc<dyn GestureFrame<R> + 'a>,
        scroll: Rc<S>,
        mouse: Rc<dyn GestureMouse<R> + 'a>,
        pointer: Rc<dyn GesturePointer + 'a>,
        selection: Rc<dyn GestureSelection<R> + 'a>,
    ) -> Self {
        Self {
            frame_: frame,
            _scroll_manager_: scroll,
            mouse_event_manager_: mouse,
            pointer_event_manager_: pointer,
            selection_controller_: selection,
            state,
        }
    }
    pub fn Clear(&self) {
        self.state.Clear();
    }
    pub fn ResetLongTapContextMenuStates(&self) {
        self.gesture_context_menu_deferred_.set(false);
        self.long_press_position_in_root_frame_
            .set(PointF::default());
        self.drag_in_progress_.set(false);
    }
    pub fn FocusedFrameChanged(&self) {
        self.lost_focus_during_drag_
            .set(self.drag_in_progress_.get());
    }
    pub fn GetHitTypeForGestureType(&self, event_type: Type) -> u32 {
        GetHitTypeForGestureType(event_type, self.frame_.ActiveElementExists())
    }
    pub fn GestureContextMenuDeferred(&self) -> bool {
        self.gesture_context_menu_deferred_.get()
    }
    pub fn HandleGestureEventInFrame(
        &self,
        e: &GestureEventWithHitTestResults<R>,
    ) -> WebInputEventResult {
        debug_assert!(!e.event.IsScrollEvent());
        if e.result.HasScrollbar() && e.result.HandleScrollbarGestureTapOrPress(&e.event) {
            return WebInputEventResult::kHandledSuppressed;
        }
        if let Some(target) = e.result.InnerNode() {
            if let Some(event) = GestureEvent::Create(self.frame_.ViewForNode(&target), &e.event) {
                let result = self.frame_.DispatchEvent(&target, &EventRef::new(event));
                if result != DispatchEventResult::kNotCanceled {
                    debug_assert!(result != DispatchEventResult::kCanceledByEventHandler);
                    return ToWebInputEventResult(result);
                }
            }
        }
        if e.event.GetType() != Type::kGestureLongPress {
            self.drag_in_progress_.set(false);
        }
        match e.event.GetType() {
            Type::kGestureTapDown => self.HandleGestureTapDown(e),
            Type::kGestureTap => self.HandleGestureTap(e),
            Type::kGestureShowPress => self.HandleGestureShowPress(),
            Type::kGestureShortPress => self.HandleGestureShortPress(e),
            Type::kGestureLongPress => self.HandleGestureLongPress(e),
            Type::kGestureLongTap => self.HandleGestureLongTap(e),
            Type::kGestureTwoFingerTap => self.HandleGestureTwoFingerTap(e),
            Type::kGestureTapCancel | Type::kGestureTapUnconfirmed => {
                WebInputEventResult::kNotHandled
            }
            _ => panic!("invalid frame gesture type"),
        }
    }
}
// Source GestureManager::GetHitTypeForGestureType; source frame activity is a
// scalar input, allowing target admission before call-scoped manager binding.
pub fn GetHitTypeForGestureType(event_type: Type, active_element_exists: bool) -> u32 {
    let mut hit = RequestType::kTouchEvent as u32;
    match event_type {
        Type::kGestureShowPress | Type::kGestureTapUnconfirmed => hit | RequestType::kActive as u32,
        Type::kGestureTapCancel => {
            if !active_element_exists {
                hit |= RequestType::kReadOnly as u32;
            }
            hit | RequestType::kRelease as u32
        }
        Type::kGestureTap => hit | RequestType::kRelease as u32,
        Type::kGestureTapDown
        | Type::kGestureShortPress
        | Type::kGestureLongPress
        | Type::kGestureLongTap
        | Type::kGestureTwoFingerTap => {
            hit | RequestType::kActive as u32 | RequestType::kReadOnly as u32
        }
        _ => panic!("invalid gesture hit-test type"),
    }
}
impl<'a, R: GestureHitResult + 'static, S: ?Sized + 'static> GestureManager<'a, R, S> {
    fn MouseFromGesture(
        &self,
        e: &WebGestureEvent,
        t: Type,
        button: Button,
        count: i32,
        modifiers: i32,
    ) -> WebMouseEvent {
        WebMouseEvent::FromGesture(
            t,
            e,
            button,
            count,
            modifiers,
            e.TimeStamp(),
            WebMouseEvent::kMousePointerId,
        )
    }
    fn FloorPoint(point: PointF) -> PointF {
        PointF::new(
            (point.x().floor() as i32) as f32,
            (point.y().floor() as i32) as f32,
        )
    }
    fn DispatchMouse(
        &self,
        node: Option<EventTargetRef>,
        kind: &str,
        e: &WebMouseEvent,
    ) -> WebInputEventResult {
        self.mouse_event_manager_
            .SetElementUnderMouseAndDispatchMouseEvent(node, kind, e, None, None)
    }
    fn HandleGestureTapDown(&self, e: &GestureEventWithHitTestResults<R>) -> WebInputEventResult {
        self.suppress_mouse_events_from_gestures_.set(
            self.pointer_event_manager_
                .PrimaryPointerdownCanceled(e.event.unique_touch_event_id),
        );
        self.lost_focus_during_drag_.set(false);
        if !self.frame_.Features().touch_text_editing_redesign
            || self.suppress_mouse_events_from_gestures_.get()
            || self.suppress_selection_on_repeated_tap_down_.get()
            || e.event.TapDownCount() <= 1
        {
            return WebInputEventResult::kNotHandled;
        }
        let mouse = self.MouseFromGesture(
            &e.event,
            Type::kMouseDown,
            Button::kLeft,
            e.event.TapDownCount(),
            e.event.GetModifiers()
                | WebInputEvent::kLeftButtonDown
                | WebInputEvent::kIsCompatibilityEventForTouch,
        );
        self.selection_controller_
            .HandleMousePressEvent(&mouse, &e.location, &e.result);
        WebInputEventResult::kNotHandled
    }
    fn HandleGestureTap(&self, e: &GestureEventWithHitTestResults<R>) -> WebInputEventResult {
        let hit_type = self.GetHitTypeForGestureType(e.event.GetType());
        let dom_version = self.frame_.DomTreeVersion();
        let style_version = self.frame_.StyleVersion();
        let mut hit = e.result.clone();
        let tapped_position = Self::FloorPoint(e.event.PositionInRootFrame());
        let mut adjusted = self.frame_.ConvertFromRootFrame(tapped_position);
        let modifiers = e.event.GetModifiers();
        if !self.suppress_mouse_events_from_gestures_.get() {
            let mouse = self.MouseFromGesture(
                &e.event,
                Type::kMouseMove,
                Button::kNoButton,
                0,
                modifiers | WebInputEvent::kIsCompatibilityEventForTouch,
            );
            self.DispatchMouse(hit.InnerPossiblyPseudoElement(), "mousemove", &mouse);
        }
        if hit.InnerNode().is_some() {
            if !self.frame_.RootHasView() || !self.frame_.UpdateRootLifecycleExceptPaint() {
                return WebInputEventResult::kNotHandled;
            }
            adjusted = self.frame_.ConvertFromRootFrame(tapped_position);
            hit = self.frame_.HitTestResultInFrame(&adjusted, hit_type);
        }
        let tapped_node = hit.InnerNode();
        let tapped_element = hit.InnerPossiblyPseudoElement();
        self.frame_.NotifyUserActivation(tapped_node.as_ref());
        self.mouse_event_manager_
            .SetMouseDownElement(tapped_element.clone());
        let down = self.MouseFromGesture(
            &e.event,
            Type::kMouseDown,
            Button::kLeft,
            e.event.TapCount(),
            modifiers
                | WebInputEvent::kLeftButtonDown
                | WebInputEvent::kIsCompatibilityEventForTouch,
        );
        let mut down_result = WebInputEventResult::kHandledSuppressed;
        self.suppress_selection_on_repeated_tap_down_.set(true);
        self.mouse_event_manager_.SetClickCount(e.event.TapCount());
        if !self.suppress_mouse_events_from_gestures_.get() {
            down_result = self.DispatchMouse(hit.InnerPossiblyPseudoElement(), "mousedown", &down);
            self.selection_controller_.InitializeSelectionState();
            if down_result == WebInputEventResult::kNotHandled {
                down_result = self
                    .mouse_event_manager_
                    .HandleMouseFocusWithTouchCapabilities(&hit);
            }
            if down_result == WebInputEventResult::kNotHandled {
                self.suppress_selection_on_repeated_tap_down_.set(false);
                down_result =
                    self.mouse_event_manager_
                        .HandleMousePressEvent(&down, &e.location, &hit);
            }
        }
        if hit.InnerNode().is_some() {
            let mut result = hit.clone();
            result.SetToShadowHostIfInUAShadowRoot();
            if let Some(node) = result.InnerNode() {
                self.frame_.DidDispatchMouseDown(&node);
            }
        }
        if hit.InnerNode().is_some() {
            if self.frame_.RootHasView() {
                self.frame_.UpdateRootLifecycleExceptPaint();
            }
            adjusted = self.frame_.ConvertFromRootFrame(tapped_position);
            hit = self.frame_.HitTestResultInFrame(&adjusted, hit_type);
        }
        let mut up = self.MouseFromGesture(
            &e.event,
            Type::kMouseUp,
            Button::kLeft,
            e.event.TapCount(),
            modifiers | WebInputEvent::kIsCompatibilityEventForTouch,
        );
        let mut up_result = if self.suppress_mouse_events_from_gestures_.get() {
            WebInputEventResult::kHandledSuppressed
        } else {
            self.DispatchMouse(hit.InnerPossiblyPseudoElement(), "mouseup", &up)
        };
        let mut click_result = WebInputEventResult::kNotHandled;
        if let Some(tapped) = tapped_element {
            if let Some(inner) = hit.InnerNode() {
                let inner_up = hit.InnerPossiblyPseudoNode().unwrap_or(inner);
                let click = self
                    .frame_
                    .AsElement(self.frame_.CommonAncestorForClick(&inner_up, &tapped));
                let id = self.GetPointerIdFromWebGestureEvent(&e.event);
                up.pointer_properties.id = id;
                up.pointer_properties.pointer_type = e.event.primary_pointer_type;
                let mut down_target = self.pointer_event_manager_.GetPointerDownTarget(id);
                let mut up_target = self.pointer_event_manager_.GetPointerUpTarget(id);
                if down_target.is_none() {
                    assert!(up_target.is_none());
                    let (x, y) = self.frame_.ClientCoordinates(&up.FlattenTransform());
                    down_target = Some(Rc::new(RefCell::new(PointerTarget::new(
                        click.as_ref(),
                        x,
                        y,
                    ))));
                    up_target = Some(Rc::new(RefCell::new(PointerTarget::new(
                        click.as_ref(),
                        x,
                        y,
                    ))));
                } else {
                    assert!(up_target.is_some());
                }
                click_result = self
                    .mouse_event_manager_
                    .SetElementUnderMouseAndDispatchMouseEvent(
                        click,
                        "click",
                        &up,
                        down_target,
                        up_target,
                    );
                self.pointer_event_manager_.RemovePointerTargets(id);
                if self.frame_.HasView() {
                    self.frame_.RegisterTapEvent(&tapped);
                }
            }
            self.mouse_event_manager_.SetMouseDownElement(None);
        }
        if up_result == WebInputEventResult::kNotHandled {
            up_result = self
                .mouse_event_manager_
                .HandleMouseReleaseEvent(&up, &e.location, &hit);
        }
        self.mouse_event_manager_.ClearDragHeuristicState();
        let result = MergeEventResult(MergeEventResult(down_result, up_result), click_result);
        if self.frame_.Features().text_fragment_tap_opens_context_menu && hit.HasInnerNodeFrame() {
            self.frame_.UpdateInnerNodeFrameLifecycleExceptPaint(&hit);
            hit = self.frame_.HitTestResultInFrame(&adjusted, hit_type);
            if self.frame_.IsOverSharedHighlight(&hit) && result == WebInputEventResult::kNotHandled
            {
                return self.SendContextMenuEventForGesture(e);
            }
        }
        if result == WebInputEventResult::kNotHandled && self.frame_.HasPage() {
            if let Some(node) = tapped_node {
                if let Some(position) = self.frame_.RootFrameToViewport(tapped_position) {
                    self.ShowUnhandledTapUIIfNeeded(
                        dom_version != self.frame_.DomTreeVersion(),
                        style_version != self.frame_.StyleVersion(),
                        &node,
                        position,
                    );
                }
            }
        }
        result
    }
    fn HandleGestureShortPress(
        &self,
        e: &GestureEventWithHitTestResults<R>,
    ) -> WebInputEventResult {
        if self
            .frame_
            .Settings()
            .is_some_and(|s| s.touch_drag_drop_enabled)
            && self.frame_.Features().touch_drag_on_short_press
            && self.HandleDragDropIfPossible(e) != DragHandlingResult::kNotHandled
        {
            WebInputEventResult::kHandledSystem
        } else {
            WebInputEventResult::kNotHandled
        }
    }
    fn TouchDragAndContextMenuEnabled(&self) -> bool {
        self.frame_.Features().touch_drag_and_context_menu
            && self
                .frame_
                .Settings()
                .is_some_and(|s| !s.modal_context_menu)
    }
    fn DragEndOpensContextMenu(&self) -> bool {
        self.frame_
            .Settings()
            .is_some_and(|s| s.touch_drag_end_context_menu)
    }
    fn HandleDragDropIfPossible(
        &self,
        e: &GestureEventWithHitTestResults<R>,
    ) -> DragHandlingResult {
        let result = self
            .mouse_event_manager_
            .HandleDragDropIfPossible(e, self.GetPointerIdFromWebGestureEvent(&e.event));
        self.drag_in_progress_
            .set(result == DragHandlingResult::kHandledDragStarted);
        result
    }
    fn HandleGestureLongPress(&self, e: &GestureEventWithHitTestResults<R>) -> WebInputEventResult {
        self.long_press_position_in_root_frame_
            .set(e.event.PositionInRootFrame());
        let location = self.frame_.ConvertFromRootFrame(Self::FloorPoint(
            self.long_press_position_in_root_frame_.get(),
        ));
        let hit = self.frame_.HitTestResultAtLocation(&location);
        self.gesture_context_menu_deferred_.set(false);
        if self.frame_.Features().touch_drag_on_short_press && self.drag_in_progress_.get() {
            if self.DragEndOpensContextMenu() {
                self.gesture_context_menu_deferred_.set(true);
                return WebInputEventResult::kNotHandled;
            }
        } else if self.TouchDragAndContextMenuEnabled() {
            self.HandleDragDropIfPossible(e);
        } else if self
            .frame_
            .Settings()
            .is_some_and(|s| s.touch_drag_drop_enabled)
            && self.frame_.HasView()
        {
            let open_now = !self.DragEndOpensContextMenu()
                && (hit.HasURLElement() || hit.HasImageURL() || hit.HasMediaURL());
            if !open_now && self.HandleDragDropIfPossible(e) != DragHandlingResult::kNotHandled {
                self.gesture_context_menu_deferred_.set(true);
                return WebInputEventResult::kHandledSystem;
            }
        }
        let node = hit.InnerNode();
        if !(self.drag_in_progress_.get() && self.TouchDragAndContextMenuEnabled())
            && node
                .as_ref()
                .is_some_and(|n| self.frame_.HasLayoutObject(n))
            && self.selection_controller_.HandleGestureLongPress(&hit)
        {
            self.mouse_event_manager_.FocusDocumentView();
        }
        if self
            .frame_
            .Settings()
            .is_some_and(|s| s.show_context_menu_on_mouse_up)
        {
            self.gesture_context_menu_deferred_.set(true);
            return WebInputEventResult::kNotHandled;
        }
        self.frame_.NotifyUserActivation(node.as_ref());
        self.SendContextMenuEventForGesture(e)
    }
    fn HandleGestureLongTap(&self, e: &GestureEventWithHitTestResults<R>) -> WebInputEventResult {
        if self.gesture_context_menu_deferred_.get() {
            self.gesture_context_menu_deferred_.set(false);
            self.SendContextMenuEventForGesture(e)
        } else {
            WebInputEventResult::kNotHandled
        }
    }
    fn HandleGestureTwoFingerTap(
        &self,
        e: &GestureEventWithHitTestResults<R>,
    ) -> WebInputEventResult {
        if e.result
            .InnerNode()
            .as_ref()
            .is_some_and(|n| self.frame_.HasLayoutObject(n))
        {
            self.selection_controller_.HandleGestureTwoFingerTap(e);
        }
        self.SendContextMenuEventForGesture(e)
    }
    pub fn HandleTouchDragEnd(&self, e: &WebMouseEvent, operation_is_none: bool) {
        if !self.drag_in_progress_.get() {
            return;
        }
        self.drag_in_progress_.set(false);
        if self.DragEndOpensContextMenu() {
            self.SendContextMenuEventTouchDragEnd(e, operation_is_none);
        }
    }
    pub fn SendContextMenuEventTouchDragEnd(&self, e: &WebMouseEvent, operation_is_none: bool) {
        if !self.gesture_context_menu_deferred_.get()
            || self.suppress_mouse_events_from_gestures_.get()
        {
            return;
        }
        let p = e.pointer_properties.PositionInWidget();
        let start = self.long_press_position_in_root_frame_.get();
        let distance = ((p.x() - start.x()).powi(2) + (p.y() - start.y()).powi(2)).sqrt();
        if !(distance <= 8. && !self.lost_focus_during_drag_.get() && operation_is_none) {
            self.ResetLongTapContextMenuStates();
            return;
        }
        self.frame_.SendContextMenuEventInAllowedScope(e);
        self.ResetLongTapContextMenuStates();
    }
    fn SendContextMenuEventForGesture(
        &self,
        e: &GestureEventWithHitTestResults<R>,
    ) -> WebInputEventResult {
        let modifiers = e.event.GetModifiers() | WebInputEvent::kIsCompatibilityEventForTouch;
        if !self.suppress_mouse_events_from_gestures_.get() {
            let mouse =
                self.MouseFromGesture(&e.event, Type::kMouseMove, Button::kNoButton, 0, modifiers);
            self.DispatchMouse(e.result.InnerElement(), "mousemove", &mouse);
        }
        let kind = if self
            .frame_
            .Settings()
            .is_some_and(|s| s.show_context_menu_on_mouse_up)
        {
            Type::kMouseUp
        } else {
            Type::kMouseDown
        };
        let mut mouse = self.MouseFromGesture(&e.event, kind, Button::kNoButton, 0, modifiers);
        if !self.suppress_mouse_events_from_gestures_.get() && self.frame_.HasView() {
            let location = self
                .frame_
                .ConvertFromRootFrame(Self::FloorPoint(e.event.PositionInRootFrame()));
            let hit = self
                .frame_
                .HitTestResultInFrame(&location, RequestType::kActive as u32);
            self.mouse_event_manager_
                .HandleMouseFocusWithTouchCapabilities(&hit);
        }
        mouse.pointer_properties.id = self.GetPointerIdFromWebGestureEvent(&e.event);
        mouse.pointer_properties.pointer_type = e.event.primary_pointer_type;
        self.frame_.SendContextMenuEvent(&mouse)
    }
    fn HandleGestureShowPress(&self) -> WebInputEventResult {
        if self.frame_.HasView() {
            for area in self.frame_.ScrollableAreas() {
                if area.ScrollsOverflow() {
                    area.CancelScrollAnimation();
                }
            }
        }
        WebInputEventResult::kNotHandled
    }
    fn ShowUnhandledTapUIIfNeeded(
        &self,
        dom_changed: bool,
        style_changed: bool,
        node: &EventTargetRef,
        position: PointF,
    ) {
        if self.frame_.Features().enable_unhandled_tap
            && !dom_changed
            && !style_changed
            && self.frame_.IsTextNode(node)
            && !self.frame_.IsContentEditable(node)
            && !self.frame_.IsInsideFocusableElementOrARIAWidget(node)
        {
            self.frame_.ShowUnhandledTapUI(position);
        }
    }
    pub fn GetPointerIdFromWebGestureEvent(&self, e: &WebGestureEvent) -> i32 {
        if !self.frame_.IsLocalRoot() {
            return self
                .frame_
                .LocalRootGestureManager()
                .GetPointerIdFromWebGestureEvent(e);
        }
        if e.primary_unique_touch_event_id == 0 {
            return 0;
        }
        self.pointer_event_manager_
            .GetPointerIdForTouchGesture(e.primary_unique_touch_event_id)
    }
}
