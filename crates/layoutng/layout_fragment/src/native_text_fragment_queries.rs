// On-demand text queries consume canonical FragmentItem/ShapeResult state.
// Keep their public cursor wrappers thin and expose one fragment sampling frame
// per query class.

use super::*;

#[inline(never)]
pub(super) fn paint_glyph_runs(
    cursor: NativePaintFragmentCursor<'_>,
) -> Option<std::sync::Arc<Vec<layout_fragment::fragment_tree::PaintGlyphRun>>> {
    let NativeFragmentCursor::Inline { item, .. } = cursor.0 else {
        return None;
    };
    item.IsText().then(|| {
        crate::internal::boundary::layout_boundary::BuildPaintGlyphRuns(
            item,
            cursor.PaintData().paint.writing_mode,
        )
    })
}

#[inline(never)]
pub(super) fn text_paint_baseline(cursor: NativePaintFragmentCursor<'_>) -> Option<f64> {
    let NativeFragmentCursor::Inline { item, .. } = cursor.0 else {
        return None;
    };
    crate::internal::boundary::layout_boundary::NativeTextPaintBaseline(item)
}

#[inline(never)]
pub(super) fn text_inline_position(
    cursor: NativePaintFragmentCursor<'_>,
    offset: u32,
) -> Option<f64> {
    let NativeFragmentCursor::Inline { item, items, .. } = cursor.0 else {
        return None;
    };
    if !item.IsText() {
        return None;
    }
    let offset = offset.clamp(item.StartOffset(), item.EndOffset());
    Some(
        item.CaretInlinePositionForOffset(item.Text(items), offset)
            .ToDouble(),
    )
}

#[inline(never)]
pub(super) fn text_range_rect(
    cursor: NativePaintFragmentCursor<'_>,
    start: u32,
    end: u32,
) -> Option<LayoutRect> {
    let NativeFragmentCursor::Inline { item, items, .. } = cursor.0 else {
        return None;
    };
    if !item.IsText() {
        return None;
    }
    let start = start.max(item.StartOffset());
    let end = end.min(item.EndOffset());
    if start >= end {
        return None;
    }
    let rect = item.LocalTextRect(item.Text(items), start, end);
    Some(LayoutRect {
        x: rect.offset.left.ToDouble(),
        y: rect.offset.top.ToDouble(),
        width: rect.size.width.ToDouble(),
        height: rect.size.height.ToDouble(),
    })
}

#[inline(never)]
pub(super) fn text_offset_for_point(
    cursor: NativePaintFragmentCursor<'_>,
    point: crate::internal::layout_input::Offset,
) -> Option<u32> {
    let NativeFragmentCursor::Inline { item, items, .. } = cursor.0 else {
        return None;
    };
    if item.Type() != layout_fragment::fragment_item::ItemType::kText {
        return None;
    }
    Some(item.TextOffsetForPoint(
        PhysicalOffset::new(
            foundation::LayoutUnit::from_f64(point.x),
            foundation::LayoutUnit::from_f64(point.y),
        ),
        items,
    ))
}
