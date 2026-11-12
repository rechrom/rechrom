//! Page contract adapter for the canonical Chromium EventDispatcher.
//! Propagation, retargeting, activation order and cleanup stay in dispatch.

#[path = "page_clipboard_commands.rs"]
mod clipboard_commands;
#[path = "page_editor_command_queries.rs"]
mod editor_command_queries;
#[path = "page_editor_commands.rs"]
mod editor_commands;
#[path = "page_editor_target_ranges.rs"]
pub(crate) mod editor_target_ranges;
#[path = "page_event_handler_frame_services.rs"]
pub mod event_handler_frame_services;
#[path = "page_event_handler_pointer.rs"]
pub mod event_handler_pointer;
#[path = "page_gesture_mouse_defaults.rs"]
pub(crate) mod gesture_mouse_defaults;
#[path = "page_gesture_selection.rs"]
pub(crate) mod gesture_selection;
#[path = "page_insert_link_commands.rs"]
mod insert_link_commands;
#[path = "page_list_indent_commands.rs"]
mod list_indent_commands;
#[path = "page_mouse_selection.rs"]
pub(crate) mod mouse_selection;
#[path = "page_mouse_selection_services.rs"]
pub(crate) mod mouse_selection_services;
#[path = "page_pointer_dom_effects.rs"]
pub mod pointer_dom_effects;
#[path = "page_pointer_event_manager.rs"]
mod pointer_input;
#[path = "page_style_commands.rs"]
mod style_commands;
pub use pointer_input::PointerInputError;
#[path = "page_gesture_event_manager.rs"]
pub mod gesture_input;
#[path = "page_gesture_mouse_frame_services.rs"]
pub(crate) mod mouse_frame_services;

#[path = "page_mouse_drag.rs"]
pub(crate) mod mouse_drag;
use super::{
    dom_event::{DomEventPaths, DomEventTargets},
    event::EventTargetResolver,
    input_event::{
        CompositionEvent as HostCompositionEvent, FocusEvent, FocusEventType, TextInputEvent,
    },
};
use crate::{
    event::{
        Event, EventListenerDispatcher, EventListenerInterestRef, EventType, MakeSyntheticEvent,
    },
    focus_controller::FocusController,
    input_type::InputTypeContext,
    ownership::{InteractionDOMMutationEmitter, InteractionDocument},
    editing_owners::{SyntheticEventDispatcher, PageEditingOwners},
};

impl interaction_default::core::page::focus_controller::FocusEventDispatch
    for EventDispatcher<'_, '_>
{
    fn DispatchElementFocusEvent(
        &self,
        target: u64,
        event_type: &'static str,
        related_target: Option<u64>,
        _focus_type: interaction_default::public::mojom::input::focus_type::FocusType,
    ) {
        let kind = match event_type {
            "blur" => EventType::kBlur,
            "focusout" => EventType::kFocusOut,
            "focus" => EventType::kFocus,
            "focusin" => EventType::kFocusIn,
            _ => EventType::kCustom,
        };
        let mut event = MakeSyntheticEvent(kind, target);
        if kind == EventType::kCustom {
            event.custom_type = event_type.into();
            event.bubbles = true;
        }
        event.related_target_node_id = related_target;
        self.Dispatch(&mut event, target);
    }

    fn DispatchElementFocusEventInDocument(
        &self,
        document: u128,
        target: u64,
        event_type: &'static str,
        related_target: Option<u64>,
        focus_type: interaction_default::public::mojom::input::focus_type::FocusType,
    ) {
        let current_document = self.document.borrow().GetDocument().RootHandle().Identity();
        if document == current_document {
            self.DispatchElementFocusEvent(target, event_type, related_target, focus_type);
            return;
        }
        let destination = self
            .focus_navigation
            .borrow()
            .as_ref()
            .and_then(|navigation| navigation.DocumentEvents(document));
        if let Some(destination) = destination {
            let kind = match event_type {
                "blur" => EventType::kBlur,
                "focusout" => EventType::kFocusOut,
                "focus" => EventType::kFocus,
                "focusin" => EventType::kFocusIn,
                _ => EventType::kCustom,
            };
            let mut event = MakeSyntheticEvent(kind, target);
            if kind == EventType::kCustom {
                event.custom_type = event_type.into();
                event.bubbles = true;
            }
            event.related_target_node_id = related_target;
            let listeners = self.listeners.borrow().clone();
            destination.Dispatch(&mut event, target, listeners);
        }
    }
    fn DispatchWindowFocusEventInDocument(&self, document: u128, event_type: &'static str) {
        let current_document = self.document.borrow().GetDocument().RootHandle().Identity();
        if document == current_document {
            self.DispatchWindowFocusEvent(event_type);
            return;
        }
        let destination = self
            .focus_navigation
            .borrow()
            .as_ref()
            .and_then(|navigation| navigation.DocumentEvents(document));
        if let Some(destination) = destination {
            let mut event = MakeSyntheticEvent(EventType::kCustom, 0);
            event.custom_type = event_type.into();
            let listeners = self.listeners.borrow().clone();
            destination.Dispatch(&mut event, 0, listeners);
        }
    }
    fn StopNodeAutoscroll(&self, node: u64) {
        crate::adapter::page_input_type::StopSelectControlAutoscroll(&self.Context(), node);
    }
    fn DispatchWindowFocusEvent(&self, event_type: &'static str) {
        let mut event = MakeSyntheticEvent(EventType::kCustom, 0);
        event.custom_type = event_type.into();
        self.Dispatch(&mut event, 0);
    }
}
use dom::UserInteractionState;
use interaction_dispatch::core::dom::events::scoped_event_queue::{
    EventQueueScope, ScopedEventQueueBinding, ScopedEventQueueOwner,
};
use interaction_dispatch::core::{
    dom::events::{
        event::{EventInterface, EventRef, EventTargetRef},
        event_dispatch_result::DispatchEventResult as NativeDispatchEventResult,
        event_dispatcher::EventDispatcher as NativeEventDispatcher,
    },
    events::{
        composition_event::{CompositionDeviceCapabilities, CompositionEvent},
        input_event::{
            EventIsComposing, InputEvent as NativeInputEvent, InputType as NativeInputType,
        },
        keyboard_event::KeyboardEventRef,
        mouse_event::{MouseCoordinateContext, MouseViewGeometry},
        pointer_event::PointerEventRef,
        text_event::{TextEvent, TextEventInputType},
        ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef},
        wheel_event::WheelEventWindow,
    },
    input::boundary_event_dispatcher::{
        BoundaryAncestorChainSnapshot, BoundaryAncestorChainSnapshotRef, BoundaryEventDOM,
    },
};
use interaction_hit_test::resident::point_queries::{
    DocumentFacts as WheelHitDocumentFacts, HitTest as ResidentWheelHitTest, ResidentFragmentSource,
};
use interaction_input::public::common::input::{
    web_mouse_event::WebMouseEvent, web_mouse_wheel_event::WebMouseWheelEvent,
    web_pointer_properties::PointF,
};
use interaction_scroll::core::input::mouse_wheel_event_manager::{
    MouseWheelEventManager, WheelDocument, WheelFrameView, WheelHitTestResult, WheelNodeQueries,
    WheelTargetQuery,
};
use layoutng_assembly::editing_state::SelectionState;
use layoutng_assembly::internal::layout_input::Offset;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

/// The Page-owned, platform-neutral LocalFrameView facts consumed while a
/// native MouseEvent is initialized. The provider may read the resident Layout
/// owner, but exposes neither LayoutEngine nor a copied layout tree.
pub trait MouseFrameGeometry {
    fn AcceptsLoadDrops(&self) -> bool {
        false
    }
    fn LoadDroppedURL(&self, url: &str, foreground: bool, user_activation: bool);
    fn DragTargetStateChanged(&self);
    fn EnumerateDroppedDirectory(
        &self,
        _target: u64,
        _directory: std::sync::Arc<fileapi::File>,
    ) -> bool {
        false
    }

    fn NodeDragImage(
        &self,
        _node: u64,
    ) -> std::io::Result<
        Option<
            std::sync::Arc<interaction_dispatch::core::clipboard::data_transfer::DragImageBitmap>,
        >,
    > {
        Ok(None)
    }
    fn SelectionDragOverlay(
        &self,
    ) -> std::io::Result<interaction_default::core::page::drag_controller::DragOverlay> {
        Ok(Default::default())
    }
    fn ResourceDragImage(
        &self,
        _node: u64,
        _size: (i32, i32),
    ) -> std::io::Result<
        Option<
            std::sync::Arc<interaction_dispatch::core::clipboard::data_transfer::DragImageBitmap>,
        >,
    > {
        Ok(None)
    }
    fn LinkDragImage(
        &self,
        _url: &str,
        _label: &[u16],
    ) -> std::io::Result<
        Option<
            std::sync::Arc<interaction_dispatch::core::clipboard::data_transfer::DragImageBitmap>,
        >,
    > {
        Ok(None)
    }
    fn HasContentLayout(&self) -> bool {
        false
    }
    fn DragFrameToViewport(
        &self,
        _point: foundation::gfx::Point,
    ) -> Option<foundation::gfx::Point> {
        None
    }

    fn AutoscrollLayoutForNode(
        &self,
        _node: u64,
    ) -> Option<Rc<dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout>>
    {
        None
    }
    fn ScheduleAutoscrollAnimation(&self);

    fn StartSystemDrag(
        &self,
        _request: interaction_default::core::page::drag_controller::SystemDragRequest,
    ) -> interaction_default::core::page::drag_controller::SystemDragSupport {
        interaction_default::core::page::drag_controller::SystemDragSupport::Unsupported
    }
    fn DragLayout(
        &self,
        _node_id: u64,
    ) -> Option<layoutng_assembly::native_mouse_coordinates::NativeMouseCoordinateLayout> {
        None
    }
    fn DragBoundingBox(&self, _node_id: u64) -> Option<foundation::gfx::Rect> {
        None
    }
    /// SimulatedEventUtil's live LayoutBox border center in root-frame space,
    /// together with that same object's effective zoom. None means no box.
    fn SimulatedClickRootFrameCenter(
        &self,
        _node: u64,
    ) -> Option<(foundation::PhysicalOffset, f32)> {
        None
    }
    fn HasFrameView(&self) -> bool;
    fn LayoutZoomFactor(&self) -> Option<f32>;
    fn DevicePixelRatio(&self) -> Option<f64>;
    fn LayoutViewportScrollOffset(&self) -> Option<(i32, i32)>;
    fn HasLayoutObject(&self, node_id: u64) -> bool;
    /// Element::IsKeyboardFocusableScroller reads the resident LayoutBox;
    /// expose only that derived source fact, never the Layout owner itself.
    fn IsUserScrollable(&self, node_id: u64) -> bool;
    /// Native LayoutObject::IsCanvas for Element focus fallback content.
    fn IsCanvasLayoutObject(&self, node_id: u64) -> bool;
    fn ConvertFromRootFrame(&self, point: PointF) -> Option<PointF>;
    /// VisualViewport's point conversion over the retained scale/scroll owner.
    fn RootFrameToViewport(&self, _point: PointF) -> Option<PointF> {
        None
    }
    fn PointerLockPosition(&self) -> Option<(PointF, PointF)>;
    /// Canonical FrameView/VisualViewport/widget values. Missing native screen
    /// origin is an unavailable capability, never a guessed last-event delta.
    fn NonLocatedContextMenuCoordinates(
        &self,
    ) -> Option<super::page_nonlocated_context_menu_coordinates::NativeContextMenuFrameCoordinates>;
    fn NonLocatedRectFromRootFrame(
        &self,
        rect: foundation::gfx::Rect,
    ) -> Option<foundation::gfx::Rect>;
    fn NonLocatedPointToRootFrame(
        &self,
        point: foundation::gfx::Point,
    ) -> Option<foundation::gfx::Point>;
    fn NonLocatedRectToRootFrame(
        &self,
        rect: foundation::gfx::Rect,
    ) -> Option<foundation::gfx::Rect>;
}

