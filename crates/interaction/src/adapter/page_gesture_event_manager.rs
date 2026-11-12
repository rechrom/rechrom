//! Call-scoped GestureManager ports over the existing pointer/selection owners.
//! Targeting follows EventHandler.cc; no resident Layout loan crosses script.
#[path = "page_event_handler_managers.rs"]
pub mod event_handler_managers;
use super::pointer_input::{PagePointerContext, PointerInputError};
use super::*;
use crate::adapter::{
    page_pointer_hit_test::{PagePointerGeometry, PointerHit, PointerHitResult},
    page_pointer_scroll_controls::PagePointerScrollControlOwners,
    source_input_managers::ResidentCursorOwners,
};
use crate::core::input::{
    gesture_manager::{
        self, GestureEventWithHitTestResults, GestureFeatures, GestureFrame, GestureHitResult,
        GestureManager, GestureMouse, GesturePointer, GesturePointerIdLookup,
        GestureScrollableArea, GestureSelection, GestureSettings,
    },
    gesture_targeting::{self, GestureTargetQuery},
    mouse_event_manager::{DragHandlingResult, MouseClickDispatchPort},
    pointer_event_manager::{
        PointerEventDOMPort, PointerEventEnvironment, PointerEventManager, PointerMouseManager,
    },
};
use foundation::{gfx, PhysicalOffset, PhysicalSize};
use interaction_dispatch::core::{
    events::pointer_event_factory::PointerTargetRef,
    input::event_handling_util::{ToWebInputEventResult, WebInputEventResult},
};
use interaction_hit_test::core::{
    layout::{hit_test_location::HitTestLocation, hit_test_request::RequestType},
    page::touch_adjustment::TouchAdjustmentCandidateType,
};
use interaction_input::{
    base::time::time::TimeTicks,
    public::common::input::{
        web_gesture_event::WebGestureEvent,
        web_input_event::{Type, WebInputEvent},
        web_pointer_properties::{Button, PointerType},
    },
};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct GestureTimerTask {
    pub document_identity: u128,
    pub generation: u64,
}
pub trait GestureTimerServices {
    fn StartActiveInterval(&self, delay: Duration, task: GestureTimerTask) -> Result<(), String>;
    fn StopActiveInterval(&self);
    fn ScrollableAreas(&self) -> Vec<Rc<dyn GestureScrollableArea>>;
}
/// ChromeClientImpl's optional Autofill/RecordReplay clients and the optional
/// LocalFrameView TapFriendlinessChecker retain their original source absence.
pub trait GestureNodeObserver {
    fn DidReceiveGestureTap(&self, node: u64);
}
#[derive(Default)]
pub struct GestureObservers {
    pub autofill: Option<Rc<dyn GestureNodeObserver>>,
    pub record_replay: Option<Rc<dyn GestureNodeObserver>>,
    pub tap_friendliness: Option<Rc<dyn GestureNodeObserver>>,
}
pub struct GestureTargetState {
    last_show_press: Cell<Option<TimeTicks>>,
    deferred_target: RefCell<Option<EventTargetRef>>,
    use_adjusted_touch: Cell<bool>,
    timer_generation: Cell<u64>,
    pub timer: RefCell<Option<Rc<dyn GestureTimerServices>>>,
    pub settings: Cell<GestureSettings>,
    pub observers: RefCell<GestureObservers>,
}
/// Source EventHandler::ActiveIntervalTimerFired, event_handler.cc:2438-2450.
pub(crate) fn RunGestureTimerTask(
    dispatcher: &EventDispatcher<'_, '_>,
    owners: &ResidentCursorOwners,
    task: GestureTimerTask,
) -> Result<(), String> {
    if dispatcher
        .document
        .borrow()
        .GetDocument()
        .RootHandle()
        .Identity()
        != task.document_identity
        || owners.gesture_target.timer_generation.get() != task.generation
    {
        return Ok(());
    }
    let target = owners.gesture_target.deferred_target.borrow_mut().take();
    if let Some(target) = target {
        let target = dispatcher.targets.NodeId(&target);
        let update = dispatcher
            .document
            .borrow_mut()
            .GetDocumentMut()
            .UpdateHoverActiveState(false, true, target);
        let mut state = dispatcher.state.borrow_mut();
        state.hovered_node_id = update.hovered_node_id;
        state.pressed_node_id = update.active_node_id;
    }
    Ok(())
}
impl Default for GestureTargetState {
    fn default() -> Self {
        Self {
            last_show_press: Cell::new(None),
            deferred_target: Default::default(),
            use_adjusted_touch: Cell::new(false),
            timer_generation: Cell::new(0),
            timer: Default::default(),
            settings: Cell::new(GestureSettings {
                modal_context_menu: true,
                ..Default::default()
            }),
            observers: Default::default(),
        }
    }
}
impl GestureTargetState {
    // EventHandler::NotifyElementActivated, cc:2452-2457.
    pub(crate) fn NotifyElementActivated(&self) {
        self.deferred_target.borrow_mut().take();
        self.timer_generation
            .set(self.timer_generation.get().wrapping_add(1));
        let timer = self.timer.borrow().clone();
        if let Some(timer) = timer {
            timer.StopActiveInterval();
        }
    }
    pub(crate) fn Clear(&self) {
        self.last_show_press.set(None);
        self.deferred_target.borrow_mut().take();
        self.use_adjusted_touch.set(false);
        self.timer_generation
            .set(self.timer_generation.get().wrapping_add(1));
        if let Some(timer) = self.timer.borrow().as_ref() {
            timer.StopActiveInterval();
        }
    }
}
#[derive(Clone)]
pub(crate) struct PageGestureHit {
    pub(crate) result: PointerHitResult,
    document: InteractionDocument,
    targets: Rc<DomEventTargets>,
    controls: Rc<PagePointerScrollControlOwners>,
    error: Rc<RefCell<Option<PointerInputError>>>,
}
impl PageGestureHit {
    fn Resolve(
        &self,
        node: Option<&interaction_hit_test::core::layout::hit_test_result::NodeRef>,
    ) -> Option<EventTargetRef> {
        let node = node?;
        let id = node.DOMNodeId()?;
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let Some(index) = document.FindNodeById(id) else {
            return None;
        };
        if document.NodeHandle(index).Identity() != node.DOMIdentity() {
            self.error
                .borrow_mut()
                .get_or_insert(PointerInputError::LocalFrameRouting);
            return None;
        }
        self.targets.TargetFromBorrowedDocument(document, id)
    }
}
impl GestureHitResult for PageGestureHit {
    fn InnerNode(&self) -> Option<EventTargetRef> {
        self.Resolve(self.result.InnerNode())
    }
    fn InnerElement(&self) -> Option<EventTargetRef> {
        self.Resolve(self.result.InnerElement())
    }
    fn InnerPossiblyPseudoElement(&self) -> Option<EventTargetRef> {
        self.Resolve(self.result.InnerPossiblyPseudoElement())
    }
    fn InnerPossiblyPseudoNode(&self) -> Option<EventTargetRef> {
        self.Resolve(self.result.InnerPossiblyPseudoNode())
    }
    fn HasScrollbar(&self) -> bool {
        self.result.GetScrollbar().is_some()
    }
    fn HandleScrollbarGestureTapOrPress(&self, event: &WebGestureEvent) -> bool {
        let Some(owner) = self
            .result
            .GetScrollbar()
            .and_then(|s| s.NativeScrollbarOwner())
            .cloned()
        else {
            return false;
        };
        match self.controls.HandleGestureTapOrPress(owner, event) {
            Ok(handled) => handled,
            Err(_) => {
                self.error
                    .borrow_mut()
                    .get_or_insert(PointerInputError::SourceLeaf(
                        "Scrollbar::HandleGestureTapOrPress",
                    ));
                false
            }
        }
    }
    fn SetToShadowHostIfInUAShadowRoot(&mut self) {
        self.result.SetToShadowHostIfInUAShadowRoot();
    }
    fn HasURLElement(&self) -> bool {
        self.result.URLElement().is_some()
    }
    fn HasImageURL(&self) -> bool {
        self.result.AbsoluteImageURL().is_some()
    }
    fn HasMediaURL(&self) -> bool {
        self.result.AbsoluteMediaURL().is_some()
    }
    fn HasInnerNodeFrame(&self) -> bool {
        self.result.InnerNodeFrame().is_some()
    }
}
struct Context<'scope, 'call, 'listener, 'frame> {
    pointer: &'scope PagePointerContext<'call, 'listener, 'frame>,
    errors: Rc<RefCell<Option<PointerInputError>>>,
}
impl Context<'_, '_, '_, '_> {
    fn Fault(&self, name: &'static str) {
        self.pointer.Fault(PointerInputError::SourceLeaf(name));
        if self.errors.borrow().is_none() {
            self.errors
                .borrow_mut()
                .get_or_insert(PointerInputError::SourceLeaf(name));
        }
    }
    fn Hit(&self, location: HitTestLocation, bits: u32) -> PageGestureHit {
        let result = match self.pointer.geometry.HitTest(location, bits) {
            Ok(hit) => hit.result,
            Err(error) => {
                self.pointer.Fault(PointerInputError::Geometry(error));
                PointerHitResult::default()
            }
        };
        PageGestureHit {
            result,
            document: self.pointer.dispatcher.document.clone(),
            targets: self.pointer.dispatcher.targets.clone(),
            controls: self.pointer.controls.clone(),
            error: self.errors.clone(),
        }
    }
    fn Record(&self, result: DispatchEventResult) -> WebInputEventResult {
        self.pointer.result.set(Some(result));
        if result.default_prevented {
            WebInputEventResult::kHandledApplication
        } else {
            WebInputEventResult::kNotHandled
        }
    }
    fn UpdateHoverActive(&self, active: bool, target: Option<EventTargetRef>) {
        let id = target
            .as_ref()
            .and_then(|target| self.pointer.dispatcher.targets.NodeId(target));
        let update = self
            .pointer
            .dispatcher
            .document
            .borrow_mut()
            .GetDocumentMut()
            .UpdateHoverActiveState(active, true, id);
        if update.notify_element_activated {
            self.pointer.owners.gesture_target.NotifyElementActivated();
        }
        let mut state = self.pointer.dispatcher.state.borrow_mut();
        state.hovered_node_id = update.hovered_node_id;
        state.pressed_node_id = update.active_node_id;
    }
}
impl crate::core::input::event_handler::LocatedContextMenuSource for Context<'_, '_, '_, '_> {
    type Target = EventTargetRef;
    type Hit = PageGestureHit;
    fn HasView(&self) -> bool {
        self.pointer.frame.HasFrameView()
    }
    fn ReleaseMousePress(&self) {
        self.pointer.owners.mouse.ReleaseMousePress();
    }
    fn LastScrollbarMouseUp(&self, event: &WebMouseEvent) {
        if let Some(scrollbar) = self.pointer.owners.event_handler.LastScrollbarUnderMouse() {
            scrollbar.MouseUp(event);
        }
    }
    fn HitTestContextMenu(&self, event: &WebMouseEvent) -> PageGestureHit {
        let root = event.PositionInRootFrame();
        let floored = PointF::new(root.x().floor(), root.y().floor());
        let point = self
            .pointer
            .frame
            .ConvertFromRootFrame(floored)
            .unwrap_or_else(|| {
                self.Fault("LocalFrameView::ConvertFromRootFrame context menu");
                PointF::default()
            });
        let hit = self.Hit(
            HitTestLocation::FromPointF(point),
            RequestType::kActive as u32,
        );
        self.UpdateHoverActive(true, hit.InnerPossiblyPseudoElement());
        hit
    }
    fn UpdateStyleAndLayoutForContextMenu(&self) {
        if let Err(error) = self.pointer.geometry.PrepareLifecycleExceptPaint() {
            self.pointer.Fault(PointerInputError::Geometry(error));
        }
    }
    fn InnerPossiblyPseudoElement(&self, hit: &PageGestureHit) -> Option<EventTargetRef> {
        hit.InnerPossiblyPseudoElement()
    }
    fn DispatchContextMenuEvent(
        &self,
        target: Option<EventTargetRef>,
        event: &WebMouseEvent,
    ) -> WebInputEventResult {
        let Some(target) =
            target.and_then(|target| self.pointer.dispatcher.targets.NodeId(&target))
        else {
            return WebInputEventResult::kNotHandled;
        };
        if self.pointer.error.borrow().is_some() {
            return WebInputEventResult::kNotHandled;
        }
        self.Record(self.pointer.dispatcher.DispatchNativeMouseEvent(
            &self.pointer.owners.mouse,
            target,
            "contextmenu",
            event,
        ))
    }
}
impl GestureTargetQuery<PageGestureHit> for Context<'_, '_, '_, '_> {
    type Error = PointerInputError;
    fn ShouldApplyTouchAdjustment(&self, e: &WebGestureEvent) -> bool {
        gesture_targeting::ShouldApplyTouchAdjustment(e)
    }
    fn GestureCorrespondsToAdjustedTouch(&self, e: &WebGestureEvent) -> bool {
        let state = &self.pointer.owners.gesture_target;
        gesture_targeting::GestureCorrespondsToAdjustedTouch(
            e,
            &state.use_adjusted_touch,
            self.pointer
                .owners
                .event_handler
                .TouchAdjustmentUniqueEventId(),
            self.pointer.owners.event_handler.TouchAdjustmentPoint(),
        )
    }
    fn AdjustedTouchPoint(&self) -> gfx::PointF {
        self.pointer.owners.event_handler.TouchAdjustmentPoint()
    }
    fn HitTestRectSize(&self, w: f32, h: f32) -> PhysicalSize {
        self.pointer.geometry.GetHitTestRectForAdjustment(w, h)
    }
    fn HitTest(&self, location: HitTestLocation, bits: u32) -> Result<PageGestureHit, Self::Error> {
        Ok(self.Hit(location, bits))
    }
    fn BestNodeForHitTestResult(
        &self,
        kind: TouchAdjustmentCandidateType,
        location: &HitTestLocation,
        result: &PageGestureHit,
    ) -> Result<Option<gfx::Point>, Self::Error> {
        self.pointer
            .geometry
            .BestTouchAdjustment(
                kind,
                &PointerHit {
                    location: location.clone(),
                    result: result.result.clone(),
                },
                self.pointer,
            )
            .map(|value| value.map(|(_, point)| point))
            .map_err(PointerInputError::Geometry)
    }
}
impl gesture_targeting::GestureTargetingQuery<PageGestureHit> for Context<'_, '_, '_, '_> {
    fn HitTypeForGesture(&self, kind: Type) -> u32 {
        gesture_manager::GetHitTypeForGestureType(kind, self.ActiveElementExists())
    }
    fn Now(&self) -> TimeTicks {
        self.pointer.dispatcher.clock.Now()
    }
    fn LastShowPressTimestamp(&self) -> Option<TimeTicks> {
        self.pointer.owners.gesture_target.last_show_press.get()
    }
    fn UpdateHoverActiveState(&self, active: bool, result: &PageGestureHit) {
        self.UpdateHoverActive(active, result.InnerPossiblyPseudoElement());
    }
    fn DeferTapActiveState(&self, result: &PageGestureHit, delay: u64) -> Result<(), Self::Error> {
        let state = &self.pointer.owners.gesture_target;
        state.deferred_target.replace(result.InnerElement());
        let generation = state.timer_generation.get().wrapping_add(1);
        state.timer_generation.set(generation);
        let document_identity = self
            .pointer
            .dispatcher
            .document
            .borrow()
            .GetDocument()
            .RootHandle()
            .Identity();
        let timer = state
            .timer
            .borrow()
            .clone()
            .ok_or(PointerInputError::SourceLeaf(
                "EventHandler active interval timer",
            ))?;
        timer
            .StartActiveInterval(
                Duration::from_micros(delay),
                GestureTimerTask {
                    document_identity,
                    generation,
                },
            )
            .map_err(|_| PointerInputError::SourceLeaf("EventHandler active interval timer"))
    }
}
impl GestureFrame<PageGestureHit> for Context<'_, '_, '_, '_> {
    fn ActiveElementExists(&self) -> bool {
        self.pointer
            .dispatcher
            .state
            .borrow()
            .pressed_node_id
            .is_some()
    }
    fn Settings(&self) -> Option<GestureSettings> {
        Some(self.pointer.owners.gesture_target.settings.get())
    }
    fn Features(&self) -> GestureFeatures {
        GestureFeatures {
            touch_text_editing_redesign:
                foundation::RuntimeEnabledFeatures::TouchTextEditingRedesignEnabled(),
            touch_drag_on_short_press:
                foundation::RuntimeEnabledFeatures::TouchDragOnShortPressEnabled(),
            touch_drag_and_context_menu:
                foundation::RuntimeEnabledFeatures::TouchDragAndContextMenuEnabled(),
            text_fragment_tap_opens_context_menu:
                foundation::RuntimeEnabledFeatures::TextFragmentTapOpensContextMenuEnabled(),
            enable_unhandled_tap: false,
        }
    }
    fn HasView(&self) -> bool {
        self.pointer.frame.HasFrameView()
    }
    fn HasPage(&self) -> bool {
        self.pointer
            .dispatcher
            .document
            .borrow()
            .GetDocument()
            .IsActive()
    }
    fn IsLocalRoot(&self) -> bool {
        true
    }
    fn LocalRootGestureManager(&self) -> Rc<dyn GesturePointerIdLookup> {
        unreachable!("the Page binder is the local root")
    }
    fn ConvertFromRootFrame(&self, point: PointF) -> HitTestLocation {
        HitTestLocation::FromPointF(
            self.pointer
                .frame
                .ConvertFromRootFrame(point)
                .unwrap_or(point),
        )
    }
    fn HitTestResultInFrame(&self, location: &HitTestLocation, bits: u32) -> PageGestureHit {
        self.Hit(location.clone(), bits)
    }
    fn HitTestResultAtLocation(&self, location: &HitTestLocation) -> PageGestureHit {
        self.Hit(
            location.clone(),
            RequestType::kReadOnly as u32 | RequestType::kActive as u32,
        )
    }
    fn UpdateRootLifecycleExceptPaint(&self) -> bool {
        match self.pointer.geometry.PrepareLifecycleExceptPaint() {
            Ok(()) => true,
            Err(error) => {
                self.pointer.Fault(PointerInputError::Geometry(error));
                false
            }
        }
    }
    fn RootHasView(&self) -> bool {
        self.HasView()
    }
    fn UpdateInnerNodeFrameLifecycleExceptPaint(&self, _: &PageGestureHit) {
        self.UpdateRootLifecycleExceptPaint();
    }
    fn IsOverSharedHighlight(&self, _: &PageGestureHit) -> bool {
        self.Fault("TextFragmentHandler::IsOverSharedHighlight");
        false
    }
    fn DomTreeVersion(&self) -> u64 {
        self.pointer
            .dispatcher
            .document
            .borrow()
            .GetDocument()
            .DomTreeVersion()
    }
    fn StyleVersion(&self) -> u64 {
        self.pointer
            .dispatcher
            .document
            .borrow()
            .GetDocument()
            .StyleVersion()
    }
    fn ViewForNode(&self, node: &EventTargetRef) -> Option<AbstractViewRef> {
        PointerEventDOMPort::ViewForElement(self.pointer, node)
    }
    fn DispatchEvent(
        &self,
        target: &EventTargetRef,
        event: &EventRef,
    ) -> NativeDispatchEventResult {
        self.pointer.DispatchCanonical(target, event.clone())
    }
    fn NotifyUserActivation(&self, _: Option<&EventTargetRef>) {
        PointerEventEnvironment::NotifyUserActivation(
            self.pointer,
            &self.pointer.owners.pointer_frame.identity,
        );
    }
    fn HasLayoutObject(&self, node: &EventTargetRef) -> bool {
        self.pointer
            .dispatcher
            .targets
            .NodeId(node)
            .is_some_and(|id| self.pointer.frame.HasLayoutObject(id))
    }
    fn DidDispatchMouseDown(&self, node: &EventTargetRef) {
        if let Some(id) = self.pointer.dispatcher.targets.NodeId(node) {
            let (autofill, replay) = {
                let clients = self.pointer.owners.gesture_target.observers.borrow();
                (clients.autofill.clone(), clients.record_replay.clone())
            };
            if let Some(client) = autofill {
                client.DidReceiveGestureTap(id);
            }
            if let Some(client) = replay {
                client.DidReceiveGestureTap(id);
            }
        }
    }
    fn CommonAncestorForClick(
        &self,
        a: &EventTargetRef,
        b: &EventTargetRef,
    ) -> Option<EventTargetRef> {
        MouseClickDispatchPort::CommonAncestorForClick(&self.pointer.MouseDispatch(), a, b)
    }
    fn AsElement(&self, node: Option<EventTargetRef>) -> Option<EventTargetRef> {
        node.filter(|n| PointerEventDOMPort::IsElement(self.pointer, n))
    }
    fn ClientCoordinates(&self, event: &WebMouseEvent) -> (f64, f64) {
        let window = self.pointer.dispatcher.targets.WindowView();
        let coordinates = self.pointer.dispatcher.MouseCoordinateContext();
        let mut initializer =
            interaction_dispatch::core::events::mouse_event_init::MouseEventInit::default();
        interaction_dispatch::core::events::mouse_event::MouseEvent::SetCoordinatesFromWebPointerProperties(
            &event.pointer_properties,Some(&window),coordinates.views.as_deref(),&mut initializer);
        (initializer.client_x, initializer.client_y)
    }
    fn RegisterTapEvent(&self, node: &EventTargetRef) {
        if let Some(id) = self.pointer.dispatcher.targets.NodeId(node) {
            let client = self
                .pointer
                .owners
                .gesture_target
                .observers
                .borrow()
                .tap_friendliness
                .clone();
            if let Some(client) = client {
                client.DidReceiveGestureTap(id);
            }
        }
    }
    fn SendContextMenuEvent(&self, event: &WebMouseEvent) -> WebInputEventResult {
        crate::core::input::event_handler::SendLocatedContextMenu(self, event, None)
    }
    fn SendContextMenuEventInAllowedScope(&self, event: &WebMouseEvent) {
        self.SendContextMenuEvent(event);
    }
    fn ScrollableAreas(&self) -> Vec<Rc<dyn GestureScrollableArea>> {
        match self.pointer.owners.gesture_target.timer.borrow().as_ref() {
            Some(host) => host.ScrollableAreas(),
            None => {
                self.Fault("LocalFrameView::ScrollableAreas gesture cancellation");
                Vec::new()
            }
        }
    }
    fn IsTextNode(&self, node: &EventTargetRef) -> bool {
        self.pointer
            .Read(node, |d, i| {
                d.Node(i).Type() == dom::persistent_document::DOMNodeType::kText
            })
            .unwrap_or(false)
    }
    fn IsContentEditable(&self, node: &EventTargetRef) -> bool {
        self.pointer
            .Read(node, |d, i| d.IsEditableForEditing(i))
            .unwrap_or(false)
    }
    fn IsInsideFocusableElementOrARIAWidget(&self, _: &EventTargetRef) -> bool {
        self.Fault("EventHandler::IsInsideFocusableElementOrARIAWidget");
        false
    }
    fn RootFrameToViewport(&self, point: PointF) -> Option<PointF> {
        let point = self.pointer.frame.RootFrameToViewport(point);
        if point.is_none() {
            self.Fault("VisualViewport::RootFrameToViewport");
        }
        point
    }
    fn ShowUnhandledTapUI(&self, _: PointF) {
        self.Fault("ChromeClient::ShowUnhandledTapUI");
    }
}

