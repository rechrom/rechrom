//! Synchronous document/client leaves for EventHandler's live frame binding.
//! Source: event_handler.cc:931-933,960,1029,1056-1068,1093-1104,
//! 1123-1128,1278-1285,2452-2457; document.cc:5766-5840.
use super::*;
use crate::adapter::{
    page_event_handler_frame::{
        PageEventHandlerFrameChrome, PageEventHandlerFrameDomain, PageEventHandlerFrameInput,
        PageEventHandlerFrameInputClient, PageEventHandlerFrameRegistry,
    },
    page_event_handler_hit_test::{PageEventHandlerHitResult, PageEventHandlerHoverState},
    page_event_handler_node::{PageEventHandlerNodes, RequiredEventHandlerPort},
};
use crate::core::input::event_handler::{EventHandlerState, EventHandlerTimer, NodeRef};
use interaction_default::core::page::autoscroll_controller::{
    AutoscrollController, AutoscrollControllerState, AutoscrollFrame,
};
use interaction_hit_test::core::layout::hit_test_location::HitTestLocation;

pub struct PageEventHandlerDocumentState<'scope, 'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
    nodes: Rc<PageEventHandlerNodes>,
    state: Rc<EventHandlerState>,
    active_interval_timer: Rc<dyn EventHandlerTimer + 'scope>,
    domain: Rc<dyn PageEventHandlerFrameDomain + 'scope>,
    registry: std::rc::Weak<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>,
}
impl<'scope, 'listener, 'frame> PageEventHandlerDocumentState<'scope, 'listener, 'frame> {
    pub fn new(
        dispatcher: EventDispatcher<'listener, 'frame>,
        nodes: Rc<PageEventHandlerNodes>,
        state: Rc<EventHandlerState>,
        active_interval_timer: Option<Rc<dyn EventHandlerTimer + 'scope>>,
        domain: Rc<dyn PageEventHandlerFrameDomain + 'scope>,
        registry: Rc<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>,
    ) -> Result<Self, RequiredEventHandlerPort> {
        assert!(Rc::ptr_eq(nodes.Targets(), &dispatcher.targets));
        Ok(Self {
            dispatcher,
            nodes,
            state,
            active_interval_timer: active_interval_timer
                .ok_or(RequiredEventHandlerPort::ActiveIntervalTimer)?,
            domain,
            registry: Rc::downgrade(&registry),
        })
    }
}
impl PageEventHandlerHoverState for PageEventHandlerDocumentState<'_, '_, '_> {
    fn UpdateHoverActiveState(
        &self,
        active: bool,
        update_active_chain: bool,
        mut node: Option<NodeRef>,
    ) {
        // document.cc:8957-8975: notify this document first, then update each
        // foreign target document before following its canonical LocalOwner.
        if active && self.dispatcher.focus_services.lifecycle.HasFrame() {
            self.state
                .NotifyElementActivated(self.active_interval_timer.as_ref());
        }
        let registry = self
            .registry
            .upgrade()
            .expect("interaction binder retains the scoped frame registry");
        while let Some(target_node) = node.as_ref() {
            let target = registry
                .EventTarget(target_node)
                .expect("hover target retains its canonical document owner");
            let identity = self
                .domain
                .NodeFrameIdentity(&target)
                .expect("hover target belongs to a live local frame");
            if Rc::ptr_eq(&identity, &self.domain.Identity()) {
                break;
            }
            registry
                .FrameForIdentity(&identity)
                .expect("binder projects the target document's LocalFrame")
                .HitTesting()
                .UpdateHoverActiveState(active, update_active_chain, node.clone());
            node = registry
                .OwnerElementForFrame(&identity)
                .and_then(|owner| registry.NodeForTarget(&owner));
        }
        let id = node.as_ref().map(|node| {
            let target = self
                .nodes
                .EventTarget(node)
                .expect("hover target belongs to the bound DOM owner");
            self.dispatcher
                .targets
                .NodeId(&target)
                .expect("hover target retains its canonical node identity")
        });
        let update = self
            .dispatcher
            .document
            .borrow_mut()
            .GetDocumentMut()
            .UpdateHoverActiveState(active, update_active_chain, id);
        let mut state = self.dispatcher.state.borrow_mut();
        state.hovered_node_id = update.hovered_node_id;
        state.pressed_node_id = update.active_node_id;
    }
}