pub type MouseFrameGeometryRef = Rc<dyn MouseFrameGeometry>;

struct PageWheelHitDocumentFacts<'a>(&'a dom::Document);

impl WheelHitDocumentFacts for PageWheelHitDocumentFacts<'_> {
    fn ContainsNode(&self, node_id: u64) -> bool {
        self.0.FindNodeById(node_id).is_some()
    }

    fn DocumentElementNodeId(&self) -> Option<u64> {
        self.0
            .DocumentElement()
            .map(|element| self.0.Node(element).Id())
    }
}

struct PageMouseViewGeometry {
    window: AbstractViewRef,
    window_document: Rc<super::dom_event::WindowDocumentOwner>,
    frame: MouseFrameGeometryRef,
}

struct PageCompositionCapabilities {
    targets: Rc<DomEventTargets>,
}

struct PageCompositionEventQueueScope<'scope> {
    _scope: EventQueueScope<'scope>,
}
impl interaction_default::core::editing::ime::input_method_controller::ScopedCompositionEvents
    for PageCompositionEventQueueScope<'_>
{
}
impl CompositionDeviceCapabilities for PageCompositionCapabilities {
    fn FiresTouchEvents(
        &self,
        _view: &AbstractViewRef,
        fires_touch_events: bool,
    ) -> InputDeviceCapabilitiesRef {
        self.targets.InputDeviceCapabilities(fires_touch_events)
    }
}

impl PageMouseViewGeometry {
    fn IsWindow(&self, view: &AbstractViewRef) -> bool {
        Rc::ptr_eq(&self.window, view)
    }
}

impl MouseViewGeometry for PageMouseViewGeometry {
    fn IsLocalDOMWindow(&self, view: &AbstractViewRef) -> bool {
        self.IsWindow(view)
    }

    fn HasFrameView(&self, view: &AbstractViewRef) -> bool {
        self.IsWindow(view) && self.window_document.IsAttached() && self.frame.HasFrameView()
    }

    fn LayoutZoomFactor(&self, view: &AbstractViewRef) -> Option<f32> {
        self.IsWindow(view)
            .then(|| self.frame.LayoutZoomFactor())
            .flatten()
    }

    fn LayoutViewportScrollOffset(&self, view: &AbstractViewRef) -> Option<(i32, i32)> {
        self.IsWindow(view)
            .then(|| self.frame.LayoutViewportScrollOffset())
            .flatten()
    }

    fn ConvertFromRootFrame(&self, view: &AbstractViewRef, point: PointF) -> Option<PointF> {
        self.IsWindow(view)
            .then(|| self.frame.ConvertFromRootFrame(point))
            .flatten()
    }

    fn PointerLockPosition(&self, view: &AbstractViewRef) -> Option<(PointF, PointF)> {
        self.IsWindow(view)
            .then(|| self.frame.PointerLockPosition())
            .flatten()
    }
}

/// Call-scoped projection of the current resident Layout result. The wheel
/// manager retains only its canonical EventTarget between phases; hit-test
/// geometry never escapes this synchronous transaction.
struct PageWheelDocumentQuery<'scope, 'listener, 'frame, 'fragment> {
    dispatcher: &'scope EventDispatcher<'listener, 'frame>,
    fragments: ResidentFragmentSource<'fragment>,
    error: Cell<Option<interaction_hit_test::resident::ResidentCursorGeometryError>>,
}

impl PageWheelDocumentQuery<'_, '_, '_, '_> {
    fn FrameGeometry(&self) -> Option<MouseFrameGeometryRef> {
        self.dispatcher.mouse_frame_geometry.borrow().clone()
    }
}

impl WheelDocument for PageWheelDocumentQuery<'_, '_, '_, '_> {
    fn HasLayoutView(&self) -> bool {
        self.FrameGeometry()
            .is_some_and(|geometry| geometry.HasFrameView())
    }

    fn Window(&self) -> WheelEventWindow {
        WheelEventWindow {
            view: self.dispatcher.targets.WindowView(),
            device_pixel_ratio: self
                .FrameGeometry()
                .and_then(|geometry| geometry.DevicePixelRatio())
                .unwrap_or(1.0),
            coordinates: self.dispatcher.MouseCoordinateContext(),
            intervention: None,
        }
    }

    fn IsVerticalScrollEnforced(&self) -> bool {
        // Rechrom has no Document vertical-scroll enforcement owner. This is
        // the source field's default value, not an adapter policy branch.
        false
    }

    fn HitTest(
        &self,
        location: &interaction_hit_test::core::layout::hit_test_location::HitTestLocation,
        _request: &interaction_hit_test::core::layout::hit_test_request::HitTestRequest<'_, (), ()>,
    ) -> WheelHitTestResult {
        let zoom = f64::from(
            self.FrameGeometry()
                .and_then(|geometry| geometry.LayoutZoomFactor())
                .unwrap_or(1.0)
                .max(f32::MIN_POSITIVE),
        );
        // FindTargetNode has already converted root-widget coordinates to
        // native layout units. ResidentWheelHitTest's public boundary accepts
        // CSS viewport units and applies the same layout zoom once.
        let point = Offset {
            x: location.Point().left.ToDouble() / zoom,
            y: location.Point().top.ToDouble() / zoom,
        };
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        let hit =
            match ResidentWheelHitTest(&PageWheelHitDocumentFacts(document), self.fragments, point)
            {
                Ok(hit) => hit,
                Err(error) => {
                    self.error.set(Some(error));
                    return WheelHitTestResult {
                        inner_node: None,
                        over_scrollbar: false,
                    };
                }
            };
        WheelHitTestResult {
            inner_node: hit.as_ref().and_then(|hit| {
                self.dispatcher
                    .targets
                    .TargetFromBorrowedDocument(document, hit.node_id)
            }),
            over_scrollbar: hit.as_ref().is_some_and(|hit| hit.overflow_controls),
        }
    }

    fn DocumentElement(&self) -> Option<EventTargetRef> {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        self.dispatcher
            .targets
            .TargetFromBorrowedDocument(document, document.Node(document.DocumentElement()?).Id())
    }
}

struct PageWheelFrameView {
    frame: Option<MouseFrameGeometryRef>,
}

impl WheelFrameView for PageWheelFrameView {
    fn ConvertFromRootFrame(&self, point: foundation::gfx::Point) -> foundation::PhysicalOffset {
        let point = PointF::new(point.x() as f32, point.y() as f32);
        let converted = self
            .frame
            .as_ref()
            .and_then(|frame| frame.ConvertFromRootFrame(point))
            .unwrap_or(point);
        foundation::PhysicalOffset::new(
            foundation::LayoutUnit::from_f64(f64::from(converted.x())),
            foundation::LayoutUnit::from_f64(f64::from(converted.y())),
        )
    }
}

struct PageWheelNodeQueries<'scope, 'listener, 'frame> {
    dispatcher: &'scope EventDispatcher<'listener, 'frame>,
    frame_geometry: Option<MouseFrameGeometryRef>,
}

impl PageWheelNodeQueries<'_, '_, '_> {
    fn Index(&self, target: &EventTargetRef) -> Option<usize> {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        (self.dispatcher.targets.OwnerDocumentHandle(target) == Some(document.RootHandle()))
            .then(|| self.dispatcher.targets.NodeId(target))
            .flatten()
            .and_then(|id| document.FindNodeById(id))
    }
}

impl WheelNodeQueries for PageWheelNodeQueries<'_, '_, '_> {
    fn IsTextNode(&self, node: &EventTargetRef) -> bool {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        self.Index(node).is_some_and(|index| {
            document.Node(index).Type() == dom::persistent_document::DOMNodeType::kText
        })
    }

    fn FlatTreeParent(&self, node: &EventTargetRef) -> Option<EventTargetRef> {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        let parent = document.FlatTreeParent(self.Index(node)?)?;
        self.dispatcher
            .targets
            .TargetFromBorrowedDocument(document, document.Node(parent).Id())
    }

    fn HasLayoutObject(&self, node: &EventTargetRef) -> bool {
        let Some(node_id) = self.dispatcher.targets.NodeId(node) else {
            return false;
        };
        self.frame_geometry
            .as_ref()
            .is_some_and(|geometry| geometry.HasLayoutObject(node_id))
    }
}

struct PageWheelTargetQuery<'scope, 'listener, 'frame, 'fragment> {
    document: PageWheelDocumentQuery<'scope, 'listener, 'frame, 'fragment>,
    view: PageWheelFrameView,
    nodes: PageWheelNodeQueries<'scope, 'listener, 'frame>,
    result: Cell<Option<DispatchEventResult>>,
}

impl WheelTargetQuery for PageWheelTargetQuery<'_, '_, '_, '_> {
    fn Document(&self) -> &dyn WheelDocument {
        &self.document
    }

    fn View(&self) -> &dyn WheelFrameView {
        &self.view
    }

    fn Nodes(&self) -> &dyn WheelNodeQueries {
        &self.nodes
    }

    fn TargetWindow(&self, _target: &EventTargetRef) -> WheelEventWindow {
        self.document.Window()
    }

    fn IsVerticalScrollEnforced(&self, _target: &EventTargetRef) -> bool {
        self.document.IsVerticalScrollEnforced()
    }

    fn DispatchEvent(
        &self,
        target: &EventTargetRef,
        event: &EventRef,
    ) -> NativeDispatchEventResult {
        let Some(target) = self.document.dispatcher.targets.NodeId(target) else {
            return NativeDispatchEventResult::kNotCanceled;
        };
        event.borrow_mut().BaseEventMut().SetTrusted(true);
        let (native, result) =
            ports::DispatchCanonicalWithResult(self.document.dispatcher, event.clone(), target);
        self.result.set(Some(result));
        native
    }
}

struct MouseRemovalRelations<'a> {
    document: &'a dom::Document,
    targets: &'a DomEventTargets,
}

