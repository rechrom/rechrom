//! Sliced inline box decorations (`box-decoration-break: slice`).
//!
//! Source: core/paint/inline_box_fragment_painter.cc
//! InlineBoxFragmentPainter::ComputeFragmentOffsetOnLine. Chromium paints
//! each fragment of a sliced LayoutInline as one piece of a single strip:
//! the strip's inline size is the sum of the object's box fragments in the
//! same inline formatting context, and each fragment is offset by the inline
//! size of the fragments before it (after it, for RTL). Rechrom resolves the
//! strips once at the start of PrePaint, before any node reads them, instead
//! of the Layout export rewriting every slot per publication (stage 5 of the
//! export removal).

#![allow(non_snake_case)]

use std::collections::HashMap;

use foundation::{LayoutOffset as Offset, LayoutSize as Size};
use layoutng_assembly::fragment_tree::{FragmentKind, FragmentPaintData, StitchedDecorationData};
use layoutng_assembly::internal::layout_input::{
    BoxDecorationBreak, Display, TextDirection, WritingMode,
};
use layoutng_assembly::layout_engine::PaintFragmentCursor;

fn StitchCandidate(input: &FragmentPaintData) -> Option<(u64, u64)> {
    (input.node_id != 0
        && input.kind == FragmentKind::kBox
        && input.paint.HasSource()
        && input.paint.establishes_paint_state
        && input.paint.Display() == Display::kInline
        && input.paint.box_decoration_break == BoxDecorationBreak::kSlice)
        .then_some((input.node_id, input.fragmentainer_instance_id))
}

/// Resolve the stitched strip of every sliced inline box under `root`.
pub(crate) fn PopulateInlineStitchedDecorations(root: PaintFragmentCursor<'_>) {
    fn collect<'a>(
        cursor: PaintFragmentCursor<'a>,
        groups: &mut HashMap<(u64, u64), Vec<PaintFragmentCursor<'a>>>,
        order: &mut Vec<(u64, u64)>,
    ) {
        if let Some(key) = StitchCandidate(cursor.PaintData()) {
            let group = groups.entry(key).or_default();
            if group.is_empty() {
                order.push(key);
            }
            group.push(cursor);
        }
        for child in cursor.Children() {
            collect(child, groups, order);
        }
    }
    let mut groups = HashMap::new();
    let mut order = Vec::new();
    collect(root, &mut groups, &mut order);
    for key in order {
        let fragments = &groups[&key];
        if fragments.len() < 2 {
            continue;
        }
        let first = fragments[0].PaintData();
        let horizontal = first.paint.WritingMode() == WritingMode::kHorizontalTb;
        let inline_size = |fragment: &FragmentPaintData| {
            if horizontal {
                fragment.size.width
            } else {
                fragment.size.height
            }
        };
        let total_inline_size: f64 = fragments
            .iter()
            .map(|cursor| inline_size(cursor.PaintData()))
            .sum();
        let mut before = 0.0;
        for cursor in fragments {
            // PrePaint is the single lifecycle writer of the resident
            // sidecars while its source lease is active.
            let fragment = unsafe { cursor.PaintDataMut() };
            let own = inline_size(fragment);
            let offset = if fragment.paint.Direction() == TextDirection::kLtr {
                before
            } else {
                total_inline_size - before - own
            };
            let stitched = StitchedDecorationData {
                fragment_origin: Offset::default(),
                fragment_offset: if horizontal {
                    Offset { x: offset, y: 0.0 }
                } else {
                    Offset { x: 0.0, y: offset }
                },
                size: if horizontal {
                    Size {
                        width: total_inline_size,
                        height: fragment.size.height,
                    }
                } else {
                    Size {
                        width: fragment.size.width,
                        height: total_inline_size,
                    }
                },
            };
            if fragment.paint.stitched_decoration.as_ref() != Some(&stitched) {
                fragment.paint.stitched_decoration = Some(stitched);
            }
            before += own;
        }
    }
}
