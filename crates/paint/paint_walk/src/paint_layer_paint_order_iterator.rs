#![allow(non_snake_case)]

use std::cmp::Ordering;
use std::collections::HashMap;
use std::marker::PhantomData;

use layout::input::Position;
use prepaint::resident_hit_test::{ResidentPaintOrderLists, ResidentPaintOrderView};

use crate::pre_paint_tree_walk::PaintTreeNode;

/// One z-order list entry borrowed from the retained stacking lists: the
/// child layer and the property ancestors its painter re-enters.
#[derive(Clone, Copy)]
pub(crate) struct PaintLayerPaintOrderEntry<'a, 'n, 'f> {
    pub(crate) layer: &'n PaintTreeNode<'f>,
    pub(crate) property_ancestors: &'a [&'n PaintTreeNode<'f>],
}

/// A collected entry while RebuildZOrderLists runs; flattened into the
/// retained ResidentPaintOrderLists when the rebuild publishes.
#[derive(Clone)]
struct CollectedEntry<'n, 'f> {
    layer: &'n PaintTreeNode<'f>,
    property_ancestors: Vec<&'n PaintTreeNode<'f>>,
}

/// Borrowed PaintLayerStackingNode z-order lists and
/// layer_to_overlay_overflow_controls_painting_after_ map of one stacking
/// root. The lists are retained on the root's resident PrePaint node and
/// rebuilt only after PrePaint dirtied them (UpdateZOrderLists,
/// paint_layer_stacking_node.cc:436-439); painting borrows them, as
/// PaintLayerPaintOrderIterator borrows the stacking node's vectors
/// (paint_layer_paint_order_iterator.cc:39-79).
pub(crate) struct PaintLayerPaintOrder<'n, 'f> {
    lists: ResidentPaintOrderView<'n>,
    _nodes: PhantomData<&'n PaintTreeNode<'f>>,
}

/// # Safety
/// `pointer` was published by RebuildZOrderLists for a resident node of the
/// tree whose root is borrowed for `'n`; PrePaint drops the list before any
/// such node can move or be freed.
unsafe fn ResidentNode<'n, 'f>(pointer: *const ()) -> &'n PaintTreeNode<'f> {
    &*pointer.cast::<PaintTreeNode<'f>>()
}

/// # Safety
/// As ResidentNode for every pointer. `*const ()` and `&PaintTreeNode` are
/// both thin, non-null-here pointers with the same layout.
unsafe fn ResidentNodes<'a, 'n, 'f>(pointers: &'a [*const ()]) -> &'a [&'n PaintTreeNode<'f>] {
    std::slice::from_raw_parts(
        pointers.as_ptr().cast::<&'n PaintTreeNode<'f>>(),
        pointers.len(),
    )
}

fn NodePointer(node: &PaintTreeNode<'_>) -> *const () {
    node as *const PaintTreeNode<'_> as *const ()
}

impl<'n, 'f> PaintLayerPaintOrder<'n, 'f> {
    pub(crate) fn len(&self) -> usize {
        self.lists.Layers().len()
    }

    /// NegZOrderList's length: the list is sorted negative z first.
    pub(crate) fn NegativeZOrderCount(&self) -> usize {
        self.lists.Layers().partition_point(|layer| {
            let layer = unsafe { ResidentNode::<'n, 'f>(*layer) };
            layer.is_stacked && layer.stacking_level < 0
        })
    }

