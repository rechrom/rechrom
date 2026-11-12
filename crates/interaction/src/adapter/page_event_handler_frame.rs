//! Call-scoped EventHandler frame projection over live LocalFrame owners.
//! Source: core/input/event_handler.h:77-101,282-288; event_handler.cc:
//! 899-930,960-967,1002-1068,1102-1133,1208-1245,1278-1286,1319-1339,
//! 1563-1640,2184-2223,2615-2631,2680-2690.
//!
//! Browser supplies only typed frame/client leaves. Handler, hit testing, DOM
//! identity, capabilities and retained state are assembled here. In particular
//! no callback has broad access to Page or a mutable DOM/Layout engine.
use super::{
    page_event_handler_hit_test::{PageEventHandlerHitResult, PageEventHandlerHitTest},
    page_event_handler_node::{PageEventHandlerNodes, RequiredEventHandlerPort},
};
use crate::{
    core::input::event_handler::{
        EventHandler, EventHandlerFactory, EventHandlerFrame, EventHandlerHitResult,
        EventHandlerHitTest, EventHandlerState, NodeRef,
    },
    ownership::InteractionDocument,
};
use interaction_dispatch::core::{
    dom::events::event::EventTargetRef, events::ui_event::InputDeviceCapabilitiesRef,
};
use interaction_hit_test::core::layout::hit_test_location::HitTestLocation;
use interaction_input::public::common::input::{
    web_gesture_event::WebGestureEvent, web_mouse_event::WebMouseEvent,
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

/// A child-frame query preserves the distinction between no embedded view and
/// a remote view. A remote frame is not treated as a missing local frame.
pub enum PageEventHandlerSubframe {
    None,
    Local(Rc<()>),
    Remote,
}
pub trait PageEventHandlerFrameDomain {
    fn Identity(&self) -> Rc<()>;
    fn LocalFrameRootIdentity(&self) -> Rc<()>;
    fn FocusedCoreFrameIdentity(&self) -> Option<Rc<()>>;
    fn FocusedLocalFrameInWidgetIdentity(&self) -> Option<Rc<()>>;
    fn ParentLocalFrameIdentity(&self) -> Option<Rc<()>>;
    fn OwnerElement(&self) -> Option<EventTargetRef>;
    fn NodeFrameIdentity(&self, target: &EventTargetRef) -> Option<Rc<()>>;
    fn HitNodeFrameIdentity(
        &self,
        node: &interaction_hit_test::core::layout::hit_test_result::NodeRef,
    ) -> Option<Rc<()>>;
    fn AdjustmentScales(&self) -> (f32, f32, f32);
    fn HasPage(&self) -> bool;
    fn IsAttached(&self) -> bool;
    fn IsOutermostMainFrame(&self) -> bool;
    fn IsDescendantOf(&self, identity: &Rc<()>) -> bool;
    fn SubframeForTarget(&self, target: &EventTargetRef) -> PageEventHandlerSubframe;
}
/// The interaction binder owns the call-scoped handler cache. Browser's frame
/// domain carries only stable frame facts, never these manager bindings.
pub trait PageEventHandlerFrameRegistry<'scope> {
    fn FrameForIdentity(
        &self,
        identity: &Rc<()>,
    ) -> Option<Rc<dyn EventHandlerFrame<'scope, PageEventHandlerHitResult> + 'scope>>;
    fn NodeForTarget(&self, target: &EventTargetRef) -> Option<NodeRef>;
    fn EventTarget(&self, node: &NodeRef) -> Option<EventTargetRef>;
    fn NodeForHit(
        &self,
        node: &interaction_hit_test::core::layout::hit_test_result::NodeRef,
    ) -> Option<NodeRef>;
    fn NodesForHit(
        &self,
        node: &interaction_hit_test::core::layout::hit_test_result::NodeRef,
    ) -> Option<Rc<PageEventHandlerNodes>>;
    fn OwnerElementForFrame(&self, identity: &Rc<()>) -> Option<EventTargetRef>;
}
/// Calls the source client, activation, AutoscrollController and FrameSelection
/// owners. All leaves are required; a missing owner is an assembly error.
pub trait PageEventHandlerFrameInputClient {
    fn HasClient(&self) -> bool;
    fn SetMouseCapture(&self, capture: bool);
    fn ShouldDiscardEventTargetingFrame(&self, event: &WebMouseEvent) -> bool;
    fn ShouldDiscardGestureEventTargetingFrame(&self, event: &WebGestureEvent) -> bool;
    fn NotifyUserActivation(&self);
    fn SetCaretBlinkingSuspended(&self, suspended: bool);
}
pub trait PageEventHandlerFrameInput: PageEventHandlerFrameInputClient {
    fn HandleMouseMoveForMiddleClickAutoscroll(&self, event: &WebMouseEvent);
    fn HandleMouseReleaseForMiddleClickAutoscroll(&self, middle_button: bool);
}
/// Source ChromeClient, AnchorElementInteractionTracker and MenuSafeTriangle
/// calls remain synchronous, including calls made after a listener reenters.
pub trait PageEventHandlerFrameChrome {
    fn WillDispatchPointerDown(&self, target: &EventTargetRef);
    fn DidDispatchMouseDown(&self, target: &EventTargetRef);
    fn MouseDidMoveOverElement(
        &self,
        location: &HitTestLocation,
        result: &PageEventHandlerHitResult,
    );
    fn ClearToolTip(&self);
    fn AnchorTrackerOnMouseMove(&self, event: &WebMouseEvent);
    fn RecheckMenuSafeTriangle(&self);
    fn SelectAndSetCursor(&self, location: &HitTestLocation, result: &PageEventHandlerHitResult);
    fn CursorUpdateTimerFired(&self);
}
pub struct PageEventHandlerFrameServices<'scope> {
    pub domain: Option<Rc<dyn PageEventHandlerFrameDomain + 'scope>>,
    pub registry: Option<Rc<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>>,
    pub input: Option<Rc<dyn PageEventHandlerFrameInput + 'scope>>,
    pub chrome: Option<Rc<dyn PageEventHandlerFrameChrome + 'scope>>,
}
pub struct PageEventHandlerFrame<'scope> {
    domain: Rc<dyn PageEventHandlerFrameDomain + 'scope>,
    registry: Weak<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>,
    input: Rc<dyn PageEventHandlerFrameInput + 'scope>,
    chrome: Rc<dyn PageEventHandlerFrameChrome + 'scope>,
    hit: PageEventHandlerHitTest<'scope>,
    nodes: Rc<PageEventHandlerNodes>,
    document: InteractionDocument,
    handler: RefCell<Weak<EventHandler<'scope, PageEventHandlerHitResult>>>,
    self_: Weak<Self>,
}
impl<'scope> PageEventHandlerFrame<'scope> {
    /// Returns only fully bound source owners: Handler() cannot observe an
    /// uninitialized projection, and the handler/frame link is non-owning in
    /// the reverse direction. The caller retains the handler for this input.
    pub fn Assemble(
        services: PageEventHandlerFrameServices<'scope>,
        hit: PageEventHandlerHitTest<'scope>,
        nodes: Rc<PageEventHandlerNodes>,
        document: InteractionDocument,
        factory: &dyn EventHandlerFactory<'scope, PageEventHandlerHitResult>,
        state: Rc<EventHandlerState>,
    ) -> Result<
        (
            Rc<Self>,
            Rc<EventHandler<'scope, PageEventHandlerHitResult>>,
        ),
        RequiredEventHandlerPort,
    > {
        let domain = services
            .domain
            .ok_or(RequiredEventHandlerPort::FrameDomain)?;
        let registry = services
            .registry
            .ok_or(RequiredEventHandlerPort::FrameRegistry)?;
        let input = services.input.ok_or(RequiredEventHandlerPort::FrameInput)?;
        let chrome = services
            .chrome
            .ok_or(RequiredEventHandlerPort::FrameChrome)?;
        assert!(Rc::ptr_eq(&document, &nodes.Targets().DocumentOwner()));
        let frame = Rc::new_cyclic(|self_| Self {
            domain,
            registry: Rc::downgrade(&registry),
            input,
            chrome,
            hit,
            nodes,
            document,
            handler: RefCell::new(Weak::new()),
            self_: self_.clone(),
        });
        let handler = Rc::new(EventHandler::WithState(frame.clone(), factory, state));
        *frame.handler.borrow_mut() = Rc::downgrade(&handler);
        Ok((frame, handler))
    }
    fn Frame(
        &self,
        identity: &Rc<()>,
    ) -> Option<Rc<dyn EventHandlerFrame<'scope, PageEventHandlerHitResult> + 'scope>> {
        if Rc::ptr_eq(identity, &self.domain.Identity()) {
            Some(self.self_.upgrade().expect("live frame binding"))
        } else {
            self.registry
                .upgrade()
                .expect("interaction binder retains the frame registry")
                .FrameForIdentity(identity)
        }
    }
    fn Subframe(&self, node: Option<&NodeRef>) -> PageEventHandlerSubframe {
        let Some(target) = node.and_then(|node| {
            self.registry
                .upgrade()
                .expect("interaction binder retains the frame registry")
                .EventTarget(node)
        }) else {
            return PageEventHandlerSubframe::None;
        };
        self.domain.SubframeForTarget(&target)
    }
}
impl<'scope> EventHandlerFrame<'scope, PageEventHandlerHitResult>
    for PageEventHandlerFrame<'scope>
{
    fn Identity(&self) -> Rc<()> {
        self.domain.Identity()
    }
    fn FrameForIdentity(
        &self,
        identity: &Rc<()>,
    ) -> Option<Rc<dyn EventHandlerFrame<'scope, PageEventHandlerHitResult> + 'scope>> {
        self.Frame(identity)
    }
    fn IsLocalRoot(&self) -> bool {
        Rc::ptr_eq(
            &self.domain.Identity(),
            &self.domain.LocalFrameRootIdentity(),
        )
    }
    fn LocalFrameRoot(
        &self,
    ) -> Rc<dyn EventHandlerFrame<'scope, PageEventHandlerHitResult> + 'scope> {
        self.Frame(&self.domain.LocalFrameRootIdentity())
            .expect("assembly registers the source LocalFrameRoot binding")
    }
    fn Handler(&self) -> Rc<EventHandler<'scope, PageEventHandlerHitResult>> {
        self.handler
            .borrow()
            .upgrade()
            .expect("assembly retains its bound EventHandler")
    }
    fn HitTesting(&self) -> &dyn EventHandlerHitTest<PageEventHandlerHitResult> {
        &self.hit
    }
    fn HasPage(&self) -> bool {
        self.domain.HasPage()
    }
    fn IsAttached(&self) -> bool {
        self.domain.IsAttached()
    }
    fn IsOutermostMainFrame(&self) -> bool {
        self.domain.IsOutermostMainFrame()
    }
    fn IsDescendantOf(
        &self,
        frame: &Rc<dyn EventHandlerFrame<'scope, PageEventHandlerHitResult> + 'scope>,
    ) -> bool {
        self.domain.IsDescendantOf(&frame.Identity())
    }
    fn HasClient(&self) -> bool {
        self.input.HasClient()
    }
    fn SetMouseCapture(&self, capture: bool) {
        self.input.SetMouseCapture(capture);
    }
    fn SetSequentialFocusNavigationStartingPoint(&self, node: NodeRef) {
        let target = self
            .nodes
            .EventTarget(&node)
            .expect("focus starting point belongs to the bound DOM owner");
        let id = target.NodeId().expect("focus starting point is a Node");
        let mut owner = self.document.borrow_mut();
        let document = owner.GetDocumentMut();
        let node = document.FindNodeById(id);
        document.SetSequentialFocusNavigationStartingPoint(node);
    }
    fn ShouldDiscardEventTargetingFrame(&self, event: &WebMouseEvent) -> bool {
        self.input.ShouldDiscardEventTargetingFrame(event)
    }
    fn ShouldDiscardGestureEventTargetingFrame(&self, event: &WebGestureEvent) -> bool {
        self.input.ShouldDiscardGestureEventTargetingFrame(event)
    }
    fn NotifyUserActivation(&self) {
        self.input.NotifyUserActivation();
    }
    fn MiddleClickAutoscrollEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::MiddleClickAutoscrollEnabled()
    }
    fn HandleMouseMoveForMiddleClickAutoscroll(&self, event: &WebMouseEvent) {
        self.input.HandleMouseMoveForMiddleClickAutoscroll(event);
    }
    fn HandleMouseReleaseForMiddleClickAutoscroll(&self, middle_button: bool) {
        self.input
            .HandleMouseReleaseForMiddleClickAutoscroll(middle_button);
    }
    fn SetCaretBlinkingSuspended(&self, suspended: bool) {
        self.input.SetCaretBlinkingSuspended(suspended);
    }
    fn FiresTouchEvents(&self, from_touch: bool) -> InputDeviceCapabilitiesRef {
        self.nodes.Targets().InputDeviceCapabilities(from_touch)
    }
    fn WillDispatchPointerDown(&self, node: NodeRef) {
        self.chrome.WillDispatchPointerDown(
            &self
                .nodes
                .EventTarget(&node)
                .expect("pointer-down node belongs to the bound DOM owner"),
        );
    }
    fn DidDispatchMouseDown(&self, node: NodeRef) {
        self.chrome.DidDispatchMouseDown(
            &self
                .nodes
                .EventTarget(&node)
                .expect("mouse-down node belongs to the bound DOM owner"),
        );
    }
    fn TargetSubframe(
        &self,
        result: &PageEventHandlerHitResult,
    ) -> (
        Option<Rc<dyn EventHandlerFrame<'scope, PageEventHandlerHitResult> + 'scope>>,
        bool,
    ) {
        match self.Subframe(result.InnerNode().as_ref()) {
            PageEventHandlerSubframe::None => (None, false),
            PageEventHandlerSubframe::Remote => (None, true),
            PageEventHandlerSubframe::Local(identity) => (
                Some(
                    self.Frame(&identity)
                        .expect("assembly binds the source local child frame"),
                ),
                false,
            ),
        }
    }
    fn SubframeForTargetNode(
        &self,
        node: Option<&NodeRef>,
    ) -> Option<Rc<dyn EventHandlerFrame<'scope, PageEventHandlerHitResult> + 'scope>> {
        match self.Subframe(node) {
            PageEventHandlerSubframe::Local(identity) => Some(
                self.Frame(&identity)
                    .expect("assembly binds the source local child frame"),
            ),
            PageEventHandlerSubframe::None | PageEventHandlerSubframe::Remote => None,
        }
    }
    fn MouseDidMoveOverElement(
        &self,
        location: &HitTestLocation,
        result: &PageEventHandlerHitResult,
    ) {
        self.chrome.MouseDidMoveOverElement(location, result);
    }
    fn ClearToolTip(&self) {
        self.chrome.ClearToolTip();
    }
    fn AnchorTrackerOnMouseMove(&self, event: &WebMouseEvent) {
        self.chrome.AnchorTrackerOnMouseMove(event);
    }
    fn RecheckMenuSafeTriangle(&self) {
        self.chrome.RecheckMenuSafeTriangle();
    }
    fn SelectAndSetCursor(&self, location: &HitTestLocation, result: &PageEventHandlerHitResult) {
        self.chrome.SelectAndSetCursor(location, result);
    }
    fn CursorUpdateTimerFired(&self) {
        self.chrome.CursorUpdateTimerFired();
    }
    fn UpdateStyleAndLayoutForContextMenu(&self) {
        self.hit.PrepareLifecycleExceptPaint();
    }
}