/// Required ChromeClient notifications. The embedding publishes its actual
/// typed client effects; optional document owners are handled below instead.
pub trait PageEventHandlerChromeClient {
    fn WillDispatchPointerDown(&self, target: &EventTargetRef);
    fn DidDispatchMouseDown(&self, target: &EventTargetRef);
    fn MouseDidMoveOverElement(
        &self,
        location: &HitTestLocation,
        result: &PageEventHandlerHitResult,
    );
    fn ClearToolTip(&self);
}
pub trait PageEventHandlerCursorClient {
    fn SelectAndSetCursor(&self, location: &HitTestLocation, result: &PageEventHandlerHitResult);
    fn CursorUpdateTimerFired(&self);
}
pub struct PageEventHandlerChrome<'scope, 'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
    client: Rc<dyn PageEventHandlerChromeClient + 'scope>,
    cursor: Rc<dyn PageEventHandlerCursorClient + 'scope>,
}
impl<'scope, 'listener, 'frame> PageEventHandlerChrome<'scope, 'listener, 'frame> {
    pub fn new(
        dispatcher: EventDispatcher<'listener, 'frame>,
        client: Option<Rc<dyn PageEventHandlerChromeClient + 'scope>>,
        cursor: Option<Rc<dyn PageEventHandlerCursorClient + 'scope>>,
    ) -> Result<Self, RequiredEventHandlerPort> {
        Ok(Self {
            dispatcher,
            client: client.ok_or(RequiredEventHandlerPort::ChromeClient)?,
            cursor: cursor.ok_or(RequiredEventHandlerPort::Cursor)?,
        })
    }
}
impl PageEventHandlerFrameChrome for PageEventHandlerChrome<'_, '_, '_> {
    fn WillDispatchPointerDown(&self, target: &EventTargetRef) {
        self.client.WillDispatchPointerDown(target);
    }
    fn DidDispatchMouseDown(&self, target: &EventTargetRef) {
        self.client.DidDispatchMouseDown(target);
    }
    fn MouseDidMoveOverElement(
        &self,
        location: &HitTestLocation,
        result: &PageEventHandlerHitResult,
    ) {
        self.client.MouseDidMoveOverElement(location, result);
    }
    fn ClearToolTip(&self) {
        self.client.ClearToolTip();
    }
    fn AnchorTrackerOnMouseMove(&self, event: &WebMouseEvent) {
        pointer_dom_effects::AnchorElementInteractionTrackerOnMouseMoveEvent(
            &self.dispatcher,
            event,
        );
    }
    fn RecheckMenuSafeTriangle(&self) {
        if let Err(error) = pointer_dom_effects::RecheckMenuTriangle(&self.dispatcher) {
            std::panic::panic_any(error);
        }
    }
    fn SelectAndSetCursor(&self, location: &HitTestLocation, result: &PageEventHandlerHitResult) {
        self.cursor.SelectAndSetCursor(location, result);
    }
    fn CursorUpdateTimerFired(&self) {
        self.cursor.CursorUpdateTimerFired();
    }
}

pub struct PageEventHandlerInput<'scope> {
    client: Rc<dyn PageEventHandlerFrameInputClient + 'scope>,
    autoscroll: Rc<AutoscrollControllerState>,
    mouse: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
    middle_click_frame: Option<Rc<dyn AutoscrollFrame + 'scope>>,
}
impl<'scope> PageEventHandlerInput<'scope> {
    pub fn new(
        client: Option<Rc<dyn PageEventHandlerFrameInputClient + 'scope>>,
        autoscroll: Rc<AutoscrollControllerState>,
        mouse: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        middle_click_frame: Option<Rc<dyn AutoscrollFrame + 'scope>>,
    ) -> Result<Self, RequiredEventHandlerPort> {
        if foundation::RuntimeEnabledFeatures::MiddleClickAutoscrollEnabled()
            && middle_click_frame.is_none()
        {
            return Err(RequiredEventHandlerPort::MiddleClickAutoscroll);
        }
        Ok(Self {
            client: client.ok_or(RequiredEventHandlerPort::FrameInput)?,
            autoscroll,
            mouse,
            middle_click_frame,
        })
    }
}
impl PageEventHandlerFrameInputClient for PageEventHandlerInput<'_> {
    fn HasClient(&self) -> bool {
        self.client.HasClient()
    }
    fn SetMouseCapture(&self, capture: bool) {
        self.client.SetMouseCapture(capture);
    }
    fn ShouldDiscardEventTargetingFrame(&self, event: &WebMouseEvent) -> bool {
        self.client.ShouldDiscardEventTargetingFrame(event)
    }
    fn ShouldDiscardGestureEventTargetingFrame(
        &self,
        event: &interaction_input::public::common::input::web_gesture_event::WebGestureEvent,
    ) -> bool {
        self.client.ShouldDiscardGestureEventTargetingFrame(event)
    }
    fn NotifyUserActivation(&self) {
        self.client.NotifyUserActivation();
    }
    fn SetCaretBlinkingSuspended(&self, suspended: bool) {
        self.client.SetCaretBlinkingSuspended(suspended);
    }
}
impl PageEventHandlerFrameInput for PageEventHandlerInput<'_> {
    fn HandleMouseMoveForMiddleClickAutoscroll(&self, _event: &WebMouseEvent) {
        let frame = self
            .middle_click_frame
            .as_ref()
            .expect("enabled middle-click autoscroll requires its source frame owner");
        AutoscrollController::WithState(frame.as_ref(), &self.autoscroll)
            .HandleMouseMoveForMiddleClickAutoscroll(self.mouse.LastKnownMouseScreenPosition());
    }
    fn HandleMouseReleaseForMiddleClickAutoscroll(&self, middle_button: bool) {
        let frame = self
            .middle_click_frame
            .as_ref()
            .expect("enabled middle-click autoscroll requires its source frame owner");
        AutoscrollController::WithState(frame.as_ref(), &self.autoscroll)
            .HandleMouseReleaseForMiddleClickAutoscroll(middle_button);
    }
}
