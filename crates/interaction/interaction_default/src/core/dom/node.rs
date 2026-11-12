// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/node.cc:3407-3469
// Source: third_party/blink/renderer/core/dom/node.h:1031-1037,1088
// The retained Node and Frame owners supply effects; this method retains the
// source event and owns only the source's default-handler ordering.

use interaction_dispatch::core::dom::events::{
    event::{EventRef, EventTargetRef},
    event_dispatch_result::DispatchEventResult,
};
use std::rc::Rc;

/// A borrowed-layout capability prepared by UpdateStyleAndLayout. It refers to
/// the resident LayoutObject, including the document's local owner boundary.
pub trait AutoscrollLayoutObject {
    fn IsBox(&self) -> bool;
    fn IsUserScrollable(&self) -> bool;
    fn IsDocumentNode(&self) -> bool;
    fn LocalOwnerLayoutObject(&self) -> Option<Rc<dyn AutoscrollLayoutObject>>;
    fn Parent(&self) -> Option<Rc<dyn AutoscrollLayoutObject>>;
    /// Another view of this same resident LayoutObject, for the source frame
    /// EventHandler's handoff to its one AutoscrollController.
    fn AutoscrollLayout(
        &self,
    ) -> Rc<dyn crate::core::page::autoscroll_controller::AutoscrollLayout>;
}

pub trait NodeDefaultEnvironment {
    fn Target(&self) -> EventTargetRef;
    fn HasFrame(&self) -> bool;
    /// Invoke the existing frame EventHandler on the same concrete event.
    fn DefaultKeyboardEventHandler(&self, event: &EventRef);
    fn DefaultTextInputEventHandler(&self, event: &EventRef);
    /// DynamicTo<UIEvent>; None is the source failed cast, not a replacement.
    fn UIEventDetail(&self, event: &EventRef) -> Option<i32>;
    fn DispatchDOMActivateEvent(&self, detail: i32, underlying: &EventRef) -> DispatchEventResult;
    fn HasPage(&self) -> bool;
    fn HandleContextMenuEvent(&self, event: &EventRef);
    fn MiddleClickAutoscrollEnabled(&self) -> bool;
    fn MouseButton(&self, event: &EventRef) -> Option<i16>;
    fn HasEnclosingLinkEventParentOrSelf(&self) -> bool;
    fn UpdateStyleAndLayoutForInput(&self);
    fn LayoutObject(&self) -> Option<Rc<dyn AutoscrollLayoutObject>>;
    fn StartMiddleClickAutoscroll(&self, object: Rc<dyn AutoscrollLayoutObject>);
}

pub struct Node {
    environment: Rc<dyn NodeDefaultEnvironment>,
}
impl Node {
    pub fn IsMiddleClickAutoscrollMouseDown(
        enabled: bool,
        kind: &str,
        mouse: bool,
        button: Option<i16>,
    ) -> bool {
        enabled && kind == "mousedown" && mouse && button == Some(1)
    }
    pub fn new(environment: Rc<dyn NodeDefaultEnvironment>) -> Self {
        Self { environment }
    }

    // Source: node.cc:3373-3396. The embedding supplies the source Now clock
    // and dispatches this same UIEvent through the existing scoped queue.
    pub fn CreateDOMActivateEvent(
        underlying: &EventRef,
        view: Option<interaction_dispatch::core::events::ui_event::AbstractViewRef>,
        detail: i32,
        clock: &dyn interaction_dispatch::core::dom::events::event::EventClock,
        is_connected: bool,
        bubbles_inheritance: bool,
    ) -> EventRef {
        use interaction_dispatch::core::{
            dom::events::event::EventInterface, events::ui_event::UIEvent,
        };
        let (bubbles, composed) = {
            let event = underlying.borrow();
            (event.BaseEvent().bubbles(), event.BaseEvent().composed())
        };
        let mut event = UIEvent::Create(clock);
        event.initUIEvent(
            "DOMActivate",
            !bubbles_inheritance || bubbles,
            true,
            view,
            detail,
        );
        event
            .BaseEventMut()
            .SetUnderlyingEvent(Some(underlying.clone()));
        event.BaseEventMut().SetComposed(composed);
        if !is_connected {
            event.BaseEventMut().SetCopyEventPathFromUnderlyingEvent();
        }
        EventRef::new(event)
    }
    // Source: node.cc:3420-3426. A borrowed event adapter supplies the same
    // synchronous DOMActivate dispatch and receives its cancellation result.
    pub fn HandleClickDefaultAction(dispatch: impl FnOnce() -> DispatchEventResult) -> bool {
        dispatch() != DispatchEventResult::kNotCanceled
    }

