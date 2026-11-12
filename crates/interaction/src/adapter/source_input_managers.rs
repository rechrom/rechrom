//! Retained native input state shared by translated ingress and managers.
//! Frame/style, scrolling and capture remain borrowed owner ports;
//! this module installs no placeholder DOM, frame, timer or drag services.

use crate::core::input::pointer_event_manager::{PointerPanAction, PointerTouchAction};
use crate::core::input::touch_event_manager::TouchEventManagerState;
use interaction_input::public::common::frame::user_activation_state::UserActivationState;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// The frame's source input effects and activation owner. The Page consumes
/// the unbuffered request inside the synchronous input callback; it is never
/// a queue of DOM work or a copy of PointerEventManager state.
pub struct PointerFrameInputState {
    pub anchor_navigation: RefCell<Option<Rc<dyn interaction_default::core::html::html_anchor_element::AnchorNavigationHost>>>,
    pub identity: Rc<()>,
    pub activation: RefCell<UserActivationState>,
    pub history_activation: RefCell<interaction_input::public::common::frame::history_user_activation_state::HistoryUserActivationState>,
    pub unbuffered_requested: Cell<bool>,
    pub touch_action: Cell<PointerTouchAction>,
    pub pan_action: Cell<PointerPanAction>,
    /// WebFrameWidget's handling-input flag, shared by input ingress and IME.
    pub handling_input_event: Cell<bool>,
    pub caret_browsing_enabled: Cell<bool>,
}
impl Default for PointerFrameInputState {
    fn default() -> Self {
        Self {
            anchor_navigation: RefCell::new(None),
            identity: Rc::new(()),
            activation: RefCell::new(Default::default()),
            history_activation: RefCell::new(Default::default()),
            unbuffered_requested: Cell::new(false),
            touch_action: Cell::new(PointerTouchAction::kAuto),
            pan_action: Cell::new(PointerPanAction::kScroll),
            handling_input_event: Cell::new(false),
            caret_browsing_enabled: Cell::new(false),
        }
    }
}

/// WebFrameWidget input scopes nest across synchronous listener reentry.
pub struct HandlingInputEventScope {
    owner: Rc<PointerFrameInputState>,
    previous: bool,
}
impl PointerFrameInputState {
    pub fn EnterHandlingInputEvent(self: &Rc<Self>) -> HandlingInputEventScope {
        HandlingInputEventScope {
            owner: self.clone(),
            previous: self.handling_input_event.replace(true),
        }
    }
}
impl Drop for HandlingInputEventScope {
    fn drop(&mut self) {
        self.owner.handling_input_event.set(self.previous);
    }
}

use interaction_default::core::editing::selection_controller::SelectionControllerState;
use interaction_input::public::common::input::{
    web_input_event::Type, web_mouse_event::WebMouseEvent, web_pointer_properties::PointF,
};

use crate::core::input::{
    event_handler::EventHandlerCursorState,
    event_handler_cursor::{
        CursorEventHandlerState, CursorFrame, CursorMouseEventManager, CursorScrollManager,
        CursorSelectionController, EventHandlerCursor, PointF as CursorPointF,
    },
    keyboard_event_manager::KeyboardEventManagerState,
    mouse_event_manager::MouseEventManagerState,
    pointer_event_manager::MousePointerEventManager,
};

/// These are the original source state owners, not snapshots for cursor reads.
/// Full source managers accept these same Rc values in their WithState
/// constructors when their dispatch/default services are connected.
#[derive(Clone, Default)]
pub struct ResidentCursorOwners {
    /// The complete handler identity is initialized on first source binding and
    /// retained alongside the original cursor-visible owner across events.
    event_handler_state:
        Rc<RefCell<Option<Rc<crate::core::input::event_handler::EventHandlerState>>>>,
    pub drag_target: Rc<crate::core::input::event_handler_drag::EventHandlerDragState>,
    pub drag: Rc<interaction_default::core::page::drag_controller::DragController>,
    pub autoscroll:
        Rc<interaction_default::core::page::autoscroll_controller::AutoscrollControllerState>,
    pub autoscroll_services:
        Rc<RefCell<Option<Rc<super::page_mouse_autoscroll::PageMouseAutoscroll>>>>,
    pub event_handler: Rc<EventHandlerCursorState>,
    pub mouse: Rc<MouseEventManagerState>,
    pub selection: Rc<SelectionControllerState>,
    pub keyboard: Rc<KeyboardEventManagerState>,
    pub pointer: Rc<MousePointerEventManager>,
    pub touch: Rc<TouchEventManagerState>,
    pub pointer_frame: Rc<PointerFrameInputState>,
    pub gesture: Rc<crate::core::input::gesture_manager::GestureManagerState>,
    pub gesture_target: Rc<super::page_event_dispatcher::gesture_input::GestureTargetState>,
}

impl ResidentCursorOwners {
    pub fn EventHandlerState(&self) -> Rc<crate::core::input::event_handler::EventHandlerState> {
        let mut state = self.event_handler_state.borrow_mut();
        state
            .get_or_insert_with(|| {
                crate::core::input::event_handler::EventHandlerState::WithCursorState(
                    self.event_handler.clone(),
                )
            })
            .clone()
    }

