//! Borrowed production ports for the canonical PointerEventManager state.
//! Source algorithms and state remain in core/input; this adapter resolves
//! live DOM identities and lends Layout/Paint for individual geometry queries.
use super::*;
use crate::adapter::{
    page_pointer_hit_test::{
        PagePointerGeometry, PointerAdjustmentDOMFacts, PointerGeometryError, PointerHit,
    },
    page_pointer_scroll_controls::PagePointerScrollControlOwners,
    source_input_managers::ResidentCursorOwners,
};
use crate::core::input::{
    pointer_event_manager::{
        self as pointer, PointerAdjustment, PointerAdjustmentCandidateType,
        PointerAdjustmentHitTest, PointerEventDOMPort, PointerEventEnvironment,
        PointerEventFeatures, PointerEventManager, PointerEventTarget, PointerMouseManager,
        PointerPanAction, PointerResizeArea, PointerTouchAction, PointerTouchManager,
    },
    touch_event_manager::{TouchEventEnvironment, TouchEventManager},
};
use foundation::{LayoutUnit, PhysicalOffset, PhysicalSize};
use interaction_dispatch::core::events::pointer_event_factory::PointerTargetRef;
use interaction_dispatch::core::input::event_handling_util::WebInputEventResult;
use interaction_hit_test::core::{
    layout::{hit_test_location::HitTestLocation, hit_test_request::RequestType},
    page::touch_adjustment::TouchAdjustmentCandidateType,
};
use interaction_input::{
    platform::widget::input::widget_base_input_handler::GetCoalescedWebPointerEventForTouchValues,
    public::common::{
        frame::user_activation_state::UserActivationNotificationType,
        input::{
            web_input_event::{Type, WebInputEventPayload},
            web_pointer_event::{TouchState, WebPointerEvent},
            web_pointer_properties::{PointerId, PointerType},
        },
    },
};

#[derive(Debug)]
pub enum PointerInputError {
    MissingGeometry,
    MissingScrollControls,
    Geometry(PointerGeometryError),
    PointQuery(interaction_hit_test::resident::ResidentCursorGeometryError),
    DOM(super::pointer_dom_effects::PointerDomError),
    Conversion(crate::input_event::WebInputConversionError),
    DetachedTarget,
    LocalFrameRouting,
    SourceLeaf(&'static str),
    GestureSelection(String),
    DragImage(String),
}
impl std::fmt::Display for PointerInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for PointerInputError {}

pub(super) struct PagePointerContext<'a, 'listener, 'frame> {
    pub(super) dispatcher: &'a EventDispatcher<'listener, 'frame>,
    pub(super) owners: &'a ResidentCursorOwners,
    pub(super) geometry: Rc<dyn PagePointerGeometry>,
    pub(super) controls: Rc<PagePointerScrollControlOwners>,
    pub(super) frame: MouseFrameGeometryRef,
    pub(super) result: Cell<Option<DispatchEventResult>>,
    pub(super) error: RefCell<Option<PointerInputError>>,
}
impl<'a, 'listener, 'frame> PagePointerContext<'a, 'listener, 'frame> {
    pub(super) fn Fault(&self, error: PointerInputError) {
        let mut slot = self.error.borrow_mut();
        if slot.is_none() {
            *slot = Some(error);
        }
    }
    pub(super) fn Read<T>(
        &self,
        node: &EventTargetRef,
        read: impl FnOnce(&dom::Document, usize) -> T,
    ) -> Option<T> {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        if self.dispatcher.targets.OwnerDocumentHandle(node) != Some(document.RootHandle()) {
            return None;
        }
        let id = self.dispatcher.targets.NodeId(node)?;
        let index = if id == 0 {
            document.Root()
        } else {
            document.FindNodeById(id)?
        };
        Some(read(document, index))
    }
    pub(super) fn Related(
        &self,
        node: &EventTargetRef,
        read: impl FnOnce(&dom::Document, usize) -> Option<usize>,
    ) -> Option<EventTargetRef> {
        let id = self.Read(node, |d, i| read(d, i).map(|i| d.Node(i).Id()))??;
        self.dispatcher.targets.ResolveTarget(id)
    }
    pub(super) fn Boundary(&self) -> PageBoundaryDOM<'_> {
        PageBoundaryDOM {
            document: &self.dispatcher.document,
            targets: &self.dispatcher.targets,
            listener_interest: &self.dispatcher.listener_interest,
        }
    }
    pub(super) fn MouseDispatch<'scope>(
        &'scope self,
    ) -> PageMouseBoundaryDispatch<'scope, 'listener, 'frame> {
        PageMouseBoundaryDispatch {
            dispatcher: self.dispatcher,
            coordinates: self.dispatcher.MouseCoordinateContext(),
            result: Cell::new(None),
        }
    }
    fn Interest(
        &self,
        read: impl FnOnce(&dyn crate::event::EventListenerInterest) -> bool,
    ) -> bool {
        let interest = self.dispatcher.listener_interest.borrow().clone();
        interest
            .as_ref()
            .is_some_and(|interest| read(interest.as_ref()))
    }
    pub(super) fn DispatchCanonical(
        &self,
        target: &EventTargetRef,
        event: EventRef,
    ) -> NativeDispatchEventResult {
        if self.error.borrow().is_some() {
            return NativeDispatchEventResult::kCanceledBeforeDispatch;
        }
        let Some(id) = self.dispatcher.targets.NodeId(target) else {
            self.Fault(PointerInputError::DetachedTarget);
            return NativeDispatchEventResult::kCanceledBeforeDispatch;
        };
        event.borrow_mut().BaseEventMut().SetTrusted(true);
        let (native, result) = ports::DispatchCanonicalWithResult(self.dispatcher, event, id);
        self.result.set(Some(result));
        native
    }
    pub(super) fn Hit(&self, point: PointF, bits: u32) -> Option<PointerHit> {
        let point = self
            .frame
            .ConvertFromRootFrame(point)
            .expect("prepared local FrameView");
        let location = HitTestLocation::FromPhysicalOffset(PhysicalOffset::new(
            LayoutUnit::FromFloatRound(point.x()),
            LayoutUnit::FromFloatRound(point.y()),
        ));
        match self.geometry.HitTest(location, bits) {
            Ok(hit) => Some(hit),
            Err(error) => {
                self.Fault(PointerInputError::Geometry(error));
                None
            }
        }
    }
    fn Target(&self, hit: PointerHit) -> PointerEventTarget {
        let target = hit
            .result
            .InnerElement()
            .and_then(|node| node.DOMNodeId())
            .and_then(|id| self.dispatcher.targets.ResolveTarget(id));
        let scrollbar = hit
            .result
            .GetScrollbar()
            .and_then(|scrollbar| scrollbar.NativeScrollbarOwner())
            .cloned()
            .map(|owner| self.controls.Scrollbar(owner));
        let frame = target.as_ref().and_then(|node| self.FrameForElement(node));
        PointerEventTarget {
            target_element: target,
            target_frame: frame,
            scrollbar,
        }
    }
    fn ScrollOffset(&self) -> PointF {
        let (x, y) = self.frame.LayoutViewportScrollOffset().unwrap_or((0, 0));
        PointF::new(x as f32, y as f32)
    }
}

