//! EventHandler's PointerEventManager port over the original Page input owners.
//! Source: core/input/event_handler.cc:269-302,327-330,932-952,1008-1017,
//! 1208-1233,1291-1296,1563-1652,2629-2631; pointer_event_manager.cc:95-114,
//! 238-263,307-349,693-874,992-1068,1103-1325,1431-1509,1545-1557.
//!
//! This binding lives for one synchronous EventHandler transaction. Each source
//! call lends fresh DOM/Layout ports through WithPointerManager while keeping
//! ResidentCursorOwners' pointer, touch, mouse and frame identities unchanged.
//! No source manager algorithm or call-scoped geometry is retained in state.
use super::{DispatchEventResult, EventDispatcher, PointerInputError};
use crate::{
    adapter::{
        page_event_handler_node::PageEventHandlerNodes, page_pointer_hit_test::PagePointerGeometry,
        page_pointer_scroll_controls::PagePointerScrollControlOwners,
        source_input_managers::ResidentCursorOwners,
    },
    core::input::{
        event_handler::{EventHandlerFrame, EventHandlerHitResult, EventHandlerPointer, NodeRef},
        pointer_event_manager::PointerEventManager,
    },
};
use interaction_dispatch::{
    core::dom::events::event::EventTargetRef,
    public::platform::web_input_event_result::WebInputEventResult,
};
use interaction_input::public::common::input::{
    pointer_id::PointerId, web_input_event::Type, web_mouse_event::WebMouseEvent,
    web_pointer_event::WebPointerEvent,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

/// A failed transaction remains failed even after its error is drained. The
/// native facade must call Finish at the end of every transaction and propagate
/// that error as SourceInput; a new input creates a new binding.
pub struct PageEventHandlerPointer<'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
    owners: ResidentCursorOwners,
    nodes: Rc<PageEventHandlerNodes>,
    geometry: Rc<dyn PagePointerGeometry>,
    controls: Rc<PagePointerScrollControlOwners>,
    failed: Cell<bool>,
    error: RefCell<Option<PointerInputError>>,
    dispatched: Cell<DispatchEventResult>,
}
impl<'listener, 'frame> PageEventHandlerPointer<'listener, 'frame> {
    pub fn new(
        dispatcher: &EventDispatcher<'listener, 'frame>,
        owners: &ResidentCursorOwners,
        nodes: Rc<PageEventHandlerNodes>,
        geometry: Option<Rc<dyn PagePointerGeometry>>,
        controls: Option<Rc<PagePointerScrollControlOwners>>,
    ) -> Result<Rc<Self>, PointerInputError> {
        let geometry = geometry.ok_or(PointerInputError::MissingGeometry)?;
        let controls = controls.ok_or(PointerInputError::MissingScrollControls)?;
        if dispatcher.mouse_frame_geometry.borrow().is_none() {
            return Err(PointerInputError::MissingGeometry);
        }
        assert!(Rc::ptr_eq(nodes.Targets(), &dispatcher.targets));
        Ok(Rc::new(Self {
            dispatcher: dispatcher.clone(),
            owners: owners.clone(),
            nodes,
            geometry,
            controls,
            failed: Cell::new(false),
            error: RefCell::new(None),
            dispatched: Cell::new(DispatchEventResult::default()),
        }))
    }
    pub fn TakeError(&self) -> Option<PointerInputError> {
        self.error.borrow_mut().take()
    }
    pub fn Finish(&self) -> Result<(), PointerInputError> {
        match self.TakeError() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
    pub fn TakeDispatchResult(&self) -> DispatchEventResult {
        self.dispatched.take()
    }
    fn Fault(&self, error: PointerInputError) {
        if !self.failed.replace(true) {
            *self.error.borrow_mut() = Some(error);
        }
    }
    fn Target(&self, node: &NodeRef) -> Option<EventTargetRef> {
        let target = self.nodes.EventTarget(node);
        if target.is_none() {
            self.Fault(PointerInputError::DetachedTarget);
        }
        target
    }
    fn Node(&self, target: Option<EventTargetRef>) -> Option<NodeRef> {
        let target = target?;
        let node = self.nodes.NodeForTarget(&target);
        if node.is_none() {
            self.Fault(PointerInputError::DetachedTarget);
        }
        node
    }
    /// Rejected values are only trait return placeholders after a recorded
    /// failure. Finish carries the failure out of the synchronous transaction.
    fn Run<T>(&self, rejected: T, run: impl FnOnce(&PointerEventManager<'_>) -> T) -> T {
        if self.failed.get() {
            return rejected;
        }
        match self.dispatcher.WithPointerManager(
            &self.owners,
            self.geometry.clone(),
            self.controls.clone(),
            |manager, _| Ok(run(manager)),
        ) {
            Ok((value, result)) => {
                let previous = self.dispatched.get();
                self.dispatched.set(DispatchEventResult {
                    target_node_id: result.target_node_id.or(previous.target_node_id),
                    default_prevented: previous.default_prevented || result.default_prevented,
                    propagation_stopped: previous.propagation_stopped || result.propagation_stopped,
                    default_handled: previous.default_handled || result.default_handled,
                });
                value
            }
            Err(error) => {
                self.Fault(error);
                rejected
            }
        }
    }
}
impl<'scope, R: EventHandlerHitResult> EventHandlerPointer<'scope, R>
    for PageEventHandlerPointer<'_, '_>
{
    fn Clear(&self) {
        self.Run((), |manager| manager.Clear());
        if !self.failed.get() {
            self.controls.Clear();
        }
    }
    fn NodeChildrenWillBeRemoved(&self, container: &NodeRef) {
        if let Some(target) = self.Target(container) {
            self.Run((), |manager| manager.NodeChildrenWillBeRemoved(&target));
        }
    }
    fn NodeWillBeRemoved(&self, node: &NodeRef) {
        if let Some(target) = self.Target(node) {
            self.Run((), |manager| manager.NodeWillBeRemoved(&target));
        }
    }
    fn HandlePseudoElementRemoval(&self, pseudo: &NodeRef) {
        if let Some(target) = self.Target(pseudo) {
            self.Run((), |manager| manager.HandlePseudoElementRemoval(&target));
        }
    }
    fn CurrentTouchDownElement(&self) -> Option<NodeRef> {
        self.Node(self.Run(None, |manager| manager.CurrentTouchDownElement()))
    }
    fn FlushEvents(&self) -> WebInputEventResult {
        self.Run(WebInputEventResult::kNotHandled, |manager| {
            manager.FlushEvents()
        })
    }
    fn HandlePointerEvent(
        &self,
        event: &WebPointerEvent,
        coalesced: &[WebPointerEvent],
        predicted: &[WebPointerEvent],
    ) -> WebInputEventResult {
        self.Run(WebInputEventResult::kNotHandled, |manager| {
            manager.HandlePointerEvent(event, coalesced, predicted)
        })
    }
    fn GetMouseCaptureTarget(&self) -> Option<NodeRef> {
        self.Node(self.Run(None, |manager| manager.GetMouseCaptureTarget()))
    }
    fn IsAnyTouchActive(&self) -> bool {
        self.Run(false, |manager| manager.IsAnyTouchActive())
    }
    fn SendMouseAndPointerBoundaryEvents(&self, target: Option<NodeRef>, event: &WebMouseEvent) {
        let target = target.as_ref().and_then(|node| self.Target(node));
        self.Run((), |manager| {
            manager.SendMouseAndPointerBoundaryEvents(target, event)
        });
    }
    fn SendEffectivePanActionAtPointer(&self, event: &WebMouseEvent, node: Option<NodeRef>) {
        let target = node.as_ref().and_then(|node| self.Target(node));
        // EventHandler.cc:1230 flattens the mouse event before constructing the
        // pointer move; preserve its coordinates and original sample time.
        let event = WebPointerEvent::FromMouse(Type::kPointerMove, &event.FlattenTransform());
        self.Run((), |manager| {
            manager.SendEffectivePanActionAtPointer(&event, target)
        });
    }
    fn RemoveLastMousePosition(&self) {
        self.Run((), |manager| manager.RemoveLastMousePosition());
    }
    fn IsPointerIdActiveOnFrame(
        &self,
        id: PointerId,
        frame: &Rc<dyn EventHandlerFrame<'scope, R> + 'scope>,
    ) -> bool {
        self.Run(false, |manager| {
            manager.IsPointerIdActiveOnFrame(id, &frame.Identity())
        })
    }
    fn AppendTouchIdForCanceledPointerDown(&self, id: u32) {
        self.Run((), |manager| {
            manager.AppendTouchIdForCanceledPointerDown(id)
        });
    }
    fn IsActive(&self, id: PointerId) -> bool {
        self.Run(false, |manager| manager.IsActive(id))
    }
    fn SetPointerCapture(&self, id: PointerId, target: &NodeRef) -> bool {
        let Some(target) = self.Target(target) else {
            return false;
        };
        self.Run(false, |manager| manager.SetPointerCapture(id, target))
    }
    fn ReleasePointerCapture(&self, id: PointerId, target: &NodeRef) -> bool {
        let Some(target) = self.Target(target) else {
            return false;
        };
        self.Run(false, |manager| manager.ReleasePointerCapture(id, &target))
    }
    fn HasPointerCapture(&self, id: PointerId, target: &NodeRef) -> bool {
        let Some(target) = self.Target(target) else {
            return false;
        };
        self.Run(false, |manager| manager.HasPointerCapture(id, &target))
    }
    fn ElementRemoved(&self, target: &NodeRef) {
        if let Some(target) = self.Target(target) {
            self.Run((), |manager| manager.ElementRemoved(&target));
        }
    }
    fn SendMousePointerEvent(
        &self,
        target: Option<NodeRef>,
        event_type: Type,
        event: &WebMouseEvent,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
        skip_click: bool,
    ) -> WebInputEventResult {
        let target = target.as_ref().and_then(|node| self.Target(node));
        self.Run(WebInputEventResult::kNotHandled, |manager| {
            manager
                .SendMousePointerEvent(target, event_type, event, coalesced, predicted, skip_click)
        })
    }
    fn DirectDispatchMousePointerEvent(
        &self,
        target: NodeRef,
        event: &WebMouseEvent,
        event_type: &str,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
    ) -> WebInputEventResult {
        let Some(target) = self.Target(&target) else {
            return WebInputEventResult::kNotHandled;
        };
        self.Run(WebInputEventResult::kNotHandled, |manager| {
            manager.DirectDispatchMousePointerEvent(target, event, event_type, coalesced, predicted)
        })
    }
    fn ReleaseMousePointerCapture(&self) {
        self.Run((), |manager| manager.ReleaseMousePointerCapture());
    }
}
