//! Post-publication fragment relationship resolution.
//!
//! Logical-parent occurrence resolution and sliced-inline decoration stitching
//! are whole-fragment-tree traversals with distinct package sampling frames.

#![allow(non_snake_case)]

use super::*;

#[inline(never)]
pub(super) fn ResolveLogicalParentOccurrences(root: crate::layout_engine::PaintFragmentCursor<'_>) {
    struct Occurrence {
        input: *mut FragmentPaintData,
        parent: Option<usize>,
        node_id: u64,
        fragment_instance_id: u64,
        fragmentainer_instance_id: u64,
        establishes_paint_state: bool,
    }
    fn collect(
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
        parent: Option<usize>,
        all: &mut Vec<Occurrence>,
        by_node_id: &mut LayoutBoundaryHashMap<u64, Vec<usize>>,
    ) {
        let input = cursor.PaintData();
        let index = all.len();
        all.push(Occurrence {
            input: input as *const FragmentPaintData as *mut FragmentPaintData,
            parent,
            node_id: input.node_id,
            fragment_instance_id: input.fragment_instance_id,
            fragmentainer_instance_id: input.fragmentainer_instance_id,
            establishes_paint_state: input.paint.establishes_paint_state,
        });
        if input.node_id != 0 && input.paint.HasSource() && input.paint.establishes_paint_state {
            by_node_id.entry(input.node_id).or_default().push(index);
        }
        for child in cursor.Children() {
            collect(child, Some(index), all, by_node_id);
        }
    }

    let mut all = Vec::new();
    let mut by_node_id = LayoutBoundaryHashMap::default();
    collect(root, None, &mut all, &mut by_node_id);
    for current_index in 0..all.len() {
        let input = unsafe { &mut *all[current_index].input };
        input.paint.logical_parent_fragment_instance_id = None;
        let Some(parent_node_id) = input.paint.logical_parent_node_id else {
            continue;
        };
        let mut ancestor = all[current_index].parent;
        let mut resolved = None;
        while let Some(index) = ancestor {
            if all[index].node_id == parent_node_id && all[index].establishes_paint_state {
                resolved = Some(index);
                break;
            }
            ancestor = all[index].parent;
        }
        if resolved.is_none() {
            if let Some(candidates) = by_node_id.get(&parent_node_id) {
                // Chromium rebuilds the missed descendant's LayoutObject spine
                // with a known fragmentainer context. The projection boundary
                // may select an occurrence only when that identity is unique;
                // geometry-nearest guessing would create a second, non-source
                // parent-selection path.
                let unique = |same_fragmentainer: bool| {
                    let mut matches = candidates.iter().copied().filter(|&index| {
                        index != current_index
                            && (!same_fragmentainer
                                || all[index].fragmentainer_instance_id
                                    == all[current_index].fragmentainer_instance_id)
                    });
                    let first = matches.next()?;
                    matches.next().is_none().then_some(first)
                };
                resolved = unique(true).or_else(|| unique(false));
            }
        }
        if let Some(index) = resolved {
            input.paint.logical_parent_fragment_instance_id = Some(all[index].fragment_instance_id);
        }
    }
}

#[inline(never)]
pub(super) fn PopulateInlineStitchedDecorations(
    root: crate::layout_engine::PaintFragmentCursor<'_>,
) {
    fn collect(
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
        groups: &mut HashMap<(u64, u64), Vec<*mut FragmentPaintData>>,
    ) {
        let input = cursor.PaintData();
        if input.node_id != 0
            && input.kind == FragmentKind::kBox
            && input.paint.HasSource()
            && input.paint.establishes_paint_state
            && input.paint.display == Display::kInline
            && input.paint.box_decoration_break == BoxDecorationBreak::kSlice
        {
            groups
                .entry((input.node_id, input.fragmentainer_instance_id))
                .or_default()
                .push(input as *const FragmentPaintData as *mut FragmentPaintData);
        }
        for child in cursor.Children() {
            collect(child, groups);
        }
    }
    let mut groups = HashMap::new();
    collect(root, &mut groups);
    for fragments in groups.values() {
        if fragments.len() < 2 {
            continue;
        }
        let first = unsafe { &*fragments[0] };
        let horizontal = first.paint.writing_mode == WritingMode::kHorizontalTb;
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
            let offset = if fragment.paint.direction == TextDirection::kLtr {
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
