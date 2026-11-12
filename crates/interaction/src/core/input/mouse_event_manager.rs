// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/mouse_event_manager.h:30-268
// Source: third_party/blink/renderer/core/input/mouse_event_manager.cc:58-777,811-1301
// Scope: all mouse-manager definitions. Includes/GC Trace (cc:159-165)
// are replaced by Rust imports and retained Rc owners. Gesture drag synthesis
// preserves the source synthetic-event then resident-hit-test ordering.
//
// Ports borrow the actual DOM, SelectionController, DragController, and resident
// Layout owners. They are not a retained input tree or a facade adapter. Every
// dispatch is synchronous and uses the same EventRef handed to listeners.
// Cells are borrowed only for individual state reads/writes, never across a
// callback: removal, navigation, and nested mouse input can reenter this owner.
//
// Lifecycle calls at cc:545,690,719-722,759-762,886-889,909-910,1066 belong to
// the embedding. Entry to focus/selection/password queries below requires the
// corresponding prepared canonical geometry, including after synchronous
// focus/selection/dragstart dispatch. This module cannot drive rendering.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use interaction_dispatch::core::dom::events::event::{
    EventClock, EventRef, EventTargetKind, EventTargetRef,
};
use interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult;
use interaction_dispatch::core::events::mouse_event::{
    MouseCoordinateContext, MouseEvent, MouseEventInit, SyntheticEventType,
};
use interaction_dispatch::core::events::pointer_event::{PointerEvent, PointerEventInit};
use interaction_dispatch::core::events::ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef};
use interaction_dispatch::core::events::ui_event_with_key_state::UIEventWithKeyState;
use interaction_dispatch::core::input::boundary_event_dispatcher::{
    BoundaryEventDOM, BoundaryEventDispatch, BoundaryEventDispatcher,
};
use interaction_dispatch::core::input::event_handling_util::{
    ToWebInputEventResult, WebInputEventResult,
};
use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::pointer_id::PointerId;
use interaction_input::public::common::input::web_input_event::{Type, WebInputEvent};
use interaction_input::public::common::input::web_mouse_event::{MenuSourceType, WebMouseEvent};
use interaction_input::public::common::input::web_pointer_properties::{
    Button, PointF, PointerType,
};

// Source: mouse_event_manager.h:30-39
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragHandlingResult {
    kNotHandled,
    kHandledDragNotStarted,
    kHandledDragStarted,
}

// Source: mouse_event_manager.h:196-207
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DragAndDropToolType {
    kUnknown = 0,
    kMouse = 1,
    kFinger = 2,
    kStylusViaGesture = 3,
    kStylusViaButton = 4,
}
// Source: mouse_event_manager.h:205
impl DragAndDropToolType {
    pub const kMaxValue: Self = Self::kStylusViaButton;
}

