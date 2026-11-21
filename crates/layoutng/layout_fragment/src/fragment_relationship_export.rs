//! Post-publication fragment relationship resolution.
//!
//! Sliced-inline decoration stitching (logical-parent occurrences are
//! resolved by PrePaint) is a whole-fragment-tree traversal.

#![allow(non_snake_case)]

use super::*;

#[inline(never)]
fn StitchCandidate(input: &FragmentPaintData) -> Option<(u64, u64)> {
    (input.node_id != 0
        && input.kind == FragmentKind::kBox
        && input.paint.HasSource()
        && input.paint.establishes_paint_state
        && input.paint.Display() == Display::kInline
        && input.paint.box_decoration_break == BoxDecorationBreak::kSlice)
        .then_some((input.node_id, input.fragmentainer_instance_id))
}

fn CollectStitchGroupsByCursor(
    root: crate::layout_engine::PaintFragmentCursor<'_>,
) -> HashMap<(u64, u64), Vec<*mut FragmentPaintData>> {
    fn collect(
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
        groups: &mut HashMap<(u64, u64), Vec<*mut FragmentPaintData>>,
    ) {
        let input = cursor.PaintData();
        if let Some(key) = StitchCandidate(input) {
            groups
                .entry(key)
                .or_default()
                .push(input as *const FragmentPaintData as *mut FragmentPaintData);
        }
        for child in cursor.Children() {
            collect(child, groups);
        }
    }
    let mut groups = HashMap::new();
    collect(root, &mut groups);
    groups
}

pub(super) fn PopulateInlineStitchedDecorations(
    root: crate::layout_engine::PaintFragmentCursor<'_>,
) {
    StitchGroups(CollectStitchGroupsByCursor(root));
}

/// PopulateInlineStitchedDecorations over a Paint publication's recorded
/// slots (the same occurrences in preorder) instead of a cursor walk.
pub(super) fn PopulateInlineStitchedDecorationsFromReplay(
    entries: &[super::layout_fragment_fragment_export_state::PopulationReplayEntry],
    check_root: Option<crate::layout_engine::PaintFragmentCursor<'_>>,
) {
    let mut groups: HashMap<(u64, u64), Vec<*mut FragmentPaintData>> = HashMap::new();
    for entry in entries {
        let input = entry.slot as *mut FragmentPaintData;
        if let Some(key) = StitchCandidate(unsafe { &*input }) {
            groups.entry(key).or_default().push(input);
        }
    }
    if let Some(root) = check_root {
        let walked = CollectStitchGroupsByCursor(root);
        browser_tracing::instant(
            "layout",
            "layout_fragment.StitchCheck",
            &[
                ("groups", groups.len() as f64),
                ("mismatch", (walked != groups) as u8 as f64),
            ],
        );
    }
    StitchGroups(groups);
}

fn StitchGroups(groups: HashMap<(u64, u64), Vec<*mut FragmentPaintData>>) {
    for fragments in groups.values() {
        if fragments.len() < 2 {
            continue;
        }
        let first = unsafe { &*fragments[0] };
        let horizontal = first.paint.WritingMode() == WritingMode::kHorizontalTb;
        let total_inline_size: f64 = fragments
            .iter()
            .map(|fragment| {
                let fragment = unsafe { &**fragment };
                if horizontal {
                    fragment.size.width
                } else {
                    fragment.size.height
                }
            })
            .sum();
        let mut before = 0.0;
        for &fragment in fragments {
            let fragment = unsafe { &mut *fragment };
            let inline_size = if horizontal {
                fragment.size.width
            } else {
                fragment.size.height
            };
            let offset = if fragment.paint.Direction() == TextDirection::kLtr {
                before
            } else {
                total_inline_size - before - inline_size
            };
            fragment.paint.stitched_decoration = Some(StitchedDecorationData {
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
            });
            before += inline_size;
        }
    }
}