impl BoundaryEventDOM for PagePointerContext<'_, '_, '_> {
    fn IsInDocument(&self, node: &EventTargetRef) -> bool {
        self.Boundary().IsInDocument(node)
    }
    fn FlatTreeAncestorChain(&self, node: &EventTargetRef) -> BoundaryAncestorChainSnapshotRef {
        self.Boundary().FlatTreeAncestorChain(node)
    }
    fn HasCapturingEventListeners(&self, node: &EventTargetRef, name: &str) -> bool {
        self.Boundary().HasCapturingEventListeners(node, name)
    }
}
impl PointerEventDOMPort for PagePointerContext<'_, '_, '_> {
    fn CoordinateContext(&self) -> MouseCoordinateContext {
        self.dispatcher.MouseCoordinateContext()
    }
    fn HasPointerEventHandlers(&self) -> bool {
        self.Interest(|interest| interest.HasPointerEventHandlers())
    }
    fn HasPointerRawUpdateEventHandlers(&self) -> bool {
        self.Interest(|interest| interest.HasPointerRawUpdateEventHandlers())
    }
    fn HasEventListeners(&self, target: &EventTargetRef, name: &str) -> bool {
        self.dispatcher.HasEventListeners(target, name)
    }
    fn IsElement(&self, node: &EventTargetRef) -> bool {
        self.Read(node, |d, i| {
            d.Node(i).Type() == dom::persistent_document::DOMNodeType::kElement
        })
        .unwrap_or(false)
    }
    fn ParentNode(&self, node: &EventTargetRef) -> Option<EventTargetRef> {
        self.Related(node, |d, i| d.Node(i).Parent())
    }
    fn ParentOrShadowHostElement(&self, node: &EventTargetRef) -> Option<EventTargetRef> {
        self.Related(node, |d, i| d.ParentOrShadowHostElement(i))
    }
    fn IsShadowIncludingInclusiveAncestorOf(
        &self,
        a: &EventTargetRef,
        node: &EventTargetRef,
    ) -> bool {
        let owner = self.dispatcher.document.borrow();
        let d = owner.GetDocument();
        let relations = MouseRemovalRelations {
            document: d,
            targets: &self.dispatcher.targets,
        };
        crate::core::input::mouse_event_manager::MouseRemovalNodeRelations::IsShadowIncludingInclusiveAncestorOf(&relations,a,node)
    }
    fn OwnerDocument(&self, node: &EventTargetRef) -> EventTargetRef {
        self.dispatcher
            .targets
            .OwnerDocumentTarget(node)
            .expect("canonical pointer document")
    }
    fn FrameForElement(&self, node: &EventTargetRef) -> Option<Rc<()>> {
        self.Read(node, |_, _| self.owners.pointer_frame.identity.clone())
    }
    fn View(&self) -> Option<AbstractViewRef> {
        self.frame
            .HasFrameView()
            .then(|| self.dispatcher.targets.WindowView())
    }
    fn ViewForElement(&self, node: &EventTargetRef) -> Option<AbstractViewRef> {
        self.FrameForElement(node).and_then(|_| self.View())
    }
    fn IsSecureContext(&self) -> bool {
        self.dispatcher
            .document
            .borrow()
            .GetDocument()
            .SecurityContext()
            .IsSecureContext()
    }
    fn NeedsUnbufferedInputEvents(&self, node: &EventTargetRef) -> bool {
        self.Read(node, |d, i| d.CanvasNeedsUnbufferedInputEvents(i))
            .unwrap_or(false)
    }
    fn RequestUnbufferedInputEvents(&self) {
        self.owners.pointer_frame.unbuffered_requested.set(true);
    }
    fn AnchorElementInteractionTrackerOnPointerEvent(
        &self,
        target: &EventTargetRef,
        event: &PointerEventRef,
    ) {
        super::pointer_dom_effects::AnchorElementInteractionTrackerOnPointerEvent(
            self.dispatcher,
            target,
            event,
        );
    }
    fn HandlePopoverLightDismiss(&self, event: &PointerEventRef, target: &EventTargetRef) {
        if let Err(error) =
            super::pointer_dom_effects::HandlePopoverLightDismiss(self.dispatcher, event, target)
        {
            self.Fault(PointerInputError::DOM(error));
        }
    }
    fn HandleDialogLightDismiss(&self, event: &PointerEventRef, target: &EventTargetRef) {
        if let Err(error) =
            super::pointer_dom_effects::HandleDialogLightDismiss(self.dispatcher, event, target)
        {
            self.Fault(PointerInputError::DOM(error));
        }
    }
    fn HandlePointerEventsForInterestFor(&self, target: &EventTargetRef, name: &str) {
        if let Err(error) = super::pointer_dom_effects::HandlePointerEventsForInterestFor(
            self.dispatcher,
            target,
            name,
        ) {
            self.Fault(PointerInputError::DOM(error));
        }
    }
    fn DispatchEvent(
        &self,
        target: &EventTargetRef,
        _: &PointerEventRef,
        event: EventRef,
    ) -> NativeDispatchEventResult {
        self.DispatchCanonical(target, event)
    }
    fn HitTestTouchPointer(&self, event: &WebPointerEvent) -> PointerEventTarget {
        self.Hit(
            *event.pointer_properties.PositionInWidget(),
            RequestType::kTouchEvent as u32
                | RequestType::kReadOnly as u32
                | RequestType::kActive as u32,
        )
        .map_or_else(PointerEventTarget::default, |hit| self.Target(hit))
    }
    fn HitTestMouseRelease(&self, event: &WebMouseEvent) -> Option<EventTargetRef> {
        self.Hit(
            *event.pointer_properties.PositionInWidget(),
            RequestType::kRelease as u32,
        )
        .and_then(|hit| self.Target(hit).target_element)
    }
    fn NodeEventContexts(&self, event: &EventRef) -> Rc<dyn pointer::PointerNodeEventContexts> {
        self.dispatcher.paths.PointerNodeEventContexts(event)
    }
    fn AppendTouchIdForCanceledPointerDown(&self, frame: &Rc<()>, id: u32) {
        if Rc::ptr_eq(frame, &self.owners.pointer_frame.identity) {
            self.owners.pointer.AppendTouchIdForCanceledPointerDown(id)
        } else {
            self.Fault(PointerInputError::LocalFrameRouting)
        }
    }
}