// Source: mouse_event_manager.cc:222-239,253-265,356-359,457-525
// These queries retain/borrow the original Node owner. CommonAncestorForClick
// invokes DOM CommonAncestor with event_handling_util::ParentForClickEvent;
// it must not use DOM parentNode or a reconstructed interaction tree.
pub trait MouseNodeRelations {
    fn IsConnected(&self, node: &EventTargetRef) -> bool;
    fn SameDocument(&self, a: &EventTargetRef, b: &EventTargetRef) -> bool;
    fn InFrameDocument(&self, node: &EventTargetRef) -> bool;
    fn HasEventListeners(&self, node: &EventTargetRef, event_type: &str) -> bool;
    fn CommonAncestorForClick(
        &self,
        a: &EventTargetRef,
        b: &EventTargetRef,
    ) -> Option<EventTargetRef>;
    fn ParentNode(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn ParentOrShadowHostElement(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn FlatTreeParent(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn IsElement(&self, node: &EventTargetRef) -> bool;
    fn IsShadowIncludingInclusiveAncestorOf(
        &self,
        ancestor: &EventTargetRef,
        node: &EventTargetRef,
    ) -> bool;
}

/// The exact DOM relation subset consumed by NodeWillBeRemoved. Keeping this
/// narrow lets Document notify the retained source mouse owner synchronously
/// while its canonical tree is still linked, without assembling drag/focus or
/// frame services that the removal algorithm never reads.
pub trait MouseRemovalNodeRelations {
    fn ParentNode(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn ParentOrShadowHostElement(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn IsElement(&self, node: &EventTargetRef) -> bool;
    fn IsShadowIncludingInclusiveAncestorOf(
        &self,
        ancestor: &EventTargetRef,
        node: &EventTargetRef,
    ) -> bool;
}

impl<T: MouseNodeRelations + ?Sized> MouseRemovalNodeRelations for T {
    fn ParentNode(&self, node: &EventTargetRef) -> Option<EventTargetRef> {
        MouseNodeRelations::ParentNode(self, node)
    }
    fn ParentOrShadowHostElement(&self, node: &EventTargetRef) -> Option<EventTargetRef> {
        MouseNodeRelations::ParentOrShadowHostElement(self, node)
    }
    fn IsElement(&self, node: &EventTargetRef) -> bool {
        MouseNodeRelations::IsElement(self, node)
    }
    fn IsShadowIncludingInclusiveAncestorOf(
        &self,
        ancestor: &EventTargetRef,
        node: &EventTargetRef,
    ) -> bool {
        MouseNodeRelations::IsShadowIncludingInclusiveAncestorOf(self, ancestor, node)
    }
}

// Source: mouse_event_manager.cc:90-111,267-288,1130-1163
// CreateDragEvent creates the real translated DragEvent, retaining exactly the
// supplied DataTransfer owner; it may not serialize or duplicate it. Dispatch
// routes the same EventRef through the canonical DOM dispatcher/default path.
pub trait MouseEventDispatchPort {
    fn DocumentWindow(&self, node: &EventTargetRef) -> Option<AbstractViewRef>;
    fn FrameDocumentWindow(&self) -> Option<AbstractViewRef>;
    fn CoordinateContext(&self) -> MouseCoordinateContext;
    fn SourceCapabilities(
        &self,
        window: &AbstractViewRef,
        from_touch: bool,
    ) -> InputDeviceCapabilitiesRef;
    fn DispatchEvent(&self, target: &EventTargetRef, event: &EventRef) -> DispatchEventResult;
    fn CreateDragEvent(
        &self,
        event_type: &str,
        initializer: &MouseEventInit,
        data_transfer: Option<&MouseDataTransferRef>,
        timestamp: TimeTicks,
        synthetic: SyntheticEventType,
    ) -> EventRef;
}

/// Exact services consumed by the ordinary MouseEvent construction/dispatch
/// branch, including boundary events. Pointer click light-dismiss, drag event
/// creation and frame services are not read by this branch.
// Source: mouse_event_manager.cc:77-112,222-229,274-295
pub trait MouseBoundaryEventDispatchPort {
    fn DocumentWindow(&self, node: &EventTargetRef) -> Option<AbstractViewRef>;
    fn CoordinateContext(&self) -> MouseCoordinateContext;
    fn SourceCapabilities(
        &self,
        window: &AbstractViewRef,
        from_touch: bool,
    ) -> InputDeviceCapabilitiesRef;
    fn HasEventListeners(&self, target: &EventTargetRef, event_type: &str) -> bool;
    fn DispatchEvent(&self, target: &EventTargetRef, event: &EventRef) -> DispatchEventResult;
}

// Source: mouse_event_manager.cc:248-265
// PointerTarget's existing owner carries dialog/popover-specific details. The
// node projection is read live, since preceding synchronous events can detach
// it. This is not a second PointerEventFactory target store.
pub trait MousePointerTarget {
    fn Node(&self) -> Option<EventTargetRef>;
    // Source: core/events/pointer_event_factory.h:117-122
    // The dialog algorithm consumes the original double-precision client
    // coordinates alongside the live weak node; neither is reconstructed.
    fn ClientPosition(&self) -> (f64, f64);
}
pub trait MouseLightDismissPort {
    fn HandlePopoverLightDismissForClick(&self, down: &EventTargetRef, up: &EventTargetRef);
    fn HandleDialogLightDismissForClick(
        &self,
        down: &dyn MousePointerTarget,
        up: &dyn MousePointerTarget,
    );
}

/// The exact retained DOM/dispatch services consumed by Chromium's click
/// branch. This keeps the production pointer-release path on the same
/// MouseEventManager state without assembling drag, focus or selection ports.
// Source: mouse_event_manager.cc:248-289,327-374
pub trait MouseClickDispatchPort: MouseBoundaryEventDispatchPort {
    fn IsConnected(&self, node: &EventTargetRef) -> bool;
    fn SameDocument(&self, a: &EventTargetRef, b: &EventTargetRef) -> bool;
    fn CommonAncestorForClick(
        &self,
        a: &EventTargetRef,
        b: &EventTargetRef,
    ) -> Option<EventTargetRef>;
    fn LightDismissFromClickEnabled(&self) -> bool;
    fn HandlePopoverLightDismissForClick(&self, down: &EventTargetRef, up: &EventTargetRef);
    fn HandleDialogLightDismissForClick(
        &self,
        down: &dyn MousePointerTarget,
        up: &dyn MousePointerTarget,
    );
}

// Source: mouse_event_manager.cc:388-429,648-650,699-707,811-815,1234-1239,1260-1261
// Geometry is a projection of the live frame view, not another frame model.
// HandleSyntheticMouseMove enters EventHandler with transformed event and empty
// coalesced/predicted lists, exactly as cc:427-429.
pub trait MouseFrameViewPort {
    fn HasView(&self) -> bool;
    fn HasPage(&self) -> bool;
    fn IsLocalRoot(&self) -> bool;
    fn IsPageActive(&self) -> bool;
    fn IsCursorVisible(&self) -> bool;
    fn IsPointerLocked(&self) -> bool;
    fn CurrentModifierState(&self) -> i32;
    fn ConvertFromRootFrame(&self, point: PointF) -> PointF;
    // gfx::Point overload: preserve the view's integer conversion/rounding.
    // Both argument and returned projection contain integral CSS coordinates.
    fn ConvertIntPointFromRootFrame(&self, point: PointF) -> PointF;
    fn ViewportToFrame(&self, point: PointF) -> PointF;
    fn HandleSyntheticMouseMove(&self, event: &WebMouseEvent);
    fn IsSvgDocumentWithZoomAndPan(&self) -> bool;
    fn StartSvgPan(&self, point: PointF);
    fn UpdateSvgPan(&self, point: PointF);
}

// Source: mouse_event_manager.cc:541-639,727,811-815
// The focus controller owns the focused node and synchronous focus events.
// Prepared focusability is read from the canonical style/layout owner.
pub trait MouseFocusPort {
    fn IsMouseFocusable(&self, element: &EventTargetRef) -> bool;
    fn IsFocusedElementInDocument(&self, element: &EventTargetRef) -> bool;
    fn IsShadowHostWithDelegatesFocus(&self, element: &EventTargetRef) -> bool;
    fn FlatTreeParentElement(&self, element: &EventTargetRef) -> Option<EventTargetRef>;
    fn IsScrollControlPseudoElement(&self, element: &EventTargetRef) -> bool;
    fn IsNodeFullyContainedInSelection(&self, element: &EventTargetRef) -> bool;
    fn IsDescendantOfFocusedElement(&self, element: &EventTargetRef) -> bool;
    fn GetFocusableArea(&self, element: &EventTargetRef) -> Option<EventTargetRef>;
    fn FocusDelegatedTargetWithMouseAndResetSelection(&self, element: &EventTargetRef);
    fn IsTextControl(&self, element: &EventTargetRef) -> bool;
    fn SetFocusedElementWithMouse(
        &self,
        element: Option<&EventTargetRef>,
        source_capabilities: Option<&InputDeviceCapabilitiesRef>,
    ) -> bool;
    fn FocusDocumentView(&self);
    fn SetSequentialFocusNavigationStartingPoint(&self, node: Option<&EventTargetRef>);
}

// Source: mouse_event_manager.cc:541-550,684-747,750-775,818-921
// H is the existing EventWithHitTestResults owner. Implementations borrow it;
// they do not construct another event, hit result, Node, or fragment tree.
pub trait MouseHitTest {
    fn Event(&self) -> &WebMouseEvent;
    fn InnerNode(&self) -> Option<EventTargetRef>;
    fn HasScrollbar(&self) -> bool;
    fn ScrollbarIsFrameViewScrollbar(&self) -> bool;
    fn IsSelectionOverLink(&self) -> bool;
    fn IsExtendingSelection(&self) -> bool;
}

// Source: mouse_event_manager.cc:541-639 takes a HitTestResult rather than a
// MouseEventWithHitTestResults. Focus only reads these scrollbar facts.
pub trait MouseFocusHitTest {
    fn HasScrollbar(&self) -> bool;
    fn ScrollbarIsFrameViewScrollbar(&self) -> bool;
}

// Source: mouse_event_manager.cc:541-639.  HandleMouseFocus only consumes
// this narrow slice of EventHandler state.  Keeping it separate lets the Page
// production adapter call the translated transaction without fabricating the
// drag, autoscroll, boundary-dispatch, or clock ports used by other methods.
pub trait MouseFocusPagePort {
    fn HasPage(&self) -> bool;
}
pub trait MouseFocusSelectionPort {
    fn IsRange(&self) -> bool;
    fn Clear(&self);
}
pub trait MouseFocusFeatures {
    fn PseudoElementsFocusableEnabled(&self) -> bool;
}
pub struct MouseFocusPorts<'a> {
    pub page: &'a dyn MouseFocusPagePort,
    pub focus: &'a dyn MouseFocusPort,
    pub selection: &'a dyn MouseFocusSelectionPort,
    pub features: &'a dyn MouseFocusFeatures,
}

impl<T: MouseHitTest + ?Sized> MouseFocusHitTest for T {
    fn HasScrollbar(&self) -> bool {
        MouseHitTest::HasScrollbar(self)
    }
    fn ScrollbarIsFrameViewScrollbar(&self) -> bool {
        MouseHitTest::ScrollbarIsFrameViewScrollbar(self)
    }
}

pub trait MouseSelectionPort<H: MouseHitTest + ?Sized> {
    fn SetMouseDownMayStartSelect(&self, value: bool);
    fn MouseDownMayStartSelect(&self) -> bool;
    fn IsRange(&self) -> bool;
    fn Clear(&self);
    fn HandleMousePressEvent(&self, event: &H) -> bool;
    fn HandleMouseReleaseEvent(&self, event: &H, drag_start_in_root: PointF) -> bool;
    fn HandleMouseDraggedEvent(
        &self,
        event: &H,
        mouse_down: PointF,
        last_known_in_root: PointF,
    ) -> WebInputEventResult;
    fn UpdateSelectionForMouseDrag(&self, drag_start_in_root: PointF, last_known_in_root: PointF);
    fn SelectedHtmlForClipboardIsEmpty(&self) -> bool;
    fn SelectionStartsInPasswordField(&self) -> bool;
}

/// The press/release source transaction does not read scrollbar focus facts.
/// Keep its event projection separate from the broader mouse/focus hit port.
pub trait MouseDefaultHitTest {
    fn DefaultEvent(&self) -> &WebMouseEvent;
    fn DefaultInnerNode(&self) -> Option<EventTargetRef>;
    fn DefaultIsSelectionOverLink(&self) -> bool;
    fn DefaultIsExtendingSelection(&self) -> bool;
}
impl<T: MouseHitTest + ?Sized> MouseDefaultHitTest for T {
    fn DefaultEvent(&self) -> &WebMouseEvent {
        MouseHitTest::Event(self)
    }
    fn DefaultInnerNode(&self) -> Option<EventTargetRef> {
        MouseHitTest::InnerNode(self)
    }
    fn DefaultIsSelectionOverLink(&self) -> bool {
        MouseHitTest::IsSelectionOverLink(self)
    }
    fn DefaultIsExtendingSelection(&self) -> bool {
        MouseHitTest::IsExtendingSelection(self)
    }
}
/// Exact call-scoped leaves consumed by press/release defaults. Geometry and
/// selection are read live again after every synchronous callback.
// Source: mouse_event_manager.cc:684-769,1205-1230.
pub trait MousePressDefaultPort<H: MouseDefaultHitTest + ?Sized> {
    fn ResetDragSource(&self);
    fn IsSvgDocumentWithZoomAndPan(&self) -> bool;
    fn ConvertFromRootFrame(&self, point: PointF) -> PointF;
    fn StartSvgPan(&self, point: PointF);
    fn FocusDocumentView(&self);
    fn SetSequentialFocusNavigationStartingPoint(&self, node: Option<&EventTargetRef>);
    fn HandleSelectionMousePress(&self, event: &H) -> bool;
    fn MouseDownMayStartSelect(&self) -> bool;
    fn HasUserScrollableLayoutBox(&self, node: &EventTargetRef) -> bool;
}
pub trait MouseReleaseDefaultPort<H: MouseDefaultHitTest + ?Sized> {
    fn StopSelectionAutoscrollIfNeeded(&self);
    fn HandleSelectionMouseRelease(&self, event: &H, drag_start: PointF) -> bool;
}

// Source: mouse_event_manager.cc:818-922. Retained layout identities are dropped
// before lifecycle/selection callbacks and looked up again at the source sites.
pub trait MouseDraggedDefaultPort<H: MouseDefaultHitTest + ?Sized> {
    fn BarrelButtonForDragEnabled(&self) -> bool;
    fn HandleDrag(
        &self,
        manager: &MouseEventManager,
        event: &H,
        initiator: DragAndDropToolType,
    ) -> DragHandlingResult;
    fn FlatTreeParent(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn GetLayoutObject(&self, node: &EventTargetRef) -> Option<MouseDragLayoutRef>;
    fn IsMenuListSelect(&self, node: &EventTargetRef) -> bool;
    fn PrepareForMouseDrag(&self);
    fn HandleSelectionMouseDragged(
        &self,
        event: &H,
        mouse_down: PointF,
        last_known_root: PointF,
    ) -> WebInputEventResult;
    fn MiddleClickAutoscrollInProgress(&self) -> bool;
    fn SelectedHtmlForClipboardIsEmpty(&self) -> bool;
    fn HasAutoscrollController(&self) -> bool;
    fn PrepareForSelectionAutoscroll(&self, layout: &MouseDragLayoutRef);
    fn StartAutoscrollForSelection(&self, layout: &MouseDragLayoutRef);
}
pub trait MouseDragLifecyclePort {
    fn PrepareForMouseDrag(&self);
    fn PrepareForSelectionAutoscroll(&self, layout: &MouseDragLayoutRef);
}

// Source: mouse_event_manager.cc:743-744,866-881,901-917
// This retained projection denotes the very same resident LayoutObject. It is
// discarded before dispatch and reread afterward at the source's lookup sites.
pub trait MouseDragLayout {
    /// Same resident layout owner consumed by Chromium's selection autoscroll.
    /// This projection carries identity, never a copied layout input tree.
    fn AutoscrollLayout(
        &self,
    ) -> Rc<dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout>;
}
pub type MouseDragLayoutRef = Rc<dyn MouseDragLayout>;
pub trait MouseLayoutPort {
    fn GetLayoutObject(&self, node: &EventTargetRef) -> Option<MouseDragLayoutRef>;
    fn HasUserScrollableLayoutBox(&self, node: &EventTargetRef) -> bool;
    fn IsMenuListSelect(&self, node: &EventTargetRef) -> bool;
}
pub trait MouseAutoscrollPort {
    fn SelectionAutoscrollInProgress(&self) -> bool;
    fn StopAutoscroll(&self);
    fn MiddleClickAutoscrollInProgress(&self) -> bool;
    fn HasAutoscrollController(&self) -> bool;
    fn StartAutoscrollForSelection(&self, layout: &MouseDragLayoutRef);
}

// Source: mouse_event_manager.cc:936-950,1021-1024,1033-1084,1139-1153,1174-1190
// The source's DataTransfer and DragState remain with DragController. The
// manager only retains shared identity handles across synchronous dispatch.
pub use interaction_dispatch::core::clipboard::data_transfer::{
    DataTransfer as MouseDataTransfer, DragOperation,
};
pub type MouseDataTransferRef = Rc<MouseDataTransfer>;

/// Source: mouse_event_manager.cc:1205-1230. The Page/DragController remain
/// the canonical source owner; only these reset leaves enter MouseEventManager.
pub trait MouseDragSourceResetPort {
    fn HasPage(&self) -> bool;
    fn DragSource(&self) -> Option<EventTargetRef>;
    fn DragSourceHasFrame(&self, source: &EventTargetRef) -> bool;
    fn DragSourceFrameIsDescendantOfFrame(&self, source: &EventTargetRef) -> bool;
    fn SetDragSource(&self, source: Option<&EventTargetRef>);
}
struct MouseDragSourceReset<'a, D: MouseDragPort + ?Sized> {
    has_page: bool,
    drag: &'a D,
}
impl<D: MouseDragPort + ?Sized> MouseDragSourceResetPort for MouseDragSourceReset<'_, D> {
    fn HasPage(&self) -> bool {
        self.has_page
    }
    fn DragSource(&self) -> Option<EventTargetRef> {
        self.drag.DragSource()
    }
    fn DragSourceHasFrame(&self, source: &EventTargetRef) -> bool {
        self.drag.DragSourceHasFrame(source)
    }
    fn DragSourceFrameIsDescendantOfFrame(&self, source: &EventTargetRef) -> bool {
        self.drag.DragSourceFrameIsDescendantOfFrame(source)
    }
    fn SetDragSource(&self, source: Option<&EventTargetRef>) {
        self.drag.SetDragSource(source)
    }
}

pub trait MouseDragPort {
    fn DragSource(&self) -> Option<EventTargetRef>;
    fn SetDragSource(&self, source: Option<&EventTargetRef>);
    fn DragSourceHasFrame(&self, source: &EventTargetRef) -> bool;
    fn DragSourceFrameIsDescendantOfFrame(&self, source: &EventTargetRef) -> bool;
    fn HitTestMouseDownReadOnly(&self, mouse_down: PointF) -> Option<EventTargetRef>;
    // DraggableNode updates the existing DragState.drag_type_ output, as in
    // cc:947-950, before the caller writes its returned drag_src_ owner.
    fn DraggableNode(
        &self,
        node: &EventTargetRef,
        mouse_down: PointF,
        delayed_selection_drag_resolution: bool,
    ) -> Option<EventTargetRef>;
    fn DataTransfer(&self) -> Option<MouseDataTransferRef>;
    fn SetDataTransfer(&self, data_transfer: &MouseDataTransferRef);
    fn CreateWritableDragAndDropDataTransfer(&self) -> MouseDataTransferRef;
    fn PopulateDragDataTransfer(&self, mouse_down: PointF, current: PointF) -> bool;
    fn DragTypeIsSelection(&self) -> bool;
    fn StartDrag(&self, event: &WebMouseEvent, mouse_down: PointF) -> bool;
    fn SetDestinationOperation(&self, operation: DragOperation);
    fn HandlePointerCausesUaAction(&self, pointer_type: PointerType, timestamp: TimeTicks);
}

pub trait MouseDragEvent {
    fn DragEvent(&self) -> &WebMouseEvent;
}
impl<T: MouseHitTest + ?Sized> MouseDragEvent for T {
    fn DragEvent(&self) -> &WebMouseEvent {
        MouseHitTest::Event(self)
    }
}
/// Only the live selection facts consumed by drag initiation. The selection
/// owner stays shared with the call-scoped SelectionController.
pub trait MouseDragSelectionPort {
    fn MouseDownMayStartSelect(&self) -> bool;
    fn SelectionStartsInPasswordField(&self) -> bool;
}
pub trait MouseDragFramePort {
    fn HasView(&self) -> bool;
    fn HasPage(&self) -> bool;
    fn ConvertIntPointFromRootFrame(&self, point: PointF) -> PointF;
}
pub trait MouseDragNodePort {
    fn SameDocument(&self, a: &EventTargetRef, b: &EventTargetRef) -> bool;
}
struct MouseDragFrameBridge<'a>(&'a dyn MouseFrameViewPort);
impl MouseDragFramePort for MouseDragFrameBridge<'_> {
    fn HasView(&self) -> bool {
        self.0.HasView()
    }
    fn HasPage(&self) -> bool {
        self.0.HasPage()
    }
    fn ConvertIntPointFromRootFrame(&self, p: PointF) -> PointF {
        self.0.ConvertIntPointFromRootFrame(p)
    }
}
struct MouseDragNodeBridge<'a>(&'a dyn MouseNodeRelations);
impl MouseDragNodePort for MouseDragNodeBridge<'_> {
    fn SameDocument(&self, a: &EventTargetRef, b: &EventTargetRef) -> bool {
        self.0.SameDocument(a, b)
    }
}
pub struct MouseDragTransactionPorts<'a, D: MouseDragPort + ?Sized> {
    pub nodes: &'a dyn MouseDragNodePort,
    pub dispatch: &'a dyn MouseEventDispatchPort,
    pub view: &'a dyn MouseDragFramePort,
    pub selection: &'a dyn MouseDragSelectionPort,
    pub drag: &'a D,
    pub suppress_pointer_stream_after_drag: bool,
}
struct MouseDragSelectionBridge<'a, H: MouseHitTest + ?Sized>(&'a dyn MouseSelectionPort<H>);
impl<H: MouseHitTest + ?Sized> MouseDragSelectionPort for MouseDragSelectionBridge<'_, H> {
    fn MouseDownMayStartSelect(&self) -> bool {
        self.0.MouseDownMayStartSelect()
    }
    fn SelectionStartsInPasswordField(&self) -> bool {
        self.0.SelectionStartsInPasswordField()
    }
}
// Source: mouse_event_manager.cc:259,392-395,410,445-447,488-489,516,573-574,829-831,988
// Runtime feature/settings inputs are read live rather than frozen across a
// dispatch. Platform conditions use cfg! and keep the upstream branches.
pub trait MouseFeatures {
    fn LightDismissFromClickEnabled(&self) -> bool;
    fn SyntheticMouseHoverOverInactivePageEnabled(&self) -> bool;
    fn SuppressPointerStreamAfterDragEnabled(&self) -> bool;
    fn BoundaryEventDispatchTracksNodeRemovalEnabled(&self) -> bool;
    fn PseudoElementsFocusableEnabled(&self) -> bool;
    fn BarrelButtonForDragEnabled(&self) -> bool;
}

