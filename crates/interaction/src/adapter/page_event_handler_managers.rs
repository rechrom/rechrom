//! Call-scoped EventHandler selection, mouse and gesture manager connections.
//! Chromium: event_handler.cc:269-302,327-351,856-1356,1689-1749,
//! 2470-2493,2583; mouse_event_manager.cc:135-157,376-526,541-775,
//! 818-939,1130-1163,1256-1263; selection_controller.h:79-81,
//! selection_controller.cc:1430-1524; gesture_manager.cc:79-835.
//!
//! The wrappers borrow the Page's live source ports for this transaction. Every
//! source manager is rebound to its original ResidentCursorOwners state; no
//! input state is allocated, copied, or reset during binding. Infallible source
//! interfaces record their first failure, which the native facade must drain
//! through Finish before returning an input result.
#[path = "page_event_handler_context_menu.rs"]
mod context_menu;
#[path = "page_event_handler_gesture_targeting.rs"]
mod gesture_targeting_binding;
#[path = "page_event_handler_mouse_view.rs"]
mod mouse_view;
use super::super::*;
use super::{Context, PageGestureHit, PointerBorrow, SelectionPorts};
use crate::adapter::{
    page_event_handler_hit_test::PageEventHandlerHitResult as Hit,
    page_event_handler_node::PageEventHandlerNodes, page_pointer_hit_test::PagePointerGeometry,
    page_pointer_scroll_controls::PagePointerScrollControlOwners,
    source_input_managers::ResidentCursorOwners,
};
use crate::core::input::{
    event_handler::{
        EventHandlerGesture, EventHandlerMouse, EventHandlerSelection,
        MouseEventWithHitTestResults, NodeRef,
    },
    gesture_manager::{GestureEventWithHitTestResults, GestureManager},
    mouse_event_manager::{MouseEventManager, MouseFeatures, MouseFrameViewPort},
    pointer_event_manager::{PointerEventManager, PointerMouseManager},
};
pub use gesture_targeting_binding::PageEventHandlerGestureTargeting;
use interaction_default::core::editing::selection_controller::{
    IsSelectionOverLink, SelectionGeometry,
};
use interaction_dispatch::core::{
    clipboard::data_transfer::DragOperation, events::ui_event::InputDeviceCapabilitiesRef,
    input::event_handling_util::WebInputEventResult,
};
use interaction_hit_test::core::layout::hit_test_location::HitTestLocation;
use interaction_input::public::common::input::{
    web_gesture_event::WebGestureEvent, web_input_event::Type, web_mouse_event::WebMouseEvent,
    web_pointer_properties::PointF,
};
use interaction_scroll::core::input::scroll_manager::ScrollManager;

#[derive(Default)]
struct InputErrors {
    failed: Cell<bool>,
    error: RefCell<Option<PointerInputError>>,
}
impl InputErrors {
    fn Fault(&self, error: PointerInputError) {
        if !self.failed.replace(true) {
            *self.error.borrow_mut() = Some(error);
        }
    }
}

/// One binding implements the three factory leaves so their synchronous
/// failures share one transaction sink. Each leaf still uses its own source
/// manager's retained identity and state.
pub struct PageEventHandlerInputManagers<'scope, 'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
    owners: ResidentCursorOwners,
    nodes: Rc<PageEventHandlerNodes>,
    geometry: Rc<dyn PagePointerGeometry>,
    controls: Rc<PagePointerScrollControlOwners>,
    scroll: Rc<ScrollManager>,
    view: Rc<dyn MouseFrameViewPort + 'scope>,
    features: Rc<dyn MouseFeatures + 'scope>,
    errors: Rc<InputErrors>,
    dispatched: Cell<DispatchEventResult>,
}
pub type PageEventHandlerSelection<'scope, 'listener, 'frame> =
    PageEventHandlerInputManagers<'scope, 'listener, 'frame>;
pub type PageEventHandlerMouse<'scope, 'listener, 'frame> =
    PageEventHandlerInputManagers<'scope, 'listener, 'frame>;
pub type PageEventHandlerGesture<'scope, 'listener, 'frame> =
    PageEventHandlerInputManagers<'scope, 'listener, 'frame>;