struct AdjustmentHit(Option<PointerHit>);
impl PointerAdjustmentHitTest for AdjustmentHit {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
}
impl PointerEventEnvironment for PagePointerContext<'_, '_, '_> {
    fn CurrentFrame(&self) -> Rc<()> {
        self.owners.pointer_frame.identity.clone()
    }
    fn PointerLockedElement(&self) -> Option<EventTargetRef> {
        // The current native host has no PointerLockController owner.
        None
    }
    fn HandlePointerEventOnFrame(
        &self,
        _: &Rc<()>,
        _: &WebPointerEvent,
        _: &[WebPointerEvent],
        _: &[WebPointerEvent],
    ) {
        self.Fault(PointerInputError::LocalFrameRouting)
    }
    fn ShouldDiscardEventTargetingFrame(&self, _: &WebPointerEvent, frame: &Rc<()>) -> bool {
        // This Page's root frame has no embedding frame rectangle to move.
        !Rc::ptr_eq(frame, &self.owners.pointer_frame.identity)
    }
    fn DomNodeId(&self, node: &EventTargetRef) -> i32 {
        self.dispatcher
            .targets
            .NodeId(node)
            .and_then(|id| i32::try_from(id).ok())
            .unwrap_or(0)
    }
    fn NotifyUserActivation(&self, frame: &Rc<()>) {
        if Rc::ptr_eq(frame, &self.owners.pointer_frame.identity) {
            self.owners.pointer_frame.activation.borrow_mut().Activate(
                UserActivationNotificationType::kInteraction,
                self.dispatcher.clock.Now(),
            );
            super::pointer_dom_effects::SetHadUserInteraction(self.dispatcher, true);
        } else {
            self.Fault(PointerInputError::LocalFrameRouting)
        }
    }
    fn EffectiveTouchActionAtPointerDown(
        &self,
        event: &WebPointerEvent,
        node: &EventTargetRef,
    ) -> PointerTouchAction {
        self.EffectiveTouchActionAtPointer(event, node)
    }
    fn EffectiveTouchActionAtPointer(
        &self,
        event: &WebPointerEvent,
        node: &EventTargetRef,
    ) -> PointerTouchAction {
        let id = self
            .dispatcher
            .targets
            .NodeId(node)
            .expect("canonical pointer node");
        PointerTouchAction(
            self.geometry
                .EffectiveTouchAction(
                    id,
                    matches!(
                        event.pointer_properties.pointer_type,
                        PointerType::kPen | PointerType::kEraser
                    ),
                )
                .bits() as u32,
        )
    }
    fn SetTouchAction(&self, action: PointerTouchAction) {
        self.owners.pointer_frame.touch_action.set(action);
    }
    fn SetPanAction(&self, action: PointerPanAction) {
        self.owners.pointer_frame.pan_action.set(action);
    }
    fn EnclosingLayerScrollableArea(
        &self,
        node: &EventTargetRef,
    ) -> Option<Rc<dyn PointerResizeArea>> {
        self.dispatcher
            .targets
            .NodeId(node)
            .and_then(|id| self.controls.EnclosingLayerScrollableArea(id))
    }
    fn ConvertFromRootFrame(&self, _: &Rc<()>, point: PointF) -> PointF {
        self.frame
            .ConvertFromRootFrame(point)
            .expect("prepared root frame view")
    }
    fn DeviceScaleFactor(&self) -> f32 {
        self.geometry.DeviceScaleFactor()
    }
    fn PageScaleFactor(&self) -> f32 {
        self.geometry.PageScaleFactor()
    }
    fn HandwritingRadius(&self) -> i32 {
        pointer::kStylusWritableAdjustmentSizeDip as i32
    }
    fn GetHitTestRectForAdjustment(&self, width: f32, height: f32) -> (f32, f32) {
        let size = self.geometry.GetHitTestRectForAdjustment(width, height);
        (size.width.ToFloat(), size.height.ToFloat())
    }
    fn HitTestAdjustmentRect(
        &self,
        point: PointF,
        size: (f32, f32),
    ) -> Rc<dyn PointerAdjustmentHitTest> {
        let point = self
            .frame
            .ConvertFromRootFrame(point)
            .expect("prepared local FrameView");
        let size = PhysicalSize::new(
            LayoutUnit::FromFloatRound(size.0),
            LayoutUnit::FromFloatRound(size.1),
        );
        match self.geometry.HitTestAdjustmentRect(point, size) {
            Ok(hit) => Rc::new(AdjustmentHit(Some(hit))),
            Err(error) => {
                self.Fault(PointerInputError::Geometry(error));
                Rc::new(AdjustmentHit(None))
            }
        }
    }
    fn BestNodeForHitTestResult(
        &self,
        kind: PointerAdjustmentCandidateType,
        hit: &dyn PointerAdjustmentHitTest,
    ) -> Option<PointerAdjustment> {
        if self.error.borrow().is_some() {
            return None;
        }
        let hit = hit
            .AsAny()
            .downcast_ref::<AdjustmentHit>()
            .expect("canonical pointer adjustment hit")
            .0
            .as_ref()?;
        let kind = match kind {
            PointerAdjustmentCandidateType::kClickable => TouchAdjustmentCandidateType::kClickable,
            PointerAdjustmentCandidateType::kStylusWritable => {
                TouchAdjustmentCandidateType::kStylusWritable
            }
        };
        match self.geometry.BestTouchAdjustment(kind, hit, self) {
            Ok(value) => value.map(|(id, point)| PointerAdjustment {
                point: PointF::new(point.x() as f32, point.y() as f32),
                node: self.dispatcher.targets.ResolveTarget(id),
            }),
            Err(error) => {
                self.Fault(PointerInputError::Geometry(error));
                None
            }
        }
    }
    fn CacheTouchAdjustmentResult(&self, id: u32, point: PointF) {
        self.owners
            .event_handler
            .CacheTouchAdjustmentResult(id, point);
    }
}

