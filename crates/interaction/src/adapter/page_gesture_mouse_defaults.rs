//! Gesture compatibility-mouse defaults enter the retained source MouseEventManager
//! and full SelectionController with call-scoped native ports.
use super::*;
use super::{
    mouse_frame_services::PageGestureMouseFrameServices,
    mouse_selection::{MouseSelectionError, PageMouseSelection},
    mouse_selection_services::PageMouseSelectionServicesOwner,
    pointer_input::PointerInputError,
};
use crate::adapter::{
    page_pointer_hit_test::{PagePointerGeometry, PointerHitResult},
    source_input_managers::ResidentCursorOwners,
};
use crate::core::input::mouse_event_manager::{
    MouseDefaultHitTest, MouseEventManager, MousePressDefaultPort, MouseReleaseDefaultPort,
};
use interaction_default::core::editing::selection_controller::{
    MouseEventWithSelectionHit, SelectionGeometry,
};
use interaction_dispatch::core::input::event_handling_util::WebInputEventResult;
use interaction_hit_test::core::layout::hit_test_location::HitTestLocation;
use interaction_input::public::common::input::{
    web_mouse_event::WebMouseEvent, web_pointer_properties::PointF,
};

pub(crate) struct PageGestureMouseDefaults<'a, 'listener, 'frame> {
    dispatcher: &'a EventDispatcher<'listener, 'frame>,
    owners: &'a ResidentCursorOwners,
    geometry: Rc<dyn PagePointerGeometry>,
    frame: PageGestureMouseFrameServices<'a, 'listener, 'frame>,
    selection_services: PageMouseSelectionServicesOwner<'a, 'listener, 'frame>,
}
impl<'a, 'listener, 'frame> PageGestureMouseDefaults<'a, 'listener, 'frame> {
    pub(crate) fn new(
        dispatcher: &'a EventDispatcher<'listener, 'frame>,
        owners: &'a ResidentCursorOwners,
        geometry: Rc<dyn PagePointerGeometry>,
        controls: Rc<crate::adapter::page_pointer_scroll_controls::PagePointerScrollControlOwners>,
    ) -> Self {
        Self {
            dispatcher,
            owners,
            geometry: geometry.clone(),
            frame: PageGestureMouseFrameServices::new(dispatcher, owners, geometry.clone()),
            selection_services: PageMouseSelectionServicesOwner::new(
                dispatcher, owners, geometry, controls,
            ),
        }
    }
    pub(crate) fn UpdateDragAutoscroll(
        &self,
        layout: Option<
            Rc<dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout>,
        >,
        event: &WebMouseEvent,
    ) -> Result<(), PointerInputError> {
        let selection = PageMouseSelection::new(
            self.dispatcher,
            self.geometry.clone(),
            &self.selection_services,
        );
        let ports = MouseDefaultPorts {
            parent: self,
            selection: &selection,
        };
        interaction_default::core::page::autoscroll_controller::AutoscrollController::WithState(
            &ports,
            &self.owners.autoscroll,
        )
        .UpdateDragAndDrop(layout, event.PositionInRootFrame(), event.TimeStamp());
        self.Finish(&selection, WebInputEventResult::kNotHandled)
            .map(|_| ())
    }
    pub(crate) fn Press(
        &self,
        event: &WebMouseEvent,
        _location: &HitTestLocation,
        result: &PointerHitResult,
    ) -> Result<WebInputEventResult, PointerInputError> {
        let selection = PageMouseSelection::new(
            self.dispatcher,
            self.geometry.clone(),
            &self.selection_services,
        );
        let ports = MouseDefaultPorts {
            parent: self,
            selection: &selection,
        };
        let hit = MouseDefaultHit {
            event,
            result,
            targets: &self.dispatcher.targets,
            draggable: selection.IsUserNodeDraggable(result),
        };
        let result = MouseEventManager::WithState(self.owners.mouse.clone())
            .HandleMousePressEventWithDefaultPorts(&ports, &hit);
        self.Finish(&selection, result)
    }
    /// Gesture repeated-tap uses SelectionController directly after the source
    /// mouse release, preserving its separate call point and retained flags.
    pub(crate) fn PassPressToSubframe(
        &self,
        event: &WebMouseEvent,
        result: &PointerHitResult,
    ) -> Result<(), PointerInputError> {
        let selection = PageMouseSelection::new(
            self.dispatcher,
            self.geometry.clone(),
            &self.selection_services,
        );
        selection.Controller().PassMousePressEventToSubframe(
            &MouseEventWithSelectionHit { event, hit: result },
            selection.as_ref(),
        );
        self.Finish(&selection, WebInputEventResult::kNotHandled)
            .map(|_| ())
    }
    pub(crate) fn SelectionPress(
        &self,
        event: &WebMouseEvent,
        _location: &HitTestLocation,
        result: &PointerHitResult,
    ) -> Result<(), PointerInputError> {
        let selection = PageMouseSelection::new(
            self.dispatcher,
            self.geometry.clone(),
            &self.selection_services,
        );
        selection.Controller().HandleMousePressEvent(
            &MouseEventWithSelectionHit { event, hit: result },
            selection.as_ref(),
        );
        self.Finish(&selection, WebInputEventResult::kNotHandled)
            .map(|_| ())
    }
    pub(crate) fn Release(
        &self,
        event: &WebMouseEvent,
        _location: &HitTestLocation,
        result: &PointerHitResult,
    ) -> Result<WebInputEventResult, PointerInputError> {
        let selection = PageMouseSelection::new(
            self.dispatcher,
            self.geometry.clone(),
            &self.selection_services,
        );
        let ports = MouseDefaultPorts {
            parent: self,
            selection: &selection,
        };
        let hit = MouseDefaultHit {
            event,
            result,
            targets: &self.dispatcher.targets,
            draggable: selection.IsUserNodeDraggable(result),
        };
        let result = MouseEventManager::WithState(self.owners.mouse.clone())
            .HandleMouseReleaseEventWithDefaultPorts(&ports, &hit);
        self.Finish(&selection, result)
    }
    fn Finish(
        &self,
        selection: &PageMouseSelection<'_, '_, '_>,
        result: WebInputEventResult,
    ) -> Result<WebInputEventResult, PointerInputError> {
        if let Some(error) = self.frame.TakeError() {
            return Err(error);
        }
        if let Some(error) = selection.TakeError() {
            return Err(match error {
                MouseSelectionError::SourceLeaf(leaf) => PointerInputError::SourceLeaf(leaf),
                MouseSelectionError::Geometry(error) => {
                    PointerInputError::GestureSelection(format!("{error:?}"))
                }
            });
        }
        // SelectionController publishes native DOM endpoints during every
        // synchronous mouse transaction, including drag and autoscroll. Text
        // controls retain those same endpoints as UTF-16 cached selection,
        // while the package-side discriminator must be restored after the
        // transaction so the final mouse event reaches caret/selection Paint.
        self.dispatcher
            .FrameSelection()
            .ActivateFocusedTextControlSelection(
                self.dispatcher.state.borrow().focused_node_id,
            );
        Ok(result)
    }
}
struct MouseDefaultHit<'a> {
    event: &'a WebMouseEvent,
    result: &'a PointerHitResult,
    targets: &'a DomEventTargets,
    draggable: bool,
}
impl MouseDefaultHitTest for MouseDefaultHit<'_> {
    fn DefaultEvent(&self) -> &WebMouseEvent {
        self.event
    }
    fn DefaultInnerNode(&self) -> Option<EventTargetRef> {
        self.targets
            .ResolveTarget(self.result.InnerNode()?.DOMNodeId()?)
    }
    fn DefaultIsSelectionOverLink(&self) -> bool {
        interaction_default::core::editing::selection_controller::IsSelectionOverLink(
            self.event,
            self.result.IsOverLink(),
        )
    }
    fn DefaultIsExtendingSelection(&self) -> bool {
        interaction_default::core::editing::selection_controller::IsExtendingSelection(
            self.event,
            self.result.IsOverLink(),
            self.result.GetImage().is_some(),
            self.draggable,
        )
    }
}
struct MouseDefaultPorts<'a, 'scope, 'listener, 'frame> {
    parent: &'a PageGestureMouseDefaults<'scope, 'listener, 'frame>,
    selection: &'a Rc<PageMouseSelection<'a, 'listener, 'frame>>,
}
impl MousePressDefaultPort<MouseDefaultHit<'_>> for MouseDefaultPorts<'_, '_, '_, '_> {
    fn ResetDragSource(&self) {
        self.parent.frame.ResetDragSource()
    }
    fn IsSvgDocumentWithZoomAndPan(&self) -> bool {
        self.parent.frame.IsSvgDocumentWithZoomAndPan()
    }
    fn ConvertFromRootFrame(&self, p: PointF) -> PointF {
        let p = self
            .selection
            .ConvertRootPointToContents((p.x() as f64, p.y() as f64));
        PointF::new(p.0 as f32, p.1 as f32)
    }
    fn StartSvgPan(&self, p: PointF) {
        self.parent.frame.StartSvgPan(p)
    }
    fn FocusDocumentView(&self) {
        self.parent.frame.FocusDocumentView()
    }
    fn SetSequentialFocusNavigationStartingPoint(&self, n: Option<&EventTargetRef>) {
        let id = n.and_then(|n| self.parent.dispatcher.targets.NodeId(n));
        crate::adapter::page_focus_controller::SetSequentialFocusNavigationStartingPoint(
            &self.parent.dispatcher.document,
            id,
        )
    }
    fn HandleSelectionMousePress(&self, h: &MouseDefaultHit<'_>) -> bool {
        self.selection.Controller().HandleMousePressEvent(
            &MouseEventWithSelectionHit {
                event: h.event,
                hit: h.result,
            },
            self.selection.as_ref(),
        )
    }
    fn MouseDownMayStartSelect(&self) -> bool {
        self.parent.owners.selection.MouseDownMayStartSelect()
    }
    fn HasUserScrollableLayoutBox(&self, n: &EventTargetRef) -> bool {
        self.parent.dispatcher.targets.NodeId(n).is_some_and(|id| {
            self.parent
                .dispatcher
                .focus_services
                .layout
                .IsUserScrollable(id)
        })
    }
}
impl MouseReleaseDefaultPort<MouseDefaultHit<'_>> for MouseDefaultPorts<'_, '_, '_, '_> {
    fn StopSelectionAutoscrollIfNeeded(&self) {
        if self
            .parent
            .owners
            .autoscroll
            .SelectionAutoscrollInProgress()
        {
            self.parent.owners.autoscroll.StopAutoscroll(self);
        }
    }
    fn HandleSelectionMouseRelease(&self, h: &MouseDefaultHit<'_>, p: PointF) -> bool {
        self.selection.Controller().HandleMouseReleaseEvent(
            &MouseEventWithSelectionHit {
                event: h.event,
                hit: h.result,
            },
            (p.x() as f64, p.y() as f64),
            self.selection.as_ref(),
        )
    }
}