// Source: mouse_event_manager.h:224-267
// frame_/scroll_manager_ are borrowed through MouseManagerPorts. None of the
// ports is stored by the manager or grants mutable access to another engine.
pub struct MouseManagerPorts<'a, H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized> {
    pub nodes: &'a dyn MouseNodeRelations,
    pub boundary_dom: &'a dyn BoundaryEventDOM,
    pub dispatch: &'a dyn MouseEventDispatchPort,
    pub light_dismiss: &'a dyn MouseLightDismissPort,
    pub view: &'a dyn MouseFrameViewPort,
    pub focus: &'a dyn MouseFocusPort,
    pub selection: &'a dyn MouseSelectionPort<H>,
    pub layout: &'a dyn MouseLayoutPort,
    pub autoscroll: &'a dyn MouseAutoscrollPort,
    pub lifecycle: &'a dyn MouseDragLifecyclePort,
    pub drag: &'a D,
    pub features: &'a dyn MouseFeatures,
    pub clock: &'a dyn EventClock,
}

impl<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized> MouseBoundaryEventDispatchPort
    for MouseManagerPorts<'_, H, D>
{
    fn DocumentWindow(&self, node: &EventTargetRef) -> Option<AbstractViewRef> {
        self.dispatch.DocumentWindow(node)
    }
    fn CoordinateContext(&self) -> MouseCoordinateContext {
        self.dispatch.CoordinateContext()
    }
    fn SourceCapabilities(
        &self,
        window: &AbstractViewRef,
        from_touch: bool,
    ) -> InputDeviceCapabilitiesRef {
        self.dispatch.SourceCapabilities(window, from_touch)
    }
    fn HasEventListeners(&self, target: &EventTargetRef, event_type: &str) -> bool {
        self.nodes.HasEventListeners(target, event_type)
    }
    fn DispatchEvent(&self, target: &EventTargetRef, event: &EventRef) -> DispatchEventResult {
        self.dispatch.DispatchEvent(target, event)
    }
}

impl<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized> MouseClickDispatchPort
    for MouseManagerPorts<'_, H, D>
{
    fn IsConnected(&self, node: &EventTargetRef) -> bool {
        self.nodes.IsConnected(node)
    }
    fn SameDocument(&self, a: &EventTargetRef, b: &EventTargetRef) -> bool {
        self.nodes.SameDocument(a, b)
    }
    fn CommonAncestorForClick(
        &self,
        a: &EventTargetRef,
        b: &EventTargetRef,
    ) -> Option<EventTargetRef> {
        self.nodes.CommonAncestorForClick(a, b)
    }
    fn LightDismissFromClickEnabled(&self) -> bool {
        self.features.LightDismissFromClickEnabled()
    }
    fn HandlePopoverLightDismissForClick(&self, down: &EventTargetRef, up: &EventTargetRef) {
        self.light_dismiss
            .HandlePopoverLightDismissForClick(down, up)
    }
    fn HandleDialogLightDismissForClick(
        &self,
        down: &dyn MousePointerTarget,
        up: &dyn MousePointerTarget,
    ) {
        self.light_dismiss
            .HandleDialogLightDismissForClick(down, up)
    }
}

impl<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized> MouseFocusPagePort
    for MouseManagerPorts<'_, H, D>
{
    fn HasPage(&self) -> bool {
        self.view.HasPage()
    }
}

impl<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized> MouseFocusSelectionPort
    for MouseManagerPorts<'_, H, D>
{
    fn IsRange(&self) -> bool {
        self.selection.IsRange()
    }
    fn Clear(&self) {
        self.selection.Clear()
    }
}

impl<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized> MouseFocusFeatures
    for MouseManagerPorts<'_, H, D>
{
    fn PseudoElementsFocusableEnabled(&self) -> bool {
        self.features.PseudoElementsFocusableEnabled()
    }
}

// Source: mouse_event_manager.h:224-267
pub struct MouseEventManagerState {
    element_under_mouse_: RefCell<Option<EventTargetRef>>,
    original_element_under_mouse_removed_: Cell<bool>,
    last_known_mouse_position_in_root_frame_: Cell<PointF>,
    last_known_mouse_position_: Cell<PointF>,
    last_known_mouse_screen_position_: Cell<PointF>,
    is_mouse_position_unknown_: Cell<bool>,
    mouse_pressed_: Cell<bool>,
    mouse_down_may_start_autoscroll_: Cell<bool>,
    svg_pan_: Cell<bool>,
    mouse_down_may_start_drag_: Cell<bool>,
    mousedown_element_: RefCell<Option<EventTargetRef>>,
    mouse_press_node_: RefCell<Option<EventTargetRef>>,
    click_count_: Cell<i32>,
    mouse_down_pos_: Cell<PointF>,
    mouse_down_timestamp_: Cell<TimeTicks>,
    mouse_down_: RefCell<WebMouseEvent>,
    drag_start_pos_in_root_frame_: Cell<PointF>,
    hover_state_dirty_: Cell<bool>,
    drag_initiator_: Cell<DragAndDropToolType>,
}

// Rust ownership refinement: the facade and full manager retain this same
// source state owner. There is no separate cursor-state projection.
impl MouseEventManagerState {
    // Source: mouse_event_manager.cc:528-530. Both full and borrowed manager
    // assembly read the same retained EventTarget identity.
    pub fn GetElementUnderMouse(&self) -> Option<EventTargetRef> {
        self.element_under_mouse_.borrow().clone()
    }
    // Source: mouse_event_manager.cc:127-133 (field initialization).
    pub fn new() -> Self {
        Self {
            element_under_mouse_: RefCell::new(None),
            original_element_under_mouse_removed_: Cell::new(false),
            last_known_mouse_position_in_root_frame_: Cell::new(PointF::default()),
            last_known_mouse_position_: Cell::new(PointF::default()),
            last_known_mouse_screen_position_: Cell::new(PointF::default()),
            is_mouse_position_unknown_: Cell::new(true),
            mouse_pressed_: Cell::new(false),
            mouse_down_may_start_autoscroll_: Cell::new(false),
            svg_pan_: Cell::new(false),
            mouse_down_may_start_drag_: Cell::new(false),
            mousedown_element_: RefCell::new(None),
            mouse_press_node_: RefCell::new(None),
            click_count_: Cell::new(0),
            mouse_down_pos_: Cell::new(PointF::default()),
            mouse_down_timestamp_: Cell::new(TimeTicks::new()),
            mouse_down_: RefCell::new(WebMouseEvent::default()),
            drag_start_pos_in_root_frame_: Cell::new(PointF::default()),
            hover_state_dirty_: Cell::new(false),
            drag_initiator_: Cell::new(DragAndDropToolType::kUnknown),
        }
    }
}
impl Default for MouseEventManagerState {
    fn default() -> Self {
        Self::new()
    }
}
pub struct MouseEventManager {
    state: Rc<MouseEventManagerState>,
}
impl std::ops::Deref for MouseEventManager {
    type Target = MouseEventManagerState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

// Source: mouse_event_manager.cc:58-75
fn UpdateMouseMovementXY(
    mouse_event: &WebMouseEvent,
    last_position: Option<PointF>,
    initializer: &mut MouseEventInit,
) {
    if !mouse_event.pointer_properties.is_raw_movement_event
        && mouse_event.GetType() == Type::kMouseMove
    {
        if let Some(last) = last_position {
            // Rust's float-to-int cast saturates/truncates like saturated_cast.
            initializer.movement_x = (mouse_event.pointer_properties.PositionInScreen().x() as i32)
                .wrapping_sub(last.x() as i32);
            initializer.movement_y = (mouse_event.pointer_properties.PositionInScreen().y() as i32)
                .wrapping_sub(last.y() as i32);
        }
    }
}

// Source: mouse_event_manager.cc:77-112
fn SetMouseEventAttributes(
    dispatch: &dyn MouseBoundaryEventDispatchPort,
    initializer: &mut MouseEventInit,
    target_node: &EventTargetRef,
    event_type: &str,
    event: &WebMouseEvent,
    last_position: Option<PointF>,
    related_target: Option<&EventTargetRef>,
    click_count: i32,
) {
    let enter_or_leave = matches!(event_type, "mouseenter" | "mouseleave");
    let window = dispatch.DocumentWindow(target_node);
    let context = dispatch.CoordinateContext();
    MouseEvent::SetCoordinatesFromWebPointerProperties(
        &event.FlattenTransform().pointer_properties,
        window.as_ref(),
        context.views.as_deref(),
        initializer,
    );
    UpdateMouseMovementXY(event, last_position, initializer);
    initializer.button = event.pointer_properties.button as i16;
    initializer.buttons = MouseEvent::WebInputEventModifiersToButtons(event.GetModifiers() as u32);
    initializer.related_target = related_target.cloned();
    UIEventWithKeyState::SetFromWebInputEventModifiers(
        &mut initializer.event_modifier_init,
        event.GetModifiers(),
    );
    let ui = &mut initializer.event_modifier_init.ui_event_init;
    ui.event_init.bubbles = !enter_or_leave;
    ui.event_init.cancelable = !enter_or_leave;
    ui.event_init.composed = !enter_or_leave;
    ui.detail = click_count;
    ui.source_capabilities = window
        .as_ref()
        .map(|window| dispatch.SourceCapabilities(window, event.FromTouch()));
    ui.view = window;
}

// Source: mouse_event_manager.cc:114-123
const kDragThresholdX: i32 = if cfg!(target_os = "macos") { 3 } else { 4 };
const kDragThresholdY: i32 = if cfg!(target_os = "macos") { 3 } else { 4 };
const kTextDragDelayMicros: i64 = if cfg!(target_os = "macos") {
    150_000
} else {
    0
};

impl MouseEventManager {
    // Source: mouse_event_manager.cc:127-133
    pub fn new<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        ports: &MouseManagerPorts<'_, H, D>,
    ) -> Self {
        let manager = Self::WithState(Rc::new(MouseEventManagerState::new()));
        manager.Clear(ports);
        manager
    }

    /// Attach the full source algorithms to the same live input state. This
    /// does not reset input already admitted by the facade.
    pub fn WithState(state: Rc<MouseEventManagerState>) -> Self {
        Self { state }
    }

    // Source: mouse_event_manager.cc:135-157
    pub fn Clear<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
    ) {
        let selection = MouseDragSelectionBridge(ports.selection);
        let nodes = MouseDragNodeBridge(ports.nodes);
        let view = MouseDragFrameBridge(ports.view);
        let drag_ports = MouseDragTransactionPorts {
            nodes: &nodes,
            dispatch: ports.dispatch,
            view: &view,
            selection: &selection,
            drag: ports.drag,
            suppress_pointer_stream_after_drag: ports
                .features
                .SuppressPointerStreamAfterDragEnabled(),
        };
        self.ClearWithDragPorts(&drag_ports);
    }
    /// Same source Clear over only the drag transaction services it consumes.
    pub fn ClearWithDragPorts<D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseDragTransactionPorts<'_, D>,
    ) {
        self.state.ClearSourceInputState();
        Self::ResetDragSourceWithDragPorts(ports);
        self.ClearDragDataTransferWithPorts(ports);
    }
}