impl PointerMouseManager for PagePointerContext<'_, '_, '_> {
    fn Clear(&self) {
        self.owners.mouse.ClearSourceInputState();
    }
    fn SetElementUnderMouse(&self, target: Option<EventTargetRef>, event: &WebMouseEvent) {
        let dispatch = self.MouseDispatch();
        pointer::MousePointerEventDispatchPort::SetElementUnderMouse(
            &dispatch,
            &self.owners.mouse,
            target.as_ref(),
            event,
        );
        if let Some(result) = dispatch.result.get() {
            self.result.set(Some(result));
        }
    }
    fn GetElementUnderMouse(&self) -> Option<EventTargetRef> {
        crate::core::input::mouse_event_manager::MouseEventManager::WithState(
            self.owners.mouse.clone(),
        )
        .GetElementUnderMouse()
    }
    fn DispatchMouseEvent(
        &self,
        target: Option<EventTargetRef>,
        name: &str,
        event: &WebMouseEvent,
        last: &PointF,
    ) -> (Option<EventRef>, WebInputEventResult) {
        if self.error.borrow().is_some() {
            return (None, WebInputEventResult::kNotHandled);
        }
        let dispatch = self.MouseDispatch();
        let result = crate::core::input::mouse_event_manager::MouseEventManager::WithState(
            self.owners.mouse.clone(),
        )
        .DispatchMouseEventWithDispatch(
            &dispatch,
            target.as_ref(),
            name,
            event,
            Some(*last),
            None,
            false,
        );
        if let Some(result) = dispatch.result.get() {
            self.result.set(Some(result));
        }
        result
    }
    fn DispatchMouseClickIfNeeded(
        &self,
        target: Option<EventTargetRef>,
        captured: Option<EventTargetRef>,
        event: &WebMouseEvent,
        id: PointerId,
        kind: &str,
        down: Option<PointerTargetRef>,
        up: Option<PointerTargetRef>,
    ) {
        if self.error.borrow().is_some() {
            return;
        }
        let dispatch = self.MouseDispatch();
        let down = down
            .as_ref()
            .map(|value| value as &dyn crate::core::input::mouse_event_manager::MousePointerTarget);
        let up = up
            .as_ref()
            .map(|value| value as &dyn crate::core::input::mouse_event_manager::MousePointerTarget);
        crate::core::input::mouse_event_manager::MouseEventManager::WithState(
            self.owners.mouse.clone(),
        )
        .DispatchMouseClickIfNeededWithDispatch(
            &dispatch,
            target.as_ref(),
            captured.as_ref(),
            event,
            id,
            kind,
            down,
            up,
        );
    }
}

