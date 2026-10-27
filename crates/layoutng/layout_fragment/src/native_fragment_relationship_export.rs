//! Post-publication fragment relationship resolution.
//!
//! Logical-parent occurrence selection and sliced-inline decoration stitching
//! are whole-fragment-tree traversals with distinct package sampling frames.

#![allow(non_snake_case)]

use super::*;

#[inline(never)]
pub(super) fn ResolveNativeLogicalParentOccurrences(
    root: crate::layout_engine::NativePaintFragmentCursor<'_>,
) {
    struct Occurrence {
        input: *mut FragmentPaintData,
        parent: Option<usize>,
        node_id: u64,
        fragment_instance_id: u64,
        fragmentainer_instance_id: u64,
        absolute_offset: Offset,
        size: Size,
        establishes_paint_state: bool,
    }
    fn collect(
        cursor: crate::layout_engine::NativePaintFragmentCursor<'_>,
        parent: Option<usize>,
        parent_offset: Offset,
        all: &mut Vec<Occurrence>,
        by_node_id: &mut LayoutBoundaryHashMap<u64, Vec<usize>>,
    ) {
        let input = cursor.PaintData();
        let absolute_offset = Offset {
            x: parent_offset.x + input.offset.x,
            y: parent_offset.y + input.offset.y,
        };
        let index = all.len();
        all.push(Occurrence {
            input: input as *const FragmentPaintData as *mut FragmentPaintData,
            parent,
            node_id: input.node_id,
            fragment_instance_id: input.fragment_instance_id,
            fragmentainer_instance_id: input.fragmentainer_instance_id,
            absolute_offset,
            size: input.size,
            establishes_paint_state: input.paint.establishes_paint_state,
        });
        if input.node_id != 0 && input.paint.has_source && input.paint.establishes_paint_state {
            by_node_id.entry(input.node_id).or_default().push(index);
        }
        for child in cursor.Children() {
            collect(child, Some(index), absolute_offset, all, by_node_id);
        }
    }
    fn distance_squared(candidate: &Occurrence, child: Offset) -> f64 {
        let right = candidate.absolute_offset.x + candidate.size.width;
        let bottom = candidate.absolute_offset.y + candidate.size.height;
        let dx = if child.x < candidate.absolute_offset.x {
            candidate.absolute_offset.x - child.x
        } else if child.x > right {
            child.x - right
        } else {
            0.0
        };
        let dy = if child.y < candidate.absolute_offset.y {
            candidate.absolute_offset.y - child.y
        } else if child.y > bottom {
            child.y - bottom
        } else {
            0.0
        };
        dx * dx + dy * dy
    }

    let mut all = Vec::new();
    let mut by_node_id = LayoutBoundaryHashMap::default();
    collect(root, None, Offset::default(), &mut all, &mut by_node_id);
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
                let child_offset = all[current_index].absolute_offset;
                let closest = |same_fragmentainer: bool| {
                    candidates
                        .iter()
                        .copied()
                        .filter(|&index| {
                            index != current_index
                                && (!same_fragmentainer
                                    || all[index].fragmentainer_instance_id
                                        == all[current_index].fragmentainer_instance_id)
                        })
                        .min_by(|&left, &right| {
                            distance_squared(&all[left], child_offset)
                                .total_cmp(&distance_squared(&all[right], child_offset))
                                .then_with(|| {
                                    all[left]
                                        .fragment_instance_id
                                        .cmp(&all[right].fragment_instance_id)
                                })
                        })
                };
                resolved = closest(true).or_else(|| closest(false));
            }
        }
        if let Some(index) = resolved {
            input.paint.logical_parent_fragment_instance_id = Some(all[index].fragment_instance_id);
        }
    }
}

#[inline(never)]
pub(super) fn PopulateNativeInlineStitchedDecorations(
    root: crate::layout_engine::NativePaintFragmentCursor<'_>,
) {
    fn collect(
        cursor: crate::layout_engine::NativePaintFragmentCursor<'_>,
        groups: &mut HashMap<(u64, u64), Vec<*mut FragmentPaintData>>,
    ) {
        let input = cursor.PaintData();
        if input.node_id != 0
            && input.kind == FragmentKind::kBox
            && input.paint.has_source
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