impl MouseEventManagerState {
    // Source mouse_event_manager.cc:135-154. Borrowed PointerManager assembly
    // clears this same retained owner; DragController cleanup stays with its
    // separately assembled ports in MouseEventManager::Clear above.
    pub fn ClearSourceInputState(&self) {
        self.element_under_mouse_.replace(None);
        self.original_element_under_mouse_removed_.set(false);
        self.mouse_press_node_.replace(None);
        self.mouse_down_may_start_autoscroll_.set(false);
        self.mouse_down_may_start_drag_.set(false);
        self.mouse_pressed_.set(false);
        self.click_count_.set(0);
        self.mousedown_element_.replace(None);
        self.mouse_down_pos_.set(PointF::default());
        self.mouse_down_timestamp_.set(TimeTicks::new());
        self.mouse_down_.replace(WebMouseEvent::default());
        self.svg_pan_.set(false);
        self.drag_start_pos_in_root_frame_.set(PointF::default());
        self.hover_state_dirty_.set(false);
    }
}

// Source: mouse_event_manager.cc:167-185
struct MouseEventBoundaryEventDispatcher<'a> {
    manager: &'a MouseEventManager,
    dispatch: &'a dyn MouseBoundaryEventDispatchPort,
    event: &'a WebMouseEvent,
}
impl BoundaryEventDispatch for MouseEventBoundaryEventDispatcher<'_> {
    fn Dispatch(
        &self,
        target: &EventTargetRef,
        related_target: Option<&EventTargetRef>,
        event_type: &str,
        check_for_listener: bool,
    ) {
        self.manager.DispatchMouseEventWithDispatch(
            self.dispatch,
            Some(target),
            event_type,
            self.event,
            None,
            related_target,
            check_for_listener,
        );
    }
}