impl<'scope, 'listener, 'frame> PageEventHandlerInputManagers<'scope, 'listener, 'frame> {
    pub fn new(
        dispatcher: &EventDispatcher<'listener, 'frame>,
        owners: &ResidentCursorOwners,
        nodes: Rc<PageEventHandlerNodes>,
        geometry: Option<Rc<dyn PagePointerGeometry>>,
        controls: Option<Rc<PagePointerScrollControlOwners>>,
        scroll: Rc<ScrollManager>,
        view: Rc<dyn MouseFrameViewPort + 'scope>,
        features: Rc<dyn MouseFeatures + 'scope>,
    ) -> Result<Rc<Self>, PointerInputError> {
        let geometry = geometry.ok_or(PointerInputError::MissingGeometry)?;
        let controls = controls.ok_or(PointerInputError::MissingScrollControls)?;
        if dispatcher.mouse_frame_geometry.borrow().is_none() {
            return Err(PointerInputError::MissingGeometry);
        }
        assert!(Rc::ptr_eq(nodes.Targets(), &dispatcher.targets));
        assert!(Rc::ptr_eq(
            &owners.selection,
            &dispatcher.selection_controller_state
        ));
        Ok(Rc::new(Self {
            dispatcher: dispatcher.clone(),
            owners: owners.clone(),
            nodes,
            geometry,
            controls,
            scroll,
            view,
            features,
            errors: Rc::new(InputErrors::default()),
            dispatched: Cell::new(DispatchEventResult::default()),
        }))
    }
    pub fn Finish(&self) -> Result<(), PointerInputError> {
        self.errors.error.borrow_mut().take().map_or(Ok(()), Err)
    }
    pub fn TakeDispatchResult(&self) -> DispatchEventResult {
        self.dispatched.take()
    }
    fn Fault(&self, error: PointerInputError) {
        self.errors.Fault(error);
    }
    fn Run<T>(&self, rejected: T, run: impl FnOnce() -> Result<T, PointerInputError>) -> T {
        if self.errors.failed.get() {
            return rejected;
        }
        match run() {
            Ok(value) => value,
            Err(error) => {
                self.Fault(error);
                rejected
            }
        }
    }
    fn Record(&self, result: DispatchEventResult) {
        let previous = self.dispatched.get();
        self.dispatched.set(DispatchEventResult {
            target_node_id: result.target_node_id.or(previous.target_node_id),
            default_prevented: previous.default_prevented || result.default_prevented,
            propagation_stopped: previous.propagation_stopped || result.propagation_stopped,
            default_handled: previous.default_handled || result.default_handled,
        });
    }
    fn WithPointer<T>(
        &self,
        rejected: T,
        run: impl FnOnce(
            &PointerEventManager<'_>,
            &super::super::pointer_input::PagePointerContext<'_, '_, '_>,
        ) -> Result<T, PointerInputError>,
    ) -> T {
        self.Run(rejected, || {
            let (value, result) = self.dispatcher.WithPointerManager(
                &self.owners,
                self.geometry.clone(),
                self.controls.clone(),
                run,
            )?;
            self.Record(result);
            Ok(value)
        })
    }
    fn Defaults(&self) -> gesture_mouse_defaults::PageGestureMouseDefaults<'_, 'listener, 'frame> {
        gesture_mouse_defaults::PageGestureMouseDefaults::new(
            &self.dispatcher,
            &self.owners,
            self.geometry.clone(),
            self.controls.clone(),
        )
    }
    fn WithSelection<T>(
        &self,
        rejected: T,
        run: impl FnOnce(&Rc<mouse_selection::PageMouseSelection<'_, 'listener, 'frame>>) -> T,
    ) -> T {
        self.Run(rejected, || {
            let services = mouse_selection_services::PageMouseSelectionServicesOwner::new(
                &self.dispatcher,
                &self.owners,
                self.geometry.clone(),
                self.controls.clone(),
            );
            let selection = mouse_selection::PageMouseSelection::new(
                &self.dispatcher,
                self.geometry.clone(),
                &services,
            );
            let value = run(&selection);
            if let Some(error) = selection.TakeError() {
                return Err(match error {
                    mouse_selection::MouseSelectionError::SourceLeaf(leaf) => {
                        PointerInputError::SourceLeaf(leaf)
                    }
                    mouse_selection::MouseSelectionError::Geometry(error) => {
                        PointerInputError::GestureSelection(format!("{error:?}"))
                    }
                });
            }
            Ok(value)
        })
    }
    fn WithGesture<T>(
        &self,
        rejected: T,
        run: impl FnOnce(
            &GestureManager<'_, PageGestureHit, ScrollManager>,
            &Context<'_, '_, '_, '_>,
        ) -> T,
    ) -> T {
        self.WithPointer(rejected, |pointer, ctx| {
            let context = Rc::new(Context {
                pointer: ctx,
                errors: Default::default(),
            });
            let selection = Rc::new(gesture_selection::PageGestureSelection::new(
                &self.dispatcher,
                self.geometry.clone(),
            ));
            let gesture = GestureManager::WithState(
                self.owners.gesture.clone(),
                context.clone(),
                self.scroll.clone(),
                context.clone(),
                Rc::new(PointerBorrow(pointer)),
                Rc::new(SelectionPorts {
                    selection: selection.clone(),
                    errors: context.errors.clone(),
                    pointer_error: &ctx.error,
                    defaults: self.Defaults(),
                }),
            );
            let value = run(&gesture, context.as_ref());
            if let Some(error) = selection.TakeError() {
                return Err(PointerInputError::GestureSelection(format!("{error:?}")));
            }
            if let Some(error) = context.errors.borrow_mut().take() {
                return Err(error);
            }
            Ok(value)
        })
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
    fn Mouse(&self) -> MouseEventManager {
        MouseEventManager::WithState(self.owners.mouse.clone())
    }
}

impl EventHandlerSelection<Hit> for PageEventHandlerInputManagers<'_, '_, '_> {
    fn MouseDownWasSingleClickInSelection(&self) -> bool {
        self.owners.selection.MouseDownWasSingleClickInSelection()
    }
    fn InitializeSelectionState(&self) {
        self.WithSelection((), |s| s.Controller().InitializeSelectionState());
    }
    fn NotifySelectionChanged(&self) {
        self.WithSelection((), |s| s.Controller().NotifySelectionChanged());
    }
    fn HasExtendedSelection(&self) -> bool {
        self.WithSelection(false, |s| s.Controller().HasExtendedSelection())
    }
    fn IsSelectionOverLink(&self, result: &Hit, event: &WebMouseEvent) -> bool {
        IsSelectionOverLink(event, result.Native().IsOverLink())
    }
    fn PassMousePressEventToSubframe(&self, event: &MouseEventWithHitTestResults<Hit>) {
        self.Run((), || {
            self.Defaults()
                .PassPressToSubframe(&event.event, event.result.Native())
        });
    }
}

