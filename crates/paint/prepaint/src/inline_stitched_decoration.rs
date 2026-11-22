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

use layout::fragment::ResidentBoxGeometry as _;
use std::collections::HashMap;

use foundation::{LayoutOffset as Offset, LayoutSize as Size};
use layout::fragment::FragmentKind;
use layout::paint::{FragmentPaintData, StitchedDecorationData};
use layout::input::{BoxDecorationBreak, Display};
use paint_input_types::layout_input::{TextDirection, WritingMode};
use layout::paint::PaintFragmentCursor;

/// `fragmentainer` is the identity of the enclosing column/page area, carried
/// by the walk the way PrePaintTreeWalk carries `fragmentainer_idx`
/// (pre_paint_tree_walk.cc); 0 outside fragmentation.
fn StitchCandidate(input: &FragmentPaintData, fragmentainer: u64) -> Option<(u64, u64)> {
    let node_id = input.ViewNodeId();
    (node_id != 0
        && input.ViewKind() == FragmentKind::kBox
        && input.paint.HasSource()
        && input.ViewEstablishesPaintState()
        && input.paint.Display() == Display::kInline
        && input.ViewBoxDecorationBreak() == BoxDecorationBreak::kSlice)
        .then_some((node_id, fragmentainer))
}

/// Resolve the stitched strip of every sliced inline box under `root`.
pub(crate) fn PopulateInlineStitchedDecorations(root: PaintFragmentCursor<'_>) {
    fn collect<'a>(
        cursor: PaintFragmentCursor<'a>,
        fragmentainer: u64,
        groups: &mut HashMap<(u64, u64), Vec<PaintFragmentCursor<'a>>>,
        order: &mut Vec<(u64, u64)>,
    ) {
        let fragmentainer = cursor.FragmentainerInstanceId().unwrap_or(fragmentainer);
        if let Some(key) = StitchCandidate(cursor.PaintData(), fragmentainer) {
            let group = groups.entry(key).or_default();
            if group.is_empty() {
                order.push(key);
            }
            group.push(cursor);
        }
        for child in cursor.Children() {
            collect(child, fragmentainer, groups, order);
        }
    }
    let mut groups = HashMap::new();
    let mut order = Vec::new();
    collect(root, 0, &mut groups, &mut order);
    for key in order {
        let fragments = &groups[&key];
        if fragments.len() < 2 {
            continue;
        }
        let first = fragments[0].PaintData();
        let horizontal = first.paint.WritingMode() == WritingMode::kHorizontalTb;
        let inline_size = |fragment: &FragmentPaintData| {
            if horizontal {
                fragment.ViewSize().width
            } else {
                fragment.ViewSize().height
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
                        height: fragment.ViewSize().height,
                    }
                } else {
                    Size {
                        width: fragment.ViewSize().width,
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