impl MouseEventManager {
    // Source: mouse_event_manager.cc:187-195
    pub fn SendBoundaryEvents<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        exited: Option<&EventTargetRef>,
        original_exited_target_removed: bool,
        entered: Option<&EventTargetRef>,
        event: &WebMouseEvent,
    ) {
        self.SendBoundaryEventsWithDispatch(
            ports.boundary_dom,
            ports,
            exited,
            original_exited_target_removed,
            entered,
            event,
        );
    }

    // Source: mouse_event_manager.h:177-194
    // Source: mouse_event_manager.cc:167-195
    pub fn SendBoundaryEventsWithDispatch(
        &self,
        boundary_dom: &dyn BoundaryEventDOM,
        dispatch: &dyn MouseBoundaryEventDispatchPort,
        exited: Option<&EventTargetRef>,
        original_exited_target_removed: bool,
        entered: Option<&EventTargetRef>,
        event: &WebMouseEvent,
    ) {
        let sender = MouseEventBoundaryEventDispatcher {
            manager: self,
            dispatch,
            event,
        };
        BoundaryEventDispatcher::new("mouseover", "mouseout", "mouseenter", "mouseleave")
            .SendBoundaryEvents(
                boundary_dom,
                &sender,
                exited,
                original_exited_target_removed,
                entered,
            );
    }

    // Source: mouse_event_manager.h:51-65
    // Source: mouse_event_manager.cc:197-295
    pub fn DispatchMouseEvent<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        target: Option<&EventTargetRef>,
        event_type: &str,
        mouse_event: &WebMouseEvent,
        last_position: Option<PointF>,
        related_target: Option<&EventTargetRef>,
        check_for_listener: bool,
        pointer_id: PointerId,
        pointer_type: &str,
        pointer_down_target: Option<&dyn MousePointerTarget>,
        pointer_up_target: Option<&dyn MousePointerTarget>,
    ) -> (Option<EventRef>, WebInputEventResult) {
        debug_assert!(matches!(
            event_type,
            "mouseup"
                | "mousedown"
                | "mousemove"
                | "mouseout"
                | "mouseover"
                | "mouseleave"
                | "mouseenter"
                | "contextmenu"
                | "click"
                | "auxclick"
        ));
        if event_type == "contextmenu" {
            return self.DispatchContextMenuEventWithDispatch(
                ports,
                target,
                mouse_event,
                last_position,
                pointer_id,
                pointer_type,
            );
        }
        if !matches!(event_type, "click" | "auxclick") {
            return self.DispatchMouseEventWithDispatch(
                ports,
                target,
                event_type,
                mouse_event,
                last_position,
                related_target,
                check_for_listener,
            );
        }
        let Some(target) = target.filter(|target| target.Kind() == EventTargetKind::Node) else {
            return (None, WebInputEventResult::kNotHandled);
        };
        let click_count = if matches!(event_type, "mouseup" | "mousedown" | "click" | "auxclick") {
            self.click_count_.get()
        } else {
            0
        };
        let should_dispatch =
            !check_for_listener || ports.nodes.HasEventListeners(target, event_type);
        let mut initializer = MouseEventInit::default();
        SetMouseEventAttributes(
            ports,
            &mut initializer,
            target,
            event_type,
            mouse_event,
            last_position,
            related_target,
            click_count,
        );
        let synthetic = if mouse_event.FromTouch() {
            SyntheticEventType::kFromTouch
        } else {
            SyntheticEventType::kRealOrIndistinguishable
        };
        let event = {
            let initializer = PointerEventInit {
                mouse_event_init: initializer,
                pointer_id: Some(pointer_id),
                pointer_type: Some(pointer_type.to_owned()),
                ..PointerEventInit::default()
            };
            let event = EventRef::new(PointerEvent::new(
                event_type,
                &initializer,
                mouse_event.TimeStamp(),
                synthetic,
                mouse_event.menu_source_type,
                false,
                ports.dispatch.CoordinateContext(),
            ));
            let down = pointer_down_target.and_then(MousePointerTarget::Node);
            let up = pointer_up_target.and_then(MousePointerTarget::Node);
            if ports.features.LightDismissFromClickEnabled()
                && event_type == "click"
                && down
                    .as_ref()
                    .is_some_and(|node| ports.nodes.IsConnected(node))
                && up
                    .as_ref()
                    .is_some_and(|node| ports.nodes.IsConnected(node))
            {
                ports.light_dismiss.HandlePopoverLightDismissForClick(
                    down.as_ref().unwrap(),
                    up.as_ref().unwrap(),
                );
                ports.light_dismiss.HandleDialogLightDismissForClick(
                    pointer_down_target.unwrap(),
                    pointer_up_target.unwrap(),
                );
            }
            event
        };
        // TODO: UIEventTiming instrumentation (cc:267).
        if should_dispatch {
            let result = ToWebInputEventResult(ports.dispatch.DispatchEvent(target, &event));
            (Some(event), result)
        } else {
            (None, WebInputEventResult::kNotHandled)
        }
    }

    /// Source contextmenu dispatch uses PointerEvent, but consumes none of the
    /// click light-dismiss or full MouseManagerPorts services. Keep it on the
    /// same native initializer/canonical dispatcher as the full manager.
    // Source: mouse_event_manager.cc:197-229,267-289
    pub fn DispatchContextMenuEventWithDispatch(
        &self,
        dispatch: &dyn MouseBoundaryEventDispatchPort,
        target: Option<&EventTargetRef>,
        mouse_event: &WebMouseEvent,
        last_position: Option<PointF>,
        pointer_id: PointerId,
        pointer_type: &str,
    ) -> (Option<EventRef>, WebInputEventResult) {
        let Some(target) = target.filter(|target| target.Kind() == EventTargetKind::Node) else {
            return (None, WebInputEventResult::kNotHandled);
        };
        let mut initializer = MouseEventInit::default();
        SetMouseEventAttributes(
            dispatch,
            &mut initializer,
            target,
            "contextmenu",
            mouse_event,
            last_position,
            None,
            0,
        );
        let synthetic = if mouse_event.FromTouch() {
            SyntheticEventType::kFromTouch
        } else {
            SyntheticEventType::kRealOrIndistinguishable
        };
        let initializer = PointerEventInit {
            mouse_event_init: initializer,
            pointer_id: Some(pointer_id),
            pointer_type: Some(pointer_type.to_owned()),
            ..PointerEventInit::default()
        };
        let event = EventRef::new(PointerEvent::new(
            "contextmenu",
            &initializer,
            mouse_event.TimeStamp(),
            synthetic,
            mouse_event.menu_source_type,
            false,
            dispatch.CoordinateContext(),
        ));
        let result = ToWebInputEventResult(dispatch.DispatchEvent(target, &event));
        (Some(event), result)
    }

    /// The source's ordinary MouseEvent branch over only the services it reads.
    /// Full mouse dispatch and boundary dispatch both construct and synchronously
    /// deliver the same native event through this entry.
    // Source: mouse_event_manager.h:51-65
    // Source: mouse_event_manager.cc:197-229,274-295
    pub fn DispatchMouseEventWithDispatch(
        &self,
        dispatch: &dyn MouseBoundaryEventDispatchPort,
        target: Option<&EventTargetRef>,
        event_type: &str,
        mouse_event: &WebMouseEvent,
        last_position: Option<PointF>,
        related_target: Option<&EventTargetRef>,
        check_for_listener: bool,
    ) -> (Option<EventRef>, WebInputEventResult) {
        debug_assert!(matches!(
            event_type,
            "mouseup"
                | "mousedown"
                | "mousemove"
                | "mouseout"
                | "mouseover"
                | "mouseleave"
                | "mouseenter"
        ));
        let Some(target) = target.filter(|target| target.Kind() == EventTargetKind::Node) else {
            return (None, WebInputEventResult::kNotHandled);
        };
        let click_count = if matches!(event_type, "mouseup" | "mousedown") {
            self.click_count_.get()
        } else {
            0
        };
        let should_dispatch = !check_for_listener || dispatch.HasEventListeners(target, event_type);
        let mut initializer = MouseEventInit::default();
        SetMouseEventAttributes(
            dispatch,
            &mut initializer,
            target,
            event_type,
            mouse_event,
            last_position,
            related_target,
            click_count,
        );
        let synthetic = if mouse_event.FromTouch() {
            SyntheticEventType::kFromTouch
        } else {
            SyntheticEventType::kRealOrIndistinguishable
        };
        let event = EventRef::new(MouseEvent::new(
            event_type,
            &initializer,
            mouse_event.TimeStamp(),
            synthetic,
            mouse_event.menu_source_type,
            None,
            dispatch.CoordinateContext(),
        ));
        // TODO: UIEventTiming instrumentation (cc:284).
        if should_dispatch {
            let result = ToWebInputEventResult(dispatch.DispatchEvent(target, &event));
            (Some(event), result)
        } else {
            (None, WebInputEventResult::kNotHandled)
        }
    }

    // Source: mouse_event_manager.cc:297-325
    pub fn SetElementUnderMouseAndDispatchMouseEvent<
        H: MouseHitTest + ?Sized,
        D: MouseDragPort + ?Sized,
    >(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        target: Option<&EventTargetRef>,
        event_type: &str,
        event: &WebMouseEvent,
        pointer_down_target: Option<&dyn MousePointerTarget>,
        pointer_up_target: Option<&dyn MousePointerTarget>,
    ) -> WebInputEventResult {
        self.SetElementUnderMouse(ports, target, event);
        let target = self.GetElementUnderMouse();
        self.DispatchMouseEvent(
            ports,
            target.as_ref(),
            event_type,
            event,
            None,
            None,
            false,
            event.pointer_properties.id,
            PointerTypeName(event.pointer_properties.pointer_type),
            pointer_down_target,
            pointer_up_target,
        )
        .1
    }

    // Source: mouse_event_manager.cc:327-374
    pub fn DispatchMouseClickIfNeeded<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        mouse_release_target: Option<&EventTargetRef>,
        captured_click_target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
        pointer_id: PointerId,
        pointer_type: &str,
        pointer_down_target: Option<&dyn MousePointerTarget>,
        pointer_up_target: Option<&dyn MousePointerTarget>,
    ) -> WebInputEventResult {
        self.DispatchMouseClickIfNeededWithDispatch(
            ports,
            mouse_release_target,
            captured_click_target,
            event,
            pointer_id,
            pointer_type,
            pointer_down_target,
            pointer_up_target,
        )
    }

    /// Call-scoped production entry for PointerEventManager's mouse-release
    /// transaction. It is the same source click algorithm and same retained
    /// MouseEventManagerState as the full MouseManagerPorts entry above.
    // Source: mouse_event_manager.cc:248-289,327-374
    pub fn DispatchMouseClickIfNeededWithDispatch(
        &self,
        ports: &dyn MouseClickDispatchPort,
        mouse_release_target: Option<&EventTargetRef>,
        captured_click_target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
        pointer_id: PointerId,
        pointer_type: &str,
        pointer_down_target: Option<&dyn MousePointerTarget>,
        pointer_up_target: Option<&dyn MousePointerTarget>,
    ) -> WebInputEventResult {
        let context_menu_event = cfg!(target_os = "macos")
            && event.pointer_properties.button == Button::kLeft
            && event.GetModifiers() & WebInputEvent::kControlKey != 0;
        let down = self.mousedown_element_.borrow().clone();
        let Some((down, release)) = down.as_ref().zip(mouse_release_target) else {
            return WebInputEventResult::kNotHandled;
        };
        if self.click_count_.get() <= 0 || context_menu_event || !ports.IsConnected(down) {
            return WebInputEventResult::kNotHandled;
        }
        let click_target = if let Some(target) = captured_click_target {
            Some(target.clone())
        } else if ports.SameDocument(down, release) {
            ports.CommonAncestorForClick(release, down)
        } else {
            None
        };
        let Some(target) = click_target else {
            return WebInputEventResult::kNotHandled;
        };
        let event_type = if event.pointer_properties.button == Button::kLeft {
            "click"
        } else {
            "auxclick"
        };
        let mut initializer = MouseEventInit::default();
        SetMouseEventAttributes(
            ports,
            &mut initializer,
            &target,
            event_type,
            event,
            None,
            None,
            self.click_count_.get(),
        );
        let synthetic = if event.FromTouch() {
            SyntheticEventType::kFromTouch
        } else {
            SyntheticEventType::kRealOrIndistinguishable
        };
        let initializer = PointerEventInit {
            mouse_event_init: initializer,
            pointer_id: Some(pointer_id),
            pointer_type: Some(pointer_type.to_owned()),
            ..PointerEventInit::default()
        };
        let click = EventRef::new(PointerEvent::new(
            event_type,
            &initializer,
            event.TimeStamp(),
            synthetic,
            event.menu_source_type,
            false,
            ports.CoordinateContext(),
        ));
        let down = pointer_down_target.and_then(MousePointerTarget::Node);
        let up = pointer_up_target.and_then(MousePointerTarget::Node);
        if ports.LightDismissFromClickEnabled()
            && event_type == "click"
            && down.as_ref().is_some_and(|node| ports.IsConnected(node))
            && up.as_ref().is_some_and(|node| ports.IsConnected(node))
        {
            ports.HandlePopoverLightDismissForClick(down.as_ref().unwrap(), up.as_ref().unwrap());
            ports.HandleDialogLightDismissForClick(
                pointer_down_target.unwrap(),
                pointer_up_target.unwrap(),
            );
        }
        ToWebInputEventResult(ports.DispatchEvent(&target, &click))
    }

    // Source: mouse_event_manager.cc:376-430
    pub fn RecomputeMouseHoverStateIfNeeded<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
    ) {
        self.RecomputeMouseHoverStateWithPorts(ports.view, ports.features, ports.drag, ports.clock);
    }

    /// Source hover algorithm over its exact call-scoped leaves.
    pub fn RecomputeMouseHoverStateWithPorts<D: MouseDragPort + ?Sized>(
        &self,
        view: &dyn MouseFrameViewPort,
        features: &dyn MouseFeatures,
        drag: &D,
        clock: &dyn EventClock,
    ) {
        if !self.HoverStateDirty(view) {
            return;
        }
        self.hover_state_dirty_.set(false);
        if self.is_mouse_position_unknown_.get() || !view.HasView() {
            return;
        }
        if !view.HasPage()
            || !(features.SyntheticMouseHoverOverInactivePageEnabled() || view.IsPageActive())
        {
            return;
        }
        if !view.IsCursorVisible() || view.IsPointerLocked() {
            return;
        }
        if features.SuppressPointerStreamAfterDragEnabled() && drag.DragSource().is_some() {
            return;
        }
        let mut button = Button::kNoButton;
        let mut modifiers = view.CurrentModifierState() | WebInputEvent::kRelativeMotionEvent;
        if self.mouse_pressed_.get() {
            button = Button::kLeft;
            modifiers |= WebInputEvent::kLeftButtonDown;
        }
        let event = WebMouseEvent::new(
            Type::kMouseMove,
            self.last_known_mouse_position_.get(),
            self.last_known_mouse_screen_position_.get(),
            button,
            0,
            modifiers,
            clock.Now(),
            MenuSourceType::kNone,
            WebMouseEvent::kMousePointerId,
        );
        view.HandleSyntheticMouseMove(&event);
    }

    // Source: mouse_event_manager.cc:432-435
    pub fn MarkHoverStateDirty(&self, view: &dyn MouseFrameViewPort) {
        debug_assert!(view.IsLocalRoot());
        self.hover_state_dirty_.set(true);
    }

    // Source: mouse_event_manager.cc:437-440
    fn HoverStateDirty(&self, view: &dyn MouseFrameViewPort) -> bool {
        debug_assert!(view.IsLocalRoot());
        self.hover_state_dirty_.get()
    }

    // Source: mouse_event_manager.cc:442-465
    pub fn SetElementUnderMouse<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
    ) {
        self.SetElementUnderMouseWithDispatch(
            ports.boundary_dom,
            ports,
            &|node| ports.nodes.InFrameDocument(node),
            ports
                .features
                .BoundaryEventDispatchTracksNodeRemovalEnabled(),
            target,
            event,
        );
    }

    // Source: mouse_event_manager.h:177-194
    // Source: mouse_event_manager.cc:167-185,442-465
    pub fn SetElementUnderMouseWithDispatch(
        &self,
        boundary_dom: &dyn BoundaryEventDOM,
        dispatch: &dyn MouseBoundaryEventDispatchPort,
        in_frame_document: &dyn Fn(&EventTargetRef) -> bool,
        boundary_event_dispatch_tracks_node_removal: bool,
        target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
    ) {
        let sender = MouseEventBoundaryEventDispatcher {
            manager: self,
            dispatch,
            event,
        };
        self.SetElementUnderMouseWithBoundaryDispatch(
            boundary_dom,
            &sender,
            in_frame_document,
            boundary_event_dispatch_tracks_node_removal,
            target,
        );
    }

    /// Source SetElementUnderMouse over the exact boundary-state inputs it
    /// consumes. The full manager entry delegates here; Page's synchronous
    /// transaction can connect the same retained state before unrelated drag,
    /// focus and autoscroll ports are assembled.
    pub fn SetElementUnderMouseWithBoundaryDispatch(
        &self,
        boundary_dom: &dyn BoundaryEventDOM,
        sender: &dyn BoundaryEventDispatch,
        in_frame_document: &dyn Fn(&EventTargetRef) -> bool,
        boundary_event_dispatch_tracks_node_removal: bool,
        target: Option<&EventTargetRef>,
    ) {
        assert!(
            !self.original_element_under_mouse_removed_.get()
                || boundary_event_dispatch_tracks_node_removal
        );
        let mut last = self.element_under_mouse_.replace(target.cloned());
        let removed = self.original_element_under_mouse_removed_.replace(false);
        if last.as_ref().is_some_and(|node| !in_frame_document(node)) {
            last = None;
        }
        let entered = self.GetElementUnderMouse();
        BoundaryEventDispatcher::new("mouseover", "mouseout", "mouseenter", "mouseleave")
            .SendBoundaryEvents(
                boundary_dom,
                sender,
                last.as_ref(),
                removed,
                entered.as_ref(),
            );
    }

    // Source: mouse_event_manager.cc:467-469
    pub fn NodeChildrenWillBeRemoved<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        node: &EventTargetRef,
    ) {
        self.NodeChildrenWillBeRemovedWithRelations(
            ports.nodes,
            ports
                .features
                .BoundaryEventDispatchTracksNodeRemovalEnabled(),
            node,
        );
    }
    pub fn NodeChildrenWillBeRemovedWithRelations<N: MouseRemovalNodeRelations + ?Sized>(
        &self,
        nodes: &N,
        boundary_event_dispatch_tracks_node_removal: bool,
        node: &EventTargetRef,
    ) {
        self.HandleRemoveSubtree(
            nodes,
            boundary_event_dispatch_tracks_node_removal,
            node,
            false,
        );
    }

    // Source: mouse_event_manager.cc:471-473
    pub fn NodeWillBeRemoved<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        node: &EventTargetRef,
    ) {
        self.HandleRemoveSubtree(
            ports.nodes,
            ports
                .features
                .BoundaryEventDispatchTracksNodeRemovalEnabled(),
            node,
            true,
        );
    }

    /// Source NodeWillBeRemoved over its actual consumed ports. Document uses
    /// this at the pre-unlink callback; the full manager entry above delegates
    /// to the same implementation.
    pub fn NodeWillBeRemovedWithRelations<N: MouseRemovalNodeRelations + ?Sized>(
        &self,
        nodes: &N,
        boundary_event_dispatch_tracks_node_removal: bool,
        node: &EventTargetRef,
    ) {
        self.HandleRemoveSubtree(
            nodes,
            boundary_event_dispatch_tracks_node_removal,
            node,
            true,
        );
    }

    // Source: mouse_event_manager.cc:475-493
    pub fn HandlePseudoElementRemoval<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        pseudo: &EventTargetRef,
    ) {
        self.HandlePseudoElementRemovalWithRelations(
            ports.nodes,
            ports
                .features
                .BoundaryEventDispatchTracksNodeRemovalEnabled(),
            pseudo,
        );
    }
    pub fn HandlePseudoElementRemovalWithRelations<N: MouseRemovalNodeRelations + ?Sized>(
        &self,
        nodes: &N,
        boundary_event_dispatch_tracks_node_removal: bool,
        pseudo: &EventTargetRef,
    ) {
        let parent = nodes.ParentOrShadowHostElement(pseudo);
        for field in [&self.mousedown_element_, &self.mouse_press_node_] {
            let target = field.borrow().clone();
            if target.as_ref().is_some_and(|node| {
                node.IsPseudoElement() && nodes.IsShadowIncludingInclusiveAncestorOf(pseudo, node)
            }) {
                field.replace(parent.clone());
            }
        }
        let target = self.GetElementUnderMouse();
        if target.as_ref().is_some_and(|node| {
            node.IsPseudoElement() && nodes.IsShadowIncludingInclusiveAncestorOf(pseudo, node)
        }) {
            self.element_under_mouse_.replace(parent);
            if boundary_event_dispatch_tracks_node_removal {
                self.original_element_under_mouse_removed_.set(true);
            }
        }
    }

    // Source: mouse_event_manager.cc:495-526
    fn HandleRemoveSubtree<N: MouseRemovalNodeRelations + ?Sized>(
        &self,
        nodes: &N,
        boundary_event_dispatch_tracks_node_removal: bool,
        node: &EventTargetRef,
        include_root: bool,
    ) {
        let remaining = if include_root {
            nodes.ParentNode(node)
        } else {
            Some(node.clone())
        };
        let affected = |target: &EventTargetRef| {
            (include_root || !Rc::ptr_eq(target, node))
                && nodes.IsShadowIncludingInclusiveAncestorOf(node, target)
        };
        let down = self.mousedown_element_.borrow().clone();
        if down.as_ref().is_some_and(affected) {
            self.mousedown_element_.replace(if node.IsPseudoElement() {
                nodes.ParentOrShadowHostElement(node)
            } else {
                None
            });
        }
        let press = self.MousePressNode();
        if press.as_ref().is_some_and(affected) {
            self.mouse_press_node_.replace(remaining.clone());
        }
        let under = self.GetElementUnderMouse();
        if boundary_event_dispatch_tracks_node_removal && under.as_ref().is_some_and(affected) {
            let remaining_element = remaining.as_ref().and_then(|node| {
                if nodes.IsElement(node) {
                    Some(node.clone())
                } else {
                    nodes.ParentOrShadowHostElement(node)
                }
            });
            self.element_under_mouse_.replace(remaining_element);
            self.original_element_under_mouse_removed_.set(true);
        }
    }

    // Source: mouse_event_manager.cc:541-639
    pub fn HandleMouseFocus<
        H: MouseHitTest + ?Sized,
        D: MouseDragPort + ?Sized,
        F: MouseFocusHitTest + ?Sized,
    >(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        hit: &F,
        capabilities: Option<&InputDeviceCapabilitiesRef>,
    ) -> WebInputEventResult {
        self.HandleMouseFocusWithPorts(
            &MouseFocusPorts {
                page: ports,
                focus: ports.focus,
                selection: ports,
                features: ports,
            },
            hit,
            capabilities,
        )
    }

    // Source: mouse_event_manager.cc:541-639.
    pub fn HandleMouseFocusWithPorts<F: MouseFocusHitTest + ?Sized>(
        &self,
        ports: &MouseFocusPorts<'_>,
        hit: &F,
        capabilities: Option<&InputDeviceCapabilitiesRef>,
    ) -> WebInputEventResult {
        if hit.ScrollbarIsFrameViewScrollbar() {
            return WebInputEventResult::kNotHandled;
        }
        let mut element = self.GetElementUnderMouse();
        while let Some(current) = element.as_ref() {
            if ports.focus.IsMouseFocusable(current)
                && ports.focus.IsFocusedElementInDocument(current)
            {
                return WebInputEventResult::kNotHandled;
            }
            if ports.focus.IsMouseFocusable(current)
                || ports.focus.IsShadowHostWithDelegatesFocus(current)
            {
                break;
            }
            element = ports.focus.FlatTreeParentElement(current);
        }
        if let Some(element) = element.as_ref().filter(|_| ports.selection.IsRange()) {
            if ports.features.PseudoElementsFocusableEnabled()
                && ports.focus.IsScrollControlPseudoElement(element)
            {
                ports.selection.Clear();
            } else if ports.focus.IsNodeFullyContainedInSelection(element)
                && ports.focus.IsDescendantOfFocusedElement(element)
            {
                return WebInputEventResult::kNotHandled;
            }
        }
        if element.is_none() && hit.HasScrollbar() {
            return WebInputEventResult::kHandledSystem;
        }
        if !ports.page.HasPage() {
            return WebInputEventResult::kNotHandled;
        }
        if let Some(element) = element
            .as_ref()
            .filter(|node| !ports.focus.IsMouseFocusable(node))
        {
            if let Some(target) = ports.focus.GetFocusableArea(element) {
                if ports.focus.IsFocusedElementInDocument(&target) {
                    return WebInputEventResult::kNotHandled;
                }
                ports
                    .focus
                    .FocusDelegatedTargetWithMouseAndResetSelection(&target);
                return if ports.focus.IsTextControl(&target) {
                    WebInputEventResult::kHandledSystem
                } else {
                    WebInputEventResult::kNotHandled
                };
            }
        }
        if !ports
            .focus
            .SetFocusedElementWithMouse(element.as_ref(), capabilities)
        {
            WebInputEventResult::kHandledSystem
        } else {
            WebInputEventResult::kNotHandled
        }
    }

    // Source: mouse_event_manager.cc:633-638
    pub fn HandleMouseReleaseEventUpdateStates<H: MouseHitTest + ?Sized>(
        &self,
        selection: &dyn MouseSelectionPort<H>,
    ) {
        self.ClearDragHeuristicState();
        self.InvalidateClick();
        selection.SetMouseDownMayStartSelect(false);
    }

    // Source: mouse_event_manager.cc:640-657
    pub fn HandleMousePressEventUpdateStates<
        H: MouseHitTest + ?Sized,
        D: MouseDragPort + ?Sized,
    >(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        event: &WebMouseEvent,
    ) {
        self.state.HandleMousePressEventUpdateStates(
            event,
            ports.view.HasView().then(|| {
                ports
                    .view
                    .ConvertIntPointFromRootFrame(Floored(event.PositionInRootFrame()))
            }),
        );
        ports.selection.SetMouseDownMayStartSelect(false);
    }

    // Source: mouse_event_manager.cc:684-748
    pub fn HandleMousePressEvent<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        event: &H,
    ) -> WebInputEventResult {
        self.HandleMousePressEventWithDefaultPorts(ports, event)
    }
    // Source: mouse_event_manager.cc:684-748.
    pub fn HandleMousePressEventWithDefaultPorts<H: MouseDefaultHitTest + ?Sized>(
        &self,
        ports: &dyn MousePressDefaultPort<H>,
        event: &H,
    ) -> WebInputEventResult {
        // TODO: TRACE_EVENT instrumentation (cc:686).
        ports.ResetDragSource();
        let single_click = event.DefaultEvent().click_count <= 1;
        self.state.HandleMousePressEventState(
            event.DefaultEvent(),
            event.DefaultIsSelectionOverLink(),
            event.DefaultIsExtendingSelection(),
        );
        if ports.IsSvgDocumentWithZoomAndPan()
            && event.DefaultEvent().GetModifiers() & WebInputEvent::kShiftKey != 0
            && single_click
        {
            self.svg_pan_.set(true);
            ports.StartSvgPan(
                ports.ConvertFromRootFrame(Floored(event.DefaultEvent().PositionInRootFrame())),
            );
            return WebInputEventResult::kHandledSystem;
        }
        if single_click {
            ports.FocusDocumentView();
        }
        let inner = event.DefaultInnerNode();
        self.mouse_press_node_.replace(inner.clone());
        ports.SetSequentialFocusNavigationStartingPoint(inner.as_ref());
        self.drag_start_pos_in_root_frame_
            .set(Floored(event.DefaultEvent().PositionInRootFrame()));
        self.mouse_pressed_.set(true);
        let swallowed = ports.HandleSelectionMousePress(event);
        let press = self.MousePressNode();
        self.mouse_down_may_start_autoscroll_.set(
            ports.MouseDownMayStartSelect()
                || press
                    .as_ref()
                    .is_some_and(|node| ports.HasUserScrollableLayoutBox(node)),
        );
        if swallowed {
            WebInputEventResult::kHandledSystem
        } else {
            WebInputEventResult::kNotHandled
        }
    }

    // Source: mouse_event_manager.cc:750-769
    pub fn HandleMouseReleaseEvent<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        event: &H,
    ) -> WebInputEventResult {
        self.HandleMouseReleaseEventWithDefaultPorts(ports, event)
    }
    // Source: mouse_event_manager.cc:750-769.
    pub fn HandleMouseReleaseEventWithDefaultPorts<H: MouseDefaultHitTest + ?Sized>(
        &self,
        ports: &dyn MouseReleaseDefaultPort<H>,
        event: &H,
    ) -> WebInputEventResult {
        ports.StopSelectionAutoscrollIfNeeded();
        if ports.HandleSelectionMouseRelease(event, self.drag_start_pos_in_root_frame_.get()) {
            WebInputEventResult::kHandledSystem
        } else {
            WebInputEventResult::kNotHandled
        }
    }

    // Source: mouse_event_manager.cc:771-776
    pub fn UpdateSelectionForMouseDrag<H: MouseHitTest + ?Sized>(
        &self,
        selection: &dyn MouseSelectionPort<H>,
    ) {
        selection.UpdateSelectionForMouseDrag(
            self.drag_start_pos_in_root_frame_.get(),
            self.last_known_mouse_position_in_root_frame_.get(),
        );
    }

    // Source: mouse_event_manager.cc:778-797. The embedding performs the
    // source read-only mouse hit test on this returned synthetic event, then
    // immediately calls HandleDragDropIfPossible with that same event/result.
    pub fn PrepareDragDropMouseEvent(
        &self,
        clock: &dyn EventClock,
        gesture: &interaction_input::public::common::input::web_gesture_event::WebGestureEvent,
        pointer_id: PointerId,
    ) -> WebMouseEvent {
        let modifiers = gesture.GetModifiers()
            | WebInputEvent::kLeftButtonDown
            | WebInputEvent::kIsCompatibilityEventForTouch;
        self.mouse_down_.replace(WebMouseEvent::FromGesture(
            Type::kMouseDown,
            gesture,
            Button::kLeft,
            1,
            modifiers,
            clock.Now(),
            WebMouseEvent::kMousePointerId,
        ));
        WebMouseEvent::FromGesture(
            Type::kMouseMove,
            gesture,
            Button::kLeft,
            1,
            modifiers,
            clock.Now(),
            pointer_id,
        )
    }
    // Source: mouse_event_manager.cc:798-809. Splitting only at the existing
    // resident hit-test boundary avoids publishing/copying the stack hit result.
    pub fn HandleDragDropIfPossibleWithPorts<
        H: MouseDragEvent + ?Sized,
        D: MouseDragPort + ?Sized,
    >(
        &self,
        ports: &MouseDragTransactionPorts<'_, D>,
        event: &H,
        primary_pointer_type: PointerType,
    ) -> DragHandlingResult {
        self.mouse_down_may_start_drag_.set(true);
        Self::ResetDragSourceWithDragPorts(ports);
        self.mouse_down_pos_.set(
            ports
                .view
                .ConvertIntPointFromRootFrame(Floored(event.DragEvent().PositionInRootFrame())),
        );
        self.HandleDragWithPorts(
            ports,
            event,
            if primary_pointer_type == PointerType::kPen {
                DragAndDropToolType::kStylusViaGesture
            } else {
                DragAndDropToolType::kFinger
            },
        )
    }

    pub fn HandleDragDropIfPossible<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        event: &H,
        primary: PointerType,
    ) -> DragHandlingResult {
        let selection = MouseDragSelectionBridge(ports.selection);
        let nodes = MouseDragNodeBridge(ports.nodes);
        let view = MouseDragFrameBridge(ports.view);
        let drag_ports = MouseDragTransactionPorts {
            nodes: &nodes,
            dispatch: ports.dispatch,
            view: &view,
            selection: &selection,
            drag: ports.drag,
            suppress_pointer_stream_after_drag: ports
                .features
                .SuppressPointerStreamAfterDragEnabled(),
        };
        self.HandleDragDropIfPossibleWithPorts(&drag_ports, event, primary)
    }
    // Source: mouse_event_manager.cc:811-816
    pub fn FocusDocumentView(&self, view: &dyn MouseFrameViewPort, focus: &dyn MouseFocusPort) {
        if view.HasPage() {
            focus.FocusDocumentView();
        }
    }

    // Source: mouse_event_manager.cc:818-922
    pub fn HandleMouseDraggedEvent<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        event: &H,
    ) -> WebInputEventResult {
        self.HandleMouseDraggedEventWithDefaultPorts(ports, event)
    }
    // Source: mouse_event_manager.cc:818-922.
    pub fn HandleMouseDraggedEventWithDefaultPorts<H: MouseDefaultHitTest + ?Sized>(
        &self,
        ports: &dyn MouseDraggedDefaultPort<H>,
        event: &H,
    ) -> WebInputEventResult {
        // TODO: TRACE_EVENT instrumentation (cc:820).
        let mouse = event.DefaultEvent();
        let is_pen = matches!(
            mouse.pointer_properties.pointer_type,
            PointerType::kPen | PointerType::kEraser
        );
        let pen_drag_button = if ports.BarrelButtonForDragEnabled() {
            Button::kBarrel
        } else {
            Button::kLeft
        };
        if (!is_pen && mouse.pointer_properties.button != Button::kLeft)
            || (is_pen && mouse.pointer_properties.button != pen_drag_button)
        {
            self.mouse_down_may_start_drag_.set(false);
            return WebInputEventResult::kNotHandled;
        }
        if !self.mouse_pressed_.get() || mouse.GetType() == Type::kMouseLeave {
            return WebInputEventResult::kNotHandled;
        }
        let should_handle_drag = !cfg!(target_os = "windows") || !is_pen;
        if should_handle_drag
            && ports.HandleDrag(
                self,
                event,
                if is_pen {
                    DragAndDropToolType::kStylusViaButton
                } else {
                    DragAndDropToolType::kMouse
                },
            ) != DragHandlingResult::kNotHandled
        {
            return WebInputEventResult::kHandledApplication;
        }
        let Some(target) = event.DefaultInnerNode() else {
            return WebInputEventResult::kNotHandled;
        };
        if ports.GetLayoutObject(&target).is_none() {
            let Some(parent) = ports.FlatTreeParent(&target) else {
                return WebInputEventResult::kNotHandled;
            };
            if ports.GetLayoutObject(&parent).is_none() || ports.IsMenuListSelect(&parent) {
                return WebInputEventResult::kNotHandled;
            }
        }
        ports.PrepareForMouseDrag();
        self.mouse_down_may_start_drag_.set(false);
        let result = ports.HandleSelectionMouseDragged(
            event,
            self.mouse_down_pos_.get(),
            self.last_known_mouse_position_in_root_frame_.get(),
        );
        // Selection dispatch may have replaced the resident LayoutObject.
        let layout = ports.GetLayoutObject(&target);
        if let Some(layout) = layout.filter(|_| {
            self.mouse_down_may_start_autoscroll_.get()
                && !ports.MiddleClickAutoscrollInProgress()
                && !ports.SelectedHtmlForClipboardIsEmpty()
                && ports.HasAutoscrollController()
        }) {
            ports.PrepareForSelectionAutoscroll(&layout);
            // The source lifecycle update may have replaced this object.
            if let Some(layout) = ports.GetLayoutObject(&target) {
                ports.StartAutoscrollForSelection(&layout);
                self.mouse_down_may_start_autoscroll_.set(false);
            }
        }
        result
    }

    // Source: mouse_event_manager.cc:924-1019
    pub fn HandleDragWithPorts<H: MouseDragEvent + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseDragTransactionPorts<'_, D>,
        event: &H,
        initiator: DragAndDropToolType,
    ) -> DragHandlingResult {
        debug_assert_eq!(event.DragEvent().GetType(), Type::kMouseMove);
        debug_assert!(ports.view.HasView());
        if !ports.view.HasPage() {
            return DragHandlingResult::kNotHandled;
        }
        if self.mouse_down_may_start_drag_.get() {
            if let Some(node) = ports
                .drag
                .HitTestMouseDownReadOnly(self.mouse_down_pos_.get())
            {
                let delayed = event.DragEvent().TimeStamp().ToInternalValue()
                    - self.mouse_down_timestamp_.get().ToInternalValue()
                    < kTextDragDelayMicros;
                let source = ports
                    .drag
                    .DraggableNode(&node, self.mouse_down_pos_.get(), delayed);
                ports.drag.SetDragSource(source.as_ref());
            } else {
                Self::ResetDragSourceWithDragPorts(ports);
            }
            if ports.drag.DragSource().is_none() {
                self.mouse_down_may_start_drag_.set(false);
            }
        }
        let initiated_by_button = matches!(
            initiator,
            DragAndDropToolType::kMouse | DragAndDropToolType::kStylusViaButton
        );
        if !self.mouse_down_may_start_drag_.get() {
            let suppressed = initiated_by_button
                && !ports.selection.MouseDownMayStartSelect()
                && !self.mouse_down_may_start_autoscroll_.get();
            return if suppressed {
                DragHandlingResult::kHandledDragNotStarted
            } else {
                DragHandlingResult::kNotHandled
            };
        }
        if initiated_by_button
            && !self.DragThresholdExceededWithDragView(
                ports.view,
                Floored(event.DragEvent().PositionInRootFrame()),
            )
        {
            Self::ResetDragSourceWithDragPorts(ports);
            return DragHandlingResult::kHandledDragNotStarted;
        }
        let started = self.TryStartDragWithPorts(ports, event);
        if !started {
            self.ClearDragDataTransferWithPorts(ports);
            Self::ResetDragSourceWithDragPorts(ports);
        } else {
            self.InvalidateClick();
            if ports.suppress_pointer_stream_after_drag {
                let pointer_type = match initiator {
                    DragAndDropToolType::kMouse => PointerType::kMouse,
                    DragAndDropToolType::kFinger => PointerType::kTouch,
                    _ => PointerType::kPen,
                };
                ports
                    .drag
                    .HandlePointerCausesUaAction(pointer_type, event.DragEvent().TimeStamp());
            } else if initiated_by_button {
                ports.drag.HandlePointerCausesUaAction(
                    PointerType::kMouse,
                    event.DragEvent().TimeStamp(),
                );
            }
            self.drag_initiator_.set(initiator);
        }
        self.mouse_down_may_start_drag_.set(false);
        if started {
            DragHandlingResult::kHandledDragStarted
        } else {
            DragHandlingResult::kHandledDragNotStarted
        }
    }

    // Source: mouse_event_manager.cc:1021-1025
    fn CreateDraggingDataTransfer<D: MouseDragPort + ?Sized>(
        &self,
        drag: &D,
    ) -> MouseDataTransferRef {
        drag.CreateWritableDragAndDropDataTransfer()
    }

    // Source: mouse_event_manager.cc:1027-1092
    fn TryStartDragWithPorts<H: MouseDragEvent + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseDragTransactionPorts<'_, D>,
        event: &H,
    ) -> bool {
        self.ClearDragDataTransferWithPorts(ports);
        ports
            .drag
            .SetDataTransfer(&self.CreateDraggingDataTransfer(ports.drag));
        if !ports.view.HasView()
            || !ports.drag.PopulateDragDataTransfer(
                self.mouse_down_pos_.get(),
                ports
                    .view
                    .ConvertIntPointFromRootFrame(Floored(event.DragEvent().PositionInRootFrame())),
            )
        {
            return false;
        }
        let mouse_down = self.mouse_down_.borrow().clone();
        if self.DispatchDragSrcEventWithPorts(ports, "dragstart", &mouse_down)
            != WebInputEventResult::kNotHandled
        {
            return false;
        }
        if !ports.view.HasPage() || ports.drag.DragSource().is_none() {
            return false;
        }
        if ports.drag.DragTypeIsSelection() && ports.selection.SelectionStartsInPasswordField() {
            return false;
        }
        ports
            .drag
            .DataTransfer()
            .expect("dragstart retains its DataTransfer owner")
            .SetAccessPolicyTypesReadable();
        if ports
            .drag
            .StartDrag(event.DragEvent(), self.mouse_down_pos_.get())
        {
            return true;
        }
        self.DispatchDragSrcEventWithPorts(ports, "dragend", event.DragEvent());
        false
    }

    // Source: mouse_event_manager.cc:1094-1105
    pub fn DispatchDragSrcEventWithPorts<D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseDragTransactionPorts<'_, D>,
        event_type: &str,
        event: &WebMouseEvent,
    ) -> WebInputEventResult {
        assert!(matches!(event_type, "drag" | "dragend" | "dragstart"));
        let source = ports.drag.DragSource();
        let transfer = ports.drag.DataTransfer();
        self.DispatchDragEventWithPorts(
            ports,
            event_type,
            source.as_ref(),
            None,
            event,
            transfer.as_ref(),
        )
    }

    // Source: mouse_event_manager.cc:1107-1172
    pub fn DispatchDragEventWithPorts<D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseDragTransactionPorts<'_, D>,
        event_type: &str,
        target: Option<&EventTargetRef>,
        related_target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
        transfer: Option<&MouseDataTransferRef>,
    ) -> WebInputEventResult {
        if !ports.view.HasView() {
            return WebInputEventResult::kNotHandled;
        }
        let target = target.expect("drag dispatch requires a live drag target");
        let related = related_target.filter(|related| ports.nodes.SameDocument(related, target));
        let mut initializer = MouseEventInit::default();
        let window = ports.dispatch.FrameDocumentWindow();
        let context = ports.dispatch.CoordinateContext();
        MouseEvent::SetCoordinatesFromWebPointerProperties(
            &event.FlattenTransform().pointer_properties,
            window.as_ref(),
            context.views.as_deref(),
            &mut initializer,
        );
        initializer.button = 0;
        initializer.buttons =
            MouseEvent::WebInputEventModifiersToButtons(event.GetModifiers() as u32);
        initializer.related_target = related.cloned();
        if matches!(event_type, "dragenter" | "dragover") {
            transfer
                .expect("drag target event has DataTransfer")
                .SetDestinationOperationFromEffectAllowed();
        } else if event_type == "dragleave" {
            transfer
                .expect("dragleave has DataTransfer")
                .SetDestinationOperationNone();
        }
        UIEventWithKeyState::SetFromWebInputEventModifiers(
            &mut initializer.event_modifier_init,
            event.GetModifiers(),
        );
        let ui = &mut initializer.event_modifier_init.ui_event_init;
        ui.event_init.bubbles = true;
        ui.event_init.cancelable = !matches!(event_type, "dragleave" | "dragend");
        ui.event_init.composed = true;
        ui.source_capabilities = window
            .as_ref()
            .map(|window| ports.dispatch.SourceCapabilities(window, event.FromTouch()));
        ui.view = window;
        let synthetic = if event.FromTouch() {
            SyntheticEventType::kFromTouch
        } else {
            SyntheticEventType::kRealOrIndistinguishable
        };
        let dispatched = ports.dispatch.CreateDragEvent(
            event_type,
            &initializer,
            transfer,
            event.TimeStamp(),
            synthetic,
        );
        let result = ToWebInputEventResult(ports.dispatch.DispatchEvent(target, &dispatched));
        if event_type == "dragleave" {
            transfer
                .expect("dragleave retains DataTransfer")
                .ResetDropEffect();
        }
        result
    }

    // Source: mouse_event_manager.cc:1174-1182
    pub fn ClearDragDataTransferWithPorts<D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseDragTransactionPorts<'_, D>,
    ) {
        if !ports.view.HasPage() {
            return;
        }
        if let Some(transfer) = ports.drag.DataTransfer() {
            transfer.ClearDragImage();
            transfer.SetAccessPolicyNumb();
        }
    }

    // Source: mouse_event_manager.cc:1184-1198
    pub fn DragSourceEndedAtWithPorts<D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseDragTransactionPorts<'_, D>,
        event: &WebMouseEvent,
        operation: DragOperation,
    ) {
        if ports.drag.DragSource().is_some() {
            ports.drag.SetDestinationOperation(operation);
            self.DispatchDragSrcEventWithPorts(ports, "dragend", event);
        }
        self.ReportDragEnd();
        self.ClearDragDataTransferWithPorts(ports);
        Self::ResetDragSourceWithDragPorts(ports);
        self.mouse_down_may_start_drag_.set(false);
    }

    /// Source: mouse_event_manager.cc:1205-1230. All source entry points use
    /// this algorithm; the default-action package owns only DragState storage.
    pub fn ResetDragSourceWithSourcePorts(ports: &dyn MouseDragSourceResetPort) {
        if !ports.HasPage() {
            return;
        }
        let Some(source) = ports.DragSource() else {
            return;
        };
        if !ports.DragSourceHasFrame(&source) || ports.DragSourceFrameIsDescendantOfFrame(&source) {
            ports.SetDragSource(None);
        }
    }
    fn ResetDragSourceWithDragPorts<D: MouseDragPort + ?Sized>(
        ports: &MouseDragTransactionPorts<'_, D>,
    ) {
        Self::ResetDragSourceWithSourcePorts(&MouseDragSourceReset {
            has_page: ports.view.HasPage(),
            drag: ports.drag,
        });
    }

    fn HandleDrag<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        event: &H,
        initiator: DragAndDropToolType,
    ) -> DragHandlingResult {
        let selection = MouseDragSelectionBridge(ports.selection);
        let nodes = MouseDragNodeBridge(ports.nodes);
        let view = MouseDragFrameBridge(ports.view);
        let drag_ports = MouseDragTransactionPorts {
            nodes: &nodes,
            dispatch: ports.dispatch,
            view: &view,
            selection: &selection,
            drag: ports.drag,
            suppress_pointer_stream_after_drag: ports
                .features
                .SuppressPointerStreamAfterDragEnabled(),
        };
        self.HandleDragWithPorts(&drag_ports, event, initiator)
    }
    pub fn DispatchDragSrcEvent<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        event_type: &str,
        event: &WebMouseEvent,
    ) -> WebInputEventResult {
        let selection = MouseDragSelectionBridge(ports.selection);
        let nodes = MouseDragNodeBridge(ports.nodes);
        let view = MouseDragFrameBridge(ports.view);
        let drag_ports = MouseDragTransactionPorts {
            nodes: &nodes,
            dispatch: ports.dispatch,
            view: &view,
            selection: &selection,
            drag: ports.drag,
            suppress_pointer_stream_after_drag: ports
                .features
                .SuppressPointerStreamAfterDragEnabled(),
        };
        self.DispatchDragSrcEventWithPorts(&drag_ports, event_type, event)
    }
    pub fn DispatchDragEvent<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        event_type: &str,
        target: Option<&EventTargetRef>,
        related_target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
        transfer: Option<&MouseDataTransferRef>,
    ) -> WebInputEventResult {
        let selection = MouseDragSelectionBridge(ports.selection);
        let nodes = MouseDragNodeBridge(ports.nodes);
        let view = MouseDragFrameBridge(ports.view);
        let drag_ports = MouseDragTransactionPorts {
            nodes: &nodes,
            dispatch: ports.dispatch,
            view: &view,
            selection: &selection,
            drag: ports.drag,
            suppress_pointer_stream_after_drag: ports
                .features
                .SuppressPointerStreamAfterDragEnabled(),
        };
        self.DispatchDragEventWithPorts(
            &drag_ports,
            event_type,
            target,
            related_target,
            event,
            transfer,
        )
    }
    fn ClearDragDataTransfer<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
    ) -> () {
        let selection = MouseDragSelectionBridge(ports.selection);
        let nodes = MouseDragNodeBridge(ports.nodes);
        let view = MouseDragFrameBridge(ports.view);
        let drag_ports = MouseDragTransactionPorts {
            nodes: &nodes,
            dispatch: ports.dispatch,
            view: &view,
            selection: &selection,
            drag: ports.drag,
            suppress_pointer_stream_after_drag: ports
                .features
                .SuppressPointerStreamAfterDragEnabled(),
        };
        self.ClearDragDataTransferWithPorts(&drag_ports)
    }
    pub fn DragSourceEndedAt<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
        event: &WebMouseEvent,
        operation: DragOperation,
    ) -> () {
        let selection = MouseDragSelectionBridge(ports.selection);
        let nodes = MouseDragNodeBridge(ports.nodes);
        let view = MouseDragFrameBridge(ports.view);
        let drag_ports = MouseDragTransactionPorts {
            nodes: &nodes,
            dispatch: ports.dispatch,
            view: &view,
            selection: &selection,
            drag: ports.drag,
            suppress_pointer_stream_after_drag: ports
                .features
                .SuppressPointerStreamAfterDragEnabled(),
        };
        self.DragSourceEndedAtWithPorts(&drag_ports, event, operation)
    }
    // Source: mouse_event_manager.cc:1200-1203
    // GetDragState is deliberately not exported as a broad mutable owner.
    // The borrowed DragController port above performs each original field
    // read/write in source order against that same state.

    // Source: mouse_event_manager.cc:1205-1230
    fn ResetDragSource<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized>(
        &self,
        ports: &MouseManagerPorts<'_, H, D>,
    ) {
        ports.ResetDragSource();
    }

    // Source: mouse_event_manager.cc:1232-1246
    fn DragThresholdExceeded(&self, view: &dyn MouseFrameViewPort, location: PointF) -> bool {
        self.DragThresholdExceededWithDragView(&MouseDragFrameBridge(view), location)
    }
    fn DragThresholdExceededWithDragView(
        &self,
        view: &dyn MouseDragFramePort,
        location_in_root: PointF,
    ) -> bool {
        if !view.HasView() {
            return false;
        }
        let location = view.ConvertIntPointFromRootFrame(location_in_root);
        let down = self.mouse_down_pos_.get();
        (location.x() as i32 - down.x() as i32).unsigned_abs() >= kDragThresholdX as u32
            || (location.y() as i32 - down.y() as i32).unsigned_abs() >= kDragThresholdY as u32
    }

    // Source: mouse_event_manager.cc:1256-1263
    pub fn HandleSvgPanIfNeeded(&self, view: &dyn MouseFrameViewPort, is_release: bool) -> bool {
        if !self.svg_pan_.get() {
            return false;
        }
        self.svg_pan_.set(!is_release);
        view.UpdateSvgPan(view.ViewportToFrame(self.last_known_mouse_position_.get()));
        true
    }
}