impl EventHandlerMouse<Hit> for PageEventHandlerInputManagers<'_, '_, '_> {
    fn DragSourceEndedAt(&self, event: &WebMouseEvent, operation: DragOperation) {
        self.Run((), || {
            mouse_drag::EndDrag(
                &self.dispatcher,
                &self.owners,
                self.geometry.clone(),
                self.controls.clone(),
                event,
                operation,
            )
        });
    }
    fn Clear(&self) {
        self.WithPointer((), |_, pointer| {
            let drag = mouse_drag::PageMouseDrag { pointer };
            self.Mouse().ClearWithDragPorts(&drag.Ports());
            Ok(())
        });
    }
    fn SetElementUnderMouse(&self, target: Option<NodeRef>, event: &WebMouseEvent) {
        let target = target.as_ref().and_then(|n| self.Target(n));
        self.WithPointer((), |_, ctx| {
            PointerMouseManager::SetElementUnderMouse(ctx, target, event);
            Ok(())
        });
    }
    fn NodeChildrenWillBeRemoved(&self, container: &NodeRef) {
        let Some(target) = self.Target(container) else {
            return;
        };
        self.Run((), || {
            let owner = self.dispatcher.document.borrow();
            self.Mouse().NodeChildrenWillBeRemovedWithRelations(
                &MouseRemovalRelations {
                    document: owner.GetDocument(),
                    targets: &self.dispatcher.targets,
                },
                self.features
                    .BoundaryEventDispatchTracksNodeRemovalEnabled(),
                &target,
            );
            Ok(())
        });
    }
    fn NodeWillBeRemoved(&self, node: &NodeRef) {
        let Some(target) = self.Target(node) else {
            return;
        };
        self.Run((), || {
            let owner = self.dispatcher.document.borrow();
            self.Mouse().NodeWillBeRemovedWithRelations(
                &MouseRemovalRelations {
                    document: owner.GetDocument(),
                    targets: &self.dispatcher.targets,
                },
                self.features
                    .BoundaryEventDispatchTracksNodeRemovalEnabled(),
                &target,
            );
            Ok(())
        });
    }
    fn HandlePseudoElementRemoval(&self, pseudo: &NodeRef) {
        let Some(target) = self.Target(pseudo) else {
            return;
        };
        self.Run((), || {
            let owner = self.dispatcher.document.borrow();
            self.Mouse().HandlePseudoElementRemovalWithRelations(
                &MouseRemovalRelations {
                    document: owner.GetDocument(),
                    targets: &self.dispatcher.targets,
                },
                self.features
                    .BoundaryEventDispatchTracksNodeRemovalEnabled(),
                &target,
            );
            Ok(())
        });
    }
    fn UpdateSelectionForMouseDrag(&self) {
        self.WithSelection((), |s| {
            let point = self.owners.mouse.LastKnownMousePositionInRootFrame();
            s.Controller().UpdateSelectionForMouseDragInRootFrame(
                (point.x() as f64, point.y() as f64),
                self.view.HasView(),
                s.as_ref(),
            );
        });
    }
    fn HandleMousePressEventUpdateStates(&self, event: &WebMouseEvent) {
        self.Run((), || {
            let p = event.PositionInRootFrame();
            let p = self.view.HasView().then(|| {
                self.view
                    .ConvertIntPointFromRootFrame(PointF::new(p.x().floor(), p.y().floor()))
            });
            self.owners
                .mouse
                .HandleMousePressEventUpdateStates(event, p);
            self.owners.selection.SetMouseDownMayStartSelect(false);
            Ok(())
        });
    }
    fn SetMousePressNode(&self, node: NodeRef) {
        let target = self.Target(&node);
        self.Run((), || {
            self.owners.mouse.SetMousePressNode(target.as_ref());
            Ok(())
        });
    }
    fn MousePressNode(&self) -> Option<NodeRef> {
        self.Node(self.owners.mouse.MousePressNode())
    }
    fn MousePressed(&self) -> bool {
        self.owners.mouse.MousePressed()
    }
    fn SetClickCount(&self, count: i32) {
        self.Run((), || {
            self.owners.mouse.SetClickCount(count);
            Ok(())
        });
    }
    fn SetMouseDownElement(&self, node: Option<NodeRef>) {
        let target = node.as_ref().and_then(|n| self.Target(n));
        self.Run((), || {
            self.owners.mouse.SetMouseDownElement(target.as_ref());
            Ok(())
        });
    }
    fn InvalidateClick(&self) {
        self.Run((), || {
            self.owners.mouse.InvalidateClick();
            Ok(())
        });
    }
    fn HandleMouseFocus(
        &self,
        result: &Hit,
        capabilities: &InputDeviceCapabilitiesRef,
    ) -> WebInputEventResult {
        self.Run(WebInputEventResult::kNotHandled, || {
            if self.dispatcher.focus_navigation.borrow().is_none() {
                return Err(PointerInputError::SourceLeaf(
                    "FocusController LocalFrame owner",
                ));
            }
            let nested = self.dispatcher.clone();
            let dispatch: SyntheticEventDispatcher<'_> = Rc::new(move |event, id| {
                nested.Dispatch(event, id);
            });
            let ports = crate::adapter::page_mouse_focus::PageMouseFocus::new(
                self.dispatcher.document.clone(),
                self.dispatcher.targets.clone(),
                self.dispatcher.state.clone(),
                self.dispatcher.editor.clone(),
                self.dispatcher.selections.clone(),
                dispatch,
                self.dispatcher.default_action_state.clone(),
                self.dispatcher.mouse_frame_geometry.borrow().clone(),
                self.dispatcher.focus_services.clone(),
                Rc::new(|| {
                    if let Err(error) = self.dispatcher.FocusNativeDocumentView() {
                        self.Fault(error);
                    }
                }),
                Rc::new(self.dispatcher.clone()),
            );
            let frame_view_scrollbar =
                match result.Native().GetScrollbar() {
                    None => false,
                    Some(scrollbar) => {
                        let owner = scrollbar.NativeScrollbarOwner().ok_or(
                            PointerInputError::SourceLeaf("HitTestResult native scrollbar owner"),
                        )?;
                        // The result roots this exact native owner for the query.
                        let native = unsafe { owner.Get().as_ref() }
                            .ok_or(PointerInputError::DetachedTarget)?;
                        native
                            .InputScrollState()
                            .ok_or(PointerInputError::SourceLeaf("Scrollbar::GetLayoutBox"))?
                            .is_layout_view
                    }
                };
            let hit = crate::adapter::page_mouse_focus::PageMouseFocusHit {
                over_scrollbar: result.Native().GetScrollbar().is_some(),
                frame_view_scrollbar,
            };
            Ok(crate::adapter::page_mouse_focus::HandleMouseFocus(
                &self.owners.mouse,
                &ports,
                &hit,
                Some(capabilities),
            ))
        })
    }
    fn HandleMousePressEvent(
        &self,
        event: &MouseEventWithHitTestResults<Hit>,
    ) -> WebInputEventResult {
        self.Run(WebInputEventResult::kNotHandled, || {
            self.Defaults()
                .Press(&event.event, &event.location, event.result.Native())
        })
    }
    fn SetLastKnownMousePosition(&self, event: &WebMouseEvent) {
        self.Run((), || {
            self.owners.mouse.SetLastKnownMousePosition(event);
            Ok(())
        });
    }
    fn HandleSvgPanIfNeeded(&self, release: bool) {
        self.Run((), || {
            self.Mouse()
                .HandleSvgPanIfNeeded(self.view.as_ref(), release);
            Ok(())
        });
    }
    fn ClearDragHeuristicState(&self) {
        self.Run((), || {
            self.owners.mouse.ClearDragHeuristicState();
            Ok(())
        });
    }
    fn HandleMouseDraggedEvent(
        &self,
        event: &MouseEventWithHitTestResults<Hit>,
    ) -> WebInputEventResult {
        self.Run(WebInputEventResult::kNotHandled, || {
            self.Defaults()
                .Drag(&event.event, &event.location, event.result.Native())
        })
    }
    fn ReleaseMousePress(&self) {
        self.Run((), || {
            self.owners.mouse.ReleaseMousePress();
            Ok(())
        });
    }
    fn HandleMouseReleaseEvent(
        &self,
        event: &MouseEventWithHitTestResults<Hit>,
    ) -> WebInputEventResult {
        self.Run(WebInputEventResult::kNotHandled, || {
            self.Defaults()
                .Release(&event.event, &event.location, event.result.Native())
        })
    }
    fn HandleMouseReleaseEventUpdateStates(&self) {
        self.Run((), || {
            self.owners.HandleMouseReleaseEventUpdateStates();
            Ok(())
        });
    }
    fn SetElementUnderMouseAndDispatchMouseUp(
        &self,
        target: Option<NodeRef>,
        event: &WebMouseEvent,
    ) -> WebInputEventResult {
        let target = target.as_ref().and_then(|n| self.Target(n));
        self.WithPointer(WebInputEventResult::kNotHandled, |_, ctx| {
            PointerMouseManager::SetElementUnderMouse(ctx, target.clone(), event);
            Ok(PointerMouseManager::DispatchMouseEvent(
                ctx,
                target,
                "mouseup",
                event,
                &self.owners.mouse.LastKnownMousePositionInRootFrame(),
            )
            .1)
        })
    }
    fn GetElementUnderMouse(&self) -> Option<NodeRef> {
        self.Node(self.owners.mouse.GetElementUnderMouse())
    }
    fn MouseDownMayStartDrag(&self) -> bool {
        self.owners.mouse.MouseDownMayStartDrag()
    }
    fn SetMouseDownMayStartAutoscroll(&self) {
        self.Run((), || {
            self.owners.mouse.SetMouseDownMayStartAutoscroll();
            Ok(())
        });
    }
    fn RecomputeMouseHoverStateIfNeeded(&self) {
        self.WithPointer((), |_, pointer| {
            let drag = mouse_drag::PageMouseDrag { pointer };
            self.Mouse().RecomputeMouseHoverStateWithPorts(
                self.view.as_ref(),
                self.features.as_ref(),
                &drag,
                self.dispatcher.clock.as_ref(),
            );
            Ok(())
        });
    }
    fn MarkHoverStateDirty(&self) {
        self.Run((), || {
            self.Mouse().MarkHoverStateDirty(self.view.as_ref());
            Ok(())
        });
    }
    fn IsMousePositionUnknown(&self) -> bool {
        self.owners.mouse.IsMousePositionUnknown()
    }
    fn SetLastMousePositionAsUnknown(&self) {
        self.Run((), || {
            self.owners.mouse.SetLastMousePositionAsUnknown();
            Ok(())
        });
    }
    fn LastKnownMousePositionInViewport(&self) -> HitTestLocation {
        HitTestLocation::FromPointF(self.owners.mouse.LastKnownMousePositionInViewport())
    }
    fn LastKnownMouseScreenPosition(&self) -> PointF {
        self.owners.mouse.LastKnownMouseScreenPosition()
    }
    fn DispatchContextMenuEvent(
        &self,
        target: Option<NodeRef>,
        event: &WebMouseEvent,
    ) -> WebInputEventResult {
        let target = target.as_ref().and_then(|n| self.Target(n));
        self.WithPointer(WebInputEventResult::kNotHandled, |_, ctx| {
            let dispatch = ctx.MouseDispatch();
            let result = self
                .Mouse()
                .DispatchContextMenuEventWithDispatch(
                    &dispatch,
                    target.as_ref(),
                    event,
                    Some(self.owners.mouse.LastKnownMousePositionInRootFrame()),
                    event.pointer_properties.id,
                    crate::core::input::mouse_event_manager::PointerTypeName(
                        event.pointer_properties.pointer_type,
                    ),
                )
                .1;
            if let Some(result) = dispatch.result.get() {
                ctx.result.set(Some(result));
            }
            Ok(result)
        })
    }
}

