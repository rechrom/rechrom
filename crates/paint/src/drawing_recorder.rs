//! DrawingRecorder's begin/end lifetime from Blink's
//! platform/graphics/paint/drawing_recorder.cc. Drawing commands remain in the
//! flat replay buffer; closing a recorder appends one DrawingDisplayItem.
use std::cell::RefCell;

use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::paint_chunker::DisplayItemClientInfo;
use crate::paint_context::PaintContext;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

pub(crate) struct DrawingRecorder<'c, 'o> {
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'c, 'o> DrawingRecorder<'c, 'o> {
    pub(crate) fn UseCachedDrawingIfPossible(
        context: &'c RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
        visual_rect: PaintRect,
    ) -> bool {
        context
            .borrow_mut()
            .UseCachedDrawingIfPossible(node, item_type, visual_rect)
    }

    pub(crate) fn UseCachedDrawingWithClientIfPossible(
        context: &'c RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
    ) -> bool {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        context.borrow_mut().UseCachedDrawingWithClient(
            id,
            client,
            fragment.pre_paint_revision,
            None,
            Some((
                fragment.paint.display_item_client_owner_node_id,
                fragment.paint.display_item_client_debug_name.clone(),
            )),
        )
    }

    pub(crate) fn new(
        context: &'c RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
        visual_rect: PaintRect,
    ) -> Self {
        context
            .borrow_mut()
            .BeginDrawingRecorder(node, item_type, visual_rect);
        Self { context }
    }

    pub(crate) fn with_client(
        context: &'c RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        visual_rect: PaintRect,
    ) -> Self {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        context.borrow_mut().BeginDrawingRecorderWithClient(
            id,
            client,
            visual_rect,
            node.paint_snap_offset,
            fragment.pre_paint_revision,
        );
        Self { context }
    }
}

impl Drop for DrawingRecorder<'_, '_> {
    fn drop(&mut self) {
        self.context.borrow_mut().EndDrawingRecorder();
    }
}