// Source: mouse_event_manager.cc:650,675,706,729,973,1040
// Mechanical projection of gfx::ToFlooredPoint at the source call sites.
fn Floored(point: PointF) -> PointF {
    PointF::new(
        point.x().floor() as i32 as f32,
        point.y().floor() as i32 as f32,
    )
}
// Preserve the native assembly path while the factory owns pointer spelling.
pub(crate) use interaction_dispatch::core::events::pointer_event_factory::PointerTypeName;

impl MouseEventManagerState {
    // Source: mouse_event_manager.cc:659-661
    pub fn IsMousePositionUnknown(&self) -> bool {
        self.is_mouse_position_unknown_.get()
    }
    // Source: mouse_event_manager.cc:663-665
    pub fn LastKnownMousePositionInViewport(&self) -> PointF {
        self.last_known_mouse_position_.get()
    }
    pub fn LastKnownMousePositionInRootFrame(&self) -> PointF {
        self.last_known_mouse_position_in_root_frame_.get()
    }
    // Source: mouse_event_manager.cc:667-669
    pub fn LastKnownMouseScreenPosition(&self) -> PointF {
        self.last_known_mouse_screen_position_.get()
    }
    // Source: mouse_event_manager.cc:671-678
    pub fn SetLastKnownMousePosition(&self, event: &WebMouseEvent) {
        self.is_mouse_position_unknown_
            .set(event.GetType() == Type::kMouseLeave);
        self.last_known_mouse_position_in_root_frame_
            .set(Floored(event.PositionInRootFrame()));
        self.last_known_mouse_position_
            .set(*event.pointer_properties.PositionInWidget());
        self.last_known_mouse_screen_position_
            .set(*event.pointer_properties.PositionInScreen());
    }
    // Source: mouse_event_manager.cc:680-682
    pub fn SetLastMousePositionAsUnknown(&self) {
        self.is_mouse_position_unknown_.set(true);
    }
    // Source: mouse_event_manager.cc:1248-1254
    pub fn ClearDragHeuristicState(&self) {
        self.mouse_pressed_.set(false);
        self.mouse_down_may_start_drag_.set(false);
        self.mouse_down_may_start_autoscroll_.set(false);
    }
    // Source: mouse_event_manager.cc:1265-1268
    pub fn InvalidateClick(&self) {
        self.click_count_.set(0);
        self.mousedown_element_.replace(None);
    }
    // Source: mouse_event_manager.cc:1270-1272
    pub fn MousePressed(&self) -> bool {
        self.mouse_pressed_.get()
    }
    // Source: mouse_event_manager.cc:1274-1276
    pub fn ReleaseMousePress(&self) {
        self.mouse_pressed_.set(false);
    }
    // Source: mouse_event_manager.cc:1278-1280
    pub fn MousePressNode(&self) -> Option<EventTargetRef> {
        self.mouse_press_node_.borrow().clone()
    }
    // Source: mouse_event_manager.cc:1282-1284
    pub fn SetMousePressNode(&self, node: Option<&EventTargetRef>) {
        self.mouse_press_node_.replace(node.cloned());
    }
    // Source: mouse_event_manager.cc:1286-1288
    pub fn SetMouseDownElement(&self, node: Option<&EventTargetRef>) {
        self.mousedown_element_.replace(node.cloned());
    }
    // Source: mouse_event_manager.cc:1290-1292
    pub fn SetClickCount(&self, count: i32) {
        self.click_count_.set(count);
    }
    // Source: mouse_event_manager.cc:1294-1296
    pub fn MouseDownMayStartDrag(&self) -> bool {
        self.mouse_down_may_start_drag_.get()
    }
    // Source: mouse_event_manager.cc:1298-1301
    pub fn ReportDragEnd(&self) {
        // TODO: UmaHistogramEnumeration("Event.DragDrop.Tool", drag_initiator_).
        self.drag_initiator_.set(DragAndDropToolType::kUnknown);
    }
    // Source: mouse_event_manager.h:155-157
    pub fn SetMouseDownMayStartAutoscroll(&self) {
        self.mouse_down_may_start_autoscroll_.set(true);
    }
}

