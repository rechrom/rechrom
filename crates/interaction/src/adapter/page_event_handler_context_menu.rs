//! Keyboard context-menu placement using EventHandler's original algorithm.
//! Source: event_handler.cc:2226-2382. Only canonical target projections cross
//! this binding; caret/viewport/outline queries retain their native owners.
use super::*;
use crate::adapter::page_event_dispatcher::mouse_selection_services::PageMouseSelectionServicesOwner;
use crate::core::input::event_handler::{
    nonlocated_context_menu::{
        NonLocatedCaret, NonLocatedContextMenuSource, NonLocatedSelectionFlags,
    },
    EventHandler,
};
use foundation::gfx;
use interaction_input::{
    base::time::time::TimeTicks, ui::base::mojom::menu_source_type::MenuSourceType,
};

struct Source<'a, 'listener, 'frame> {
    source: PageMouseSelectionServicesOwner<'a, 'listener, 'frame>,
    nodes: &'a PageEventHandlerNodes,
}
impl Source<'_, '_, '_> {
    fn Target(&self, node: &NodeRef) -> EventTargetRef {
        self.nodes
            .EventTarget(node)
            .expect("context-menu target retains canonical node identity")
    }
}
impl NonLocatedContextMenuSource for Source<'_, '_, '_> {
    fn HasDocument(&self) -> bool {
        self.source.HasDocument()
    }
    fn FocusedElement(&self) -> Option<NodeRef> {
        self.source
            .FocusedElement()
            .and_then(|target| self.nodes.NodeForTarget(&target))
    }
    fn SelectionFlags(&self) -> NonLocatedSelectionFlags {
        self.source.SelectionFlags()
    }
    fn SelectionHasFocus(&self) -> bool {
        self.source.SelectionHasFocus()
    }
    fn SelectionStartCaret(&self) -> Option<NonLocatedCaret> {
        self.source.SelectionStartCaret()
    }
    fn SelectionEndCaret(&self) -> Option<NonLocatedCaret> {
        self.source.SelectionEndCaret()
    }
    fn EnclosingAnchorAtSelectionStart(&self) -> Option<NodeRef> {
        self.source
            .EnclosingAnchorAtSelectionStart()
            .and_then(|target| self.nodes.NodeForTarget(&target))
    }
    fn ElementVisibleBoundsInLocalRoot(&self, element: &NodeRef) -> gfx::Rect {
        self.source
            .ElementVisibleBoundsInLocalRoot(&self.Target(element))
    }
    fn ElementOutlineRectsInWidget(&self, element: &NodeRef) -> Vec<gfx::Rect> {
        self.source
            .ElementOutlineRectsInWidget(&self.Target(element))
    }
    fn ConvertRectFromRootFrame(&self, rect: gfx::Rect) -> gfx::Rect {
        self.source.ConvertRectFromRootFrame(rect)
    }
    fn ConvertPointToRootFrame(&self, point: gfx::Point) -> gfx::Point {
        self.source.ConvertPointToRootFrame(point)
    }
    fn RootFrameToViewport(&self, rect: gfx::Rect) -> gfx::Rect {
        self.source.RootFrameToViewport(rect)
    }
    fn ViewportToRootFrame(&self, rect: gfx::Rect) -> gfx::Rect {
        self.source.ViewportToRootFrame(rect)
    }
    fn VisualViewportSize(&self) -> gfx::Size {
        self.source.VisualViewportSize()
    }
    fn VisualViewportScrollOffset(&self) -> gfx::PointF {
        self.source.VisualViewportScrollOffset()
    }
    fn SetPointerCursor(&self) {
        self.source.SetPointerCursor();
    }
    fn RootFramePointToScreenDIPs(&self, point: gfx::Point) -> gfx::Point {
        self.source.RootFramePointToScreenDIPs(point)
    }
    fn ShowContextMenuOnMouseUp(&self) -> bool {
        self.source.ShowContextMenuOnMouseUp()
    }
    fn Now(&self) -> TimeTicks {
        self.source.Now()
    }
}

impl PageEventHandlerInputManagers<'_, '_, '_> {
    pub fn ShowKeyboardContextMenu(
        &self,
        handler: &EventHandler<'_, Hit>,
    ) -> Result<(), PointerInputError> {
        let source = Source {
            source: PageMouseSelectionServicesOwner::new(
                &self.dispatcher,
                &self.owners,
                self.geometry.clone(),
                self.controls.clone(),
            ),
            nodes: self.nodes.as_ref(),
        };
        handler.ShowNonLocatedContextMenu(None, MenuSourceType::kKeyboard, &source);
        source
            .source
            .Finish()
            .map_err(PointerInputError::SourceLeaf)
    }
}