impl TouchEventEnvironment for PagePointerContext<'_, '_, '_> {
    fn OwnerDocument(&self, node: &EventTargetRef) -> EventTargetRef {
        PointerEventDOMPort::OwnerDocument(self, node)
    }
    fn FrameHasView(&self, document: &EventTargetRef) -> bool {
        self.FrameForElement(document).is_some() && self.frame.HasFrameView()
    }
    fn DocumentHasPage(&self, document: &EventTargetRef) -> bool {
        self.FrameForElement(document).is_some()
            && self.dispatcher.document.borrow().GetDocument().IsActive()
    }
    fn HasTouchHandlers(&self, _: &EventTargetRef) -> bool {
        self.Interest(|interest| interest.HasTouchEventHandlers())
    }
    fn ViewForDocument(&self, document: &EventTargetRef) -> Option<AbstractViewRef> {
        self.ViewForElement(document)
    }
    fn TouchCapabilitiesForDocument(
        &self,
        document: &EventTargetRef,
    ) -> Option<InputDeviceCapabilitiesRef> {
        self.FrameForElement(document)
            .map(|_| self.dispatcher.targets.InputDeviceCapabilities(true))
    }
    fn LayoutZoomFactor(&self, _: &EventTargetRef) -> f32 {
        self.frame
            .LayoutZoomFactor()
            .expect("prepared native frame zoom")
    }
    fn RootFrameToDocument(&self, _: &EventTargetRef, point: PointF) -> PointF {
        let point = self
            .frame
            .ConvertFromRootFrame(point)
            .expect("prepared native frame view");
        let offset = self.ScrollOffset();
        PointF::new(point.x() + offset.x(), point.y() + offset.y())
    }
    fn LayoutViewportScrollOffset(&self, _: &EventTargetRef) -> PointF {
        self.ScrollOffset()
    }
    fn DocumentToFrame(&self, _: &EventTargetRef, point: PointF) -> PointF {
        let offset = self.ScrollOffset();
        PointF::new(point.x() - offset.x(), point.y() - offset.y())
    }
    fn HitTestTouchNode(
        &self,
        _: &EventTargetRef,
        event: &WebPointerEvent,
    ) -> Option<EventTargetRef> {
        self.HitTestTouchPointer(event).target_element
    }
    fn IsTextNode(&self, node: &EventTargetRef) -> bool {
        self.Read(node, |d, i| {
            d.Node(i).Type() == dom::persistent_document::DOMNodeType::kText
        })
        .unwrap_or(false)
    }
    fn FlatTreeParent(&self, node: &EventTargetRef) -> Option<EventTargetRef> {
        self.Related(node, |d, i| d.FlatTreeParent(i))
    }
    fn IsElement(&self, node: &EventTargetRef) -> bool {
        PointerEventDOMPort::IsElement(self, node)
    }
    fn IsPseudoElement(&self, node: &EventTargetRef) -> bool {
        self.Read(node, |d, i| d.PseudoElementFacts(i).is_some())
            .unwrap_or(false)
    }
    fn ParentOrShadowHostElement(&self, node: &EventTargetRef) -> Option<EventTargetRef> {
        PointerEventDOMPort::ParentOrShadowHostElement(self, node)
    }
    fn IsShadowIncludingInclusiveAncestorOf(
        &self,
        a: &EventTargetRef,
        node: &EventTargetRef,
    ) -> bool {
        PointerEventDOMPort::IsShadowIncludingInclusiveAncestorOf(self, a, node)
    }
    fn IsVerticalScrollEnforced(&self, _: &EventTargetRef) -> bool {
        // Same source field default as PageWheelDocumentQuery. No host policy
        // currently enables Document's vertical-scroll enforcement.
        false
    }
    fn HasBlockingTouchHandlerInAncestorPath(&self, node: &EventTargetRef) -> bool {
        let Some(interest) = self.dispatcher.listener_interest.borrow().clone() else {
            return false;
        };
        let owner = self.dispatcher.document.borrow();
        let d = owner.GetDocument();
        if interest.HasBlockingTouchHandler(d.RootHandle(), 0) {
            return true;
        }
        let Some(id) = self.dispatcher.targets.NodeId(node) else {
            return false;
        };
        let mut current = d.FindNodeById(id);
        while let Some(index) = current {
            if interest.HasBlockingTouchHandler(d.RootHandle(), d.Node(index).Id()) {
                return true;
            }
            current = d.Node(index).Parent();
        }
        false
    }
    fn SetTouchAction(&self, action: PointerTouchAction) {
        PointerEventEnvironment::SetTouchAction(self, action)
    }
    fn DispatchEvent(&self, target: &EventTargetRef, event: EventRef) -> NativeDispatchEventResult {
        self.DispatchCanonical(target, event)
    }
}