    pub(crate) fn Entry(&self, index: usize) -> PaintLayerPaintOrderEntry<'_, 'n, 'f> {
        PaintLayerPaintOrderEntry {
            layer: unsafe { ResidentNode(self.lists.Layers()[index]) },
            property_ancestors: unsafe { ResidentNodes(self.lists.EntryAncestors(index)) },
        }
    }

    #[cfg(test)]
    pub(crate) fn Entries(&self) -> impl Iterator<Item = PaintLayerPaintOrderEntry<'_, 'n, 'f>> + '_ {
        (0..self.len()).map(move |index| self.Entry(index))
    }

    pub(crate) fn ControlsAfter<'s>(
        &'s self,
        layer: &PaintTreeNode<'f>,
    ) -> impl ExactSizeIterator<Item = PaintLayerPaintOrderEntry<'s, 'n, 'f>> + 's {
        self.lists
            .ControlsAfter(NodePointer(layer))
            .iter()
            .map(move |&(scroller, ancestors)| PaintLayerPaintOrderEntry {
                layer: unsafe { ResidentNode(scroller) },
                property_ancestors: unsafe { ResidentNodes(self.lists.Ancestors(ancestors)) },
            })
    }

    pub(crate) fn NeedsReorderOverlayOverflowControls(&self, layer: &PaintTreeNode<'f>) -> bool {
        layer.needs_reorder_overlay_overflow_controls.get()
    }
}

fn ZIndex(node: &PaintTreeNode<'_>) -> i32 {
    node.stacking_level
}