impl MouseRemovalRelations<'_> {
    fn Index(&self, target: &EventTargetRef) -> Option<usize> {
        (self.targets.OwnerDocumentHandle(target) == Some(self.document.RootHandle()))
            .then(|| self.targets.NodeId(target))
            .flatten()
            .and_then(|id| (id != 0).then_some(id))
            .and_then(|id| self.document.FindNodeById(id))
    }

    fn Target(&self, index: usize) -> Option<EventTargetRef> {
        self.targets
            .TargetFromBorrowedDocument(self.document, self.document.Node(index).Id())
    }
}

impl crate::core::input::mouse_event_manager::MouseRemovalNodeRelations
    for MouseRemovalRelations<'_>
{
    fn ParentNode(&self, node: &EventTargetRef) -> Option<EventTargetRef> {
        self.Target(self.document.Node(self.Index(node)?).Parent()?)
    }

    fn ParentOrShadowHostElement(&self, node: &EventTargetRef) -> Option<EventTargetRef> {
        self.Target(self.document.ParentOrShadowHostElement(self.Index(node)?)?)
    }

    fn IsElement(&self, node: &EventTargetRef) -> bool {
        self.Index(node).is_some_and(|index| {
            self.document.Node(index).Type() == dom::persistent_document::DOMNodeType::kElement
        })
    }

    fn IsShadowIncludingInclusiveAncestorOf(
        &self,
        ancestor: &EventTargetRef,
        node: &EventTargetRef,
    ) -> bool {
        let (Some(ancestor), Some(node)) = (self.Index(ancestor), self.Index(node)) else {
            return false;
        };
        self.document
            .IsShadowIncludingInclusiveAncestor(ancestor, node)
    }
}

struct PageNodeRemovalObserver {
    targets: Rc<DomEventTargets>,
    mouse: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
    pointer: Rc<crate::core::input::pointer_event_manager::MousePointerEventManager>,
    wheel: Rc<
        RefCell<
            Option<Rc<interaction_scroll::core::input::event_handler::EventHandlerScrollRouting>>,
        >,
    >,
}

struct PageBoundaryAncestorChain(Vec<EventTargetRef>);
impl BoundaryAncestorChainSnapshot for PageBoundaryAncestorChain {
    fn Ancestors(&self) -> &[EventTargetRef] {
        &self.0
    }
}

struct PageBoundaryDOM<'a> {
    document: &'a InteractionDocument,
    targets: &'a DomEventTargets,
    listener_interest: &'a Rc<RefCell<Option<EventListenerInterestRef>>>,
}

impl PageBoundaryDOM<'_> {
    fn Interest(
        &self,
        target: &EventTargetRef,
    ) -> Option<(
        EventListenerInterestRef,
        dom::persistent_document::DOMOwnerHandle,
        u64,
    )> {
        Some((
            self.listener_interest.borrow().as_ref()?.clone(),
            self.targets.OwnerDocumentHandle(target)?,
            self.targets.NodeId(target)?,
        ))
    }
}

impl BoundaryEventDOM for PageBoundaryDOM<'_> {
    fn IsInDocument(&self, target: &EventTargetRef) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let relations = MouseRemovalRelations {
            document,
            targets: self.targets,
        };
        relations
            .Index(target)
            .is_some_and(|index| document.IsConnectedToDocument(index))
    }

    fn FlatTreeAncestorChain(&self, target: &EventTargetRef) -> BoundaryAncestorChainSnapshotRef {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let relations = MouseRemovalRelations {
            document,
            targets: self.targets,
        };
        let mut ancestors = Vec::new();
        let mut current = relations.Index(target);
        while let Some(index) = current {
            if let Some(target) = relations.Target(index) {
                ancestors.push(target);
            }
            current = document.FlatTreeParent(index);
        }
        Rc::new(PageBoundaryAncestorChain(ancestors))
    }

    fn HasCapturingEventListeners(&self, target: &EventTargetRef, event_type: &str) -> bool {
        self.Interest(target)
            .is_some_and(|(interest, document, target)| {
                interest.HasCapturingEventListeners(document, target, event_type)
            })
    }
}

struct PageMouseBoundaryDispatch<'a, 'listener, 'frame> {
    dispatcher: &'a EventDispatcher<'listener, 'frame>,
    coordinates: MouseCoordinateContext,
    result: Cell<Option<DispatchEventResult>>,
}

impl crate::core::input::mouse_event_manager::MouseBoundaryEventDispatchPort
    for PageMouseBoundaryDispatch<'_, '_, '_>
{
    fn DocumentWindow(&self, node: &EventTargetRef) -> Option<AbstractViewRef> {
        let owner = self.dispatcher.document.borrow().GetDocument().RootHandle();
        (self.dispatcher.targets.OwnerDocumentHandle(node) == Some(owner))
            .then(|| self.dispatcher.targets.WindowView())
    }

    fn CoordinateContext(&self) -> MouseCoordinateContext {
        self.coordinates.clone()
    }

    fn SourceCapabilities(
        &self,
        _: &AbstractViewRef,
        from_touch: bool,
    ) -> InputDeviceCapabilitiesRef {
        self.dispatcher.targets.InputDeviceCapabilities(from_touch)
    }

    fn HasEventListeners(&self, target: &EventTargetRef, event_type: &str) -> bool {
        self.dispatcher.HasEventListeners(target, event_type)
    }

    fn DispatchEvent(
        &self,
        target: &EventTargetRef,
        event: &EventRef,
    ) -> NativeDispatchEventResult {
        let Some(target) = self.dispatcher.targets.NodeId(target) else {
            return NativeDispatchEventResult::kNotCanceled;
        };
        event.borrow_mut().BaseEventMut().SetTrusted(true);
        let (native, result) =
            ports::DispatchCanonicalWithResult(self.dispatcher, event.clone(), target);
        self.result.set(Some(result));
        native
    }
}

impl crate::core::input::mouse_event_manager::MouseClickDispatchPort
    for PageMouseBoundaryDispatch<'_, '_, '_>
{
    fn IsConnected(&self, node: &EventTargetRef) -> bool {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        let relations = MouseRemovalRelations {
            document,
            targets: &self.dispatcher.targets,
        };
        relations
            .Index(node)
            .is_some_and(|index| document.IsConnectedToDocument(index))
    }

    fn SameDocument(&self, a: &EventTargetRef, b: &EventTargetRef) -> bool {
        self.dispatcher.targets.OwnerDocumentHandle(a)
            == self.dispatcher.targets.OwnerDocumentHandle(b)
    }

    fn CommonAncestorForClick(
        &self,
        a: &EventTargetRef,
        b: &EventTargetRef,
    ) -> Option<EventTargetRef> {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        let relations = MouseRemovalRelations {
            document,
            targets: &self.dispatcher.targets,
        };
        let mut ancestors = std::collections::HashSet::new();
        let mut current = Some(relations.Index(a)?);
        while let Some(index) = current {
            ancestors.insert(index);
            current = document.FlatTreeParent(index);
        }
        let mut current = Some(relations.Index(b)?);
        while let Some(index) = current {
            if ancestors.contains(&index) {
                return relations.Target(index);
            }
            current = document.FlatTreeParent(index);
        }
        None
    }

    fn LightDismissFromClickEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::LightDismissFromClickEnabled()
    }

    fn HandlePopoverLightDismissForClick(&self, _: &EventTargetRef, _: &EventTargetRef) {
        debug_assert!(!self.LightDismissFromClickEnabled());
    }

    fn HandleDialogLightDismissForClick(
        &self,
        _: &dyn crate::core::input::mouse_event_manager::MousePointerTarget,
        _: &dyn crate::core::input::mouse_event_manager::MousePointerTarget,
    ) {
        debug_assert!(!self.LightDismissFromClickEnabled());
    }
}

impl crate::core::input::pointer_event_manager::MousePointerEventDispatchPort
    for PageMouseBoundaryDispatch<'_, '_, '_>
{
    fn View(&self) -> Option<AbstractViewRef> {
        Some(self.dispatcher.targets.WindowView())
    }

    fn IsInDocument(&self, target: &EventTargetRef) -> bool {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        let relations = MouseRemovalRelations {
            document,
            targets: &self.dispatcher.targets,
        };
        relations
            .Index(target)
            .is_some_and(|index| document.IsConnectedToDocument(index))
    }

    fn OwnerDocument(&self, target: &EventTargetRef) -> EventTargetRef {
        self.dispatcher
            .targets
            .OwnerDocumentTarget(target)
            .expect("pointer target belongs to the current canonical Document")
    }

    fn HasPointerEventListeners(&self, target: &EventTargetRef, event_type: &str) -> bool {
        self.dispatcher.HasEventListeners(target, event_type)
    }

    fn CoordinateContext(&self) -> MouseCoordinateContext {
        self.coordinates.clone()
    }

    fn DispatchPointerEvent(
        &self,
        target: &EventTargetRef,
        event: &PointerEventRef,
    ) -> interaction_dispatch::core::input::event_handling_util::WebInputEventResult {
        use interaction_dispatch::core::input::event_handling_util::ToWebInputEventResult;
        let event_ref = EventRef::FromShared(event.clone());
        event_ref.borrow_mut().BaseEventMut().SetTrusted(true);
        ToWebInputEventResult(
            crate::core::input::mouse_event_manager::MouseBoundaryEventDispatchPort::DispatchEvent(
                self, target, &event_ref,
            ),
        )
    }

    fn DispatchMouseEvent(
        &self,
        mouse: &Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        target: &EventTargetRef,
        event_type: &str,
        event: &WebMouseEvent,
        last_position: PointF,
    ) -> interaction_dispatch::core::input::event_handling_util::WebInputEventResult {
        crate::core::input::mouse_event_manager::MouseEventManager::WithState(mouse.clone())
            .DispatchMouseEventWithDispatch(
                self,
                Some(target),
                event_type,
                event,
                Some(last_position),
                None,
                false,
            )
            .1
    }

    fn SetElementUnderMouse(
        &self,
        mouse: &Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
    ) {
        let boundary_dom = PageBoundaryDOM {
            document: &self.dispatcher.document,
            targets: &self.dispatcher.targets,
            listener_interest: &self.dispatcher.listener_interest,
        };
        crate::core::input::mouse_event_manager::MouseEventManager::WithState(mouse.clone())
            .SetElementUnderMouseWithDispatch(
                &boundary_dom,
                self,
                &|target| {
                    crate::core::input::pointer_event_manager::MousePointerEventDispatchPort::IsInDocument(
                        self, target,
                    )
                },
                foundation::RuntimeEnabledFeatures::BoundaryEventDispatchTracksNodeRemovalEnabled(),
                target,
                event,
            );
    }

    fn DispatchMouseClickIfNeeded(
        &self,
        mouse: &Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        target: Option<&EventTargetRef>,
        captured_target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
        pointer_id: interaction_input::public::common::input::pointer_id::PointerId,
        pointer_type: &str,
        down_target: Option<&dyn crate::core::input::mouse_event_manager::MousePointerTarget>,
        up_target: Option<&dyn crate::core::input::mouse_event_manager::MousePointerTarget>,
    ) {
        // PointerEventManager returns the merged pointerup/mouseup disposition;
        // click is synchronously dispatched afterwards but does not replace it.
        let release_result = self.result.get();
        crate::core::input::mouse_event_manager::MouseEventManager::WithState(mouse.clone())
            .DispatchMouseClickIfNeededWithDispatch(
                self,
                target,
                captured_target,
                event,
                pointer_id,
                pointer_type,
                down_target,
                up_target,
            );
        self.result.set(release_result);
    }
}