    pub fn DefaultEventHandler(&self, event: &EventRef) {
        let (target, kind, keyboard, mouse, text) = {
            let event = event.borrow();
            (
                event.BaseEvent().RawTarget(),
                event.BaseEvent().r#type().to_owned(),
                event.IsKeyboardEvent(),
                event.IsMouseEvent(),
                event.HasInterface("TextEvent"),
            )
        };
        if !target.is_some_and(|target| Rc::ptr_eq(&target, &self.environment.Target())) {
            return;
        }
        if matches!(kind.as_str(), "keydown" | "keypress" | "keyup") {
            if keyboard && self.environment.HasFrame() {
                self.environment.DefaultKeyboardEventHandler(event);
            }
        } else if kind == "click" {
            let detail = self.environment.UIEventDetail(event).unwrap_or(0);
            if Self::HandleClickDefaultAction(|| {
                self.environment.DispatchDOMActivateEvent(detail, event)
            }) {
                event.borrow_mut().BaseEventMut().SetDefaultHandled();
            }
        } else if kind == "contextmenu" && mouse {
            if self.environment.HasPage() {
                self.environment.HandleContextMenuEvent(event);
            }
        } else if kind == "textInput" {
            if text && self.environment.HasFrame() {
                self.environment.DefaultTextInputEventHandler(event);
            }
        } else if Self::IsMiddleClickAutoscrollMouseDown(
            self.environment.MiddleClickAutoscrollEnabled(),
            &kind,
            mouse,
            self.environment.MouseButton(event),
        ) {
            if self.environment.HasEnclosingLinkEventParentOrSelf() {
                return;
            }
            self.environment.UpdateStyleAndLayoutForInput();
            let mut object = self.environment.LayoutObject();
            while let Some(current) = &object {
                if current.IsBox() && current.IsUserScrollable() {
                    break;
                }
                object = if current.IsDocumentNode() {
                    current.LocalOwnerLayoutObject()
                } else {
                    current.Parent()
                };
            }
            if let Some(object) = object {
                if self.environment.HasFrame() {
                    self.environment.StartMiddleClickAutoscroll(object);
                }
            }
        }
    }
}

// Source: node.cc:3501-3508; event_util.cc:18-24.
pub trait NodeMouseClickResponseEnvironment {
    fn IsDisabledFormControl(&self) -> bool;
    fn UpdateStyleAndLayoutTree(&self);
    fn IsEditable(&self) -> bool;
    fn HasEventListeners(&self, event_type: &str) -> bool;
}
pub fn WillRespondToMouseClickEvents(owner: &dyn NodeMouseClickResponseEnvironment) -> bool {
    if owner.IsDisabledFormControl() {
        return false;
    }
    owner.UpdateStyleAndLayoutTree();
    owner.IsEditable()
        || ["click", "mousedown", "mouseup", "DOMActivate"]
            .iter()
            .any(|kind| owner.HasEventListeners(kind))
}
// Source: node.cc:3493-3499. Subclass HTMLDiv base calls this method directly.
pub fn WillRespondToMouseMoveEvents(owner: &dyn NodeMouseClickResponseEnvironment) -> bool {
    !owner.IsDisabledFormControl()
        && ["mousemove", "mouseover", "mouseout"]
            .iter()
            .any(|kind| owner.HasEventListeners(kind))
}
