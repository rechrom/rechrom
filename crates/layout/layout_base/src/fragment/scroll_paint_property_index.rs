//! Chromium directory from scroll/sticky dirty objects to retained fragments.
//!
//! This source is compiled as a child of the LayoutNG boundary module. The
//! directory is valid for one completed layout and owns neither source objects
//! nor fragments; it only provides package-attributed targeted refresh.

use super::layout_fragment_scroll_paint_property_refresh::{
    refresh_fragment_pre_paint_invalidation, refresh_fragment_scroll_paint_properties,
};
use super::{
    FragmentPaintData, InputId, LayoutObject, LayoutResultHandle, Position, UpdateStickyOffsets,
};
use crate::fragment_tree::ResidentBoxGeometry as _;
use crate::layout_engine::PaintFragmentCursor;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

struct FragmentEntry {
    path_start: usize,
    path_len: usize,
    fragment: *mut FragmentPaintData,
    node_id: u64,
    source_id: u64,
    logical_tree_order: u64,
}

pub(crate) struct ScrollPaintPropertyIndex {
    root: *mut LayoutObject,
    // Preorder is required for nested sticky constraints, just as in the full
    // refresh. Include anonymous source sticky objects even without an output.
    sticky_objects: Arc<Vec<*mut LayoutObject>>,
    fragments: Vec<FragmentEntry>,
    // Chromium's resident FragmentData directory does not allocate one owned
    // path per fragment. Keep the immutable boundary equivalent contiguous:
    // entries identify slices in this arena for admission and refresh.
    path_components: Vec<*mut FragmentPaintData>,
    by_source_id: HashMap<u64, Vec<usize>>,
    sticky_fragments: Vec<usize>,
}

impl ScrollPaintPropertyIndex {
    #[inline(never)]
    pub(crate) fn new(
        root: &mut LayoutObject,
        fragments: &LayoutResultHandle,
        cached_objects: Option<(Arc<Vec<*mut LayoutObject>>, Arc<Vec<*mut LayoutObject>>)>,
        reuse: Option<Self>,
    ) -> Self {
        let root = root as *mut LayoutObject;
        // Fragments name their source through the bound LayoutObject, so only
        // the cached sticky list is needed from the boundary object index.
        let sticky_objects = match cached_objects {
            Some((_, sticky_objects)) => sticky_objects,
            None => {
                let mut sticky_objects = Vec::new();
                let mut object = root;
                while !object.is_null() {
                    let current = unsafe { &*object };
                    if current.StyleRef().HasStickyConstrainedPosition() {
                        sticky_objects.push(object);
                    }
                    object = current.NextInPreOrder(root);
                }
                Arc::new(sticky_objects)
            }
        };
        let mut index = reuse.unwrap_or_else(|| Self {
            root,
            sticky_objects: Arc::new(Vec::new()),
            fragments: Vec::new(),
            path_components: Vec::new(),
            by_source_id: HashMap::new(),
            sticky_fragments: Vec::new(),
        });
        index.root = root;
        index.sticky_objects = sticky_objects;
        index.fragments.clear();
        index.path_components.clear();
        for entries in index.by_source_id.values_mut() {
            entries.clear();
        }
        index.sticky_fragments.clear();
        fn collect(
            index: &mut ScrollPaintPropertyIndex,
            fragment: PaintFragmentCursor<'_>,
            path: &mut Vec<*mut FragmentPaintData>,
        ) {
            let input = fragment.PaintData() as *const FragmentPaintData as *mut FragmentPaintData;
            path.push(input);
            index.admit(input, path);
            for fragment in fragment.Children() {
                collect(index, fragment, path);
            }
            path.pop();
        }
        collect(
            &mut index,
            PaintFragmentCursor::FromHandle(fragments),
            &mut Vec::new(),
        );
        // Source identities may disappear after DOM mutations. Drop empty
        // buckets after their capacity had a chance to serve this rebuild.
        index.by_source_id.retain(|_, entries| !entries.is_empty());
        index
    }