impl MouseEventManagerState {
    // Source: mouse_event_manager.cc:640-657; the frame projection supplies
    // the same converted integer point, or the source's absent-view branch.
    pub fn HandleMousePressEventUpdateStates(
        &self,
        event: &WebMouseEvent,
        mouse_down: Option<PointF>,
    ) {
        self.mouse_pressed_.set(true);
        self.SetLastKnownMousePosition(event);
        self.mouse_down_may_start_drag_.set(false);
        self.mouse_down_may_start_autoscroll_.set(false);
        self.mouse_down_timestamp_.set(event.TimeStamp());
        if let Some(point) = mouse_down {
            self.mouse_down_pos_.set(point);
        } else {
            self.InvalidateClick();
        }
    }
    // Source: mouse_event_manager.cc:687-693. Shared by the full source press
    // handler and its call-scoped Page ports.
    pub fn HandleMousePressEventState(
        &self,
        event: &WebMouseEvent,
        selection_over_link: bool,
        extending_selection: bool,
    ) {
        self.mouse_down_may_start_drag_
            .set(event.click_count <= 1 && !selection_over_link && !extending_selection);
        self.mouse_down_.replace(event.clone());
    }
}

impl<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized> MousePressDefaultPort<H>
    for MouseManagerPorts<'_, H, D>
{
    fn ResetDragSource(&self) {
        MouseEventManager::ResetDragSourceWithSourcePorts(&MouseDragSourceReset {
            has_page: self.view.HasPage(),
            drag: self.drag,
        });
    }
    fn IsSvgDocumentWithZoomAndPan(&self) -> bool {
        self.view.IsSvgDocumentWithZoomAndPan()
    }
    fn ConvertFromRootFrame(&self, p: PointF) -> PointF {
        self.view.ConvertFromRootFrame(p)
    }
    fn StartSvgPan(&self, p: PointF) {
        self.view.StartSvgPan(p)
    }
    fn FocusDocumentView(&self) {
        if self.view.HasPage() {
            self.focus.FocusDocumentView();
        }
    }
    fn SetSequentialFocusNavigationStartingPoint(&self, n: Option<&EventTargetRef>) {
        self.focus.SetSequentialFocusNavigationStartingPoint(n)
    }
    fn HandleSelectionMousePress(&self, e: &H) -> bool {
        self.selection.HandleMousePressEvent(e)
    }
    fn MouseDownMayStartSelect(&self) -> bool {
        self.selection.MouseDownMayStartSelect()
    }
    fn HasUserScrollableLayoutBox(&self, n: &EventTargetRef) -> bool {
        self.layout.HasUserScrollableLayoutBox(n)
    }
}
impl<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized> MouseReleaseDefaultPort<H>
    for MouseManagerPorts<'_, H, D>
{
    fn StopSelectionAutoscrollIfNeeded(&self) {
        if self.autoscroll.HasAutoscrollController()
            && self.autoscroll.SelectionAutoscrollInProgress()
        {
            self.autoscroll.StopAutoscroll();
        }
    }
    fn HandleSelectionMouseRelease(&self, e: &H, p: PointF) -> bool {
        self.selection.HandleMouseReleaseEvent(e, p)
    }
}

