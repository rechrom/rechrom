// On-demand text queries consume canonical FragmentItem/ShapeResult state.
// Keep their public cursor wrappers thin and expose one fragment sampling frame
// per query class.

use super::*;

#[inline(never)]
pub(super) fn paint_glyph_runs(
    cursor: PaintFragmentCursor<'_>,
) -> Option<std::sync::Arc<Vec<layout_fragment::fragment_tree::PaintGlyphRun>>> {
    let FragmentCursor::Inline { item, .. } = cursor.Cursor() else {
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
pub(super) fn text_paint_baseline(cursor: PaintFragmentCursor<'_>) -> Option<f64> {
    let FragmentCursor::Inline { item, .. } = cursor.Cursor() else {
        return None;
    };
    crate::internal::boundary::layout_boundary::ExportTextPaintBaseline(item)
}

#[inline(never)]
pub(super) fn text_inline_position(cursor: PaintFragmentCursor<'_>, offset: u32) -> Option<f64> {
    let FragmentCursor::Inline { item, items, .. } = cursor.Cursor() else {
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
pub(super) fn text_next_character_width(
    cursor: PaintFragmentCursor<'_>,
    offset: u32,
) -> Option<f64> {
    use font_engine::fonts::shaping::text_width::ComputeTextWidth;
    use font_engine::text::implementation::character_break_iterator::LengthOfGraphemeCluster;

    let FragmentCursor::Inline { item, items, .. } = cursor.Cursor() else {
        return None;
    };
    if !item.IsText() {
        return None;
    }
    let text = item.Text(items);
    let local_offset = offset
        .clamp(item.StartOffset(), item.EndOffset())
        .saturating_sub(item.StartOffset());
    let width = if local_offset < text.length() {
        let cluster_size = LengthOfGraphemeCluster(&text, local_offset);
        ComputeTextWidth(
            &text.Substring(local_offset, cluster_size),
            item.Style().GetFont(),
        ) as f64
    } else {
        0.0
    };
    if width != 0.0 {
        return Some(width);
    }
    let primary = unsafe { &*item.Style().GetFont() }.PrimaryFont();
    (!primary.is_null()).then(|| unsafe { &*primary }.GetFontMetrics().ZeroWidth() as f64)
}

#[inline(never)]
pub(super) fn text_range_rect(
    cursor: PaintFragmentCursor<'_>,
    start: u32,
    end: u32,
) -> Option<LayoutRect> {
    let FragmentCursor::Inline { item, items, .. } = cursor.Cursor() else {
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
pub(super) fn text_selection_range_rect(
    cursor: PaintFragmentCursor<'_>,
    start: u32,
    end: u32,
    select_soft_line_break: bool,
) -> Option<LayoutRect> {
    let FragmentCursor::Inline {
        item,
        items,
        container,
        ..
    } = cursor.Cursor()
    else {
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
    let mut inline = layout_fragment::inline_cursor::InlineCursor::new_with_items(container, items);
    inline.MoveToFragmentItem(item);
    let rect = inline.CurrentLocalSelectionRectForText(
        start,
        end,
        select_soft_line_break && inline.IsBeforeSoftLineBreak(),
    );
    Some(LayoutRect {
        x: rect.offset.left.ToDouble(),
        y: rect.offset.top.ToDouble(),
        width: rect.size.width.ToDouble(),
        height: rect.size.height.ToDouble(),
    })
}

// highlight_painter.cc:50-60. Unlike a selection background rect, this is
// line-relative geometry used to clip the selected glyph pass. Keep the
// ShapeResult query beside FragmentItem and return only the small geometry
// value Paint needs for the active range.
#[inline(never)]
pub(super) fn text_highlight_range_rect(
    cursor: PaintFragmentCursor<'_>,
    start: u32,
    end: u32,
) -> Option<LayoutRect> {
    let FragmentCursor::Inline { item, items, .. } = cursor.Cursor() else {
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
    let (left, right) = item.LineLeftAndRightForOffsets(item.Text(items), start, end);
    Some(LayoutRect {
        x: left.ToDouble(),
        y: 0.0,
        width: (right - left).ToDouble(),
        height: item.InkOverflowRect().Height().ToDouble(),
    })
}

#[inline(never)]
pub(super) fn text_offset_for_point(
    cursor: PaintFragmentCursor<'_>,
    point: crate::internal::layout_input::Offset,
) -> Option<u32> {
    let FragmentCursor::Inline { item, items, .. } = cursor.Cursor() else {
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
