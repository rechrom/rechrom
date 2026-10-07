//! Blink paints the bar caret after its editor foreground, under its normal
//! clip/transform/effect state (BoxFragmentPainter::PaintCaretsIfNeeded).
use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::drawing_recorder::DrawingRecorder;
use crate::paint_chunker::DisplayItemClientInfo;
use crate::{
    geometry_mapper::MapRectToRoot,
    paint_context::PaintContext,
    paint_engine::{CaretGeometry, DisplayItem, DisplayItemType, PaintPhase},
    pre_paint_tree_walk::PaintTreeNode,
    PaintRect,
};
use layoutng_assembly::{caret::geometry::TextControlCaretRect, fragment_tree::FragmentKind};
use std::cell::RefCell;

#[allow(non_snake_case)]
pub(crate) fn PaintCaret(node: &PaintTreeNode<'_>, context: &RefCell<PaintContext<'_>>) {
    let Some(caret) = context.borrow().caret else {
        return;
    };
    let fragment = node.fragment.as_deref().expect("caret owner");
    if fragment.node_id != caret.node_id
        || fragment.kind != FragmentKind::kBox
        || !fragment.paint.establishes_paint_state
    {
        return;
    }
    let Some(local) = TextControlCaretRect(fragment, caret.offset, caret.empty) else {
        return;
    };
    let rect = PaintRect {
        x: (node.paint_offset.x + local.offset.x).round(),
        y: (node.paint_offset.y + local.offset.y).round(),
        width: local.size.width,
        height: local.size.height.round().max(1.0),
    };
    let border = fragment.paint.border;
    let padding = fragment.paint.padding;
    let content = PaintRect {
        x: node.paint_offset.x + border.left + padding.left,
        y: node.paint_offset.y + border.top + padding.top,
        width: (fragment.size.width - border.left - border.right - padding.left - padding.right)
            .max(0.0),
        height: (fragment.size.height - border.top - border.bottom - padding.top - padding.bottom)
            .max(0.0),
    };
    if let Some(root_rect) = MapRectToRoot(rect, &node.transforms) {
        context.borrow_mut().SetCaretGeometry(CaretGeometry {
            node_id: caret.node_id,
            rect: root_rect,
            visible: caret.visible,
        });
    }
    let _caret_properties =
        crate::paint_context::ScopedPaintChunkProperties::caret(context, caret, node);
    // The flat replay backend has no property-tree compositor. Apply the same
    // effect opacity outside the drawing record; blinking does not remove the
    // caret DrawingDisplayItem (FrameCaret::CaretEffectNodeState .cc:90-94).
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kSaveLayerAlpha,
            phase: PaintPhase::kForeground,
            node_id: caret.node_id,
            rect,
            opacity: if caret.visible { 1.0 } else { 0.001 },
            ..Default::default()
        },
        node,
    );
    // FrameCaret::PaintCaret uses the independent CaretDisplayItemClient and
    // kCaret (frame_caret.cc:298), inheriting the active fragment scope via
    // PaintController::CreateAndAppend (paint_controller.h:219). If an outer recorder
    // is active (text-combine), caret operations belong to that existing record
    // (caret_display_item_client.cc:386-394).
    let in_drawing = context.borrow().InDrawingRecorder();
    let drawing = (!in_drawing).then(|| {
        DrawingRecorder::with_client(
            context,
            node,
            DisplayItemId {
                client_id: caret.display_item_client_id,
                r#type: DisplayItemIdType::kCaret,
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo {
                is_cacheable: caret.display_item_client_is_cacheable,
                is_just_created: caret.display_item_client_is_just_created,
            },
            rect,
        )
    });
    let mut paint_context = context.borrow_mut();
    for item in [
        DisplayItem {
            r#type: DisplayItemType::kSave,
            ..Default::default()
        },
        DisplayItem {
            r#type: DisplayItemType::kClipRect,
            rect: content,
            ..Default::default()
        },
        DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect,
            color: fragment.paint.style.color,
            ..Default::default()
        },
        DisplayItem {
            r#type: DisplayItemType::kRestore,
            ..Default::default()
        },
    ] {
        paint_context.Append(
            DisplayItem {
                node_id: caret.node_id,
                phase: PaintPhase::kForeground,
                ..item
            },
            node,
        );
    }
    drop(paint_context);
    drop(drawing);
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kRestore,
            phase: PaintPhase::kForeground,
            node_id: caret.node_id,
            ..Default::default()
        },
        node,
    );
}