struct FocusGeometry<'a>(&'a dyn MouseFrameGeometry);
impl crate::adapter::page_element_focusability::PageElementFocusLayout for FocusGeometry<'_> {
    fn HasLayoutObject(&self, id: u64) -> bool {
        self.0.HasLayoutObject(id)
    }
    fn IsUserScrollable(&self, id: u64) -> bool {
        self.0.IsUserScrollable(id)
    }
    fn IsCanvasLayoutObject(&self, id: u64) -> bool {
        self.0.IsCanvasLayoutObject(id)
    }
}
impl PointerAdjustmentDOMFacts for PagePointerContext<'_, '_, '_> {
    fn WillRespondToMouseClickEvents(&self, id: u64) -> bool {
        let Some(target) = self.dispatcher.targets.ResolveTarget(id) else {
            return false;
        };
        let state = self
            .Read(&target, |d, i| {
                (!d.IsDisabledFormControl(i), d.IsEditableForEditing(i))
            })
            .unwrap_or((false, false));
        state.0
            && (state.1
                || ["click", "mousedown", "mouseup", "DOMActivate"]
                    .iter()
                    .any(|name| self.dispatcher.HasEventListeners(&target, name)))
    }
    fn WillRespondToMouseMoveEvents(&self, id: u64) -> bool {
        let Some(target) = self.dispatcher.targets.ResolveTarget(id) else {
            return false;
        };
        self.Read(&target, |d, i| !d.IsDisabledFormControl(i))
            .unwrap_or(false)
            && ["mousemove", "mouseover", "mouseout"]
                .iter()
                .any(|name| self.dispatcher.HasEventListeners(&target, name))
    }
    fn IsMouseFocusable(&self, id: u64) -> bool {
        let owner = self.dispatcher.document.borrow();
        let d = owner.GetDocument();
        let Some(index) = d.FindNodeById(id) else {
            return false;
        };
        crate::adapter::page_element_focusability::PageElementFocusability::new(
            d,
            &FocusGeometry(self.frame.as_ref()),
            self.dispatcher.state.borrow().focused_node_id,
        )
        .IsMouseFocusable(index)
    }
    fn ChildrenOrSiblingsAffectedByActiveOrHover(&self, id: u64) -> bool {
        self.geometry.ChildrenOrSiblingsAffectedByActiveOrHover(id)
    }
}