impl PageGestureMouseDefaults<'_, '_, '_> {
    pub(crate) fn Drag(
        &self,
        event: &WebMouseEvent,
        _location: &HitTestLocation,
        result: &PointerHitResult,
    ) -> Result<WebInputEventResult, PointerInputError> {
        let selection = PageMouseSelection::new(
            self.dispatcher,
            self.geometry.clone(),
            &self.selection_services,
        );
        let ports = MouseDefaultPorts {
            parent: self,
            selection: &selection,
        };
        let hit = MouseDefaultHit {
            event,
            result,
            targets: &self.dispatcher.targets,
            draggable: selection.IsUserNodeDraggable(result),
        };
        let result = MouseEventManager::WithState(self.owners.mouse.clone())
            .HandleMouseDraggedEventWithDefaultPorts(&ports, &hit);
        self.Finish(&selection, result)
    }
    pub(crate) fn AnimateSelectionAutoscroll(
        &self,
        now: interaction_input::base::time::time::TimeTicks,
    ) -> Result<(), PointerInputError> {
        let Some(owner) = self.owners.autoscroll_services.borrow().clone() else {
            return Ok(());
        };
        let selection = PageMouseSelection::new(
            self.dispatcher,
            self.geometry.clone(),
            &self.selection_services,
        );
        let ports = MouseDefaultPorts {
            parent: self,
            selection: &selection,
        };
        owner.Animate(&ports, now);
        self.Finish(&selection, WebInputEventResult::kNotHandled)
            .map(|_| ())
    }
}
struct MouseSelectionDragLayout(
    Rc<dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout>,
);
impl crate::core::input::mouse_event_manager::MouseDragLayout for MouseSelectionDragLayout {
    fn AutoscrollLayout(
        &self,
    ) -> Rc<dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout> {
        self.0.clone()
    }
}
impl crate::core::input::mouse_event_manager::MouseDraggedDefaultPort<MouseDefaultHit<'_>>
    for MouseDefaultPorts<'_, '_, '_, '_>
{
    // Source Settings.json5:354-355; this Page has no overriding mutation.
    fn BarrelButtonForDragEnabled(&self) -> bool {
        false
    }
    fn HandleDrag(
        &self,
        _: &MouseEventManager,
        hit: &MouseDefaultHit<'_>,
        initiator: crate::core::input::mouse_event_manager::DragAndDropToolType,
    ) -> crate::core::input::mouse_event_manager::DragHandlingResult {
        let result = super::mouse_drag::HandleDrag(
            self.parent.dispatcher,
            self.parent.owners,
            self.parent.geometry.clone(),
            self.parent.selection_services.Controls(),
            hit.event,
            &HitTestLocation::FromPointF(hit.event.PositionInRootFrame()),
            hit.result,
            initiator,
        );
        match result {
            Ok(result) => result,
            Err(error) => {
                self.parent.frame.RecordError(error);
                crate::core::input::mouse_event_manager::DragHandlingResult::kNotHandled
            }
        }
    }
    fn FlatTreeParent(&self, n: &EventTargetRef) -> Option<EventTargetRef> {
        let id = self.parent.dispatcher.targets.NodeId(n)?;
        let owner = self.parent.dispatcher.document.borrow();
        let d = owner.GetDocument();
        self.parent
            .dispatcher
            .targets
            .TargetFromBorrowedDocument(d, d.Node(d.FlatTreeParent(d.FindNodeById(id)?)?).Id())
    }
    fn GetLayoutObject(
        &self,
        n: &EventTargetRef,
    ) -> Option<crate::core::input::mouse_event_manager::MouseDragLayoutRef> {
        let id = self.parent.dispatcher.targets.NodeId(n)?;
        let frame = self.parent.dispatcher.mouse_frame_geometry.borrow();
        Some(Rc::new(MouseSelectionDragLayout(
            frame.as_ref()?.AutoscrollLayoutForNode(id)?,
        )))
    }
    fn IsMenuListSelect(&self, n: &EventTargetRef) -> bool {
        let Some(id) = self.parent.dispatcher.targets.NodeId(n) else {
            return false;
        };
        let owner = self.parent.dispatcher.document.borrow();
        let d = owner.GetDocument();
        d.FindNodeById(id)
            .is_some_and(|i| d.Node(i).IsHTMLElement("select") && d.SelectUsesMenuList(i))
    }
    fn PrepareForMouseDrag(&self) {
        if let Err(error) = self.parent.geometry.PrepareLifecycleExceptPaint() {
            self.parent
                .frame
                .RecordError(PointerInputError::Geometry(error));
        }
    }
    fn HandleSelectionMouseDragged(
        &self,
        h: &MouseDefaultHit<'_>,
        down: PointF,
        _last: PointF,
    ) -> WebInputEventResult {
        self.selection.Controller().HandleMouseDraggedEvent(
            &MouseEventWithSelectionHit {
                event: h.event,
                hit: h.result,
            },
            (down.x() as f64, down.y() as f64),
            self.selection.as_ref(),
        )
    }
    fn MiddleClickAutoscrollInProgress(&self) -> bool {
        self.parent
            .owners
            .autoscroll
            .MiddleClickAutoscrollInProgress()
    }
    fn SelectedHtmlForClipboardIsEmpty(&self) -> bool {
        match self
            .parent
            .dispatcher
            .focus_services
            .selection
            .SelectedHtmlForClipboard()
        {
            Ok(html) => html.is_empty(),
            Err(error) => {
                self.parent
                    .frame
                    .RecordError(PointerInputError::GestureSelection(format!("{error:?}")));
                true
            }
        }
    }
    fn HasAutoscrollController(&self) -> bool {
        self.parent.owners.autoscroll_services.borrow().is_some()
    }
    fn PrepareForSelectionAutoscroll(
        &self,
        _: &crate::core::input::mouse_event_manager::MouseDragLayoutRef,
    ) {
        if let Err(error) = self.parent.geometry.PrepareLifecycleExceptPaint() {
            self.parent
                .frame
                .RecordError(PointerInputError::Geometry(error));
        }
    }
    fn StartAutoscrollForSelection(
        &self,
        layout: &crate::core::input::mouse_event_manager::MouseDragLayoutRef,
    ) {
        if let Some(owner) = self.parent.owners.autoscroll_services.borrow().clone() {
            owner.StartAutoscrollForSelection(self, layout.AutoscrollLayout());
        }
    }
}
impl interaction_default::core::page::autoscroll_controller::AutoscrollStopServices
    for MouseDefaultPorts<'_, '_, '_, '_>
{
    fn StopNodeAutoscroll(
        &self,
        layout: &dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout,
    ) {
        if let Some(id) = layout.NodeId() {
            crate::adapter::page_input_type::StopSelectControlAutoscroll(
                &self.parent.dispatcher.Context(),
                id,
            );
        }
    }

    fn NotifyAutoscrollForSelectionInMainFrame(&self, active: bool) {
        if let Some(owner) = self.parent.owners.autoscroll_services.borrow().clone() {
            owner
                .StopServices()
                .NotifyAutoscrollForSelectionInMainFrame(active);
        }
    }
    fn SystemAutoscrollEnd(&self) {
        if let Some(owner) = self.parent.owners.autoscroll_services.borrow().clone() {
            owner.StopServices().SystemAutoscrollEnd();
        }
    }
    fn SetCursorOverridden(&self, value: bool) {
        if let Some(owner) = self.parent.owners.autoscroll_services.borrow().clone() {
            owner.StopServices().SetCursorOverridden(value);
        }
    }
    fn UpdateCursor(&self) {
        if let Some(owner) = self.parent.owners.autoscroll_services.borrow().clone() {
            owner.StopServices().UpdateCursor();
        }
    }
}
impl interaction_default::core::page::autoscroll_controller::SelectionAutoscrollFrame
    for MouseDefaultPorts<'_, '_, '_, '_>
{
    fn LastKnownMousePositionInRootFrame(
        &self,
        _: &dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout,
    ) -> PointF {
        self.parent.owners.mouse.LastKnownMousePositionInRootFrame()
    }
    fn MousePressed(
        &self,
        _: &dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout,
    ) -> bool {
        self.parent.owners.mouse.MousePressed()
    }
    fn UpdateSelectionForMouseDrag(
        &self,
        _: &dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout,
    ) {
        let point = self.parent.owners.mouse.LastKnownMousePositionInRootFrame();
        self.selection
            .Controller()
            .UpdateSelectionForMouseDragInRootFrame(
                (point.x() as f64, point.y() as f64),
                self.parent
                    .frame
                    .dispatcher
                    .focus_services
                    .lifecycle
                    .HasFrame(),
                self.selection.as_ref(),
            );
    }
    fn AssertScrollLifecyclePrepared(
        &self,
        _: &dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout,
    ) {
        debug_assert!(self.parent.dispatcher.focus_services.lifecycle.HasFrame());
    }
    fn ScheduleMainThreadAnimation(
        &self,
        _: &dyn interaction_default::core::page::autoscroll_controller::AutoscrollLayout,
    ) {
        if let Some(frame) = self
            .parent
            .dispatcher
            .mouse_frame_geometry
            .borrow()
            .as_ref()
        {
            frame.ScheduleAutoscrollAnimation();
        }
    }
}