struct PointerBorrow<'a>(&'a PointerEventManager<'a>);
impl GesturePointer for PointerBorrow<'_> {
    fn PrimaryPointerdownCanceled(&self, id: u32) -> bool {
        self.0.PrimaryPointerdownCanceled(id)
    }
    fn GetPointerIdForTouchGesture(&self, id: u32) -> i32 {
        self.0.GetPointerIdForTouchGesture(id)
    }
    fn GetPointerDownTarget(&self, id: i32) -> Option<PointerTargetRef> {
        self.0.GetPointerDownTarget(id)
    }
    fn GetPointerUpTarget(&self, id: i32) -> Option<PointerTargetRef> {
        self.0.GetPointerUpTarget(id)
    }
    fn RemovePointerTargets(&self, id: i32) {
        self.0.RemovePointerTargets(id);
    }
}
struct SelectionPorts<'scope, 'a, 'listener, 'frame> {
    selection: Rc<super::gesture_selection::PageGestureSelection<'a, 'listener, 'frame>>,
    errors: Rc<RefCell<Option<PointerInputError>>>,
    pointer_error: &'scope RefCell<Option<PointerInputError>>,
    defaults: super::gesture_mouse_defaults::PageGestureMouseDefaults<'a, 'listener, 'frame>,
}
impl SelectionPorts<'_, '_, '_, '_> {
    fn RecordError(&self) {
        if let Some(error) = self.selection.TakeError() {
            let message = format!("{error:?}");
            self.errors
                .borrow_mut()
                .get_or_insert(PointerInputError::GestureSelection(message.clone()));
            self.pointer_error
                .borrow_mut()
                .get_or_insert(PointerInputError::GestureSelection(message));
        }
    }
}
impl GestureSelection<PageGestureHit> for SelectionPorts<'_, '_, '_, '_> {
    fn InitializeSelectionState(&self) {
        self.selection.InitializeSelectionState();
    }
    fn HandleMousePressEvent(
        &self,
        event: &WebMouseEvent,
        location: &HitTestLocation,
        hit: &PageGestureHit,
    ) {
        if let Err(error) = self.defaults.SelectionPress(event, location, &hit.result) {
            self.pointer_error.borrow_mut().get_or_insert(error);
        }
    }
    fn HandleGestureLongPress(&self, hit: &PageGestureHit) -> bool {
        let result = self.selection.HandleGestureLongPress(&hit.result);
        self.RecordError();
        result
    }
    fn HandleGestureTwoFingerTap(&self, event: &GestureEventWithHitTestResults<PageGestureHit>) {
        self.selection
            .HandleGestureTwoFingerTap(&event.result.result);
        self.RecordError();
    }
}

