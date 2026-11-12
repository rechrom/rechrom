//! Blink paints block carets before editor text and bar/underscore carets
//! after it, under the normal clip/transform/effect state
//! (BoxFragmentPainter::PaintCaretsIfNeeded).
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
use layoutng_assembly::fragment_tree::FragmentKind;
use std::cell::RefCell;

#[allow(non_snake_case)]
pub(crate) fn PaintCaret(node: &PaintTreeNode<'_>, context: &RefCell<PaintContext<'_>>) {
    let Some(caret) = context.borrow().caret else {
        return;
    };
    let Some(local) = context.borrow().caret_local_rect else {
        return;
    };
    PaintCaretWithState(node, context, caret, local, false);
}

#[allow(non_snake_case)]
pub(crate) fn PaintDragCaret(node: &PaintTreeNode<'_>, context: &RefCell<PaintContext<'_>>) {
    let Some(caret) = context.borrow().drag_caret else {
        return;
    };
    let Some(local) = context.borrow().drag_caret_local_rect else {
        return;
    };
    PaintCaretWithState(node, context, caret, local, true);
}

#[allow(non_snake_case)]
fn PaintCaretWithState(
    node: &PaintTreeNode<'_>,
    context: &RefCell<PaintContext<'_>>,
    caret: crate::paint_engine::CaretPosition,
    local: layoutng_assembly::caret::geometry::LocalCaretRect,
    drag: bool,
) {
    if caret.selection.is_some() && caret.composition.is_none() {
        return;
    }
    let fragment = node.fragment.as_deref().expect("caret owner");
    let matches_painter = caret
        .native
        .map_or(fragment.node_id == caret.node_id, |native| {
            node.SourceCursor()
                .is_some_and(|cursor| native.Matches(cursor))
        });
    if !matches_painter
        || fragment.kind != FragmentKind::kBox
        || (caret.native.is_none() && !fragment.paint.establishes_paint_state)
    {
        return;
    }
    let mut color = caret
        .native
        .map_or_else(|| fragment.paint.style.caret_color(), |native| native.color);
    if caret.native.is_none()
        && context.borrow().caret_shape
            == Some(layoutng_assembly::internal::caret_rect::CaretShape::kBlock)
        && fragment
            .paint
            .style
            .Source()
            .is_some_and(|style| style.IsCaretColorAuto() && style.IsCaretTextColorAuto())
    {
        color.alpha = 0.5;
    }
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
    if let Some(root_rect) = (!drag)
        .then(|| MapRectToRoot(rect, &node.transforms))
        .flatten()
    {
        context.borrow_mut().SetCaretGeometry(CaretGeometry {
            node_id: caret.node_id,
            rect: root_rect,
            visible: caret.visible && caret.selection.is_none(),
        });
    }
    // A selected preedit still publishes its focus rectangle to the platform
    // candidate window, while the selected text has no visible insertion bar.
    if caret.selection.is_some() {
        return;
    }
    let _caret_properties = if drag {
        crate::paint_context::ScopedPaintChunkProperties::drag_caret(context, caret, node)
    } else {
        crate::paint_context::ScopedPaintChunkProperties::caret(context, caret, node)
    };
    // The flat replay backend has no property-tree compositor. Apply the same
    // effect opacity outside the drawing record; blinking does not remove the
    // caret DrawingDisplayItem (FrameCaret::CaretEffectNodeState .cc:90-94).
    if !drag {
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
    }
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
                r#type: if drag {
                    DisplayItemIdType::kDragCaret
                } else {
                    DisplayItemIdType::kCaret
                },
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo {
                is_cacheable: caret.display_item_client_is_cacheable,
                invalidation_reason: Some(caret.display_item_client_invalidation_reason),
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
            color,
            ..Default::default()
        },
        DisplayItem {
            r#type: DisplayItemType::kRestore,
            ..Default::default()
        },
    ]
    .into_iter()
    .filter(|item| {
        // Generic painters inherit their normal overflow clip; only controls
        // additionally clip the insertion caret to their inner editor.
        caret.native.is_none() || item.r#type != DisplayItemType::kClipRect
    }) {
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
    if !drag {
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
}