impl dom::node_removal_observer::DocumentNodeRemovalObserver for PageNodeRemovalObserver {
    fn NodeWillBeRemoved(&self, document: &dom::Document, node: usize) {
        let Some(target) = self
            .targets
            .TargetFromBorrowedDocument(document, document.Node(node).Id())
        else {
            return;
        };
        let relations = MouseRemovalRelations {
            document,
            targets: &self.targets,
        };
        crate::core::input::mouse_event_manager::MouseEventManager::WithState(self.mouse.clone())
            .NodeWillBeRemovedWithRelations(
                &relations,
                foundation::RuntimeEnabledFeatures::BoundaryEventDispatchTracksNodeRemovalEnabled(),
                &target,
            );
        self.pointer.NodeWillBeRemovedWithRelations(
            &relations,
            foundation::RuntimeEnabledFeatures::BoundaryEventDispatchTracksNodeRemovalEnabled(),
            &target,
        );
        if let Some(wheel) = self.wheel.borrow().as_ref() {
            wheel.WheelElementRemoved(Some(&target));
        }
    }

    fn ElementRemoved(&self, document: &dom::Document, node: usize) {
        let Some(target) = self
            .targets
            .TargetFromBorrowedDocument(document, document.Node(node).Id())
        else {
            return;
        };
        self.pointer.ElementRemoved(&target);
    }
}

#[path = "page_dispatch_ports.rs"]
mod ports;
// cpp: interaction/event_dispatcher.h:16-21
#[derive(Clone, Copy, Default, Debug)]
pub struct DispatchEventResult {
    pub target_node_id: Option<u64>,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub default_handled: bool,
}
/// The canonical dispatch identities and path factory for one retained DOM
/// owner. Interaction retains its lazy slot across ordinary event transactions.
/// The source queue retains pending Events only for its active queue scopes;
/// no copied path snapshot or listener invocation is stored here.
pub struct PageDispatchState {
    document: InteractionDocument,
    root: dom::persistent_document::DOMOwnerHandle,
    targets: Rc<DomEventTargets>,
    paths: DomEventPaths,
    scoped_queue: Rc<ScopedEventQueueOwner>,
}
impl PageDispatchState {
    pub(crate) fn ForDocument(
        slot: &RefCell<Option<Rc<Self>>>,
        document: InteractionDocument,
        listener_interest: Rc<RefCell<Option<EventListenerInterestRef>>>,
    ) -> Rc<Self> {
        let root = document.borrow().GetDocument().RootHandle();
        if let Some(state) = slot.borrow().as_ref() {
            if Rc::ptr_eq(&state.document, &document) && state.root == root {
                return state.clone();
            }
        }
        let targets = DomEventTargets::new(document.clone());
        let paths = DomEventPaths::new(
            targets.clone(),
            Rc::new(move |target, event_type| {
                listener_interest
                    .borrow()
                    .as_ref()
                    .is_some_and(|interest| interest.HasEventListeners(root, target, event_type))
            }),
        );
        let state = Rc::new(Self {
            document,
            root,
            targets,
            paths,
            scoped_queue: ScopedEventQueueOwner::new(),
        });
        *slot.borrow_mut() = Some(state.clone());
        state
    }

    pub(crate) fn MatchesDocument(&self, document: &InteractionDocument) -> bool {
        Rc::ptr_eq(&self.document, document)
            && self.root == document.borrow().GetDocument().RootHandle()
    }

    pub(crate) fn ResolveTarget(&self, node_id: u64) -> Option<EventTargetRef> {
        self.targets.ResolveTarget(node_id)
    }

    pub(crate) fn Targets(&self) -> Rc<DomEventTargets> {
        self.targets.clone()
    }

    pub(crate) fn NodeRemovalObserver(
        &self,
        mouse: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        pointer: Rc<crate::core::input::pointer_event_manager::MousePointerEventManager>,
        wheel: Rc<
            RefCell<
                Option<
                    Rc<interaction_scroll::core::input::event_handler::EventHandlerScrollRouting>,
                >,
            >,
        >,
    ) -> Rc<dyn dom::node_removal_observer::DocumentNodeRemovalObserver> {
        Rc::new(PageNodeRemovalObserver {
            targets: self.targets.clone(),
            mouse,
            pointer,
            wheel,
        })
    }

    pub(crate) fn WheelTargetIdentity(
        &self,
    ) -> Rc<dyn super::page_scroll_manager::PageWheelTargetIdentity> {
        self.targets.clone()
    }
}
// cpp: interaction/event_dispatcher.h:26-40
// cpp: interaction/event_dispatcher.cc:10-20
#[derive(Clone)]
pub struct EventDispatcher<'listener, 'frame> {
    pub(crate) script_invocation: Option<&'listener dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation>,
    input_dispatch_result: Option<Rc<Cell<DispatchEventResult>>>,
    pub(crate) event_timing: Option<Rc<interaction_default::core::timing::window_performance::WindowPerformance>>,
    pub(crate) interaction_output: Option<super::interaction_contract::InteractionOutputEmitter>,
    pub(crate) input_frame: Option<Rc<super::source_input_managers::PointerFrameInputState>>,
    pub(crate) editor_scroll: Option<Rc<super::page_scroll_manager::PageScrollManagerOwners>>,
    pub(crate) input_method_state: Option<Rc<interaction_default::core::editing::ime::input_method_controller::InputMethodControllerState>>,
    clock: Rc<dyn interaction_dispatch::core::dom::events::event::EventClock>,
    document: InteractionDocument,
    emit: InteractionDOMMutationEmitter,
    listeners: Rc<RefCell<Option<EventListenerDispatcher<'listener>>>>,
    listener_interest: Rc<RefCell<Option<EventListenerInterestRef>>>,
    mouse_frame_geometry: Rc<RefCell<Option<MouseFrameGeometryRef>>>,
    mouse_state: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
    focus_navigation: Rc<RefCell<Option<Rc<super::page_focus_navigation::PageFocusNavigation>>>>,
    state: Rc<RefCell<UserInteractionState>>,
    editor: Rc<PageEditingOwners>,
    selections: Rc<SelectionState>,
    submit_form: Option<Rc<dyn Fn(Result<Option<crate::input_type::PrepareFormEntryListResult>, crate::input_type::FormDataAdmission>) + 'listener>>,
    targets: Rc<DomEventTargets>,
    paths: DomEventPaths,
    scoped_queue: Rc<ScopedEventQueueOwner>,
    default_action_state: Rc<crate::input_type::DefaultActionState>,
    focus_services: Rc<super::page_focus_controller::DocumentFocusServices>,
    selection_controller_state:
        Rc<interaction_default::core::editing::selection_controller::SelectionControllerState>,
    editing_geometry: Option<super::page_editing_context::PreparedEditingGeometry<'frame>>,
    editing_geometry_revision: Option<u64>,
    keyboard_default: Rc<RefCell<Option<Rc<dyn Fn(&KeyboardEventRef, u64) + 'frame>>>>,
    text_input_default: Rc<RefCell<Option<Rc<dyn Fn(&EventRef) + 'frame>>>>,
    editing_command_event: Rc<RefCell<Option<EventRef>>>,
    middle_click_default: Rc<RefCell<Option<Rc<dyn Fn(u64) + 'frame>>>>,
}
#[derive(Debug)]
pub enum EditorCommandError {
    KillRing(interaction_default::core::editing::kill_ring::RequiredKillRingPlatform),
    Aborted(
        interaction_default::core::editing::commands::paragraph_editing_context::EditingAborted,
    ),
    RequiredPort(&'static str),
}
impl<'listener, 'frame> EventDispatcher<'listener, 'frame> {
    fn HostInputElement(&self, target: Option<u64>) -> Option<u64> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let mut node = target.and_then(|id| document.FindNodeById(id));
        while let Some(index) = node {
            let current = document.Node(index);
            if current.Type() == dom::persistent_document::DOMNodeType::kElement {
                return Some(current.Id());
            }
            node = current.Parent();
        }
        None
    }