impl GestureMouse<PageGestureHit> for Context<'_, '_, '_, '_> {
    fn SetElementUnderMouseAndDispatchMouseEvent(
        &self,
        node: Option<EventTargetRef>,
        kind: &str,
        event: &WebMouseEvent,
        down: Option<PointerTargetRef>,
        up: Option<PointerTargetRef>,
    ) -> WebInputEventResult {
        PointerMouseManager::SetElementUnderMouse(self.pointer, node.clone(), event);
        if kind == "click" {
            PointerMouseManager::DispatchMouseClickIfNeeded(
                self.pointer,
                node,
                None,
                event,
                event.pointer_properties.id,
                crate::core::input::mouse_event_manager::PointerTypeName(
                    event.pointer_properties.pointer_type,
                ),
                down,
                up,
            );
            return self
                .pointer
                .result
                .get()
                .map_or(WebInputEventResult::kNotHandled, |result| {
                    if result.default_prevented {
                        WebInputEventResult::kHandledApplication
                    } else {
                        WebInputEventResult::kNotHandled
                    }
                });
        }
        let last = self
            .pointer
            .owners
            .mouse
            .LastKnownMousePositionInRootFrame();
        PointerMouseManager::DispatchMouseEvent(self.pointer, node, kind, event, &last).1
    }
    fn SetMouseDownElement(&self, node: Option<EventTargetRef>) {
        self.pointer.owners.mouse.SetMouseDownElement(node.as_ref());
    }
    fn SetClickCount(&self, count: i32) {
        self.pointer.owners.mouse.SetClickCount(count);
    }
    fn HandleMouseFocusWithTouchCapabilities(&self, hit: &PageGestureHit) -> WebInputEventResult {
        let mut event = WebMouseEvent::new(
            Type::kMouseDown,
            PointF::default(),
            PointF::default(),
            Button::kNoButton,
            0,
            WebInputEvent::kIsCompatibilityEventForTouch,
            self.pointer.dispatcher.clock.Now(),
            interaction_dispatch::core::events::mouse_event::MenuSourceType::kTouch,
            WebMouseEvent::kMousePointerId,
        );
        event.pointer_properties.pointer_type = PointerType::kTouch;
        self.pointer.dispatcher.HandleNativeMouseFocus(
            &self.pointer.owners.mouse,
            &event,
            hit.HasScrollbar(),
            false,
        )
    }
    fn HandleMousePressEvent(
        &self,
        event: &WebMouseEvent,
        location: &HitTestLocation,
        hit: &PageGestureHit,
    ) -> WebInputEventResult {
        match super::gesture_mouse_defaults::PageGestureMouseDefaults::new(
            self.pointer.dispatcher,
            self.pointer.owners,
            self.pointer.geometry.clone(),
            self.pointer.controls.clone(),
        )
        .Press(event, location, &hit.result)
        {
            Ok(result) => result,
            Err(error) => {
                self.pointer.Fault(error);
                WebInputEventResult::kNotHandled
            }
        }
    }
    fn HandleMouseReleaseEvent(
        &self,
        event: &WebMouseEvent,
        location: &HitTestLocation,
        hit: &PageGestureHit,
    ) -> WebInputEventResult {
        match super::gesture_mouse_defaults::PageGestureMouseDefaults::new(
            self.pointer.dispatcher,
            self.pointer.owners,
            self.pointer.geometry.clone(),
            self.pointer.controls.clone(),
        )
        .Release(event, location, &hit.result)
        {
            Ok(result) => result,
            Err(error) => {
                self.pointer.Fault(error);
                WebInputEventResult::kNotHandled
            }
        }
    }
    fn ClearDragHeuristicState(&self) {
        self.pointer.owners.mouse.ClearDragHeuristicState();
    }
    fn FocusDocumentView(&self) {
        if let Err(error) = self.pointer.dispatcher.FocusNativeDocumentView() {
            self.pointer.Fault(error);
        }
    }
    fn HandleDragDropIfPossible(
        &self,
        event: &GestureEventWithHitTestResults<PageGestureHit>,
        pointer_id: i32,
    ) -> DragHandlingResult {
        match super::mouse_drag::HandleTouchDrag(
            self.pointer.dispatcher,
            self.pointer.owners,
            self.pointer.geometry.clone(),
            self.pointer.controls.clone(),
            event,
            pointer_id,
        ) {
            Ok(result) => result,
            Err(error) => {
                self.pointer.Fault(error);
                DragHandlingResult::kNotHandled
            }
        }
    }
}