// Source: core/paint/paint_layer_paint_order_iterator.cc:39-79.
fn PaintOrderKey(node: &PaintTreeNode<'_>) -> (u8, i32) {
    if !node.is_stacked {
        (1, 0)
    } else if ZIndex(node) < 0 {
        (0, ZIndex(node))
    } else {
        (2, ZIndex(node))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayerType {
    AbsolutePosition,
    FixedPosition,
    InFlowStacked,
}

impl LayerType {
    fn index(self) -> usize {
        match self {
            Self::AbsolutePosition => 0,
            Self::FixedPosition => 1,
            Self::InFlowStacked => 2,
        }
    }
}

// Source: core/paint/paint_layer_stacking_node.cc:92-185.
#[derive(Default)]
struct HighestLayers<'n, 'f> {
    highest_layers: [Option<&'n PaintTreeNode<'f>>; 3],
    highest_layers_order: Vec<LayerType>,
}

fn SetIfHigher<'n, 'f>(
    first: &mut Option<&'n PaintTreeNode<'f>>,
    second: Option<&'n PaintTreeNode<'f>>,
) -> bool {
    let Some(second) = second else {
        return false;
    };
    debug_assert!(ZIndex(second) >= 0);
    if first.is_none_or(|first| ZIndex(second) >= ZIndex(first)) {
        *first = Some(second);
        return true;
    }
    false
}

impl<'n, 'f> HighestLayers<'n, 'f> {
    fn GetLayerType(layer: &PaintTreeNode<'f>) -> LayerType {
        debug_assert!(layer.is_stacked);
        match layer
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment")
            .paint
            .Position()
        {
            Position::kAbsolute => LayerType::AbsolutePosition,
            Position::kFixed => LayerType::FixedPosition,
            _ => LayerType::InFlowStacked,
        }
    }

    fn UpdateOrderForSubtreeHighestLayers(
        &mut self,
        layer_type: LayerType,
        layer: Option<&'n PaintTreeNode<'f>>,
    ) {
        if SetIfHigher(&mut self.highest_layers[layer_type.index()], layer) {
            self.highest_layers_order
                .retain(|candidate| *candidate != layer_type);
            self.highest_layers_order.push(layer_type);
        }
    }

    fn Update(&mut self, layer: &'n PaintTreeNode<'f>) {
        if !layer.is_stacked || ZIndex(layer) < 0 {
            return;
        }
        if !layer.is_z_order_list_visible {
            return;
        }
        self.UpdateOrderForSubtreeHighestLayers(Self::GetLayerType(layer), Some(layer));
    }

    fn Merge(&mut self, child: &HighestLayers<'n, 'f>, current: &PaintTreeNode<'f>) {
        for layer_type in &child.highest_layers_order {
            let mut propagation_type = *layer_type;
            if current.is_stacked
                && ((*layer_type == LayerType::AbsolutePosition
                    && current.can_contain_absolute_position_objects)
                    || (*layer_type == LayerType::FixedPosition
                        && current.can_contain_fixed_position_objects)
                    || *layer_type == LayerType::InFlowStacked)
            {
                propagation_type = Self::GetLayerType(current);
            }
            self.UpdateOrderForSubtreeHighestLayers(
                propagation_type,
                child.highest_layers[layer_type.index()],
            );
        }
    }
}

// Source: core/paint/paint_layer_stacking_node.cc:187-254.
fn OrderLessThan(first: &PaintTreeNode<'_>, second: &PaintTreeNode<'_>) -> bool {
    match (
        first.paint_layer_order_parent_unique_id,
        second.paint_layer_order_parent_unique_id,
    ) {
        (Some(first_parent), Some(second_parent)) if first_parent == second_parent => {
            first.paint_layer_order < second.paint_layer_order
        }
        _ => false,
    }
}

fn SortChildrenByOrder(children: &mut [CollectedEntry<'_, '_>]) {
    children.sort_by(|first, second| {
        if OrderLessThan(first.layer, second.layer) {
            Ordering::Less
        } else if OrderLessThan(second.layer, first.layer) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
}

fn CollectDirectLayerChildren<'n, 'f>(
    parent: &'n PaintTreeNode<'f>,
    path: &mut Vec<&'n PaintTreeNode<'f>>,
    children: &mut Vec<CollectedEntry<'n, 'f>>,
) {
    for child_owner in &parent.children {
        let child = child_owner.as_ref();
        if child.is_embedded_local_frame_root {
            continue;
        }
        if child.is_paint_layer {
            children.push(CollectedEntry {
                layer: child,
                property_ancestors: path.clone(),
            });
        } else {
            path.push(child);
            CollectDirectLayerChildren(child, path, children);
            path.pop();
        }
    }
}

struct Collector<'n, 'f> {
    entries: Vec<CollectedEntry<'n, 'f>>,
    overscroll_area_parents: Vec<CollectedEntry<'n, 'f>>,
    top_or_view_transition_layers: Vec<CollectedEntry<'n, 'f>>,
    collect_top_or_view_transition_layers: bool,
    controls_after: HashMap<*const PaintTreeNode<'f>, Vec<CollectedEntry<'n, 'f>>>,
}

impl<'n, 'f> Collector<'n, 'f> {
    // Source: core/paint/paint_layer_stacking_node.cc:328-401.
    fn CollectLayer(
        &mut self,
        entry: CollectedEntry<'n, 'f>,
        mut highest_layers: Option<&mut HighestLayers<'n, 'f>>,
    ) {
        let layer = entry.layer;
        // Source: paint_layer_stacking_node.cc:332. Rebuild always clears the
        // previous lifecycle value before considering the current subtree.
        layer.needs_reorder_overlay_overflow_controls.set(false);
        if layer.is_in_top_or_view_transition_layer {
            if self.collect_top_or_view_transition_layers
                && layer.is_direct_layout_view_child
                && layer.is_stacked
            {
                self.top_or_view_transition_layers.push(entry);
            }
            return;
        }
        if layer
            .fragment
            .as_deref()
            .is_some_and(|fragment| fragment.paint.IsOverscrollAreaParent())
        {
            debug_assert!(layer.is_stacked);
            self.overscroll_area_parents.push(entry);
            return;
        }

        if let Some(highest_layers) = highest_layers.as_deref_mut() {
            highest_layers.Update(layer);
        }

        if layer.is_z_order_list_visible {
            self.entries.push(entry.clone());
        }
        if layer.is_stacking_context {
            return;
        }

        let mut subtree_highest_layers = HighestLayers::default();
        let mut child_path = entry.property_ancestors.clone();
        child_path.push(layer);
        let mut children = Vec::new();
        CollectDirectLayerChildren(layer, &mut child_path, &mut children);
        SortChildrenByOrder(&mut children);
        for child in children {
            self.CollectLayer(child, Some(&mut subtree_highest_layers));
        }

        if layer.has_overlay_overflow_controls {
            let mut paint_after = None;
            for layer_type in &subtree_highest_layers.highest_layers_order {
                if *layer_type == LayerType::FixedPosition
                    && !layer.can_contain_fixed_position_objects
                {
                    continue;
                }
                if *layer_type == LayerType::AbsolutePosition
                    && !layer.can_contain_absolute_position_objects
                {
                    continue;
                }
                SetIfHigher(
                    &mut paint_after,
                    subtree_highest_layers.highest_layers[layer_type.index()],
                );
            }
            if let Some(paint_after) = paint_after {
                self.controls_after
                    .entry(paint_after as *const PaintTreeNode<'f>)
                    .or_default()
                    .push(entry.clone());
                layer.needs_reorder_overlay_overflow_controls.set(true);
            }
        }

        if let Some(highest_layers) = highest_layers {
            highest_layers.Merge(&subtree_highest_layers, layer);
        }
    }
}

/// The z-order lists of `root`, rebuilt only when PrePaint dirtied them:
/// PaintLayerStackingNode::UpdateZOrderLists (paint_layer_stacking_node.cc:
/// 436-439), then a borrow of the retained lists as
/// PaintLayerPaintOrderIterator reads them (paint_layer_paint_order_iterator
/// .cc:39-79). PrePaint drops the lists before any topology or stacking
/// change (DirtyZOrderLists, paint_layer_stacking_node.cc:67-83).
pub(crate) fn CollectPaintLayerChildren<'n, 'f>(
    root: &'n PaintTreeNode<'f>,
) -> PaintLayerPaintOrder<'n, 'f> {
    if !root.HasResidentLayerOrder() {
        RebuildZOrderLists(root);
    }
    PaintLayerPaintOrder {
        lists: root
            .ResidentPaintOrder()
            .expect("RebuildZOrderLists published the lists"),
        _nodes: PhantomData,
    }
}

/// PaintLayerStackingNode::RebuildZOrderLists() projected over the immutable
/// PrePaint tree, including the overlay-control insertion map
/// (paint_layer_stacking_node.cc:283-326).
fn RebuildZOrderLists(root: &PaintTreeNode<'_>) {
    let mut trace = browser_tracing::span("paint_walk", "PaintWalk.CollectLayerChildren");
    let mut children = Vec::new();
    // Source: paint_layer_stacking_node.cc:289. Only a stacking context owns
    // a PaintLayerStackingNode (paint_layer_stacking_node.cc:62-65); a
    // non-stacking root collected for a hit-test query leaves the reorder
    // bit its enclosing stacking context's retained lists set.
    if root.is_stacking_context || root.is_root {
        root.needs_reorder_overlay_overflow_controls.set(false);
    }
    CollectDirectLayerChildren(root, &mut Vec::new(), &mut children);
    SortChildrenByOrder(&mut children);

    let mut collector = Collector {
        entries: Vec::new(),
        overscroll_area_parents: Vec::new(),
        top_or_view_transition_layers: Vec::new(),
        collect_top_or_view_transition_layers: root.is_root,
        controls_after: HashMap::new(),
    };
    for child in children {
        collector.CollectLayer(child, None);
    }
    collector
        .entries
        .sort_by_key(|child| PaintOrderKey(child.layer));
    collector.overscroll_area_parents.reverse();
    collector.entries.extend(collector.overscroll_area_parents);
    collector
        .top_or_view_transition_layers
        .sort_by_key(|entry| {
            entry
                .layer
                .fragment
                .as_deref()
                .expect("PrePaint node must own a fragment")
                .paint
                .LogicalTreeOrder()
        });
    collector
        .entries
        .extend(collector.top_or_view_transition_layers);
    trace.set("children", collector.entries.len() as f64);
    // Source: PaintLayerStackingNode z-order vectors and overlay-control map.
    // Retain these exact source-selected owners for input queries; do not sort
    // a second list in Browser or rebuild a fragment tree for hit testing.
    let mut canonical_controls = HashMap::<u64, Vec<*const ()>>::new();
    for child in &collector.entries {
        let Some(controls) = collector.controls_after.get(&(child.layer as *const _)) else {
            continue;
        };
        let client = child
            .layer
            .fragment
            .as_deref()
            .unwrap()
            .paint
            .PaintLayerClientId();
        // Fragmented instances retain one source PaintLayer client. Preserve
        // every selected control in source paint order when those aliases meet;
        // HashMap iteration or replacing one instance would lose controls.
        canonical_controls.entry(client).or_default().extend(
            controls
                .iter()
                .map(|entry| entry.layer as *const PaintTreeNode<'_> as *const ()),
        );
    }
    // The paint walk's form of the same lists: each entry's property
    // ancestors in one flattened buffer, and the overlay-control map keyed
    // by the target layer's resident node.
    let mut lists = ResidentPaintOrderLists::default();
    for entry in &collector.entries {
        let ancestors = lists.PushAncestors(
            entry
                .property_ancestors
                .iter()
                .map(|ancestor| NodePointer(ancestor)),
        );
        lists.PushEntry(ancestors);
    }
    for (target, scrollers) in &collector.controls_after {
        for scroller in scrollers {
            let ancestors = lists.PushAncestors(
                scroller
                    .property_ancestors
                    .iter()
                    .map(|ancestor| NodePointer(ancestor)),
            );
            lists.PushControlsAfter(*target as *const (), NodePointer(scroller.layer), ancestors);
        }
    }
    // Every entry was borrowed directly from root's resident child owners.
    // PrePaint drops the lists before exposing a changed topology.
    unsafe {
        root.PublishResidentLayerOrder(
            collector
                .entries
                .iter()
                .map(|entry| NodePointer(entry.layer))
                .collect(),
            canonical_controls,
            lists,
        );
    }
}

/// Source stacking lists are lazily updated when a never-drawn/cull-skipped
/// layer is queried. Use the owning collector, including all source ordering
/// and overlay-control rules, rather than another hit-test sorting algorithm.
pub fn EnsureResidentHitTestLayerOrder(root: &PaintTreeNode<'_>) {
    if !root.HasResidentLayerOrder() {
        RebuildZOrderLists(root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout::paint::FragmentPaintData;

    fn layer(node_id: u64) -> Box<PaintTreeNode<'static>> {
        let mut fragment = FragmentPaintData::default();
        fragment.node_id = node_id;
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.is_paint_layer = true;
        node.is_z_order_list_visible = true;
        Box::new(node)
    }

    #[test]
    fn normal_flow_is_between_negative_and_zero_level_stacked_layers() {
        let mut negative = PaintTreeNode::default();
        negative.is_stacked = true;
        negative.stacking_level = -1;
        let normal = PaintTreeNode::default();
        let mut positive = PaintTreeNode::default();
        positive.is_stacked = true;

        assert!(PaintOrderKey(&negative) < PaintOrderKey(&normal));
        assert!(PaintOrderKey(&normal) < PaintOrderKey(&positive));
    }

    #[test]
    fn overscroll_area_parents_follow_normal_layers_in_reverse_chain_order() {
        let mut root = PaintTreeNode::default();
        let first = layer(1);
        let mut second = layer(2);
        second
            .fragment
            .as_mut()
            .unwrap()
            .paint
            .is_overscroll_area_parent = true;
        second.is_stacked = true;
        let mut third = layer(3);
        third
            .fragment
            .as_mut()
            .unwrap()
            .paint
            .is_overscroll_area_parent = true;
        third.is_stacked = true;
        root.children.push(first);
        root.children.push(second);
        root.children.push(third);
        let order: Vec<_> = CollectPaintLayerChildren(&root)
            .Entries()
            .map(|entry| entry.layer.fragment.as_deref().unwrap().node_id)
            .collect();
        assert_eq!(order, [1, 3, 2]);
    }

    #[test]
    fn overlay_controls_are_inserted_after_highest_contained_descendant() {
        let mut root = PaintTreeNode::default();
        let mut scroller = layer(1);
        scroller.has_overlay_overflow_controls = true;
        scroller.can_contain_absolute_position_objects = true;
        let mut low = layer(2);
        low.is_stacked = true;
        low.stacking_level = 2;
        let mut high = layer(3);
        high.is_stacked = true;
        high.stacking_level = 5;
        scroller.children.push(low);
        scroller.children.push(high);
        root.children.push(scroller);

        let order = CollectPaintLayerChildren(&root);
        let target = order
            .Entries()
            .find(|entry| entry.layer.fragment.as_deref().unwrap().node_id == 3)
            .unwrap();
        assert_eq!(order.ControlsAfter(target.layer).len(), 1);
        assert_eq!(
            order.ControlsAfter(target.layer).next().unwrap()
                .layer
                .fragment
                .as_deref()
                .unwrap()
                .node_id,
            1
        );
        assert!(
            order.NeedsReorderOverlayOverflowControls(
                order.ControlsAfter(target.layer).next().unwrap().layer
            )
        );
    }

    #[test]
    fn uncontained_absolute_descendant_does_not_reparent_controls() {
        let mut root = PaintTreeNode::default();
        let mut scroller = layer(1);
        scroller.has_overlay_overflow_controls = true;
        // Rebuilding the stacking list must clear a stale bit from an earlier
        // tree state before deciding that this descendant is uncontained.
        scroller.needs_reorder_overlay_overflow_controls.set(true);
        let mut absolute = layer(2);
        absolute.is_stacked = true;
        absolute.stacking_level = 7;
        absolute.fragment.as_mut().unwrap().paint.position = Position::kAbsolute;
        scroller.children.push(absolute);
        root.children.push(scroller);

        let order = CollectPaintLayerChildren(&root);
        assert!(order
            .Entries()
            .all(|entry| order.ControlsAfter(entry.layer).len() == 0));
        let scroller = order
            .Entries()
            .find(|entry| entry.layer.fragment.as_deref().unwrap().node_id == 1)
            .unwrap();
        assert!(!order.NeedsReorderOverlayOverflowControls(scroller.layer));
    }

    #[test]
    fn one_target_retains_all_scrollers_in_collection_order() {
        let mut root = PaintTreeNode::default();
        let mut outer = layer(1);
        outer.has_overlay_overflow_controls = true;
        let mut inner = layer(2);
        inner.has_overlay_overflow_controls = true;
        let mut target = layer(3);
        target.is_stacked = true;
        target.stacking_level = 1;
        inner.children.push(target);
        outer.children.push(inner);
        root.children.push(outer);

        let order = CollectPaintLayerChildren(&root);
        let target = order
            .Entries()
            .find(|entry| entry.layer.fragment.as_deref().unwrap().node_id == 3)
            .unwrap();
        let scrollers: Vec<_> = order
            .ControlsAfter(target.layer)
            .map(|entry| entry.layer.fragment.as_deref().unwrap().node_id)
            .collect();
        assert_eq!(scrollers, [2, 1]);
    }

    #[test]
    fn flex_order_projection_stably_orders_matching_groups() {
        let mut first = PaintTreeNode::default();
        first.paint_layer_order_parent_unique_id = Some(9);
        first.paint_layer_order = 4;
        let mut second = PaintTreeNode::default();
        second.paint_layer_order_parent_unique_id = Some(9);
        second.paint_layer_order = -1;
        assert!(OrderLessThan(&second, &first));
        assert!(!OrderLessThan(&first, &second));
    }

    #[test]
    fn root_appends_top_layers_after_normal_and_ignores_them_in_nested_contexts() {
        let mut root = PaintTreeNode::default();
        root.is_root = true;
        let normal = layer(1);
        let mut top = layer(2);
        top.is_stacked = true;
        top.is_in_top_or_view_transition_layer = true;
        top.is_direct_layout_view_child = true;
        top.fragment.as_mut().unwrap().paint.logical_tree_order = 9;
        root.children.push(top);
        root.children.push(normal);
        let order: Vec<_> = CollectPaintLayerChildren(&root)
            .Entries()
            .map(|entry| entry.layer.fragment.as_deref().unwrap().node_id)
            .collect();
        assert_eq!(order, [1, 2]);

        root.is_root = false;
        root.DirtyZOrderLists();
        let order: Vec<_> = CollectPaintLayerChildren(&root)
            .Entries()
            .map(|entry| entry.layer.fragment.as_deref().unwrap().node_id)
            .collect();
        assert_eq!(order, [1]);
    }
}