    /// The engine invalidates this index before every non-scroll input change.
    /// Its resident tree roots all borrowed source pointers for this layout.
    /// All admission checks precede updates; failure keeps the full refresh.
    #[inline(never)]
    pub(crate) fn refresh(&self, root: &mut LayoutObject, changed_ids: &HashSet<u64>) -> bool {
        let mut trace = layout_commit::fragment_trace::span(
            layout_commit::fragment_trace::TracePhase::RefreshScrollPaintIndex,
        );
        trace.set("changed_ids", changed_ids.len() as f64);
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
        trace.set("fragments", entries.len() as f64);
        for &entry in &entries {
            let entry = &self.fragments[entry];
            let fragment = unsafe { &*entry.fragment };
            if !fragment.paint.HasSource()
                || fragment.ViewNodeId() != entry.node_id
                || fragment.paint.LogicalTreeOrder() != entry.logical_tree_order
            {
                return false;
            }
        }
        // Blink PaintLayerScrollableArea::InvalidatePaintForStickyDescendants
        // visits consumed sticky descendants rather than unrelated objects.
        // Keep the full real sticky set here (a conservative superset) and the
        // existing constraint/axis/scroll-container calculation unchanged.
        UpdateStickyOffsets(self.sticky_objects.iter().copied(), true);
        let mut changed_fragments = 0usize;
        let mut refreshed_sources = HashSet::new();
        let mut republished_ancestors = HashSet::new();
        for entry in entries {
            let entry = &self.fragments[entry];
            let path = &self.path_components[entry.path_start..entry.path_start + entry.path_len];
            let own_changed =
                refresh_fragment_scroll_paint_properties(unsafe { &mut *entry.fragment });
            // The refreshed object republished its own dirty-bit snapshot.
            // Its ancestors carry DescendantNeedsPaintPropertyUpdate from the
            // scroll/sticky SetNeedsPaintPropertyUpdate propagation; publish
            // each ancestor slot once (core/paint/
            // paint_layer_scrollable_area.cc:2417-2440).
            for &ancestor in &path[..path.len().saturating_sub(1)] {
                if republished_ancestors.insert(ancestor) {
                    refresh_fragment_pre_paint_invalidation(unsafe { &mut *ancestor });
                }
            }
            if own_changed {
                changed_fragments += 1;
                refreshed_sources.insert(entry.source_id);
            }
        }
        trace.set("changed_fragments", changed_fragments as f64);
        trace.set("refreshed_sources", refreshed_sources.len() as f64);
        // The global viewport scroller may be represented by a source object
        // other than the DOM node whose scroll offset changed. Chromium finds
        // that owner through PaintLayerScrollableArea. If the narrow DOM-id
        // directory selected no fragment whose live paint input changed, it
        // did not perform the update. Require this proof for every dirty
        // source: one successful sibling must not hide a missed owner when
        // several scrollers change in the same frame. The caller then uses
        // the existing owner walk instead of certifying an old scroll
        // transform as current.
        changed_ids.iter().all(|id| refreshed_sources.contains(id))
    }
}

impl ScrollPaintPropertyIndex {
    /// Index one occurrence whose ancestor path (itself last) is `path`.
    fn admit(&mut self, input: *mut FragmentPaintData, path: &[*mut FragmentPaintData]) {
        let fragment = unsafe { &*input };
        if !fragment.paint.HasSource() {
            return;
        }
        // Slot initialization binds every sourced slot to its LayoutObject; HasSource()
        // reads that binding, so the bound owner is the source object.
        let object = fragment.paint.SourceObjectPointer();
        if object.is_null() {
            return;
        }
        let entry = self.fragments.len();
        let path_start = self.path_components.len();
        self.path_components.extend_from_slice(path);
        // ScrollingCoordinator reports the DOM owner associated
        // with the display-item client. For the global viewport
        // this intentionally differs from LayoutView's own node.
        let source_id = fragment.paint.DisplayItemClientOwnerNodeId() as u64;
        let source_id = if source_id == 0 {
            InputId(object)
        } else {
            source_id
        };
        self.fragments.push(FragmentEntry {
            path_start,
            path_len: path.len(),
            fragment: input,
            node_id: fragment.ViewNodeId(),
            source_id,
            logical_tree_order: fragment.paint.LogicalTreeOrder(),
        });
        self.by_source_id.entry(source_id).or_default().push(entry);
        if fragment.paint.Position() == Position::kSticky {
            self.sticky_fragments.push(entry);
        }
    }
}
