//! Bridge the resident native layout's scroll/sticky dirty objects to its owned
//! exported fragments. This directory is valid only for one completed layout;
//! it owns neither fragments nor native objects and adds no paint properties.

use super::{
    FragmentNode, InputId, LayoutObject, Position, RefreshFragmentScrollPaintProperties,
    UpdateStickyOffsets,
};
use layoutng_fragment_tree::fragment_tree::pre_paint_revision::MarkScrollPropertyChange;
use std::collections::{HashMap, HashSet};

struct FragmentEntry {
    path: Vec<usize>,
    node_id: u64,
    logical_tree_order: u64,
}

pub(crate) struct ScrollPaintPropertyIndex {
    root: *mut LayoutObject,
    sources: Vec<*mut LayoutObject>,
    // Preorder is required for nested sticky constraints, just as in the full
    // refresh. Include anonymous native sticky objects even without an output.
    sticky_objects: Vec<*mut LayoutObject>,
    fragments: Vec<FragmentEntry>,
    by_source_id: HashMap<u64, Vec<usize>>,
    sticky_fragments: Vec<usize>,
}

impl ScrollPaintPropertyIndex {
    pub(crate) fn new(root: &mut LayoutObject, fragments: &FragmentNode) -> Self {
        let root = root as *mut LayoutObject;
        let mut index = Self {
            root,
            sources: Vec::new(),
            sticky_objects: Vec::new(),
            fragments: Vec::new(),
            by_source_id: HashMap::new(),
            sticky_fragments: Vec::new(),
        };
        let mut object = root;
        while !object.is_null() {
            let current = unsafe { &*object };
            if !current.GetNode().is_null() {
                index.sources.push(object);
            }
            if current.StyleRef().HasStickyConstrainedPosition() {
                index.sticky_objects.push(object);
            }
            object = current.NextInPreOrder(root);
        }
        fn collect(
            index: &mut ScrollPaintPropertyIndex,
            fragment: &FragmentNode,
            path: &mut Vec<usize>,
        ) {
            if fragment.paint.has_source {
                if let Some(&object) = fragment
                    .paint
                    .logical_tree_order
                    .checked_sub(1)
                    .and_then(|order| usize::try_from(order).ok())
                    .and_then(|order| index.sources.get(order))
                {
                    let entry = index.fragments.len();
                    index.fragments.push(FragmentEntry {
                        path: path.clone(),
                        node_id: fragment.node_id,
                        logical_tree_order: fragment.paint.logical_tree_order,
                    });
                    index
                        .by_source_id
                        .entry(InputId(object))
                        .or_default()
                        .push(entry);
                    if fragment.paint.position == Position::kSticky {
                        index.sticky_fragments.push(entry);
                    }
                }
            }
            for (child, fragment) in fragment.children.iter().enumerate() {
                path.push(child);
                collect(index, fragment, path);
                path.pop();
            }
        }
        collect(&mut index, fragments, &mut Vec::new());
        index
    }

    /// The engine invalidates this index before every non-scroll input change.
    /// Its resident tree roots all borrowed native pointers for this layout.
    /// All admission checks precede updates; failure keeps the full refresh.
    pub(crate) fn refresh(
        &self,
        root: &mut LayoutObject,
        fragments: &mut FragmentNode,
        changed_ids: &HashSet<u64>,
    ) -> bool {
        if changed_ids.is_empty() || self.root != root as *mut LayoutObject {
            return false;
        }
        let mut entries = self.sticky_fragments.clone();
        for id in changed_ids {
            let Some(selected) = self.by_source_id.get(id) else {
                return false;
            };
            entries.extend_from_slice(selected);
        }
        entries.sort_unstable();
        entries.dedup();
        for &entry in &entries {
            let entry = &self.fragments[entry];
            let mut fragment: &FragmentNode = fragments;
            for &child in &entry.path {
                let Some(next) = fragment.children.get(child) else {
                    return false;
                };
                fragment = next;
            }
            if !fragment.paint.has_source
                || fragment.node_id != entry.node_id
                || fragment.paint.logical_tree_order != entry.logical_tree_order
            {
                return false;
            }
        }
        // Blink PaintLayerScrollableArea::InvalidatePaintForStickyDescendants
        // visits consumed sticky descendants rather than unrelated objects.
        // Keep the full real sticky set here (a conservative superset) and the
        // existing constraint/axis/scroll-container calculation unchanged.
        UpdateStickyOffsets(self.sticky_objects.iter().copied(), true);
        for entry in entries {
            refresh_path(fragments, &self.fragments[entry].path, &mut |fragment| {
                RefreshFragmentScrollPaintProperties(fragment, &self.sources)
            });
        }
        true
    }
}