    /// Host focus enters the same retained FocusController and canonical event
    /// dispatcher as pointer/keyboard focus. This is the sole native binding
    /// port for host focus input.
    pub(crate) fn DispatchHostFocus(&self, event: &FocusEvent) -> DispatchEventResult {
        let Some(target) = self.HostInputElement(
            (event.target_node_id != 0)
                .then_some(event.target_node_id)
                .or(self.state.borrow().focused_node_id),
        ) else {
            return DispatchEventResult::default();
        };
        let nested = self.clone();
        let dispatch: SyntheticEventDispatcher<'_> = Rc::new(move |event, id| {
            nested.Dispatch(event, id);
        });
        let focus = FocusController::new(
            self.document.clone(),
            self.state.clone(),
            self.editor.clone(),
            dispatch,
            self.default_action_state.clone(),
            self.focus_services.clone(),
        )
        .WithFocusDispatch(Rc::new(self.clone()));
        if event.r#type == FocusEventType::kFocus {
            focus.Focus(target, true);
        } else {
            focus.Blur(target);
        }
        DispatchEventResult {
            target_node_id: Some(target),
            ..Default::default()
        }
    }

    /// Native IME input is owned by the retained InputMethodController state
    /// already installed on this dispatcher. The result is projected once at
    /// the Page boundary; there is no second facade editing transaction.
    pub(crate) fn DispatchHostComposition(
        &self,
        event: &HostCompositionEvent,
    ) -> DispatchEventResult {
        let prepare = |_: u32| {
            if let Some(geometry) = self.editing_geometry {
                geometry.AssertPrepared();
            }
        };
        super::page_input_method_controller::DispatchPageCompositionEvent(
            self.document.clone(),
            self,
            self.editor.clone(),
            self.selections.clone(),
            self.state.clone(),
            self.emit.clone(),
            self.input_method_state
                .clone()
                .expect("native dispatcher has InputMethodController state"),
            Some(self.focus_services.selection.clone()),
            event,
            &super::page_input_method_controller::PageCompositionGeometry(&prepare),
        )
    }

    pub(crate) fn DispatchHostTextInput(&self, event: &TextInputEvent) -> DispatchEventResult {
        let prepare = |_: u32| {
            if let Some(geometry) = self.editing_geometry {
                geometry.AssertPrepared();
            }
        };
        super::page_input_method_controller::DispatchPageTextInputEvent(
            self.document.clone(),
            self,
            self.editor.clone(),
            self.selections.clone(),
            self.state.clone(),
            self.emit.clone(),
            self.input_method_state
                .clone()
                .expect("native dispatcher has InputMethodController state"),
            Some(self.focus_services.selection.clone()),
            event,
            &super::page_input_method_controller::PageCompositionGeometry(&prepare),
        )
    }

    pub(crate) fn BindInputDispatchResult(&mut self, result: Rc<Cell<DispatchEventResult>>) {
        self.input_dispatch_result = Some(result);
    }
    pub(crate) fn FrameSelection(&self) -> &Rc<super::page_frame_selection::PageFrameSelection> {
        &self.focus_services.selection
    }
    /// The canonical descriptor selects source execution; native selectors
    /// never create a second name-to-operation table in the dispatcher.
    pub(crate) fn EditingSelectionState(&self) -> Rc<SelectionState> {
        self.selections.clone()
    }
    pub(crate) fn EditingOwners(&self) -> Rc<PageEditingOwners> {
        self.editor.clone()
    }
    pub(crate) fn EditorCommandMutationEmitter(&self) -> InteractionDOMMutationEmitter {
        self.emit.clone()
    }
    pub(crate) fn EditorCommandEventQueueScope<'scope>(&self) -> EventQueueScope<'scope>
    where
        'listener: 'scope,
        'frame: 'scope,
    {
        self.ScopedQueueBinding().Scope()
    }
    pub(crate) fn ExecuteEditorCommand(
        &self,
        command: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
    ) -> Result<bool, EditorCommandError> {
        self.ExecuteEditorCommandWithValue(command, "")
    }
    pub(crate) fn ExecuteEditorCommandWithValue(
        &self,
        command: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
        value: &str,
    ) -> Result<bool, EditorCommandError> {
        self.ExecuteEditorCommandFromSource(command, value, interaction_default::core::editing::commands::editor_command_table::EditorCommandSource::kMenuOrKeyBinding)
    }
    pub(crate) fn ExecuteEditorCommandFromSource(
        &self,
        command: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
        value: &str,
        source: interaction_default::core::editing::commands::editor_command_table::EditorCommandSource,
    ) -> Result<bool, EditorCommandError> {
        use interaction_default::core::editing::{
            commands::{
                editor_command_input_type::{CommandInputTypeContext, InputTypeFromCommandType},
                editor_command_table::ExecuteFunction as F,
            },
            text_granularity::TextGranularity as G,
        };
        let Some(descriptor) = command.Descriptor() else {
            return Ok(false);
        };
        if source == interaction_default::core::editing::commands::editor_command_table::EditorCommandSource::kDom {
            let can_execute = descriptor
                .CanExecute(&editor_command_queries::CommandQuery(self), source)
                .map_err(EditorCommandError::RequiredPort)?;
            if !can_execute {
                return Ok(false);
            }
        }
        if descriptor.execute == F::ExecuteDefaultParagraphSeparator {
            use interaction_default::core::editing::editor::EditorParagraphSeparator;
            let separator = if value.eq_ignore_ascii_case("div") {
                Some(EditorParagraphSeparator::kIsDiv)
            } else if value.eq_ignore_ascii_case("p") {
                Some(EditorParagraphSeparator::kIsP)
            } else {
                None
            };
            if let Some(separator) = separator {
                self.editor
                    .ime_typing
                    .editor
                    .SetDefaultParagraphSeparator(separator);
            }
            return Ok(true);
        }
        if let Some(result) = self.ExecuteMovementCommand(descriptor.execute) {
            return result;
        }
        if let Some(result) = self.ExecuteSelectionCommand(descriptor.execute, source) {
            return result;
        }
        if let Some(result) = self.ExecuteExtendedSelectionCommand(command, source) {
            return result;
        }
        if let Some(result) = self.ExecuteStyleCommand(command, value, source) {
            return result;
        }
        if let Some(result) = self.ExecuteListIndentCommand(command, source) {
            return result;
        }
        if let Some(result) = self.ExecuteInsertLinkCommand(command, value, source) {
            return result;
        }
        if let Some(result) = self.ExecuteEditorHostCommand(descriptor.execute) {
            return result;
        }
        if matches!(
            descriptor.execute,
            F::ExecuteUndo
                | F::ExecuteRedo
                | F::InsertCommands_ExecuteInsertLineBreak
                | F::InsertCommands_ExecuteInsertParagraph
                | F::InsertCommands_ExecuteInsertNewline
                | F::InsertCommands_ExecuteInsertNewlineInQuotedContent
                | F::InsertCommands_ExecuteInsertTab
                | F::InsertCommands_ExecuteInsertBacktab
                | F::InsertCommands_ExecuteInsertText
        ) {
            if !matches!(descriptor.execute, F::ExecuteUndo | F::ExecuteRedo)
                && source == interaction_default::core::editing::commands::editor_command_table::EditorCommandSource::kMenuOrKeyBinding
                && !super::page_input_method_controller::CanExecuteEditingCommand(self)
            {
                return Ok(false);
            }
            let editor = self.EditingOwners();
            if (descriptor.execute == F::ExecuteUndo
                && !editor.ime_typing.editor.GetUndoStack().CanUndo())
                || (descriptor.execute == F::ExecuteRedo
                    && !editor.ime_typing.editor.GetUndoStack().CanRedo())
            {
                return Ok(false);
            }
            struct InputTypeContext<'a, 'b, 'c>(&'a EventDispatcher<'b, 'c>);
            impl CommandInputTypeContext for InputTypeContext<'_, '_, '_> {
                fn CanEditRichly(&self) -> bool {
                    super::page_input_method_controller::PasteCanEditRichly(self.0)
                }
                fn VisibleSelectionIsRange(&self) -> bool {
                    self.0
                        .FrameSelection()
                        .CreateVisibleSelection(self.0.FrameSelection().Current())
                        .is_some_and(|s| s.anchor != s.focus)
                }
            }
            let identity = self.document.borrow().GetDocument().RootHandle().Identity();
            if !self.EditingBeforeInputFromSource(
                command,
                InputTypeFromCommandType(command, &InputTypeContext(self)),
                source,
            )? {
                return Ok(true);
            }
            if !self.FrameSelection().IsAvailable()
                || self.document.borrow().GetDocument().RootHandle().Identity() != identity
            {
                return Ok(false);
            }
            if source == interaction_default::core::editing::commands::editor_command_table::EditorCommandSource::kMenuOrKeyBinding && self
                .input_method_state
                .as_ref()
                .is_some_and(|state| state.GetActiveEditContext().is_some())
                && matches!(
                    descriptor.execute,
                    F::InsertCommands_ExecuteInsertTab
                        | F::InsertCommands_ExecuteInsertBacktab
                        | F::InsertCommands_ExecuteInsertNewline
                        | F::InsertCommands_ExecuteInsertLineBreak
                )
            {
                return Ok(true);
            }
            self.FrameSelection().PrepareEditing();
            return Ok(
                super::page_input_method_controller::ExecuteTypingEditorCommand(
                    self,
                    descriptor.execute,
                    value,
                ),
            );
        }
        if let Some(result) =
            clipboard_commands::ExecuteFromSource(self, descriptor.execute, source)
        {
            return result.map_err(EditorCommandError::RequiredPort);
        }
        let deletion = match descriptor.execute {
            F::ExecuteDelete => Some((G::kCharacter, false, true, false)),
            F::ExecuteDeleteBackward | F::ExecuteDeleteBackwardByDecomposingPreviousCharacter => {
                Some((G::kCharacter, false, false, true))
            }
            F::ExecuteDeleteForward | F::ExecuteForwardDelete => {
                Some((G::kCharacter, true, false, true))
            }
            F::ExecuteDeleteToBeginningOfLine => Some((G::kLineBoundary, false, true, false)),
            F::ExecuteDeleteToEndOfLine => Some((G::kLineBoundary, true, true, false)),
            F::ExecuteDeleteToBeginningOfParagraph => {
                Some((G::kParagraphBoundary, false, true, false))
            }
            F::ExecuteDeleteToEndOfParagraph => Some((G::kParagraphBoundary, true, true, false)),
            F::ExecuteDeleteWordBackward => Some((G::kWord, false, true, false)),
            F::ExecuteDeleteWordForward => Some((G::kWord, true, true, false)),
            F::ExecuteYank | F::ExecuteYankAndSelect => None,
            _ => return Ok(false),
        };
        if source == interaction_default::core::editing::commands::editor_command_table::EditorCommandSource::kMenuOrKeyBinding
            && !super::page_input_method_controller::CanExecuteEditingCommand(self) {
            return Ok(false);
        }
        if descriptor.execute == F::ExecuteDelete
            && !self
                .FrameSelection()
                .CreateVisibleSelection(self.FrameSelection().Current())
                .is_some_and(|s| s.anchor != s.focus)
        {
            return Ok(false);
        }
        if let Some((granularity, forward, kill_ring, typing)) = deletion {
            struct InputTypeContext<'a, 'b, 'c>(&'a EventDispatcher<'b, 'c>);
            impl CommandInputTypeContext for InputTypeContext<'_, '_, '_> {
                fn CanEditRichly(&self) -> bool {
                    super::page_input_method_controller::PasteCanEditRichly(self.0)
                }
                fn VisibleSelectionIsRange(&self) -> bool {
                    self.0
                        .FrameSelection()
                        .CreateVisibleSelection(self.0.FrameSelection().Current())
                        .is_some_and(|selection| selection.anchor != selection.focus)
                }
            }
            let input_type = InputTypeFromCommandType(command, &InputTypeContext(self));
            let identity = self.document.borrow().GetDocument().RootHandle().Identity();
            if !self.EditingBeforeInputFromSource(command, input_type, source)? {
                return Ok(true);
            }
            if !self.FrameSelection().IsAvailable()
                || self.document.borrow().GetDocument().RootHandle().Identity() != identity
            {
                return Ok(false);
            }
            if let Some(context) = self
                .input_method_state
                .as_ref()
                .and_then(|state| state.GetActiveEditContext())
            {
                use interaction_default::core::editing::commands::editor_command_table::EditingCommandType as C;
                match command {
                    C::DeleteBackward => {
                        context.DeleteBackward();
                        return Ok(true);
                    }
                    C::DeleteForward => {
                        context.DeleteForward();
                        return Ok(true);
                    }
                    C::DeleteWordBackward => {
                        context.DeleteWordBackward();
                        return Ok(true);
                    }
                    C::DeleteWordForward => {
                        context.DeleteWordForward();
                        return Ok(true);
                    }
                    _ => {}
                }
            }
            self.FrameSelection().PrepareEditing();
            super::page_input_method_controller::ExecuteDeleteWithDirection(
                self,
                self.document.clone(),
                self.editor.clone(),
                self.selections.clone(),
                self.state.clone(),
                self.emit.clone(),
                self.FrameSelection().clone(),
                granularity,
                forward,
                kill_ring,
                typing,
                descriptor.execute == F::ExecuteDelete,
            )
            .map_err(EditorCommandError::Aborted)
        } else {
            self.FrameSelection().PrepareEditing();
            super::page_input_method_controller::ExecuteYank(
                self,
                self.document.clone(),
                self.editor.clone(),
                self.selections.clone(),
                self.state.clone(),
                self.emit.clone(),
                self.FrameSelection().clone(),
                descriptor.execute == F::ExecuteYankAndSelect,
            )
            .map_err(EditorCommandError::KillRing)
        }
    }
    // Source: editor_key_bindings.cc:51-62; editing_utilities.cc:1006-1016.
    pub(crate) fn EditingKeyboardWritingMode(
        &self,
    ) -> Result<foundation::WritingMode, &'static str> {
        use super::page_frame_selection::{CompositionTextQuery, CompositionTextResult};
        let node = self
            .FrameSelection()
            .Current()
            .and_then(|selection| selection.focus.node_id)
            .or(self.state.borrow().focused_node_id);
        let Some(node) = node else {
            return Ok(foundation::WritingMode::kHorizontalTb);
        };
        {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            let Some(index) = document.FindNodeById(node) else {
                return Ok(foundation::WritingMode::kHorizontalTb);
            };
            if document.Node(index).Type() == dom::persistent_document::DOMNodeType::kElement {
                return Ok(document
                    .ResolvedStyleFor(index)
                    .and_then(|style| style.ComputedStyle())
                    .map_or(foundation::WritingMode::kHorizontalTb, |style| {
                        style.GetWritingMode()
                    }));
            }
        }
        // Text/Document nodes use their own native LayoutObject style. An
        // ancestor's CSSOM style is not a replacement for that source owner.
        match self
            .FrameSelection()
            .CompositionText(CompositionTextQuery::NodeWritingMode(node))
        {
            Some(CompositionTextResult::NodeWritingMode(mode)) => {
                Ok(mode.unwrap_or(foundation::WritingMode::kHorizontalTb))
            }
            _ => Err("Editor keyboard resident writing-mode geometry"),
        }
    }
    pub(super) fn EditingBeforeInputFromSource(
        &self,
        command: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
        input_type: NativeInputType,
        source: interaction_default::core::editing::commands::editor_command_table::EditorCommandSource,
    ) -> Result<bool, EditorCommandError> {
        if source == interaction_default::core::editing::commands::editor_command_table::EditorCommandSource::kDom {
            return Ok(true);
        }
        self.EditingBeforeInput(command, input_type)
    }
    fn EditingBeforeInput(
        &self,
        command: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
        input_type: NativeInputType,
    ) -> Result<bool, EditorCommandError> {
        if input_type == NativeInputType::kNone {
            return Ok(true);
        }
        use interaction_default::core::editing::commands::editor_command_table::EditingCommandType as C;
        let history_root =
            if foundation::RuntimeEnabledFeatures::UseUndoStepElementDispatchBeforeInputEnabled()
                && matches!(command, C::Undo | C::Redo)
            {
                let stack = self.editor.ime_typing.editor.GetUndoStack();
                let step = if command == C::Undo {
                    stack.LastUndoStep()
                } else {
                    stack.LastRedoStep()
                };
                step.and_then(|step| {
                    let owner = self.document.borrow();
                    foundation::editable::RootEditableElement(
                        &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
                        step.StartingSelection().0.anchor.node_id?,
                        Default::default(),
                    )
                })
            } else {
                None
            };
        let Some(target) = history_root
            .and_then(|root| self.targets.ResolveTarget(root))
            .or_else(|| {
                self.targets
                    .EventTargetNodeForDocument(self.state.borrow().focused_node_id)
            })
        else {
            return Ok(true);
        };
        let ranges = self.EditorCommandTargetRanges(command)?;
        let mut event = NativeInputEvent::CreateBeforeInputDataUTF16(
            input_type,
            None,
            EventIsComposing::kNotComposing,
            ranges.as_deref(),
            self.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        Ok(self
            .DispatchCanonicalEventTarget(&target, EventRef::new(event))
            .0
            == NativeDispatchEventResult::kNotCanceled)
    }
    pub(crate) fn EventClock(
        &self,
    ) -> &dyn interaction_dispatch::core::dom::events::event::EventClock {
        self.clock.as_ref()
    }
    fn MouseCoordinateContext(&self) -> MouseCoordinateContext {
        let window = self.targets.WindowView();
        MouseCoordinateContext {
            views: self.mouse_frame_geometry.borrow().as_ref().map(|frame| {
                Rc::new(PageMouseViewGeometry {
                    window,
                    window_document: self.targets.WindowDocument(),
                    frame: frame.clone(),
                }) as Rc<dyn MouseViewGeometry>
            }),
            relative: None,
        }
    }

    pub fn new(
        document: InteractionDocument,
        emit: InteractionDOMMutationEmitter,
        listeners: Rc<RefCell<Option<EventListenerDispatcher<'listener>>>>,
        listener_interest: Rc<RefCell<Option<EventListenerInterestRef>>>,
        mouse_frame_geometry: Rc<RefCell<Option<MouseFrameGeometryRef>>>,
        mouse_state: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        focus_navigation: Rc<
            RefCell<Option<Rc<super::page_focus_navigation::PageFocusNavigation>>>,
        >,
        state: Rc<RefCell<UserInteractionState>>,
        editor: Rc<PageEditingOwners>,
        selections: Rc<SelectionState>,
        submit_form: Option<
            Rc<
                dyn Fn(
                        Result<
                            Option<crate::input_type::PrepareFormEntryListResult>,
                            crate::input_type::FormDataAdmission,
                        >,
                    ) + 'listener,
            >,
        >,
        default_action_state: Rc<crate::input_type::DefaultActionState>,
        dispatch_state: Rc<RefCell<Option<Rc<PageDispatchState>>>>,
        editing_geometry: Option<super::page_editing_context::PreparedEditingGeometry<'frame>>,
        selection_controller_state: Rc<
            interaction_default::core::editing::selection_controller::SelectionControllerState,
        >,
        clock: Rc<dyn interaction_dispatch::core::dom::events::event::EventClock>,
        focus_services: Rc<super::page_focus_controller::DocumentFocusServices>,
    ) -> Self {
        let editing_geometry_revision =
            editing_geometry.map(|_| document.borrow().GetMeasurementGeometryRevision());
        let state_owner = PageDispatchState::ForDocument(
            &dispatch_state,
            document.clone(),
            listener_interest.clone(),
        );
        let targets = state_owner.targets.clone();
        let paths = state_owner.paths.clone();
        Self {
            input_dispatch_result: None,
            interaction_output: None,
            event_timing: None,
            editor_scroll: None,
            input_frame: None,
            script_invocation: None,
            input_method_state: None,
            clock,
            targets,
            paths,
            scoped_queue: state_owner.scoped_queue.clone(),
            document,
            emit,
            listeners,
            listener_interest,
            mouse_frame_geometry,
            mouse_state,
            focus_navigation,
            state,
            editor,
            selections,
            submit_form,
            default_action_state,
            focus_services,
            selection_controller_state,
            editing_geometry,
            editing_geometry_revision,
            keyboard_default: Rc::new(RefCell::new(None)),
            text_input_default: Rc::new(RefCell::new(None)),
            editing_command_event: Rc::new(RefCell::new(None)),
            middle_click_default: Rc::new(RefCell::new(None)),
        }
    }

    /// Installs Node::DefaultEventHandler's call-scoped EventHandler keyboard
    /// edge. The callback is shared by dispatcher clones created for nested
    /// events, but never survives this synchronous input transaction.
    // Source: node.cc:3406-3422; event_handler.cc:2468-2471.
    pub(crate) fn SetKeyboardDefaultHandler(
        &self,
        handler: Option<Rc<dyn Fn(&KeyboardEventRef, u64) + 'frame>>,
    ) {
        *self.keyboard_default.borrow_mut() = handler;
    }

    pub(super) fn KeyboardDefaultHandler(
        &self,
    ) -> Option<Rc<dyn Fn(&KeyboardEventRef, u64) + 'frame>> {
        self.keyboard_default.borrow().clone()
    }

    pub(crate) fn SetTextInputDefaultHandler(
        &self,
        handler: Option<Rc<dyn Fn(&EventRef) + 'frame>>,
    ) {
        *self.text_input_default.borrow_mut() = handler;
    }
    pub(crate) fn SetMiddleClickDefaultHandler(&self, handler: Rc<dyn Fn(u64) + 'frame>) {
        *self.middle_click_default.borrow_mut() = Some(handler);
    }
    pub(crate) fn MiddleClickDefaultHandler(&self) -> Option<Rc<dyn Fn(u64) + 'frame>> {
        self.middle_click_default.borrow().clone()
    }
    pub(crate) fn TextInputDefaultHandler(&self) -> Option<Rc<dyn Fn(&EventRef) + 'frame>> {
        self.text_input_default.borrow().clone()
    }

    pub(crate) fn ReplaceEditingCommandEvent(&self, event: Option<EventRef>) -> Option<EventRef> {
        self.editing_command_event.replace(event)
    }

    /// Source EventTarget::GetAttributeEventListener read against the original
    /// retained listener map, used by ReplacementFragment's preparation gate.
    pub(crate) fn HasAttributeEventListener(&self, node: u64, event_type: &str) -> bool {
        let document = self.document.borrow().GetDocument().RootHandle();
        self.listener_interest
            .borrow()
            .as_ref()
            .is_some_and(|interest| interest.HasAttributeEventListener(document, node, event_type))
    }

    pub(crate) fn KeyboardTargets(&self) -> Rc<DomEventTargets> {
        self.targets.clone()
    }

    pub(crate) fn KeyboardState(&self) -> Rc<RefCell<UserInteractionState>> {
        self.state.clone()
    }

    pub(crate) fn DispatchCanonicalKeyboardEvent(
        &self,
        target: &EventTargetRef,
        event: EventRef,
    ) -> (NativeDispatchEventResult, DispatchEventResult) {
        // KeyboardEventManager reaches this edge through Node::DispatchEvent.
        // EventTarget::DispatchEvent marks a browser-created event trusted
        // before listener dispatch; default actions are intentionally skipped
        // for an untrusted event. Keep that source lifecycle at the package
        // boundary instead of relying on each keyboard constructor to repeat
        // it.
        event.borrow_mut().BaseEventMut().SetTrusted(true);
        self.DispatchCanonicalEventTarget(target, event)
    }

    fn DispatchCanonicalEventTarget(
        &self,
        target: &EventTargetRef,
        event: EventRef,
    ) -> (NativeDispatchEventResult, DispatchEventResult) {
        let Some(id) = self.targets.NodeId(target) else {
            return (
                NativeDispatchEventResult::kNotCanceled,
                DispatchEventResult::default(),
            );
        };
        ports::DispatchCanonicalWithResult(self, event, id)
    }

    pub(crate) fn FocusKeyboardTarget(
        &self,
        target: &EventTargetRef,
        params: &interaction_default::core::dom::focus_params::FocusParams,
    ) {
        let Some(id) = self.targets.NodeId(target) else {
            return;
        };
        let nested = self.clone();
        let dispatch: SyntheticEventDispatcher<'_> = Rc::new(move |event, id| {
            nested.Dispatch(event, id);
        });
        FocusController::new(
            self.document.clone(),
            self.state.clone(),
            self.editor.clone(),
            dispatch,
            self.default_action_state.clone(),
            self.focus_services.clone(),
        )
        .WithFocusDispatch(Rc::new(self.clone()))
        .FocusWithParams(id, params);
    }

    /// Accessibility and other source callers enter the same native simulated
    /// click sequence. Missing native coordinate capabilities are reported to
    /// the caller; an element without a LayoutBox still dispatches normally.
    pub fn DispatchSimulatedClick(
        &self,
        target: u64,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    ) -> Result<
        (),
        interaction_dispatch::core::events::simulated_event_util::RequiredSimulatedEventInput,
    > {
        ports::DispatchSimulatedWithoutUnderlying(self, target, scope)
    }

    pub(crate) fn DispatchAccessKeyAction(
        &self,
        target: &EventTargetRef,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    ) -> Result<
        (),
        interaction_dispatch::core::events::simulated_event_util::RequiredSimulatedEventInput,
    > {
        match self.targets.NodeId(target) {
            Some(id) => self.DispatchSimulatedClick(id, scope),
            None => Ok(()),
        }
    }

    pub(crate) fn KeyboardHadEventChanged(&self, target: &EventTargetRef) {
        let Some(id) = self.targets.NodeId(target) else {
            return;
        };
        if !self
            .mouse_frame_geometry
            .borrow()
            .as_ref()
            .is_some_and(|geometry| geometry.HasLayoutObject(id))
        {
            return;
        }
        self.state.borrow_mut().focus_visible_node_id = Some(id);
        self.document
            .borrow_mut()
            .GetDocumentMut()
            .StyleStateMut()
            .impact
            .Merge(dom::style_state::StyleUpdateImpact {
                paint: true,
                ..Default::default()
            });
    }

    pub(crate) fn KeyboardFrameHasComposition(&self) -> Option<bool> {
        let focused = self.state.borrow().focused_node_id?;
        Some(self.editor.HasComposition(focused))
    }

    pub(crate) fn CompositionDispatchEvent(
        &self,
        target: &EventTargetRef,
        event_type: &'static str,
        data: &[u16],
        scoped: bool,
    ) {
        let capabilities = PageCompositionCapabilities {
            targets: self.targets.clone(),
        };
        let mut event = CompositionEvent::newUTF16(
            event_type,
            Some(self.targets.WindowView()),
            Rc::from(data),
            &capabilities,
            self.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        let event = EventRef::new(event);
        if scoped {
            let queue = self.ScopedQueueBinding();
            NativeEventDispatcher::DispatchScopedEventWithQueue(
                target,
                event,
                &self.paths,
                queue.as_ref(),
            );
        } else {
            self.DispatchCanonicalEventTarget(target, event);
        }
    }

    pub(crate) fn DragCommandOwners(
        &self,
    ) -> super::page_input_method_controller::DragCommandOwners {
        super::page_input_method_controller::DragCommandOwners {
            document: self.document.clone(),
            editor: self.editor.clone(),
            selections: self.selections.clone(),
            interaction: self.state.clone(),
            emit: self.emit.clone(),
            frame: self.focus_services.selection.clone(),
        }
    }
    pub(crate) fn CompositionBeforeInput(
        &self,
        target: &EventTargetRef,
        input_type: NativeInputType,
        data: &[u16],
        composing: bool,
    ) -> NativeDispatchEventResult {
        let mut event = NativeInputEvent::CreateBeforeInputUTF16(
            input_type,
            Rc::from(data),
            if composing {
                EventIsComposing::kIsComposing
            } else {
                EventIsComposing::kNotComposing
            },
            None,
            self.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        self.DispatchCanonicalEventTarget(target, EventRef::new(event))
            .0
    }

    // Source editing_utilities.cc:1750-1779.
    pub(crate) fn KeyboardBeforeInputInsertText(
        &self,
        target: Option<&EventTargetRef>,
        data: &[u16],
    ) -> Result<NativeDispatchEventResult, EditorCommandError> {
        let Some(target) = target else {
            return Ok(NativeDispatchEventResult::kNotCanceled);
        };
        let ranges = self.TextInputTargetRanges(target)?;
        let mut event = NativeInputEvent::CreateBeforeInputUTF16(
            NativeInputType::kInsertText,
            Rc::from(data),
            EventIsComposing::kNotComposing,
            ranges.as_deref(),
            self.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        Ok(self
            .DispatchCanonicalEventTarget(target, EventRef::new(event))
            .0)
    }

    pub(crate) fn CompositionEventQueueScope(
        &self,
    ) -> Box<
        dyn interaction_default::core::editing::ime::input_method_controller::ScopedCompositionEvents
            + '_,
    >{
        Box::new(PageCompositionEventQueueScope {
            _scope: self.ScopedQueueBinding().Scope(),
        })
    }

    pub(crate) fn CompositionEventsAreScoped(&self) -> bool {
        self.scoped_queue.ShouldQueueEvents()
    }

    pub(crate) fn CompositionDispatchInput(
        &self,
        target: u64,
        data: &[u16],
        input_type: NativeInputType,
        composing: bool,
    ) -> DispatchEventResult {
        self.CompositionDispatchInputWithTransfer(target, data, input_type, composing, None)
    }
    pub(crate) fn CompositionDispatchInputWithTransfer(
        &self,
        target: u64,
        data: &[u16],
        input_type: NativeInputType,
        composing: bool,
        transfer: Option<Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>>,
    ) -> DispatchEventResult {
        let mut event = if let Some(transfer) = transfer {
            NativeInputEvent::CreateInputWithDataTransfer(
                input_type,
                Some(transfer),
                EventIsComposing::kNotComposing,
                None,
                self.clock.as_ref(),
            )
        } else {
            NativeInputEvent::CreateInputUTF16(
                input_type,
                Rc::from(data),
                if composing {
                    EventIsComposing::kIsComposing
                } else {
                    EventIsComposing::kNotComposing
                },
                None,
                self.clock.as_ref(),
            )
        };
        event.BaseEventMut().SetTrusted(true);
        let Some(target_node) = self.targets.ResolveTarget(target) else {
            return DispatchEventResult::default();
        };
        let queue = self.ScopedQueueBinding();
        NativeEventDispatcher::DispatchScopedEventWithQueue(
            &target_node,
            EventRef::new(event),
            &self.paths,
            queue.as_ref(),
        );
        DispatchEventResult {
            target_node_id: Some(target),
            ..Default::default()
        }
    }

    pub(crate) fn CompositionDispatchTextInput(
        &self,
        target: &EventTargetRef,
        data: &[u16],
        composing: bool,
    ) -> NativeDispatchEventResult {
        let mut event = TextEvent::new(
            Some(self.targets.WindowView()),
            data,
            if composing {
                TextEventInputType::Composition
            } else {
                TextEventInputType::Keyboard
            },
            self.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        self.DispatchCanonicalEventTarget(target, EventRef::new(event))
            .0
    }
    // Source: typing_command.cc:100-113. This event's default is owned by
    // the initiating TypingCommand; Editor leaves IncrementalInsertion alone.
    pub(crate) fn TypingDispatchIncrementalTextInput(
        &self,
        target: &EventTargetRef,
        data: &[u16],
    ) -> NativeDispatchEventResult {
        let mut event = TextEvent::new(
            Some(self.targets.WindowView()),
            data,
            TextEventInputType::IncrementalInsertion,
            self.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        self.DispatchCanonicalEventTarget(target, EventRef::new(event))
            .0
    }

    pub(crate) fn EditorCommandDispatchTextInput(
        &self,
        data: &[u16],
        line_break: bool,
    ) -> Option<NativeDispatchEventResult> {
        let underlying = self.editing_command_event.borrow().clone();
        let target = if let Some(event) = underlying.as_ref() {
            event.borrow().BaseEvent().RawTarget()
        } else {
            self.targets
                .EventTargetNodeForDocument(self.state.borrow().focused_node_id)
        }?;
        let mut event = TextEvent::new(
            Some(self.targets.WindowView()),
            data,
            if line_break {
                TextEventInputType::LineBreak
            } else {
                TextEventInputType::Keyboard
            },
            self.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        event.BaseEventMut().SetUnderlyingEvent(underlying);
        Some(
            self.DispatchCanonicalEventTarget(&target, EventRef::new(event))
                .0,
        )
    }

    pub(super) fn Context<'scope>(&'scope self) -> InputTypeContext<'scope> {
        let nested = self.clone();
        let dispatch: SyntheticEventDispatcher<'scope> = Rc::new(move |event, id| {
            nested.Dispatch(event, id);
        });
        let focus = FocusController::new(
            self.document.clone(),
            self.state.clone(),
            self.editor.clone(),
            dispatch.clone(),
            self.default_action_state.clone(),
            self.focus_services.clone(),
        )
        .WithFocusDispatch(Rc::new(self.clone()));
        let activate_dispatcher = self.clone();
        let simulated_dispatcher = self.clone();
        let scoped_dispatcher = self.clone();
        let queue_dispatcher = self.clone();
        let formdata_dispatcher = self.clone();
        let dialog_dispatcher = self.clone();
        InputTypeContext {
            anchor_navigation: self
                .input_frame
                .as_ref()
                .and_then(|frame| frame.anchor_navigation.borrow().clone()),
            event_targets: self.targets.clone(),
            script_invocation: self.script_invocation,
            close_dialog: Rc::new(move |dialog, result| {
                pointer_dom_effects::CloseDialogForForm(&dialog_dispatcher, dialog, &result);
            }),
            dispatch_form_data: Rc::new(move |target, data| {
                ports::DispatchFormData(&formdata_dispatcher, target, data);
            }),
            dispatch_dom_activate: Rc::new(move |target, underlying| {
                ports::DispatchDOMActivate(&activate_dispatcher, target, underlying)
            }),
            dispatch_simulated_click: Rc::new(move |target, underlying| {
                // Control defaults carry their triggering timestamp and do
                // not require accessibility geometry.
                let _ = ports::DispatchSimulated(&simulated_dispatcher, target, underlying);
            }),
            editing_geometry: self.editing_geometry,
            editing_geometry_revision: self.editing_geometry_revision,
            selection_controller_state: self.selection_controller_state.clone(),
            begin_event_queue_scope: Rc::new(move || {
                Box::new(queue_dispatcher.ScopedQueueBinding().Scope())
            }),
            dispatch_scoped_synthetic: Rc::new(move |event, target| {
                ports::DispatchScoped(&scoped_dispatcher, event, target);
            }),
            default_action_state: self.default_action_state.clone(),
            submit_form: self.submit_form.clone(),
            document: self.document.clone(),
            emit_mutation: self.emit.clone(),
            interaction_state: self.state.clone(),
            editor: self.editor.clone(),
            focus_controller: focus,
            dispatch_synthetic: dispatch,
        }
    }

    fn HasEventListeners(&self, target: &EventTargetRef, event_type: &str) -> bool {
        let Some(document) = self.targets.OwnerDocumentHandle(target) else {
            return false;
        };
        let Some(target) = self.targets.NodeId(target) else {
            return false;
        };
        self.listener_interest
            .borrow()
            .as_ref()
            .is_some_and(|interest| interest.HasEventListeners(document, target, event_type))
    }
    pub(super) fn FormStateEventQueueScope<'scope>(
        &self,
    ) -> Box<dyn interaction_default::core::html::forms::input_type::EventQueueScope + 'scope>
    where
        'listener: 'scope,
        'frame: 'scope,
    {
        Box::new(self.ScopedQueueBinding().Scope())
    }
    fn ScopedQueueBinding<'scope>(&self) -> Rc<ScopedEventQueueBinding<'scope>>
    where
        'listener: 'scope,
        'frame: 'scope,
    {
        let dispatcher = self.clone();
        ScopedEventQueueBinding::new(
            self.scoped_queue.clone(),
            Rc::new(move |event| {
                ports::DispatchQueued(&dispatcher, event);
            }),
        )
    }
    pub fn PrepareFormEntryList(
        &self,
        form: u64,
        submitter: Option<u64>,
        encoding: &str,
    ) -> Result<
        Option<crate::input_type::PrepareFormEntryListResult>,
        crate::input_type::FormDataAdmission,
    > {
        crate::input_type::PrepareFormEntryList(&self.Context(), form, submitter, encoding)
    }
    pub fn ResumeFormEntryList(
        &self,
        token: crate::input_type::FormSubmissionToken,
    ) -> Result<
        Option<crate::input_type::PrepareFormEntryListResult>,
        crate::input_type::FormDataAdmission,
    > {
        crate::input_type::ResumeFormEntryList(&self.Context(), token)
    }
    pub fn FinishEntryList(
        &self,
        pending: crate::input_type::PendingFormEntryList,
    ) -> Result<
        Rc<interaction_dispatch::core::fileapi::form_data::FormData>,
        crate::input_type::FormDataAdmission,
    > {
        crate::input_type::FinishEntryList(&self.Context(), pending)
    }
    pub fn ConstructFormEntryList(
        &self,
        form: u64,
        encoding: &str,
    ) -> Result<
        Option<Rc<interaction_dispatch::core::fileapi::form_data_object::FormDataObject>>,
        crate::input_type::FormDataAdmission,
    > {
        crate::input_type::ConstructFormEntryList(&self.Context(), form, encoding)
    }
    pub fn SubmitForm(
        &self,
        form: u64,
        submitter: Option<u64>,
        dispatch_event: bool,
    ) -> Result<(), crate::input_type::RequestSubmitError> {
        if dispatch_event {
            crate::input_type::RequestSubmit(&self.Context(), form, submitter)
        } else {
            crate::input_type::SubmitForm(&self.Context(), form, submitter, false);
            Ok(())
        }
    }
    pub fn Dispatch(&self, event: &mut Event, target: u64) -> DispatchEventResult {
        ports::Dispatch(self, event, target)
    }

    pub fn SetElementUnderMouseAndDispatchBoundaryEvents(
        &self,
        pointer: &crate::core::input::pointer_event_manager::MousePointerEventManager,
        mouse: &Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        event: &WebMouseEvent,
        entered: Option<u64>,
    ) {
        let entered = entered.and_then(|id| self.targets.ResolveTarget(id));
        let boundary_dom = PageBoundaryDOM {
            document: &self.document,
            targets: &self.targets,
            listener_interest: &self.listener_interest,
        };
        let dispatch = PageMouseBoundaryDispatch {
            dispatcher: self,
            coordinates: self.MouseCoordinateContext(),
            result: Cell::new(None),
        };
        let document_owner = self.document.borrow().GetDocument().RootHandle();
        pointer.SendMousePointerBoundaryEvents(&dispatch, &boundary_dom, entered.clone(), event);
        crate::core::input::mouse_event_manager::MouseEventManager::WithState(mouse.clone())
            .SetElementUnderMouseWithDispatch(
                &boundary_dom,
                &dispatch,
                &|target| self.targets.OwnerDocumentHandle(target) == Some(document_owner),
                foundation::RuntimeEnabledFeatures::BoundaryEventDispatchTracksNodeRemovalEnabled(),
                entered.as_ref(),
                event,
            );
    }

    /// Runs Chromium's retained wheel target/phase transaction against the
    /// current resident fragment lease. The manager owns target latching;
    /// this adapter owns no wheel target or copied hit-test tree.
    pub(crate) fn DispatchNativeWheelEvent<M: ?Sized>(
        &self,
        manager: &MouseWheelEventManager<M>,
        event: &WebMouseWheelEvent,
        fragments: ResidentFragmentSource<'_>,
    ) -> Result<DispatchEventResult, interaction_hit_test::resident::ResidentCursorGeometryError>
    {
        self.DispatchNativeWheelEventWithResult(manager, event, fragments)
            .map(|result| result.0)
    }

    pub(crate) fn DispatchNativeWheelEventWithResult<M: ?Sized>(
        &self,
        manager: &MouseWheelEventManager<M>,
        event: &WebMouseWheelEvent,
        fragments: ResidentFragmentSource<'_>,
    ) -> Result<
        (
            DispatchEventResult,
            interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult,
        ),
        interaction_hit_test::resident::ResidentCursorGeometryError,
    > {
        let frame_geometry = self.mouse_frame_geometry.borrow().clone();
        let query = PageWheelTargetQuery {
            document: PageWheelDocumentQuery {
                dispatcher: self,
                fragments,
                error: Cell::new(None),
            },
            view: PageWheelFrameView {
                frame: frame_geometry.clone(),
            },
            nodes: PageWheelNodeQueries {
                dispatcher: self,
                frame_geometry,
            },
            result: Cell::new(None),
        };
        let input_result = manager.HandleWheelEventWithTargetQuery(event, &query);
        if let Some(error) = query.document.error.get() {
            return Err(error);
        }
        Ok((query.result.get().unwrap_or_default(), input_result))
    }

    pub fn DispatchNativeMouseEvent(
        &self,
        mouse: &Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        target: u64,
        event_type: &str,
        event: &WebMouseEvent,
    ) -> DispatchEventResult {
        let Some(target_ref) = self.targets.ResolveTarget(target) else {
            return DispatchEventResult::default();
        };
        let dispatch = PageMouseBoundaryDispatch {
            dispatcher: self,
            coordinates: self.MouseCoordinateContext(),
            result: Cell::new(None),
        };
        let manager =
            crate::core::input::mouse_event_manager::MouseEventManager::WithState(mouse.clone());
        if event_type == "contextmenu" {
            manager.DispatchContextMenuEventWithDispatch(
                &dispatch,
                Some(&target_ref),
                event,
                None,
                event.pointer_properties.id,
                crate::core::input::mouse_event_manager::PointerTypeName(
                    event.pointer_properties.pointer_type,
                ),
            );
        } else {
            manager.DispatchMouseEventWithDispatch(
                &dispatch,
                Some(&target_ref),
                event_type,
                event,
                None,
                None,
                false,
            );
        }
        dispatch.result.get().unwrap_or(DispatchEventResult {
            target_node_id: Some(target),
            ..Default::default()
        })
    }

    /// Runs Chromium's retained PointerEventManager mouse down/up transaction:
    /// pointer event, compatibility mouse event, capture release, then click.
    pub fn DispatchNativeMousePointerEvent(
        &self,
        pointer: &crate::core::input::pointer_event_manager::MousePointerEventManager,
        mouse: &Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        target: u64,
        event_type: interaction_input::public::common::input::web_input_event::Type,
        event: &WebMouseEvent,
    ) -> DispatchEventResult {
        let Some(target_ref) = self.targets.ResolveTarget(target) else {
            return DispatchEventResult::default();
        };
        let dispatch = PageMouseBoundaryDispatch {
            dispatcher: self,
            coordinates: self.MouseCoordinateContext(),
            result: Cell::new(None),
        };
        if event_type
            == interaction_input::public::common::input::web_input_event::Type::kPointerDown
        {
            mouse.SetClickCount(event.click_count);
            mouse.SetMouseDownElement(Some(&target_ref));
        }
        let boundary_dom = PageBoundaryDOM {
            document: &self.document,
            targets: &self.targets,
            listener_interest: &self.listener_interest,
        };
        pointer.SendMousePointerEvent(
            &dispatch,
            &boundary_dom,
            mouse,
            target_ref,
            event_type,
            event,
        );
        dispatch.result.get().unwrap_or(DispatchEventResult {
            target_node_id: Some(target),
            ..Default::default()
        })
    }

    /// Source: event_handler.cc:864-873; mouse_event_manager.cc:541-639.
    /// The DOM mousedown has already returned uncanceled and PointerEventManager
    /// has installed the effective element-under-mouse on the same retained
    /// MouseEventManager state before this transaction runs.
    pub(crate) fn HandleNativeMouseFocus(
        &self,
        mouse: &Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        event: &WebMouseEvent,
        over_scrollbar: bool,
        frame_view_scrollbar: bool,
    ) -> interaction_dispatch::core::input::event_handling_util::WebInputEventResult {
        let nested = self.clone();
        let dispatch: SyntheticEventDispatcher<'_> = Rc::new(move |event, id| {
            nested.Dispatch(event, id);
        });
        let frame = self.mouse_frame_geometry.borrow().clone();
        let focus_dispatcher = self.clone();
        let ports = super::page_mouse_focus::PageMouseFocus::new(
            self.document.clone(),
            self.targets.clone(),
            self.state.clone(),
            self.editor.clone(),
            self.selections.clone(),
            dispatch,
            self.default_action_state.clone(),
            frame,
            self.focus_services.clone(),
            Rc::new(move || {
                focus_dispatcher
                    .FocusNativeDocumentView()
                    .unwrap_or_else(|error| std::panic::panic_any(error))
            }),
            Rc::new(self.clone()),
        );
        let capabilities = self.targets.InputDeviceCapabilities(event.FromTouch());
        super::page_mouse_focus::HandleMouseFocus(
            mouse,
            &ports,
            &super::page_mouse_focus::PageMouseFocusHit {
                over_scrollbar,
                frame_view_scrollbar,
            },
            Some(&capabilities),
        )
    }
}

impl interaction_default::core::loader::frame_navigation_lifecycle::FrameLifecycleEventDispatcher
    for EventDispatcher<'_, '_>
{
    fn DispatchReadyStateChanged(&self) {
        let target = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            document.Node(document.Root()).Id()
        };
        let mut event = MakeSyntheticEvent(EventType::kReadyStateChange, target);
        event.bubbles = false;
        event.cancelable = false;
        event.composed = false;
        (self.Context().dispatch_synthetic)(&mut event, target);
    }
}