impl<H: MouseHitTest + ?Sized, D: MouseDragPort + ?Sized> MouseDraggedDefaultPort<H>
    for MouseManagerPorts<'_, H, D>
{
    fn BarrelButtonForDragEnabled(&self) -> bool {
        self.features.BarrelButtonForDragEnabled()
    }
    fn HandleDrag(
        &self,
        manager: &MouseEventManager,
        event: &H,
        initiator: DragAndDropToolType,
    ) -> DragHandlingResult {
        manager.HandleDrag(self, event, initiator)
    }
    fn FlatTreeParent(&self, n: &EventTargetRef) -> Option<EventTargetRef> {
        self.nodes.FlatTreeParent(n)
    }
    fn GetLayoutObject(&self, n: &EventTargetRef) -> Option<MouseDragLayoutRef> {
        self.layout.GetLayoutObject(n)
    }
    fn IsMenuListSelect(&self, n: &EventTargetRef) -> bool {
        self.layout.IsMenuListSelect(n)
    }
    fn PrepareForMouseDrag(&self) {
        self.lifecycle.PrepareForMouseDrag()
    }
    fn HandleSelectionMouseDragged(
        &self,
        e: &H,
        down: PointF,
        last: PointF,
    ) -> WebInputEventResult {
        self.selection.HandleMouseDraggedEvent(e, down, last)
    }
    fn MiddleClickAutoscrollInProgress(&self) -> bool {
        self.autoscroll.MiddleClickAutoscrollInProgress()
    }
    fn SelectedHtmlForClipboardIsEmpty(&self) -> bool {
        self.selection.SelectedHtmlForClipboardIsEmpty()
    }
    fn HasAutoscrollController(&self) -> bool {
        self.autoscroll.HasAutoscrollController()
    }
    fn PrepareForSelectionAutoscroll(&self, layout: &MouseDragLayoutRef) {
        self.lifecycle.PrepareForSelectionAutoscroll(layout)
    }
    fn StartAutoscrollForSelection(&self, layout: &MouseDragLayoutRef) {
        self.autoscroll.StartAutoscrollForSelection(layout)
    }
}