impl EventDispatcher<'_, '_> {
    pub(super) fn WithPointerManager<T>(
        &self,
        owners: &ResidentCursorOwners,
        geometry: Rc<dyn PagePointerGeometry>,
        controls: Rc<PagePointerScrollControlOwners>,
        run: impl FnOnce(
            &PointerEventManager<'_>,
            &PagePointerContext<'_, '_, '_>,
        ) -> Result<T, PointerInputError>,
    ) -> Result<(T, DispatchEventResult), PointerInputError> {
        let frame = self
            .mouse_frame_geometry
            .borrow()
            .clone()
            .ok_or(PointerInputError::MissingGeometry)?;
        let context = Rc::new(PagePointerContext {
            dispatcher: self,
            owners,
            geometry,
            controls,
            frame,
            result: Cell::new(None),
            error: RefCell::new(None),
        });
        let touch = Rc::new(TouchEventManager::WithState(
            owners.touch.clone(),
            context.clone(),
        ));
        let features = PointerEventFeatures {
            light_dismiss_from_click:
                foundation::RuntimeEnabledFeatures::LightDismissFromClickEnabled(),
            boundary_event_dispatch_tracks_node_removal:
                foundation::RuntimeEnabledFeatures::BoundaryEventDispatchTracksNodeRemovalEnabled(),
            suppress_pointer_stream_after_drag:
                foundation::RuntimeEnabledFeatures::SuppressPointerStreamAfterDragEnabled(),
            pointer_raw_update_only_in_secure_context:
                foundation::RuntimeEnabledFeatures::PointerRawUpdateOnlyInSecureContextEnabled(),
            ..Default::default()
        };
        let manager = PointerEventManager::WithState(
            owners.pointer.clone(),
            context.clone(),
            touch,
            context.clone(),
            context.clone(),
            features,
        );
        let value = run(&manager, context.as_ref())?;
        let error = context.error.borrow_mut().take();
        if let Some(error) = error {
            return Err(error);
        }
        Ok((value, context.result.get().unwrap_or_default()))
    }

