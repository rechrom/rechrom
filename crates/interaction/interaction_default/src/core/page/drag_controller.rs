//! Retained source-side DragController state.
//! Chromium core/page/drag_controller.h:154-166, .cc:1507-1515.
use super::drag_state::{DragSourceAction, DragState};
use foundation::{gfx, EUserDrag};
use interaction_dispatch::core::clipboard::data_transfer::{
    DataTransfer, DataTransferAccessPolicy, DragDataObject, DragImageBitmap, DragOperation,
    WebDragData,
};
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_input::public::common::input::{
    web_mouse_event::WebMouseEvent, web_pointer_properties::PointF,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

// Source: drag_controller.cc:1023-1031.
fn MaxDragImageSize(device_scale: f32) -> (f32, f32) {
    #[cfg(target_os = "macos")]
    let max = 400.;
    #[cfg(not(target_os = "macos"))]
    let max = 200.;
    let max = (max * device_scale).floor();
    (max, max)
}

// Source: drag_controller.cc:1077-1079. The clamping algorithm belongs to
// DragImage; this controller supplies the platform-specific maximum size.
pub fn ClampedImageScale(image: (u32, u32), element: (i32, i32), device_scale: f32) -> (f32, f32) {
    super::drag_image::ClampedImageScaleForSize(image, element, MaxDragImageSize(device_scale))
}

// Source: drag_controller.cc:1085-1102.
pub fn DragRectForImage(
    size: Option<(u32, u32)>,
    origin: (i32, i32),
    image_origin: (i32, i32),
    element: (i32, i32),
) -> gfx::Rect {
    let Some(size) = size else {
        return gfx::Rect::new(gfx::Point::new(origin.0, origin.1), gfx::Size::default());
    };
    let scale = size.0 as f32 / element.0 as f32;
    gfx::Rect::new(
        gfx::Point::new(
            origin.0 + ((image_origin.0 - origin.0) as f32 * scale).round() as i32,
            origin.1 + ((image_origin.1 - origin.1) as f32 * scale).round() as i32,
        ),
        gfx::Size::new(size.0 as i32, size.1 as i32),
    )
}

#[derive(Default)]
pub struct DragController {
    drag_state: RefCell<Option<Rc<DragState>>>,
    did_initiate_drag: Cell<bool>,
    drag_pointer_id: Cell<Option<i32>>,
    prepared_overlay: RefCell<Option<DragOverlay>>,
    target_data: RefCell<Option<Rc<DragDataObject>>>,
    target_operations: Cell<u32>,
    document_is_handling_drag: Cell<bool>,
    document_under_mouse: Cell<bool>,
    file_input_under_mouse: RefCell<Option<EventTargetRef>>,
    drag_caret: RefCell<Option<Rc<dyn DragCaretPosition>>>,
}
impl DragController {
    // Source: drag_controller.cc:1507-1511. Reentry returns this same owner.
    pub fn GetDragState(&self) -> Rc<DragState> {
        self.drag_state
            .borrow_mut()
            .get_or_insert_with(|| Rc::new(DragState::default()))
            .clone()
    }
    // Source: drag_controller.cc:1513-1515.
    pub fn ContextDestroyed(&self) {
        self.target_data.borrow_mut().take();
        self.file_input_under_mouse.borrow_mut().take();
        self.drag_caret.borrow_mut().take();
        self.document_is_handling_drag.set(false);
        self.document_under_mouse.set(false);
        self.drag_state.borrow_mut().take();
        self.did_initiate_drag.set(false);
        self.drag_pointer_id.set(None);
        self.prepared_overlay.borrow_mut().take();
    }
}

/// Immutable ChromeClient::StartDragging packet. The live DataTransfer and DOM
/// identities stay on the document sequence; only serialized data and immutable
/// raster pixels cross to the native host.
#[derive(Clone)]
pub struct SystemDragRequest {
    pub data: WebDragData,
    pub source_operations: u32,
    pub image: Option<Arc<DragImageBitmap>>,
    pub cursor_offset: (i32, i32),
    pub drag_object_rect: gfx::Rect,
    pub pointer_id: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemDragSupport {
    Supported,
    Unsupported,
}
#[derive(Clone, Default)]
pub struct DragOverlay {
    pub image: Option<Arc<DragImageBitmap>>,
    pub rect: gfx::Rect,
}
/// A borrowed cursor over the actual LayoutObject parent chain. Anonymous and
/// pseudo objects preserve the source traversal; no DOM ancestor approximation
/// or second layout model is admitted here.
pub trait DragLayoutCursor {
    fn NonPseudoNode(&self) -> Option<EventTargetRef>;
    fn Parent(&self) -> Option<Rc<dyn DragLayoutCursor>>;
    fn UserDrag(&self) -> EUserDrag;
    fn IsImage(&self) -> bool;
}
pub trait DraggableNodeContext {
    fn SelectionContains(&self, origin: PointF) -> bool;
    fn LayoutObject(&self, node: &EventTargetRef) -> Option<Rc<dyn DragLayoutCursor>>;
    fn IsElement(&self, node: &EventTargetRef) -> bool;
    fn SelectTextInsteadOfDrag(&self, node: &EventTargetRef) -> bool;
    fn IsLiveHTMLAnchor(&self, node: &EventTargetRef) -> bool;
    fn LoadsImagesAutomatically(&self) -> bool;
}
/// Native source leaves used at Populate/Start's original call sites. DOM
/// serialization, image resources, selection, and Paint retain their owners.
/// Each leaf ends its layout loan before this controller returns to dispatch.
pub trait DragSourceContext {
    fn HasViewAndContentLayout(&self) -> bool;
    fn GenerateOverlayBeforeDragStart(&self) -> bool;
    fn SourceContainsHitAt(&self, source: &EventTargetRef, origin: PointF) -> bool;
    fn IsLiveHTMLAnchor(&self, source: &EventTargetRef) -> bool;
    fn LinkURLAt(&self, origin: PointF) -> Option<String>;
    fn WriteURL(&self, transfer: &DataTransfer, source: &EventTargetRef, origin: PointF);
    fn WriteSelection(&self, transfer: &DataTransfer);
    fn CanDragImage(&self, source: &EventTargetRef, origin: PointF) -> bool;
    fn WriteImage(&self, transfer: &DataTransfer, source: &EventTargetRef, origin: PointF) -> bool;
    fn BoundingBoxIncludingDescendants(&self, source: &EventTargetRef) -> Option<gfx::Rect>;
    fn SelectEnclosingAnchorIfContentEditable(&self);
    fn DetermineDragOverlay(
        &self,
        state: &DragState,
        origin: PointF,
        current: PointF,
    ) -> DragOverlay;
    fn DataTransferImageOverlay(
        &self,
        state: &DragState,
        origin: PointF,
        current: PointF,
    ) -> Option<DragOverlay>;
    fn ConvertFromRootFrame(&self, point: PointF) -> PointF;
    fn FrameToViewport(&self, point: gfx::Point) -> gfx::Point;
    fn SerializeDragData(&self, transfer: &DataTransfer) -> WebDragData;
    fn StartSystemDrag(&self, request: SystemDragRequest) -> SystemDragSupport;
}
impl DragController {
    // Chromium drag_controller.cc:891-972. Selection priority and delayed
    // resolution are source policy, not a DOM/style adapter heuristic.
    pub fn DraggableNode(
        &self,
        context: &dyn DraggableNodeContext,
        start: &EventTargetRef,
        origin: PointF,
        delayed: bool,
    ) -> Option<EventTargetRef> {
        let state = self.GetDragState();
        let selected = context.SelectionContains(origin);
        state.drag_type.set(if selected {
            DragSourceAction::Selection
        } else {
            DragSourceAction::None
        });
        if selected && !delayed {
            return Some(start.clone());
        }
        let mut layout = context.LayoutObject(start);
        let mut candidate = None;
        while let Some(current) = layout {
            layout = current.Parent();
            let Some(node) = current.NonPseudoNode() else {
                continue;
            };
            if !selected && context.SelectTextInsteadOfDrag(&node) {
                return None;
            }
            if !context.IsElement(&node) || current.UserDrag() == EUserDrag::kNone {
                continue;
            }
            if current.IsImage() && context.LoadsImagesAutomatically() {
                state.drag_type.set(DragSourceAction::Image);
                return Some(node);
            }
            let action = if current.UserDrag() == EUserDrag::kElement {
                DragSourceAction::DHTML
            } else if context.IsLiveHTMLAnchor(&node) {
                DragSourceAction::Link
            } else {
                continue;
            };
            candidate = Some((node, action));
            break;
        }
        let (node, action) = candidate?;
        if selected {
            Some(start.clone())
        } else {
            state.drag_type.set(action);
            Some(node)
        }
    }
    // Chromium drag_controller.cc:1282-1383. This runs before dragstart, with
    // the same writable DataTransfer later exposed to that event listener.
    pub fn PopulateDragDataTransfer(
        &self,
        context: &dyn DragSourceContext,
        origin: PointF,
        current: PointF,
    ) -> bool {
        let state = self.GetDragState();
        let source = state.drag_src.borrow().clone();
        let transfer = state.drag_data_transfer.borrow().clone();
        let (Some(source), Some(transfer)) = (source, transfer) else {
            return false;
        };
        if !context.HasViewAndContentLayout() {
            return false;
        }
        if context.GenerateOverlayBeforeDragStart() {
            self.prepared_overlay.borrow_mut().take();
        }
        if !context.SourceContainsHitAt(&source, origin) {
            return false;
        }
        if context.IsLiveHTMLAnchor(&source) && context.LinkURLAt(origin).is_some() {
            context.WriteURL(&transfer, &source, origin);
        }
        match state.drag_type.get() {
            DragSourceAction::Selection => context.WriteSelection(&transfer),
            DragSourceAction::Image => {
                if context.GenerateOverlayBeforeDragStart()
                    && !context.CanDragImage(&source, origin)
                {
                    return false;
                }
                if !context.WriteImage(&transfer, &source, origin) {
                    return false;
                }
            }
            DragSourceAction::Link => {
                if context.LinkURLAt(origin).is_none() {
                    return false;
                }
            }
            DragSourceAction::DHTML => {
                let Some(rect) = context.BoundingBoxIncludingDescendants(&source) else {
                    return false;
                };
                transfer.SetDragImageElement(
                    source,
                    (origin.x() as i32 - rect.x(), origin.y() as i32 - rect.y()),
                );
            }
            DragSourceAction::None => return false,
        }
        if context.GenerateOverlayBeforeDragStart() {
            if state.drag_type.get() == DragSourceAction::Link {
                context.SelectEnclosingAnchorIfContentEditable();
            }
            self.prepared_overlay
                .replace(Some(context.DetermineDragOverlay(&state, origin, current)));
        }
        true
    }
    // Chromium drag_controller.cc:1386-1493. A platform with no drag service
    // reports Unsupported; it never fabricates an active native drag session.
    pub fn StartDrag(
        &self,
        context: &dyn DragSourceContext,
        event: &WebMouseEvent,
        origin: PointF,
    ) -> bool {
        if !context.HasViewAndContentLayout() {
            return false;
        }
        let state = self.GetDragState();
        let source = state.drag_src.borrow().clone();
        let transfer = state.drag_data_transfer.borrow().clone();
        let (Some(source), Some(transfer)) = (source, transfer) else {
            return false;
        };
        if state.drag_type.get() == DragSourceAction::None {
            return false;
        }
        let current = context.ConvertFromRootFrame(event.PositionInRootFrame());
        let mut overlay = if context.GenerateOverlayBeforeDragStart() {
            let Some(overlay) = self.prepared_overlay.borrow().clone() else {
                return false;
            };
            overlay
        } else {
            if !context.SourceContainsHitAt(&source, origin) {
                return false;
            }
            match state.drag_type.get() {
                DragSourceAction::Image if !context.CanDragImage(&source, origin) => return false,
                DragSourceAction::Link => {
                    if context.LinkURLAt(origin).is_none() {
                        return false;
                    }
                    context.SelectEnclosingAnchorIfContentEditable();
                }
                _ => {}
            }
            context.DetermineDragOverlay(&state, origin, current)
        };
        if context.GenerateOverlayBeforeDragStart() {
            if let Some(custom) = context.DataTransferImageOverlay(&state, origin, current) {
                overlay = custom;
            }
        }
        let initiation = if state.drag_type.get() == DragSourceAction::Link {
            current
        } else {
            origin
        };
        let event_pos = context.FrameToViewport(gfx::Point::new(
            initiation.x() as i32,
            initiation.y() as i32,
        ));
        let image_pos = context.FrameToViewport(overlay.rect.origin());
        let mut data = context.SerializeDragData(&transfer);
        if data.source_effect_allowed.is_none() {
            data.source_effect_allowed = Some(transfer.effectAllowed());
        }
        let request = SystemDragRequest {
            data,
            source_operations: transfer.SourceOperation(),
            image: overlay.image.clone(),
            cursor_offset: (event_pos.x() - image_pos.x(), event_pos.y() - image_pos.y()),
            drag_object_rect: overlay.rect,
            pointer_id: event.pointer_properties.id,
        };
        // Publish the source state before entering the host, which may reenter.
        self.did_initiate_drag.set(true);
        self.drag_pointer_id.set(Some(event.pointer_properties.id));
        self.prepared_overlay.replace(Some(overlay));
        if context.StartSystemDrag(request) == SystemDragSupport::Unsupported {
            self.did_initiate_drag.set(false);
            self.drag_pointer_id.set(None);
            self.prepared_overlay.borrow_mut().take();
            return false;
        }
        true
    }
    pub fn DidInitiateDrag(&self) -> bool {
        self.did_initiate_drag.get()
    }
    pub fn DragPointerId(&self) -> Option<i32> {
        self.drag_pointer_id.get()
    }
    // Source DragEnded clears these owners; MouseEventManager separately
    // dispatches dragend and numbs the original shared DataTransfer.
    pub fn DragEnded(&self) {
        self.drag_caret.borrow_mut().take();
        self.did_initiate_drag.set(false);
        self.drag_pointer_id.set(None);
        self.prepared_overlay.borrow_mut().take();
    }
}

#[path = "drag_controller_target.rs"]
mod target;
pub use target::*;