impl EventDispatcher<'_, '_> {
    pub(crate) fn SendGestureContextMenuEvent(
        &self,
        owners: &ResidentCursorOwners,
        geometry: Rc<dyn PagePointerGeometry>,
        controls: Rc<PagePointerScrollControlOwners>,
        event: &WebMouseEvent,
        override_target: Option<EventTargetRef>,
    ) -> Result<WebInputEventResult, PointerInputError> {
        let (result, _) = self.WithPointerManager(owners, geometry, controls, |_, pointer| {
            let context = Context {
                pointer,
                errors: Default::default(),
            };
            let result = crate::core::input::event_handler::SendLocatedContextMenu(
                &context,
                event,
                override_target,
            );
            if let Some(error) = context.errors.borrow_mut().take() {
                return Err(error);
            }
            Ok(result)
        })?;
        Ok(result)
    }
    pub(crate) fn DispatchNativeGesturePacket(
        &self,
        owners: &ResidentCursorOwners,
        geometry: Rc<dyn PagePointerGeometry>,
        controls: Rc<PagePointerScrollControlOwners>,
        scroll: Rc<interaction_scroll::core::input::scroll_manager::ScrollManager>,
        packet: crate::adapter::frame_aligned_input_queue::BorrowedInputPacket<'_>,
    ) -> Result<(DispatchEventResult, WebInputEventResult), PointerInputError> {
        let event = packet
            .Event()
            .AsAny()
            .downcast_ref::<WebGestureEvent>()
            .expect("canonical gesture packet");
        if event.IsScrollEvent() {
            return Err(PointerInputError::SourceLeaf(
                "compositor gesture scroll ingress",
            ));
        }
        if !matches!(
            event.GetType(),
            Type::kGestureTapDown
                | Type::kGestureTap
                | Type::kGestureShowPress
                | Type::kGestureShortPress
                | Type::kGestureLongPress
                | Type::kGestureLongTap
                | Type::kGestureTwoFingerTap
                | Type::kGestureTapCancel
                | Type::kGestureTapUnconfirmed
        ) {
            return Err(PointerInputError::SourceLeaf(
                "WidgetEventHandler fling/pinch gesture routing",
            ));
        }
        let (input_result, result) =
            self.WithPointerManager(owners, geometry, controls, |pointer, ctx| {
                let context = Rc::new(Context {
                    pointer: ctx,
                    errors: Default::default(),
                });
                let selection = Rc::new(super::gesture_selection::PageGestureSelection::new(
                    self,
                    ctx.geometry.clone(),
                ));
                let gesture = GestureManager::WithState(
                    owners.gesture.clone(),
                    context.clone(),
                    scroll.clone(),
                    context.clone(),
                    Rc::new(PointerBorrow(pointer)),
                    Rc::new(SelectionPorts {
                        selection: selection.clone(),
                        errors: context.errors.clone(),
                        pointer_error: &ctx.error,
                        defaults: super::gesture_mouse_defaults::PageGestureMouseDefaults::new(
                            self,
                            owners,
                            ctx.geometry.clone(),
                            ctx.controls.clone(),
                        ),
                    }),
                );
                let targeted =
                    gesture_targeting::TargetGestureEvent(context.as_ref(), event, false)?;
                if event.GetType() == Type::kGestureShowPress {
                    owners
                        .gesture_target
                        .last_show_press
                        .set(Some(self.clock.Now()));
                }
                let result = gesture.HandleGestureEventInFrame(&targeted);
                if let Some(error) = selection.TakeError() {
                    return Err(PointerInputError::GestureSelection(format!("{error:?}")));
                }
                if let Some(error) = context.errors.borrow_mut().take() {
                    return Err(error);
                }
                Ok(result)
            })?;
        Ok((result, input_result))
    }
}