    pub(crate) fn DispatchNativePointerPacket(
        &self,
        owners: &ResidentCursorOwners,
        geometry: Rc<dyn PagePointerGeometry>,
        controls: Rc<PagePointerScrollControlOwners>,
        packet: crate::adapter::frame_aligned_input_queue::BorrowedInputPacket<'_>,
    ) -> Result<DispatchEventResult, PointerInputError> {
        let touch_packet=interaction_input::public::common::input::web_input_event::WebInputEvent::IsTouchEventType(packet.Event().Event().GetType());
        let(result,mut dispatched)=self.WithPointerManager(owners,geometry,controls,|manager,context|{
            if touch_packet{
                let touch=packet.Event().AsAny().downcast_ref::<interaction_input::public::common::input::web_touch_event::WebTouchEvent>().expect("canonical touch packet");
                if touch.GetType()==Type::kTouchScrollStarted{
                    let event=WebPointerEvent::CreatePointerCausesUaActionEvent(PointerType::kUnknown,touch.TimeStamp());
                    return Ok(manager.HandlePointerEvent(&event,&[],&[]));
                }
                for point in &touch.touches[..touch.touches_length]{
                    if point.state==TouchState::kStateStationary{continue;}
                    let mut event=WebPointerEvent::FromTouch(touch,point);
                    event.SetQueuedTimeStamp(touch.QueuedTimeStamp());
                    let sample=GetCoalescedWebPointerEventForTouchValues(&event,packet.CoalescedEvents(),packet.PredictedEvents(),&());
                    let coalesced=sample.GetCoalescedEventsPointers().iter().map(|value|value.AsAny().downcast_ref::<WebPointerEvent>().expect("touch pointer sample").clone()).collect::<Vec<_>>();
                    let predicted=sample.GetPredictedEventsPointers().iter().map(|value|value.AsAny().downcast_ref::<WebPointerEvent>().expect("touch pointer sample").clone()).collect::<Vec<_>>();
                    manager.HandlePointerEvent(&event,&coalesced,&predicted);
                    if context.error.borrow().is_some(){return Ok(WebInputEventResult::kNotHandled);}
                }
                Ok(manager.FlushEvents())
            }else{
                let event=packet.Event().AsAny().downcast_ref::<WebPointerEvent>().expect("canonical pointer packet");
                let coalesced=packet.CoalescedEvents().map(|value|value.AsAny().downcast_ref::<WebPointerEvent>().expect("pointer sample").clone()).collect::<Vec<_>>();
                let predicted=packet.PredictedEvents().map(|value|value.AsAny().downcast_ref::<WebPointerEvent>().expect("pointer sample").clone()).collect::<Vec<_>>();
                Ok(manager.HandlePointerEvent(event,&coalesced,&predicted))
            }
        })?;
        // WidgetBaseInputHandler returns the buffered touch result; pointer
        // cancellation has its own source suppression/capture state.
        if touch_packet {
            dispatched.default_prevented = result == WebInputEventResult::kHandledApplication;
        }
        Ok(dispatched)
    }

    pub(crate) fn DispatchNativeMousePointerPacket(
        &self,
        owners: &ResidentCursorOwners,
        geometry: Rc<dyn PagePointerGeometry>,
        controls: Rc<PagePointerScrollControlOwners>,
        target: Option<u64>,
        event_type: Type,
        event: &WebMouseEvent,
        packet: Option<crate::adapter::frame_aligned_input_queue::BorrowedInputPacket<'_>>,
    ) -> Result<(DispatchEventResult, WebInputEventResult), PointerInputError> {
        let target = target
            .map(|target| {
                self.targets
                    .ResolveTarget(target)
                    .ok_or(PointerInputError::DetachedTarget)
            })
            .transpose()?;
        let coalesced = packet
            .map(|packet| {
                packet
                    .CoalescedEvents()
                    .map(|value| {
                        value
                            .AsAny()
                            .downcast_ref::<WebMouseEvent>()
                            .expect("mouse sample")
                            .clone()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let predicted = packet
            .map(|packet| {
                packet
                    .PredictedEvents()
                    .map(|value| {
                        value
                            .AsAny()
                            .downcast_ref::<WebMouseEvent>()
                            .expect("mouse sample")
                            .clone()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let (input_result, dispatched) =
            self.WithPointerManager(owners, geometry, controls, |manager, _| {
                Ok(manager.SendMousePointerEvent(
                    target, event_type, event, &coalesced, &predicted, false,
                ))
            })?;
        Ok((dispatched, input_result))
    }

    pub(crate) fn DispatchNativeLastScrollbarMouseUp(
        &self,
        owners: &ResidentCursorOwners,
        geometry: Rc<dyn PagePointerGeometry>,
        controls: Rc<PagePointerScrollControlOwners>,
        event: &WebMouseEvent,
        packet: Option<crate::adapter::frame_aligned_input_queue::BorrowedInputPacket<'_>>,
        release_widget_capture: impl FnOnce(),
    ) -> Option<Result<(DispatchEventResult, WebInputEventResult), PointerInputError>> {
        if !crate::core::input::event_handler::HandleLastScrollbarMouseUp(
            &owners.event_handler,
            event,
            || owners.mouse.InvalidateClick(),
            || {
                release_widget_capture();
                owners.event_handler.ClearMouseCapture();
            },
        ) {
            return None;
        }
        let target = owners
            .mouse
            .GetElementUnderMouse()
            .and_then(|target| self.targets.NodeId(&target));
        Some(self.DispatchNativeMousePointerPacket(
            owners,
            geometry,
            controls,
            target,
            Type::kPointerUp,
            event,
            packet,
        ))
    }
}