fn refresh_path(
    fragment: &mut FragmentNode,
    path: &[usize],
    update: &mut impl FnMut(&mut FragmentNode) -> bool,
) -> bool {
    let (changed, own_changed) = if let Some((&child, rest)) = path.split_first() {
        (
            refresh_path(&mut fragment.children[child], rest, update),
            false,
        )
    } else {
        let changed = update(fragment);
        (changed, changed)
    };
    if changed {
        MarkScrollPropertyChange(fragment, own_changed);
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_scroll_refresh_matches_full_walk_and_revision_paths() {
        fn node(id: u64) -> FragmentNode {
            FragmentNode {
                node_id: id,
                ..Default::default()
            }
        }
        let mut input = node(1);
        input.children = vec![node(2), node(3)];
        input.children[0].children = vec![node(4)];
        layoutng_fragment_tree::fragment_tree::pre_paint_revision::PrePaintRevisionTracker::new()
            .update(&mut input);
        let before = input.clone();
        let mut full = input.clone();
        let mut indexed = input;
        let mut update = |node: &mut FragmentNode| {
            if node.node_id != 4 {
                return false;
            }
            let changed = node.paint.scroll_offset.y != 20.0;
            node.paint.scroll_offset.y = 20.0;
            changed
        };
        fn walk(
            node: &mut FragmentNode,
            update: &mut impl FnMut(&mut FragmentNode) -> bool,
        ) -> bool {
            let own = update(node);
            let mut changed = own;
            for child in &mut node.children {
                changed |= walk(child, update);
            }
            if changed {
                MarkScrollPropertyChange(node, own);
            }
            changed
        }
        walk(&mut full, &mut update);
        refresh_path(&mut indexed, &[0, 0], &mut update);
        fn equal(a: &FragmentNode, b: &FragmentNode, before: &FragmentNode) {
            assert_eq!(a.offset, b.offset);
            assert_eq!(a.size, b.size);
            assert_eq!(a.content_size, b.content_size);
            assert_eq!(a.paint.scroll_offset, b.paint.scroll_offset);
            // Independent refreshes use globally unique tokens; compare the
            // exact dirty paths, not the numeric generation of each pass.
            assert_eq!(
                a.pre_paint_revision != before.pre_paint_revision,
                b.pre_paint_revision != before.pre_paint_revision
            );
            assert_eq!(
                a.pre_paint_subtree_revision != before.pre_paint_subtree_revision,
                b.pre_paint_subtree_revision != before.pre_paint_subtree_revision
            );
            assert_ne!(a.pre_paint_revision, 0);
            assert_ne!(b.pre_paint_revision, 0);
            assert_ne!(a.pre_paint_subtree_revision, 0);
            assert_ne!(b.pre_paint_subtree_revision, 0);
            for ((a, b), before) in a.children.iter().zip(&b.children).zip(&before.children) {
                equal(a, b, before);
            }
        }
        equal(&full, &indexed, &before);
        assert!(!refresh_path(&mut indexed, &[0, 0], &mut update));
        assert_eq!(
            indexed.children[1].pre_paint_subtree_revision,
            before.children[1].pre_paint_subtree_revision
        );
        let mut unobserved = FragmentNode::default();
        MarkScrollPropertyChange(&mut unobserved, true);
        assert_eq!(unobserved.pre_paint_revision, 0);
        assert_eq!(unobserved.pre_paint_subtree_revision, 0);
    }
}
