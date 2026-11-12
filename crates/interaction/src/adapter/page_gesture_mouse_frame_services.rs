//! Source mouse press/release frame leaves. All retained state comes from
//! EventHandler; this call-scoped adapter owns neither a node tree nor Layout.
use super::*;
use crate::adapter::{
    page_pointer_hit_test::PagePointerGeometry, source_input_managers::ResidentCursorOwners,
};
use crate::core::input::mouse_event_manager::{MouseDragSourceResetPort, MouseEventManager};
use interaction_default::core::page::focus_controller::FocusDispatchScope;

pub(crate) struct PageGestureMouseFrameServices<'a, 'listener, 'frame> {
    pub(crate) dispatcher: &'a EventDispatcher<'listener, 'frame>,
    pub(crate) owners: &'a ResidentCursorOwners,
    pub(crate) geometry: Rc<dyn PagePointerGeometry>,
    error: RefCell<Option<PointerInputError>>,
}
impl<'a, 'listener, 'frame> PageGestureMouseFrameServices<'a, 'listener, 'frame> {
    pub(crate) fn new(
        dispatcher: &'a EventDispatcher<'listener, 'frame>,
        owners: &'a ResidentCursorOwners,
        geometry: Rc<dyn PagePointerGeometry>,
    ) -> Self {
        Self {
            dispatcher,
            owners,
            geometry,
            error: RefCell::new(None),
        }
    }
    pub(crate) fn Fault(&self, leaf: &'static str) {
        self.error
            .borrow_mut()
            .get_or_insert(PointerInputError::SourceLeaf(leaf));
    }
    pub(crate) fn RecordError(&self, error: PointerInputError) {
        self.error.borrow_mut().get_or_insert(error);
    }
    pub(crate) fn TakeError(&self) -> Option<PointerInputError> {
        self.error.borrow_mut().take()
    }
    pub(crate) fn ResetDragSource(&self) {
        MouseEventManager::ResetDragSourceWithSourcePorts(self);
    }
    pub(crate) fn StopSelectionAutoscrollIfNeeded(&self) {
        let services = self.owners.autoscroll_services.borrow().clone();
        match services {
            Some(services) => services.StopSelectionAutoscrollIfNeeded(),
            None => self.Fault("AutoscrollController frame services"),
        }
    }
    pub(crate) fn FocusDocumentView(&self) {
        if let Err(error) = self.dispatcher.FocusNativeDocumentView() {
            self.error.borrow_mut().get_or_insert(error);
        }
    }
    pub(crate) fn IsSvgDocumentWithZoomAndPan(&self) -> bool {
        let owner = self.dispatcher.document.borrow();
        let document = owner.GetDocument();
        document.DocumentType(document.Root()) == dom::persistent_document::DOMDocumentType::kSVG
            && document.SVGZoomAndPanEnabled()
    }
    pub(crate) fn StartSvgPan(&self, point: PointF) {
        self.dispatcher
            .document
            .borrow_mut()
            .GetDocumentMut()
            .StartSVGPan([point.x(), point.y()]);
    }
}
impl EventDispatcher<'_, '_> {
    pub(crate) fn FocusNativeDocumentView(&self) -> Result<(), PointerInputError> {
        let navigation =
            self.focus_navigation
                .borrow()
                .clone()
                .ok_or(PointerInputError::SourceLeaf(
                    "FocusController LocalFrame owner",
                ))?;
        navigation.Controller().FocusDocumentViewWithScope(
            Some(navigation.Frame()),
            true,
            &FocusDispatchScope {
                focus_events: Some(self),
                ..Default::default()
            },
        );
        Ok(())
    }
}
impl MouseDragSourceResetPort for PageGestureMouseFrameServices<'_, '_, '_> {
    fn DragSource(&self) -> Option<EventTargetRef> {
        self.owners.drag.GetDragState().drag_src.borrow().clone()
    }
    fn SetDragSource(&self, source: Option<&EventTargetRef>) {
        self.owners
            .drag
            .GetDragState()
            .drag_src
            .replace(source.cloned());
    }
    fn HasPage(&self) -> bool {
        self.dispatcher.focus_services.lifecycle.HasFrame()
    }
    fn DragSourceHasFrame(&self, target: &EventTargetRef) -> bool {
        let Some(id) = self.dispatcher.targets.NodeId(target) else {
            return false;
        };
        let owner = self.dispatcher.document.borrow();
        // Detached nodes retain their Document/Frame; only removal of the
        // source Window's Frame takes source ResetDragSource's !GetFrame branch.
        owner.GetDocument().FindNodeById(id).is_some() && self.HasPage()
    }
    fn DragSourceFrameIsDescendantOfFrame(&self, target: &EventTargetRef) -> bool {
        // DomEventTargets only resolves nodes belonging to this LocalFrame's
        // canonical Document. Foreign-frame identities are never remapped.
        self.dispatcher.targets.NodeId(target).is_some()
    }
}
