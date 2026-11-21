//! Chromium box and inline ink-overflow projection.
//!
//! Kept in `layout_fragment` so glyph-bound and visual-overflow work is
//! distinguishable from generic boundary conversion in sampling profiles.

#![allow(non_snake_case)]

use super::*;

pub(super) fn ExportInkRect(rect: PhysicalRect) -> FragmentPaintRect {
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
pub(super) fn ExportInlineInkOverflow(item: &FragmentItem, paint: &mut PaintProperties) {
    // Box items and items with computed ink overflow are read live
    // (ResidentBoxGeometry::ViewInkOverflows); only the uncomputed-text
    // fallback below is stored.
    if !item.BoxFragment().is_null() || item.IsInkOverflowComputed() {
        return;
    }
    if !item.IsText() || item.IsSvgText() || item.IsSymbolMarker() {
        return;
    }
    let style = item.Style();
    let decoration = &paint.style.text_decoration;
    // This export fallback is only for a text item whose source overflow was
    // not recalculated. Decorated/emphasized/shadowed text must have gone
    // through FragmentItem's retained InlinePaintContext lifecycle; do not
    // fabricate those effects from the flattened Paint snapshot here.
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
        paint.text_glyph_ink = Some(ExportInkRect(glyph_rect));
        if !glyph_rect.IsEmpty() && !rect.ContainsRect(&glyph_rect) {
            rect.Unite(&glyph_rect);
            rect.ExpandEdgesToPixelBoundaries();
        }
    }
    if plain {
        paint.self_ink_overflow = Some(ExportInkRect(rect));
        paint.ink_overflow = paint.self_ink_overflow;
    }
}
