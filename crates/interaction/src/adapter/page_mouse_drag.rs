//! Call-scoped source drag transaction. DragState and DataTransfer retain their
//! canonical owners; each native Layout query releases its borrow before JS.
use super::pointer_input::PagePointerContext;
use super::*;
use crate::adapter::{
    page_pointer_hit_test::{PagePointerGeometry, PointerHitResult},
    page_pointer_scroll_controls::PagePointerScrollControlOwners,
    source_input_managers::ResidentCursorOwners,
};
use crate::core::input::mouse_event_manager::{
    DragAndDropToolType, DragHandlingResult, MouseDragEvent, MouseDragFramePort, MouseDragNodePort,
    MouseDragPort, MouseDragSelectionPort, MouseDragTransactionPorts, MouseEventDispatchPort,
    MouseEventManager,
};
use crate::core::input::pointer_event_manager::PointerEventDOMPort;
use foundation::{gfx, EUserDrag};

use interaction_default::core::editing::gesture_selection::GestureSelectionGeometry;
use interaction_default::core::page::drag_controller::{
    DragLayoutCursor, DragOverlay, DragSourceContext, DraggableNodeContext, SystemDragRequest,
    SystemDragSupport,
};
use interaction_default::core::page::drag_state::{DragSourceAction, DragState};
use interaction_dispatch::core::{
    clipboard::data_transfer::{DataTransfer, DragOperation, WebDragData},
    events::{
        drag_event::DragEvent,
        mouse_event::{MouseEventInit, SyntheticEventType},
    },
};
use interaction_hit_test::core::layout::{
    hit_test_location::HitTestLocation, hit_test_request::RequestType,
};
use interaction_input::{
    base::time::time::TimeTicks,
    public::common::input::{
        web_pointer_event::WebPointerEvent, web_pointer_properties::PointerType,
    },
};
use layoutng_assembly::native_mouse_coordinates::NativeMouseCoordinateLayout;

