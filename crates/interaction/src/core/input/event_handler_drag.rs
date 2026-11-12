//! EventHandler's target drag transaction, event_handler.cc:1376-1520.
//! Call-scoped ports dispatch against canonical DOM identities; no borrows of
//! retained state survive author callbacks.
use interaction_dispatch::core::{
    clipboard::data_transfer::DataTransfer, dom::events::event::EventTargetRef,
    input::event_handling_util::WebInputEventResult,
};
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Default)]
pub struct EventHandlerDragState {
    target: RefCell<Option<EventTargetRef>>,
    only_drag_over: Cell<bool>,
}
pub trait DragTargetPort {
    fn HasView(&self) -> bool;
    fn TargetAt(&self, event: &WebMouseEvent) -> Option<EventTargetRef>;
    fn UpdateAutoscroll(&self, target: Option<&EventTargetRef>, event: &WebMouseEvent);
    fn DispatchSourceDrag(&self, event: &WebMouseEvent);
    fn DispatchDrag(
        &self,
        kind: &str,
        target: &EventTargetRef,
        related: Option<&EventTargetRef>,
        event: &WebMouseEvent,
        data: &Rc<DataTransfer>,
    ) -> WebInputEventResult;
    fn ClearDragState(&self);
}
impl EventHandlerDragState {
    pub fn Clear(&self) {
        self.target.borrow_mut().take();
        self.only_drag_over.set(false);
    }
    pub fn UpdateDragAndDrop(
        &self,
        port: &dyn DragTargetPort,
        event: &WebMouseEvent,
        data: &Rc<DataTransfer>,
    ) -> WebInputEventResult {
        use WebInputEventResult::kNotHandled;
        if !port.HasView() {
            return kNotHandled;
        }
        let target = port.TargetAt(event);
        port.UpdateAutoscroll(target.as_ref(), event);
        let old = self.target.borrow().clone();
        let same = match (&old, &target) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        let mut result = kNotHandled;
        if !same {
            if let Some(target) = &target {
                port.DispatchSourceDrag(event);
                result = port.DispatchDrag("dragenter", target, old.as_ref(), event, data);
            }
            if let Some(old) = &old {
                port.DispatchDrag("dragleave", old, target.as_ref(), event, data);
            }
            if target.is_some() {
                self.only_drag_over.set(true);
            }
        } else if let Some(target) = &target {
            if !self.only_drag_over.get() {
                port.DispatchSourceDrag(event);
            }
            result = port.DispatchDrag("dragover", target, None, event, data);
            self.only_drag_over.set(false);
        }
        self.target.replace(target);
        result
    }
    pub fn CancelDragAndDrop(
        &self,
        port: &dyn DragTargetPort,
        event: &WebMouseEvent,
        data: &Rc<DataTransfer>,
    ) {
        let target = self.target.borrow().clone();
        if let Some(target) = target {
            port.DispatchSourceDrag(event);
            port.DispatchDrag("dragleave", &target, None, event, data);
        }
        port.ClearDragState();
        self.Clear();
    }
    pub fn PerformDragAndDrop(
        &self,
        port: &dyn DragTargetPort,
        event: &WebMouseEvent,
        data: &Rc<DataTransfer>,
    ) -> WebInputEventResult {
        let target = self.target.borrow().clone();
        let result = target.map_or(WebInputEventResult::kNotHandled, |target| {
            port.DispatchDrag("drop", &target, None, event, data)
        });
        port.ClearDragState();
        self.Clear();
        result
    }
}