impl EventHandlerGesture<Hit> for PageEventHandlerInputManagers<'_, '_, '_> {
    fn HandleTouchDragEnd(&self, event: &WebMouseEvent, operation_is_none: bool) {
        self.WithGesture((), |manager, _| {
            manager.HandleTouchDragEnd(event, operation_is_none)
        });
    }
    fn GetPointerIdFromWebGestureEvent(&self, event: &WebGestureEvent) -> i32 {
        self.WithGesture(0, |manager, _| {
            manager.GetPointerIdFromWebGestureEvent(event)
        })
    }
    fn Clear(&self) {
        self.WithGesture((), |manager, _| manager.Clear());
    }
    fn GetHitTypeForGestureType(&self, event_type: Type) -> u32 {
        self.WithGesture(0, |manager, _| manager.GetHitTypeForGestureType(event_type))
    }
    fn HandleGestureEventInFrame(
        &self,
        event: &GestureEventWithHitTestResults<Hit>,
    ) -> WebInputEventResult {
        self.WithGesture(WebInputEventResult::kNotHandled, |manager, context| {
            manager.HandleGestureEventInFrame(&GestureEventWithHitTestResults {
                event: event.event.clone(),
                location: event.location.clone(),
                result: PageGestureHit {
                    result: event.result.Native().clone(),
                    document: self.dispatcher.document.clone(),
                    targets: self.dispatcher.targets.clone(),
                    controls: self.controls.clone(),
                    error: context.errors.clone(),
                },
            })
        })
    }
    fn GestureContextMenuDeferred(&self) -> bool {
        self.WithGesture(false, |manager, _| manager.GestureContextMenuDeferred())
    }
    fn ResetLongTapContextMenuStates(&self) {
        self.WithGesture((), |manager, _| manager.ResetLongTapContextMenuStates());
    }
}