pub(crate) fn HandleDrag(
    dispatcher: &EventDispatcher<'_, '_>,
    owners: &ResidentCursorOwners,
    geometry: Rc<dyn PagePointerGeometry>,
    controls: Rc<PagePointerScrollControlOwners>,
    event: &WebMouseEvent,
    _location: &HitTestLocation,
    _hit: &PointerHitResult,
    initiator: DragAndDropToolType,
) -> Result<DragHandlingResult, PointerInputError> {
    let (result, _) = dispatcher.WithPointerManager(owners, geometry, controls, |_, pointer| {
        let context = PageMouseDrag { pointer };
        let hit = DragHit { event };
        let result = MouseEventManager::WithState(owners.mouse.clone()).HandleDragWithPorts(
            &context.Ports(),
            &hit,
            initiator,
        );
        Ok(result)
    })?;
    Ok(result)
}
pub(crate) fn HandleTouchDrag(
    dispatcher: &EventDispatcher<'_, '_>,
    owners: &ResidentCursorOwners,
    geometry: Rc<dyn PagePointerGeometry>,
    controls: Rc<PagePointerScrollControlOwners>,
    gesture: &crate::core::input::gesture_manager::GestureEventWithHitTestResults<
        super::gesture_input::PageGestureHit,
    >,
    pointer_id: i32,
) -> Result<DragHandlingResult, PointerInputError> {
    let (result, _) = dispatcher.WithPointerManager(owners, geometry, controls, |_, pointer| {
        let context = PageMouseDrag { pointer };
        let manager = MouseEventManager::WithState(owners.mouse.clone());
        let event = manager.PrepareDragDropMouseEvent(
            dispatcher.clock.as_ref(),
            &gesture.event,
            pointer_id,
        );
        let _hit = pointer
            .geometry
            .HitTest(
                HitTestLocation::FromPointF(event.PositionInRootFrame()),
                RequestType::kReadOnly as u32,
            )
            .map_err(PointerInputError::Geometry)?;
        let hit = DragHit { event: &event };
        Ok(manager.HandleDragDropIfPossibleWithPorts(
            &context.Ports(),
            &hit,
            gesture.event.primary_pointer_type,
        ))
    })?;
    Ok(result)
}
pub(crate) fn EndDrag(
    dispatcher: &EventDispatcher<'_, '_>,
    owners: &ResidentCursorOwners,
    geometry: Rc<dyn PagePointerGeometry>,
    controls: Rc<PagePointerScrollControlOwners>,
    event: &WebMouseEvent,
    operation: DragOperation,
) -> Result<(), PointerInputError> {
    dispatcher.WithPointerManager(owners, geometry, controls, |_, pointer| {
        let context = PageMouseDrag { pointer };
        MouseEventManager::WithState(owners.mouse.clone()).DragSourceEndedAtWithPorts(
            &context.Ports(),
            event,
            operation,
        );
        owners.drag.DragEnded();
        Ok(())
    })?;
    Ok(())
}
struct DragHit<'a> {
    event: &'a WebMouseEvent,
}
impl MouseDragEvent for DragHit<'_> {
    fn DragEvent(&self) -> &WebMouseEvent {
        self.event
    }
}
pub(super) struct PageMouseDrag<'a, 'scope, 'listener, 'frame> {
    pub(super) pointer: &'a PagePointerContext<'scope, 'listener, 'frame>,
}
impl<'a, 'scope, 'listener, 'frame> PageMouseDrag<'a, 'scope, 'listener, 'frame> {
    pub(super) fn Ports(&self) -> MouseDragTransactionPorts<'_, Self> {
        MouseDragTransactionPorts {
            nodes: self,
            dispatch: self,
            view: self,
            selection: self,
            drag: self,
            suppress_pointer_stream_after_drag:
                foundation::RuntimeEnabledFeatures::SuppressPointerStreamAfterDragEnabled(),
        }
    }
    fn Fault(&self, leaf: &'static str) {
        self.pointer.Fault(PointerInputError::SourceLeaf(leaf));
    }
    fn Hit(&self, p: PointF) -> Option<PointerHitResult> {
        match self.pointer.geometry.HitTest(
            HitTestLocation::FromPointF(p),
            RequestType::kReadOnly as u32,
        ) {
            Ok(hit) => Some(hit.result),
            Err(error) => {
                self.pointer.Fault(PointerInputError::Geometry(error));
                None
            }
        }
    }
    fn GestureSelection(
        &self,
    ) -> super::gesture_selection::PageGestureSelection<'scope, 'listener, 'frame> {
        super::gesture_selection::PageGestureSelection::new(
            self.pointer.dispatcher,
            self.pointer.geometry.clone(),
        )
    }
    fn ImageContent(
        &self,
        node: &EventTargetRef,
    ) -> Option<Rc<dom::persistent_document::DOMImageResourceContent>> {
        let layout = self.pointer.frame.DragLayout(self.SourceId(node)?)?;
        let facts = layout.DragResourceImageFacts()?;
        self.pointer
            .dispatcher
            .document
            .borrow()
            .GetDocument()
            .ImageResourceContentById(facts.resource_id?)
    }
    fn SourceId(&self, n: &EventTargetRef) -> Option<u64> {
        self.pointer.dispatcher.targets.NodeId(n)
    }
}
impl MouseDragNodePort for PageMouseDrag<'_, '_, '_, '_> {
    fn SameDocument(&self, a: &EventTargetRef, b: &EventTargetRef) -> bool {
        self.pointer.dispatcher.targets.OwnerDocumentHandle(a)
            == self.pointer.dispatcher.targets.OwnerDocumentHandle(b)
    }
}
impl MouseDragFramePort for PageMouseDrag<'_, '_, '_, '_> {
    fn HasView(&self) -> bool {
        self.pointer.frame.HasFrameView()
    }
    fn HasPage(&self) -> bool {
        self.pointer.dispatcher.focus_services.lifecycle.HasFrame()
    }
    fn ConvertIntPointFromRootFrame(&self, p: PointF) -> PointF {
        self.pointer
            .frame
            .ConvertFromRootFrame(p)
            .map(|p| PointF::new(p.x().round(), p.y().round()))
            .unwrap_or_default()
    }
}
impl MouseDragSelectionPort for PageMouseDrag<'_, '_, '_, '_> {
    fn MouseDownMayStartSelect(&self) -> bool {
        self.pointer.owners.selection.MouseDownMayStartSelect()
    }
    fn SelectionStartsInPasswordField(&self) -> bool {
        let Some(selection) = self.pointer.dispatcher.focus_services.selection.Current() else {
            return false;
        };
        let owner = self.pointer.dispatcher.document.borrow();
        let d = owner.GetDocument();
        let mut node = selection.anchor.node_id.and_then(|id| d.FindNodeById(id));
        while let Some(i) = node {
            let n = d.Node(i);
            if n.IsHTMLElement("input")
                && n.FindAttribute("type")
                    .is_some_and(|a| a.value.eq_ignore_ascii_case("password"))
            {
                return true;
            }
            node = n.Parent().or_else(|| d.OwnerShadowHost(i));
        }
        false
    }
}
impl MouseEventDispatchPort for PageMouseDrag<'_, '_, '_, '_> {
    fn DocumentWindow(&self, n: &EventTargetRef) -> Option<AbstractViewRef> {
        PointerEventDOMPort::ViewForElement(self.pointer, n)
    }
    fn FrameDocumentWindow(&self) -> Option<AbstractViewRef> {
        PointerEventDOMPort::View(self.pointer)
    }
    fn CoordinateContext(&self) -> MouseCoordinateContext {
        self.pointer.dispatcher.MouseCoordinateContext()
    }
    fn SourceCapabilities(&self, _: &AbstractViewRef, touch: bool) -> InputDeviceCapabilitiesRef {
        self.pointer
            .dispatcher
            .targets
            .InputDeviceCapabilities(touch)
    }
    fn DispatchEvent(
        &self,
        target: &EventTargetRef,
        event: &EventRef,
    ) -> NativeDispatchEventResult {
        self.pointer.DispatchCanonical(target, event.clone())
    }
    fn CreateDragEvent(
        &self,
        kind: &str,
        init: &MouseEventInit,
        transfer: Option<&Rc<DataTransfer>>,
        stamp: TimeTicks,
        synthetic: SyntheticEventType,
    ) -> EventRef {
        EventRef::new(DragEvent::new(
            kind,
            init,
            transfer.cloned(),
            stamp,
            synthetic,
            self.CoordinateContext(),
        ))
    }
}
impl MouseDragPort for PageMouseDrag<'_, '_, '_, '_> {
    fn DragSource(&self) -> Option<EventTargetRef> {
        self.pointer
            .owners
            .drag
            .GetDragState()
            .drag_src
            .borrow()
            .clone()
    }
    fn SetDragSource(&self, source: Option<&EventTargetRef>) {
        self.pointer
            .owners
            .drag
            .GetDragState()
            .drag_src
            .replace(source.cloned());
    }
    fn DragSourceHasFrame(&self, source: &EventTargetRef) -> bool {
        self.SourceId(source).is_some() && self.HasPage()
    }
    fn DragSourceFrameIsDescendantOfFrame(&self, source: &EventTargetRef) -> bool {
        self.SourceId(source).is_some()
    }
    fn HitTestMouseDownReadOnly(&self, p: PointF) -> Option<EventTargetRef> {
        self.pointer
            .dispatcher
            .targets
            .ResolveTarget(self.Hit(p)?.InnerNode()?.DOMNodeId()?)
    }
    fn DraggableNode(
        &self,
        node: &EventTargetRef,
        p: PointF,
        delayed: bool,
    ) -> Option<EventTargetRef> {
        self.pointer
            .owners
            .drag
            .DraggableNode(self, node, p, delayed)
    }
    fn DataTransfer(&self) -> Option<Rc<DataTransfer>> {
        self.pointer
            .owners
            .drag
            .GetDragState()
            .drag_data_transfer
            .borrow()
            .clone()
    }
    fn SetDataTransfer(&self, t: &Rc<DataTransfer>) {
        self.pointer
            .owners
            .drag
            .GetDragState()
            .drag_data_transfer
            .replace(Some(t.clone()));
    }
    fn CreateWritableDragAndDropDataTransfer(&self) -> Rc<DataTransfer> {
        DataTransfer::CreateWritableDragAndDrop()
    }
    fn PopulateDragDataTransfer(&self, origin: PointF, current: PointF) -> bool {
        self.pointer
            .owners
            .drag
            .PopulateDragDataTransfer(self, origin, current)
    }
    fn DragTypeIsSelection(&self) -> bool {
        self.pointer.owners.drag.GetDragState().drag_type.get() == DragSourceAction::Selection
    }
    fn StartDrag(&self, event: &WebMouseEvent, p: PointF) -> bool {
        self.pointer.owners.drag.StartDrag(self, event, p)
    }
    fn SetDestinationOperation(&self, op: DragOperation) {
        if let Some(transfer) = self.DataTransfer() {
            transfer.SetDestinationOperation(op);
        }
    }
    fn HandlePointerCausesUaAction(&self, ty: PointerType, stamp: TimeTicks) {
        let event = WebPointerEvent::CreatePointerCausesUaActionEvent(ty, stamp);
        let result = self.pointer.dispatcher.WithPointerManager(
            self.pointer.owners,
            self.pointer.geometry.clone(),
            self.pointer.controls.clone(),
            |manager, _| {
                manager.HandlePointerInterruption(&event);
                Ok(())
            },
        );
        if let Err(error) = result {
            self.pointer.Fault(error);
        }
    }
}
struct LayoutCursor {
    native: NativeMouseCoordinateLayout,
    targets: Rc<DomEventTargets>,
}
impl DragLayoutCursor for LayoutCursor {
    fn NonPseudoNode(&self) -> Option<EventTargetRef> {
        self.targets.ResolveTarget(self.native.NonPseudoNodeId()?)
    }
    fn Parent(&self) -> Option<Rc<dyn DragLayoutCursor>> {
        Some(Rc::new(Self {
            native: self.native.Parent()?,
            targets: self.targets.clone(),
        }))
    }
    fn UserDrag(&self) -> EUserDrag {
        self.native.UserDrag()
    }
    fn IsImage(&self) -> bool {
        self.native.IsImage()
    }
}
impl DraggableNodeContext for PageMouseDrag<'_, '_, '_, '_> {
    fn SelectionContains(&self, p: PointF) -> bool {
        let Some(settings) = self.pointer.geometry.InteractionTextSettings() else {
            self.Fault("Settings drag selection containment");
            return false;
        };
        let frame = &self.pointer.dispatcher.focus_services.selection;
        match frame.ContainsPoint(
            self.pointer.dispatcher.document.clone(),
            self.pointer.geometry.clone(),
            settings,
            foundation::PhysicalOffset::new(
                foundation::LayoutUnit::FromFloatRound(p.x()),
                foundation::LayoutUnit::FromFloatRound(p.y()),
            ),
            self.HasView(),
        ) {
            Ok(v) => v,
            Err(_) => {
                self.Fault("FrameSelection::Contains drag geometry");
                false
            }
        }
    }
    fn LayoutObject(&self, n: &EventTargetRef) -> Option<Rc<dyn DragLayoutCursor>> {
        Some(Rc::new(LayoutCursor {
            native: self.pointer.frame.DragLayout(self.SourceId(n)?)?,
            targets: self.pointer.dispatcher.targets.clone(),
        }))
    }
    fn IsElement(&self, n: &EventTargetRef) -> bool {
        PointerEventDOMPort::IsElement(self.pointer, n)
    }
    fn SelectTextInsteadOfDrag(&self, n: &EventTargetRef) -> bool {
        let facts = self
            .pointer
            .Read(n, |d, i| {
                if d.Node(i).Type() != dom::DOMNodeType::kText {
                    return (false, false);
                }
                if d.IsEditableForEditing(i) {
                    return (true, false);
                }
                let mut node = Some(i);
                while let Some(i) = node {
                    if d.IsHTMLElementDraggable(i) {
                        return (false, false);
                    }
                    node = d.Node(i).Parent();
                }
                (false, true)
            })
            .unwrap_or_default();
        facts.0
            || (facts.1 && GestureSelectionGeometry::CanStartSelection(&self.GestureSelection(), n))
    }
    fn IsLiveHTMLAnchor(&self, n: &EventTargetRef) -> bool {
        self.pointer
            .Read(n, |d, i| {
                d.Node(i).IsHTMLElement("a") && d.Node(i).FindAttribute("href").is_some()
            })
            .unwrap_or(false)
    }
    fn LoadsImagesAutomatically(&self) -> bool {
        true
    }
}
impl DragSourceContext for PageMouseDrag<'_, '_, '_, '_> {
    fn HasViewAndContentLayout(&self) -> bool {
        self.HasView() && self.pointer.frame.HasContentLayout()
    }
    fn GenerateOverlayBeforeDragStart(&self) -> bool {
        false
    }
    fn SourceContainsHitAt(&self, source: &EventTargetRef, p: PointF) -> bool {
        self.Hit(p)
            .and_then(|h| {
                self.pointer
                    .dispatcher
                    .targets
                    .ResolveTarget(h.InnerNode()?.DOMNodeId()?)
            })
            .is_some_and(|n| {
                PointerEventDOMPort::IsShadowIncludingInclusiveAncestorOf(self.pointer, source, &n)
            })
    }
    fn IsLiveHTMLAnchor(&self, n: &EventTargetRef) -> bool {
        DraggableNodeContext::IsLiveHTMLAnchor(self, n)
    }
    fn LinkURLAt(&self, p: PointF) -> Option<String> {
        self.Hit(p)?
            .AbsoluteLinkURL()
            .map(|url| url.GetString().to_owned())
            .filter(|s| !s.is_empty())
    }
    fn WriteURL(&self, transfer: &DataTransfer, node: &EventTargetRef, p: PointF) {
        let Some(hit) = self.Hit(p) else {
            return;
        };
        let Some(url) = hit.AbsoluteLinkURL() else {
            return;
        };
        let Some(id) = self.SourceId(node) else {
            return;
        };
        let markup = match crate::adapter::page_selection_markup::CreateMarkup(
            &self.pointer.dispatcher.document,
            id,
            interaction_default::core::editing::serializers::markup_formatter::ResolveUrls::All,
        ) {
            Ok(markup) => markup,
            Err(_) => {
                self.Fault("CreateMarkup drag source serialization");
                return;
            }
        };
        let title = foundation::String::from(hit.TextContent()).SimplifyWhiteSpace();
        transfer.WriteURLUTF16(url.GetString(), &title.Utf8(), &markup);
    }
    fn WriteSelection(&self, transfer: &DataTransfer) {
        use crate::adapter::page_editing_visible_position::{
            CompositionTextQuery as Q, CompositionTextResult as V,
        };
        let frame = &self.pointer.dispatcher.focus_services.selection;
        frame.PrepareEditing();
        let Some(selection) = frame.Current() else {
            return;
        };
        let Some(V::Text(text)) = frame.CompositionText(Q::SelectedTextForClipboard {
            selection,
            include_image_alt_text: false,
        }) else {
            self.Fault("FrameSelection::SelectedTextForClipboard native query");
            return;
        };
        let (in_control, base) = {
            let owner = self.pointer.dispatcher.document.borrow();
            let d = owner.GetDocument();
            (crate::adapter::page_editing_visible_position::EnclosingTextControlForSelectionForFocus(d,Some(selection)).is_some(),d.DocumentURL().to_owned())
        };
        let html = if in_control {
            None
        } else {
            match frame.SelectedHtmlForClipboard() {
                Ok(html) => Some(html),
                Err(error) => {
                    self.pointer
                        .Fault(PointerInputError::GestureSelection(format!("{error:?}")));
                    return;
                }
            }
        };
        transfer.WriteSelectionUTF16(html.as_deref().map(|html| (html, base.as_str())), &text);
    }
    fn CanDragImage(&self, node: &EventTargetRef, p: PointF) -> bool {
        self.Hit(p).and_then(|hit| hit.AbsoluteImageURL()).is_some()
            && self.ImageContent(node).is_some_and(|content| {
                !content.ErrorOccurred()
                    && content.HasImage()
                    && content
                        .FilenameExtension()
                        .is_some_and(|extension| !extension.is_empty())
                    && content
                        .ResourceBuffer()
                        .is_some_and(|buffer| !buffer.bytes.is_empty())
            })
    }
    fn WriteImage(&self, transfer: &DataTransfer, node: &EventTargetRef, p: PointF) -> bool {
        let Some(hit) = self.Hit(p) else {
            return false;
        };
        let Some(image) = hit.AbsoluteImageURL() else {
            return false;
        };
        let Some(id) = self.SourceId(node) else {
            return false;
        };
        let markup = match crate::adapter::page_selection_markup::CreateMarkup(
            &self.pointer.dispatcher.document,
            id,
            interaction_default::core::editing::serializers::markup_formatter::ResolveUrls::All,
        ) {
            Ok(markup) => markup,
            Err(_) => {
                self.Fault("CreateMarkup image drag serialization");
                return false;
            }
        };
        let richly = self
            .pointer
            .Read(node, |d, _| {
                foundation::editable::IsRichlyEditable(
                    &dom::persistent_document::DOMEditableFacts(d),
                    id,
                    Default::default(),
                )
            })
            .unwrap_or(false);
        if richly {
            use layoutng_assembly::{
                EditingPosition as P, EditingPositionAnchorType as A, EditingTextAffinity,
            };
            self.pointer
                .dispatcher
                .focus_services
                .selection
                .SetSelection(
                    Some(layoutng_assembly::editing_state::DOMSelection {
                        anchor: P::WithAnchor(id, 0, A::BeforeAnchor),
                        focus: P::WithAnchor(id, 0, A::AfterAnchor),
                        affinity: EditingTextAffinity::Downstream,
                    }),
                    Default::default(),
                );
        }
        let file=self.ImageContent(node).and_then(|content|{
            if content.ErrorOccurred() || !content.HasImage(){return None;}
            let buffer=content.ResourceBuffer()?;let extension=content.FilenameExtension()?;
            Some(interaction_dispatch::core::clipboard::data_transfer::DragDataFile::FromImageResource(buffer.bytes.clone(),content.Metadata().is_some_and(|m|m.is_cors_same_origin),image.GetString().to_owned(),extension.to_owned(),buffer.content_disposition.clone()))
        });
        let link = hit.AbsoluteLinkURL();
        let title = hit.AltDisplayString().unwrap_or_default();
        transfer.DeclareAndWriteDragImageUTF16(
            link.as_ref().map(|url| url.GetString()),
            image.GetString(),
            &title,
            &markup,
            file,
        );
        true
    }
    fn BoundingBoxIncludingDescendants(&self, n: &EventTargetRef) -> Option<gfx::Rect> {
        self.pointer.frame.DragBoundingBox(self.SourceId(n)?)
    }
    fn SelectEnclosingAnchorIfContentEditable(&self) {
        use crate::adapter::page_editing_visible_position::{
            CompositionTextQuery as Q, CompositionTextResult as V, NonLocatedGeometryQuery as NQ,
            NonLocatedGeometryResult as NV,
        };
        let frame = &self.pointer.dispatcher.focus_services.selection;
        frame.PrepareEditing();
        let Some(V::NonLocatedGeometry(NV::Selection {
            flags,
            anchor: Some(anchor),
            ..
        })) = frame.CompositionText(Q::NonLocatedGeometry(NQ::Selection(frame.Current())))
        else {
            return;
        };
        if !flags.is_caret || !flags.has_editable_root {
            return;
        }
        let children = {
            let owner = self.pointer.dispatcher.document.borrow();
            let d = owner.GetDocument();
            let Some(i) = d.FindNodeById(anchor) else {
                return;
            };
            d.Node(i).Children().len()
        };
        use layoutng_assembly::{
            EditingPosition as P, EditingPositionAnchorType as A, EditingTextAffinity,
        };
        frame.SetSelection(
            Some(layoutng_assembly::editing_state::DOMSelection {
                anchor: P::WithAnchor(anchor, 0, A::OffsetInAnchor),
                focus: P::WithAnchor(anchor, children, A::OffsetInAnchor),
                affinity: EditingTextAffinity::Downstream,
            }),
            Default::default(),
        );
    }
    fn DetermineDragOverlay(
        &self,
        state: &DragState,
        origin: PointF,
        current: PointF,
    ) -> DragOverlay {
        if let Some(overlay) = self.DataTransferImageOverlay(state, origin, current) {
            return overlay;
        }
        let result = match state.drag_type.get() {
            DragSourceAction::Selection => self.pointer.frame.SelectionDragOverlay(),
            DragSourceAction::Image => (|| {
                let hit = self.Hit(origin);
                let rect = hit.as_ref().map(|h| h.ImageRect()).unwrap_or_default();
                let source = state.drag_src.borrow().clone();
                let image = if let Some(id) = source.as_ref().and_then(|s| self.SourceId(s)) {
                    self.pointer
                        .frame
                        .ResourceDragImage(id, (rect.width(), rect.height()))?
                } else {
                    None
                };
                let bounds = interaction_default::core::page::drag_image::DragRectForImage(
                    image.as_ref().map(|i| (i.width, i.height)),
                    (origin.x() as i32, origin.y() as i32),
                    (rect.x(), rect.y()),
                    (rect.width(), rect.height()),
                );
                Ok(DragOverlay {
                    image,
                    rect: bounds,
                })
            })(),
            DragSourceAction::Link => (|| {
                let Some(hit) = self.Hit(origin) else {
                    return Ok(DragOverlay::default());
                };
                let Some(url) = hit.AbsoluteLinkURL() else {
                    return Ok(DragOverlay::default());
                };
                let image = self.pointer.frame.LinkDragImage(
                    url.GetString(),
                    &hit.TextContent().encode_utf16().collect::<Vec<_>>(),
                )?;
                let scale = self.pointer.frame.DevicePixelRatio().unwrap_or(1.) as f32;
                let rect = image
                    .as_ref()
                    .map(|image| {
                        gfx::Rect::new(
                            gfx::Point::new(
                                (current.x() - image.width as f32 / (2. * scale)).round() as i32,
                                (current.y() - 2. / scale).round() as i32,
                            ),
                            gfx::Size::new(image.width as i32, image.height as i32),
                        )
                    })
                    .unwrap_or_else(|| {
                        gfx::Rect::new(
                            gfx::Point::new(current.x() as i32, current.y() as i32),
                            gfx::Size::default(),
                        )
                    });
                Ok(DragOverlay { image, rect })
            })(),
            _ => Ok(DragOverlay::default()),
        };
        result.unwrap_or_else(|error| {
            self.pointer
                .Fault(PointerInputError::DragImage(error.to_string()));
            DragOverlay::default()
        })
    }
    fn DataTransferImageOverlay(
        &self,
        state: &DragState,
        origin: PointF,
        current: PointF,
    ) -> Option<DragOverlay> {
        use interaction_dispatch::core::clipboard::data_transfer::DragImageSource;
        let custom = state.drag_data_transfer.borrow().clone()?.DragImage()?;
        let image = match custom.source {
            DragImageSource::Node(node) => self
                .pointer
                .frame
                .NodeDragImage(self.SourceId(&node)?)
                .unwrap_or_else(|error| {
                    self.pointer
                        .Fault(PointerInputError::DragImage(error.to_string()));
                    None
                }),
            DragImageSource::Resource(resource) => {
                resource.BitmapScaled(self.pointer.frame.DevicePixelRatio().unwrap_or(1.) as f32)
            }
        }?;
        let point = if self.LinkURLAt(origin).is_some() {
            current
        } else {
            origin
        };
        let rect = gfx::Rect::new(
            gfx::Point::new(
                point.x() as i32 - custom.location.0,
                point.y() as i32 - custom.location.1,
            ),
            gfx::Size::new(image.width as i32, image.height as i32),
        );
        Some(DragOverlay {
            image: Some(image),
            rect,
        })
    }
    fn ConvertFromRootFrame(&self, p: PointF) -> PointF {
        MouseDragFramePort::ConvertIntPointFromRootFrame(self, p)
    }
    fn FrameToViewport(&self, p: gfx::Point) -> gfx::Point {
        self.pointer
            .frame
            .DragFrameToViewport(p)
            .unwrap_or_else(|| {
                self.Fault("LocalFrameView::FrameToViewport drag coordinates");
                Default::default()
            })
    }
    fn SerializeDragData(&self, t: &DataTransfer) -> WebDragData {
        t.GetDataObject().ToWebDragData(None)
    }
    fn StartSystemDrag(&self, request: SystemDragRequest) -> SystemDragSupport {
        if self.pointer.error.borrow().is_some() {
            SystemDragSupport::Unsupported
        } else {
            self.pointer.frame.StartSystemDrag(request)
        }
    }
}

