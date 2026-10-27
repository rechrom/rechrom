//! Native box and inline ink-overflow projection.
//!
//! Kept in `layout_fragment` so glyph-bound and visual-overflow work is
//! distinguishable from generic boundary conversion in sampling profiles.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:1736-1848
pub(super) fn ExportNativeInkRect(rect: PhysicalRect) -> FragmentPaintRect {
    FragmentPaintRect {
        offset: Offset {
            x: Number(rect.offset.left),
            y: Number(rect.offset.top),
        },
        size: Size {
            width: Number(rect.size.width),
            height: Number(rect.size.height),
        },
    }
}

#[inline(never)]
pub(super) fn ExportBoxInkOverflow(fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    paint.box_fragment_is_inline_box = fragment.IsInlineBox();
    if fragment.IsInkOverflowComputed() {
        paint.self_ink_overflow = Some(ExportNativeInkRect(fragment.SelfInkOverflowRect()));
        paint.contents_ink_overflow = Some(ExportNativeInkRect(fragment.ContentsInkOverflowRect()));
        paint.ink_overflow = Some(ExportNativeInkRect(fragment.InkOverflowRect()));
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() {
        return;
    }
    let layout_box = DynamicTo::<LayoutBox>(object);
    if !layout_box.is_null() {
        // BoxPainter::VisualRect uses the stitched LayoutBox self rect.
        // The native overflow lifecycle is not yet complete. Its unset
        // border-box default is valid for plain boxes, but cannot be exported
        // as completed shadow/outline overflow.
        let layout_box = unsafe { &*layout_box };
        let plain = paint.style.box_shadows.is_empty()
            && paint.style.outline_width == 0.0
            && paint.style.border_image.is_none()
            && paint.style.mask_box_image.is_none()
            && !fragment.IsTableRow();
        if layout_box.VisualOverflowIsSet() || (plain && !fragment.IsInlineBox()) {
            paint.box_self_visual_overflow =
                Some(ExportNativeInkRect(layout_box.SelfVisualOverflowRect()));
        }
    }
}

#[inline(never)]
pub(super) fn ExportInlineInkOverflow(item: &FragmentItem, paint: &mut PaintProperties) {
    let box_fragment = item.BoxFragment();
    if !box_fragment.is_null() {
        ExportBoxInkOverflow(unsafe { &*box_fragment }, paint);
        return;
    }
    if item.IsInkOverflowComputed() {
        paint.self_ink_overflow = Some(ExportNativeInkRect(item.SelfInkOverflowRect()));
        paint.ink_overflow = Some(ExportNativeInkRect(item.InkOverflowRect()));
        return;
    }
    if !item.IsText() || item.IsSvgText() || item.IsSymbolMarker() {
        return;
    }
    let style = item.Style();
    let decoration = &paint.style.text_decoration;
    // Only the implemented plain-text branch is computed here. Decoration
    // boxes, emphasis, shadows and selection/marker overflow require the
    // original InlinePaintContext lifecycle; do not mark those as computed.
    let plain = paint.style.applied_text_decorations.is_empty()
        && !style.HasAppliedTextDecorations()
        && !(decoration.underline || decoration.overline || decoration.line_through)
        && style.TextShadow().is_null()
        && paint.style.text_shadows.is_empty()
        && style.GetTextEmphasisMark() == foundation::TextEmphasisMark::kNone;
    // ink_overflow.cc:439-489 / SetTextInkOverflow:361-369. Glyph bounds are
    // logical baseline coordinates, not a font ascent/descent approximation.
    let size = item.Size();
    let mut rect = PhysicalRect::new(PhysicalOffset::default(), size);
    let shape = item.TextShapeResult();
    if !shape.is_null() {
        let font = item.GetUsedFont();
        let mut bounds = if font.GetFont().ShouldSkipDrawing() {
            gfx::RectF::default()
        } else {
            unsafe { &*shape }.ComputeInkBounds()
        };
        bounds.Scale(font.ScalingFactor(), font.ScalingFactor());
        let mut logical = LogicalRect::EnclosingRect(&bounds);
        logical.offset.block_offset += font.FixedAscent();
        let stroke_width = style.TextStrokeWidth();
        if stroke_width != 0.0 {
            logical.Inflate(LayoutUnit::FromFloatCeil(stroke_width / 2.0));
        }
        let converter = WritingModeConverter::new(
            foundation::WritingDirectionMode::new(
                item.GetWritingMode(),
                foundation::TextDirection::kLtr,
            ),
            size,
        );
        let glyph_rect = converter.ToPhysicalRect(logical);
        paint.text_glyph_ink = Some(ExportNativeInkRect(glyph_rect));
        if !glyph_rect.IsEmpty() && !rect.ContainsRect(&glyph_rect) {
            rect.Unite(&glyph_rect);
            rect.ExpandEdgesToPixelBoundaries();
        }
    }
    if plain {
        paint.self_ink_overflow = Some(ExportNativeInkRect(rect));
        paint.ink_overflow = paint.self_ink_overflow;
    }
}