    /// Scroll/autoscroll and capture facts must come from live source owners.
    /// They are required ports because SelectCursor reads them unconditionally.
    pub fn CursorHandler<'a>(
        &'a self,
        frame: &'a dyn CursorFrame,
        scroll: &'a dyn CursorScrollManager,
        state: &'a dyn CursorEventHandlerState,
    ) -> EventHandlerCursor<'a> {
        EventHandlerCursor {
            frame,
            mouse_event_manager: self.mouse.as_ref(),
            scroll_manager: scroll,
            selection_controller: self.selection.as_ref(),
            state,
        }
    }

    /// Prepared resident queries use exactly SelectCursor's frame subset;
    /// this borrows the same retained managers as the full EventHandler path.
    pub fn CursorPolicy<'a>(
        &'a self,
        frame: &'a dyn crate::core::input::event_handler_cursor::CursorPolicyFrame,
        scroll: &'a dyn CursorScrollManager,
        state: &'a dyn CursorEventHandlerState,
    ) -> crate::core::input::event_handler_cursor::EventHandlerCursorPolicy<'a> {
        crate::core::input::event_handler_cursor::EventHandlerCursorPolicy {
            frame,
            mouse_event_manager: self.mouse.as_ref(),
            scroll_manager: scroll,
            selection_controller: self.selection.as_ref(),
            state,
        }
    }

    /// The embedding has one local root: these are its already transformed
    /// CSS viewport coordinates. Timestamps remain on the original Web event.
    pub fn HandleMouseEventUpdateStates(&self, event: &WebMouseEvent) {
        match event.GetType() {
            Type::kMouseDown => {
                self.event_handler.ClearMouseCapture();
                let point = event.PositionInRootFrame();
                self.mouse.HandleMousePressEventUpdateStates(
                    event,
                    Some(PointF::new(point.x().floor(), point.y().floor())),
                );
                // EventHandler.cc:950 installs the host-provided native click
                // count on the same retained MouseEventManager before pointer
                // and compatibility mouse dispatch.
                self.mouse.SetClickCount(event.click_count);
                self.selection.SetMouseDownMayStartSelect(false);
            }
            Type::kMouseUp => {
                self.event_handler.ClearMouseCapture();
                self.mouse.SetLastKnownMousePosition(event);
                self.mouse.ReleaseMousePress();
            }
            Type::kMouseMove | Type::kMouseEnter | Type::kMouseLeave => {
                self.mouse.SetLastKnownMousePosition(event);
            }
            _ => (),
        }
    }

    /// Source release cleanup follows pointer/mouse dispatch and selection.
    pub fn HandleMouseReleaseEventUpdateStates(&self) {
        self.mouse.ClearDragHeuristicState();
        self.mouse.InvalidateClick();
        self.selection.SetMouseDownMayStartSelect(false);
    }

    /// Source mouse press heuristic, reached only after uncanceled mousedown.
    /// Selection facts are taken from the existing live selection transaction.
    pub fn HandleMousePressDefault(
        &self,
        event: &WebMouseEvent,
        selection_over_link: bool,
        extending_selection: bool,
    ) {
        self.mouse
            .HandleMousePressEventState(event, selection_over_link, extending_selection);
    }
}

impl CursorMouseEventManager for MouseEventManagerState {
    fn LastKnownMousePositionInViewport(&self) -> CursorPointF {
        let point = MouseEventManagerState::LastKnownMousePositionInViewport(self);
        CursorPointF {
            x: point.x(),
            y: point.y(),
        }
    }
    fn MousePressed(&self) -> bool {
        MouseEventManagerState::MousePressed(self)
    }
    fn MouseDownMayStartDrag(&self) -> bool {
        MouseEventManagerState::MouseDownMayStartDrag(self)
    }
}
impl CursorSelectionController for SelectionControllerState {
    fn MouseDownMayStartSelect(&self) -> bool {
        SelectionControllerState::MouseDownMayStartSelect(self)
    }
}

/// Call-scoped projections of the canonical DOM/style/image owners consumed
/// by SelectionController's source event predicates. No ancestry is retained.
pub(crate) struct MousePressFacts {
    pub is_over_link: bool,
    pub has_image: bool,
    pub is_user_node_draggable: bool,
}
impl MousePressFacts {
    // Source: selection_controller.cc:1495-1524; hit_test_result.cc:437-447.
    pub fn FromDocument(document: &dom::Document, target: u64) -> Option<Self> {
        use dom::persistent_document::DOMNamespace;
        let target = document.FindNodeById(target)?;
        let node = document.Node(target);
        let is_user_node_draggable = document
            .ResolvedStyleFor(target)
            .and_then(|style| style.ComputedStyle())
            .is_some_and(|style| style.UserDrag() == foundation::EUserDrag::kElement);
        let image_content = document
            .ExistingHTMLImageLoader(target)
            .and_then(|loader| loader.GetContent())
            .or_else(|| {
                node.IsElement("image", DOMNamespace::kSVG)
                    .then(|| {
                        node.FindAttribute("href")
                            .or_else(|| node.FindAttribute("xlink:href"))
                            .and_then(|attribute| {
                                document.ImageResourceContentFor(&attribute.value)
                            })
                    })
                    .flatten()
            });
        let has_image =
            image_content.is_some_and(|image| !image.ErrorOccurred() && image.HasImage());
        let mut ancestor = Some(target);
        let mut is_over_link = false;
        while let Some(index) = ancestor {
            let node = document.Node(index);
            if (node.IsHTMLElement("a")
                || node.IsHTMLElement("area")
                || node.IsElement("a", DOMNamespace::kSVG))
                && (node.FindAttribute("href").is_some()
                    || node.FindAttribute("xlink:href").is_some())
            {
                is_over_link = true;
                break;
            }
            ancestor = node.Parent();
        }
        Some(Self {
            is_over_link,
            has_image,
            is_user_node_draggable,
        })
    }
}