/// Target-side EventHandler uses the same current dispatch and native hit owner
/// as the source transaction. Page supplies its retained ScrollManager only for
/// ClearDragState; no manager or callback is retained in the target state.
pub(crate) fn TargetDrag(
    dispatcher: &EventDispatcher<'_, '_>,
    owners: &ResidentCursorOwners,
    geometry: Rc<dyn PagePointerGeometry>,
    controls: Rc<PagePointerScrollControlOwners>,
    scroll: Option<Rc<interaction_scroll::core::input::scroll_manager::ScrollManager>>,
    event: &WebMouseEvent,
    action: &interaction_default::core::page::drag_controller::DragTargetAction,
) -> Result<interaction_default::core::page::drag_controller::DragTargetOperation, PointerInputError>
{
    let (result, _) = dispatcher.WithPointerManager(owners, geometry, controls, |_, pointer| {
        let source = PageMouseDrag { pointer };
        let context = PageDragTarget { source, scroll };
        owners
            .drag
            .HandleTargetDrag(&context, action, event)
            .map_err(PointerInputError::SourceLeaf)
    })?;
    Ok(result)
}
struct PageDragTarget<'a, 'scope, 'listener, 'frame> {
    source: PageMouseDrag<'a, 'scope, 'listener, 'frame>,
    scroll: Option<Rc<interaction_scroll::core::input::scroll_manager::ScrollManager>>,
}
impl interaction_default::core::page::drag_controller::DragTargetContext
    for PageDragTarget<'_, '_, '_, '_>
{
    fn HasDocumentAt(&self, event: &WebMouseEvent) -> bool {
        self.HasDocumentAt(event)
    }
    fn DocumentIsEditable(&self) -> bool {
        self.DocumentIsEditable()
    }
    fn NotifyDropUserActivation(&self) {
        self.source.pointer.owners.pointer_frame.activation.borrow_mut().Activate(
            interaction_input::public::common::frame::user_activation_state::UserActivationNotificationType::kInteraction,
            self.source.pointer.dispatcher.clock.Now(),
        );
        super::pointer_dom_effects::SetHadUserInteraction(self.source.pointer.dispatcher, true);
    }
    fn EditTarget(
        &self,
        event: &WebMouseEvent,
        data: &interaction_default::core::page::drag_data::DragData,
        did_initiate: bool,
    ) -> Result<
        Option<interaction_default::core::page::drag_controller::DragEditTarget>,
        &'static str,
    > {
        self.EditTarget(event, data, did_initiate)
    }
    fn SetCanReceiveDroppedFiles(&self, target: &EventTargetRef, value: bool) {
        self.SetCanReceiveDroppedFiles(target, value)
    }
    fn SelectionFacts(
        &self,
    ) -> interaction_default::core::page::drag_controller::DragSelectionFacts {
        self.SelectionFacts()
    }
    fn SmartInsertDeleteEnabled(&self) -> bool {
        false
    }
    fn DispatchDropTextInput(
        &self,
        caret: &dyn interaction_default::core::page::drag_controller::DragCaretPosition,
        data: &interaction_default::core::page::drag_data::DragData,
    ) -> bool {
        self.DispatchDropTextInput(caret, data)
    }
    fn ReceiveDroppedFiles(
        &self,
        target: &EventTargetRef,
        data: &interaction_default::core::page::drag_data::DragData,
    ) -> Result<bool, &'static str> {
        self.ReceiveDroppedFiles(target, data)
    }
    fn CreateDropFragment(
        &self,
        data: &interaction_default::core::page::drag_data::DragData,
        caret: &dyn interaction_default::core::page::drag_controller::DragCaretPosition,
        move_selection: bool,
    ) -> Result<
        Option<(
            Rc<dyn interaction_default::core::page::drag_controller::DragFragment>,
            bool,
        )>,
        &'static str,
    > {
        self.CreateDropFragment(data, caret, move_selection)
    }
    fn BeginDragAndDropCommandGroup(&self) {
        self.BeginDragAndDropCommandGroup()
    }
    fn DeleteSelectionAfterDraggingWithEvents(
        &self,
        caret: &dyn interaction_default::core::page::drag_controller::DragCaretPosition,
        smart: bool,
    ) -> Result<bool, &'static str> {
        self.DeleteSelectionAfterDraggingWithEvents(caret, smart)
    }
    fn SetSelectionToDragCaret(
        &self,
        caret: &dyn interaction_default::core::page::drag_controller::DragCaretPosition,
    ) -> bool {
        self.SetSelectionToDragCaret(caret)
    }
    fn ReplaceSelectionAfterDraggingWithEvents(
        &self,
        target: &EventTargetRef,
        data: &interaction_default::core::page::drag_data::DragData,
        fragment: &dyn interaction_default::core::page::drag_controller::DragFragment,
        smart: bool,
        plain: bool,
    ) -> Result<bool, &'static str> {
        self.ReplaceSelectionAfterDraggingWithEvents(target, data, fragment, smart, plain)
    }
    fn RootEditableElement(&self) -> Option<EventTargetRef> {
        self.RootEditableElement()
    }
    fn UpdateDragStateAfterEdit(&self, root: Option<&EventTargetRef>) {
        self.UpdateDragStateAfterEdit(root)
    }
    fn AcceptsLoadDrops(&self) -> bool {
        self.source.pointer.frame.AcceptsLoadDrops()
    }
    fn LoadDroppedURL(&self, url: &str, foreground: bool) {
        self.source.pointer.frame.LoadDroppedURL(
            url,
            foreground,
            self.source
                .pointer
                .owners
                .pointer_frame
                .activation
                .borrow()
                .IsActive(self.source.pointer.dispatcher.clock.Now()),
        )
    }
    fn HasView(&self) -> bool {
        self.source.HasView()
    }
    fn UpdateDragAndDrop(&self, event: &WebMouseEvent, data: &Rc<DataTransfer>) -> bool {
        self.source.pointer.owners.drag_target.UpdateDragAndDrop(self, event, data) != interaction_dispatch::core::input::event_handling_util::WebInputEventResult::kNotHandled
    }
    fn CancelDragAndDrop(&self, event: &WebMouseEvent, data: &Rc<DataTransfer>) {
        self.source
            .pointer
            .owners
            .drag_target
            .CancelDragAndDrop(self, event, data);
    }
    fn PerformDragAndDrop(&self, event: &WebMouseEvent, data: &Rc<DataTransfer>) -> bool {
        self.source.pointer.owners.drag_target.PerformDragAndDrop(self, event, data) != interaction_dispatch::core::input::event_handling_util::WebInputEventResult::kNotHandled
    }
}
impl crate::core::input::event_handler_drag::DragTargetPort for PageDragTarget<'_, '_, '_, '_> {
    fn HasView(&self) -> bool {
        self.source.HasView()
    }
    fn TargetAt(&self, event: &WebMouseEvent) -> Option<EventTargetRef> {
        let hit = self.source.Hit(event.PositionInRootFrame())?;
        let id = hit.InnerElement()?.DOMNodeId()?;
        // The hit owner already resolves decorative pseudo-elements. Cross the
        // existing UA shadow roots only, preserving authored shadow ownership.
        let id = {
            let owner = self.source.pointer.dispatcher.document.borrow();
            let d = owner.GetDocument();
            let mut node = d.FindNodeById(id)?;
            while let Some(root) = d.ContainingShadowRoot(node) {
                if d.ShadowRootMode(root)
                    != Some(dom::persistent_document::DOMShadowRootMode::kUserAgent)
                {
                    break;
                }
                node = d.OwnerShadowHost(node)?;
            }
            d.Node(node).Id()
        };
        self.source.pointer.dispatcher.targets.ResolveTarget(id)
    }
    fn UpdateAutoscroll(&self, target: Option<&EventTargetRef>, event: &WebMouseEvent) {
        let p = self.source.pointer;
        let layout = target
            .and_then(|n| self.source.SourceId(n))
            .and_then(|id| p.frame.AutoscrollLayoutForNode(id));
        let defaults = super::gesture_mouse_defaults::PageGestureMouseDefaults::new(
            p.dispatcher,
            p.owners,
            p.geometry.clone(),
            p.controls.clone(),
        );
        if let Err(error) = defaults.UpdateDragAutoscroll(layout, event) {
            p.Fault(error);
        }
    }
    fn DispatchSourceDrag(&self, event: &WebMouseEvent) {
        if self.source.DragSource().is_some() {
            MouseEventManager::WithState(self.source.pointer.owners.mouse.clone())
                .DispatchDragSrcEventWithPorts(&self.source.Ports(), "drag", event);
        }
    }
    fn DispatchDrag(
        &self,
        kind: &str,
        target: &EventTargetRef,
        related: Option<&EventTargetRef>,
        event: &WebMouseEvent,
        data: &Rc<DataTransfer>,
    ) -> interaction_dispatch::core::input::event_handling_util::WebInputEventResult {
        MouseEventManager::WithState(self.source.pointer.owners.mouse.clone())
            .DispatchDragEventWithPorts(
                &self.source.Ports(),
                kind,
                Some(target),
                related,
                event,
                Some(data),
            )
    }
    fn ClearDragState(&self) {
        let p = self.source.pointer;
        let release = |id| {
            crate::adapter::page_input_type::StopSelectControlAutoscroll(
                &p.dispatcher.Context(),
                id,
            )
        };
        if let Some(scroll) = &self.scroll {
            scroll.StopAutoscrollWithNodeRelease(&release);
        } else if let Some(owner) = p.owners.autoscroll_services.borrow().clone() {
            owner.StopAutoscrollWithNodeRelease(&release);
        }
        p.owners.event_handler.ClearMouseCapture();
    }
}

#[path = "page_drag_edit.rs"]
mod editing;

pub(crate) use editing::{CurrentDragCaret, DirectoryFilesChosen};
