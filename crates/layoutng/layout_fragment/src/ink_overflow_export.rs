//! Chromium box and inline ink-overflow projection.
//!
//! Kept in `layout_fragment` so glyph-bound and visual-overflow work is
//! distinguishable from generic boundary conversion in sampling profiles.

#![allow(non_snake_case)]

use super::*;
use foundation::OverlayScrollbarClipBehavior;

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
pub(super) fn ExportBoxInkOverflow(fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    paint.box_fragment_is_inline_box = fragment.IsInlineBox();
    if paint.ShouldClipOverflowAlongEitherAxis() {
        paint.hit_test_overflow_clip = Some(ExportInkRect(fragment.OverflowClipRect(
            OverlayScrollbarClipBehavior::kExcludeOverlayScrollbarSizeForHitTesting,
        )));
    }
    if !fragment.GetLayoutObject().is_null() {
        let scrolled = fragment.PixelSnappedScrolledContentOffset();
        paint.hit_test_scrolled_content_offset = Offset {
            x: scrolled.x() as f64,
            y: scrolled.y() as f64,
        };
    }
    if fragment.IsInkOverflowComputed() {
        paint.self_ink_overflow = Some(ExportInkRect(fragment.SelfInkOverflowRect()));
        paint.contents_ink_overflow = Some(ExportInkRect(fragment.ContentsInkOverflowRect()));
        let ink_overflow = fragment.InkOverflowRect();
        paint.ink_overflow = Some(ExportInkRect(ink_overflow));
        let box_model = DynamicTo::<LayoutBoxModelObject>(fragment.GetLayoutObject());
        if !box_model.is_null() {
            // Source: core/paint/box_fragment_painter.cc:2122-2133. Keep the
            // source filter expansion here; Paint must not approximate filter
            // geometry from its flattened style snapshot.
            paint.filtered_ink_overflow = Some(ExportInkRect(
                unsafe { &*box_model }.ApplyFiltersToRect(&ink_overflow),
            ));
        }
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() {
        return;
    }
    let layout_box = DynamicTo::<LayoutBox>(object);
    if !layout_box.is_null() {
        // Source: core/paint/box_painter.cc:124-129 and
        // core/paint/box_fragment_painter.cc:3021-3030. LayoutBox owns the
        // visual-overflow lifecycle; SelfVisualOverflowRect() deliberately
        // returns its border-box value when no overflow storage is needed.
        // Paint consumes that retained result and never re-estimates shadows
        // or outlines from style.
        let layout_box = unsafe { &*layout_box };
        paint.box_self_visual_overflow = Some(ExportInkRect(layout_box.SelfVisualOverflowRect()));
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
        paint.self_ink_overflow = Some(ExportInkRect(item.SelfInkOverflowRect()));
        paint.ink_overflow = Some(ExportInkRect(item.InkOverflowRect()));
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
