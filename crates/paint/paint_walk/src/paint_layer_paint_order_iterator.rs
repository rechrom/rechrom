#![allow(non_snake_case, non_upper_case_globals)]

//! PaintLayerPaintOrderIterator over Layout's PaintLayer and its
//! PaintLayerStackingNode, as PaintLayerPainter::PaintChildren reads them.
//!
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: third_party/blink/renderer/core/paint/paint_layer_paint_order_iterator.h:29-89
//! Source: third_party/blink/renderer/core/paint/paint_layer_paint_order_iterator.cc:39-80
//!
//! The z-order lists are the translated stacking node's, rebuilt by Layout's
//! PaintLayer::UpdateDescendantDependentFlags before PrePaint
//! (paint_layer.cc:864-882); normal-flow children are the PaintLayer's own
//! FirstChild()/NextSibling() links. Until the PrePaint node tree is deleted,
//! each listed PaintLayer is mapped back to its resident PrePaint
//! occurrence(s) through `ResidentLayerNodes`, which also carries the
//! non-layer property ancestors the flat replay stream re-enters.

use std::cmp::Ordering;
use std::collections::HashMap;

use layout::input::Position;
use layout::views::paint_layer::{PaintLayer, PaintLayers};
use prepaint::resident_hit_test::ResidentPaintOrderLists;

use crate::pre_paint_tree_walk::PaintTreeNode;

// Source: paint_layer_paint_order_iterator.h:29-37 (PaintLayerIteration).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PaintLayerIteration(u32);

impl PaintLayerIteration {
    pub(crate) const kNegativeZOrderChildren: Self = Self(1);
    pub(crate) const kNormalFlowChildren: Self = Self(1 << 1);
    pub(crate) const kPositiveZOrderChildren: Self = Self(1 << 2);
    pub(crate) const kNormalFlowAndPositiveZOrderChildren: Self = Self((1 << 1) | (1 << 2));
    pub(crate) const kAllChildren: Self = Self(1 | (1 << 1) | (1 << 2));
}

// Source: paint_layer_paint_order_iterator.h:48-89. No Clone/Copy, matching
// the deleted source copy operations. The debug-only
// PaintLayerListMutationDetector (paint_layer.h:877-894) is folded in: it
// saves the root's mutation-allowed state for the iterator's lifetime.
//
// Rechrom's PaintLayer derives FirstChild() from the LayoutObject subtree
// instead of a stored link, so the normal-flow cursor is seeded when the
// normal-flow pass starts (None) rather than at construction; the visited
// sequence is the source's.
pub(crate) struct PaintLayerPaintOrderIterator<'a> {
    root_: &'a PaintLayer,
    remaining_children_: u32,
    index_: usize,
    current_normal_flow_child_: Option<*mut PaintLayer>,
    #[cfg(debug_assertions)]
    previous_mutation_allowed_state_: bool,
}

impl<'a> PaintLayerPaintOrderIterator<'a> {
    // Source: paint_layer_paint_order_iterator.h:55-69.
    pub(crate) fn new(root: &'a PaintLayer, which_children: PaintLayerIteration) -> Self {
        Self {
            root_: root,
            remaining_children_: which_children.0,
            index_: 0,
            current_normal_flow_child_: None,
            #[cfg(debug_assertions)]
            previous_mutation_allowed_state_: root.SetLayerListMutationAllowed(false),
        }
    }

    // Source: paint_layer_paint_order_iterator.h:73-79.
    pub(crate) fn LayersPaintingOverlayOverflowControlsAfter(
        &self,
        layer: &PaintLayer,
    ) -> Option<&'a PaintLayers> {
        self.root_
            .StackingNode()?
            .LayersPaintingOverlayOverflowControlsAfter(layer as *const PaintLayer)
    }

    // Source: paint_layer_paint_order_iterator.cc:39-80.
    pub(crate) fn Next(&mut self) -> Option<&'a PaintLayer> {
        if self.remaining_children_ & PaintLayerIteration::kNegativeZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                let list = node.NegZOrderList();
                if self.index_ < list.len() {
                    let layer = list[self.index_].Get();
                    self.index_ += 1;
                    // The stacking node's Members name live PaintLayers of
                    // this tree; the lifecycle keeps them through Paint.
                    return Some(unsafe { &*layer });
                }
            }
            self.index_ = 0;
            self.remaining_children_ &= !PaintLayerIteration::kNegativeZOrderChildren.0;
        }

        if self.remaining_children_ & PaintLayerIteration::kNormalFlowChildren.0 != 0 {
            let root = self.root_;
            let mut current = *self
                .current_normal_flow_child_
                .get_or_insert_with(|| root.FirstChild());
            // PaintLayer links name live layers of this LayoutObject tree.
            while let Some(child) = unsafe { current.as_ref() } {
                current = child.NextSibling();
                self.current_normal_flow_child_ = Some(current);
                if child.GetLayoutObject().IsStacked() {
                    continue;
                }
                return Some(child);
            }
            // We reset the iterator in case we reuse it.
            self.current_normal_flow_child_ = None;
            self.remaining_children_ &= !PaintLayerIteration::kNormalFlowChildren.0;
        }

        if self.remaining_children_ & PaintLayerIteration::kPositiveZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                let list = node.PosZOrderList();
                if self.index_ < list.len() {
                    let layer = list[self.index_].Get();
                    self.index_ += 1;
                    return Some(unsafe { &*layer });
                }
            }
            self.index_ = 0;
            self.remaining_children_ &= !PaintLayerIteration::kPositiveZOrderChildren.0;
        }

        None
    }
}

#[cfg(debug_assertions)]
impl Drop for PaintLayerPaintOrderIterator<'_> {
    fn drop(&mut self) {
        self.root_
            .SetLayerListMutationAllowed(self.previous_mutation_allowed_state_);
    }
}

/// The source PaintLayer of a resident PrePaint node, read through its
/// borrowed fragment cursor (LayoutBoxModelObject::Layer()). None for
/// hand-built fixture nodes, which have no Layout owner.
pub(crate) fn SourcePaintLayer<'n>(node: &'n PaintTreeNode<'_>) -> Option<&'n PaintLayer> {
    node.SourceCursor()?.OwnPaintLayer()
}

/// One resident PrePaint occurrence of a child PaintLayer: the layer node and
/// the nodes between the painting layer and it (non-layer nodes, and for a
/// stacked descendant also its non-stacking-context ancestor layers), whose
/// property scopes the child painter re-enters in the flat replay stream.
#[derive(Clone, Copy)]
pub(crate) struct PaintLayerPaintOrderEntry<'a, 'n, 'f> {
    pub(crate) layer: &'n PaintTreeNode<'f>,
    pub(crate) property_ancestors: &'a [&'n PaintTreeNode<'f>],
}

struct Occurrence<'n, 'f> {
    node: &'n PaintTreeNode<'f>,
    ancestors: (u32, u32),
    next: u32,
}

const kNoOccurrence: u32 = u32::MAX;

/// PaintLayer -> resident PrePaint node map below one painting layer node.
///
/// Temporary bridge until the PrePaint node tree is removed: Chromium's
/// PaintLayerPainter(*child) paints the child's own LayoutObject fragments,
/// while Rechrom still paints the resident node PrePaint built for each
/// fragment. Built once per painted layer by one walk of the node subtree its
/// iterator can name (direct child layers; for a layer owning a
/// PaintLayerStackingNode, every layer up to nested stacking contexts, the
/// domain of PaintLayerStackingNode::CollectLayers); each lookup is one hash
/// probe. A fragmented PaintLayer keeps all its occurrences in node-tree
/// order, so they paint together as PaintLayerPainter::PaintWithPhase
/// iterates the layer's FragmentData (paint_layer_painter.cc:752-786).
pub(crate) struct ResidentLayerNodes<'n, 'f> {
    ancestors: Vec<&'n PaintTreeNode<'f>>,
    occurrences: Vec<Occurrence<'n, 'f>>,
    /// First and last occurrence index per PaintLayer.
    layers: HashMap<*const PaintLayer, (u32, u32)>,
}

impl<'n, 'f> ResidentLayerNodes<'n, 'f> {
    /// `through_layers`: also descend through non-stacking-context layer
    /// nodes, as PaintLayerStackingNode::CollectLayers does
    /// (paint_layer_stacking_node.cc:328-401). Only a layer that owns a
    /// stacking node lists such descendants.
    pub(crate) fn Collect(root: &'n PaintTreeNode<'f>, through_layers: bool) -> Self {
        let mut trace = browser_tracing::span("paint_walk", "PaintWalk.CollectLayerNodes");
        let mut nodes = Self {
            ancestors: Vec::new(),
            occurrences: Vec::new(),
            layers: HashMap::new(),
        };
        nodes.Visit(root, &mut Vec::new(), through_layers);
        trace.set("occurrences", nodes.occurrences.len() as f64);
        nodes
    }

    fn Visit(
        &mut self,
        parent: &'n PaintTreeNode<'f>,
        path: &mut Vec<&'n PaintTreeNode<'f>>,
        through_layers: bool,
    ) {
        for child_owner in &parent.children {
            let child: &'n PaintTreeNode<'f> = child_owner.as_ref();
            // A LocalFrameView owns a separate PaintLayer tree, painted by
            // FramePainter from the embedding object's foreground phase.
            if child.is_embedded_local_frame_root {
                continue;
            }
            if child.is_paint_layer {
                if let Some(layer) = SourcePaintLayer(child) {
                    self.Push(layer, child, path);
                }
                if !through_layers || child.is_stacking_context {
                    continue;
                }
            }
            path.push(child);
            self.Visit(child, path, through_layers);
            path.pop();
        }
    }

    fn Push(
        &mut self,
        layer: &PaintLayer,
        node: &'n PaintTreeNode<'f>,
        path: &[&'n PaintTreeNode<'f>],
    ) {
        let start = self.ancestors.len() as u32;
        self.ancestors.extend_from_slice(path);
        let index = self.occurrences.len() as u32;
        self.occurrences.push(Occurrence {
            node,
            ancestors: (start, self.ancestors.len() as u32),
            next: kNoOccurrence,
        });
        match self.layers.entry(layer as *const PaintLayer) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let tail = entry.get().1;
                self.occurrences[tail as usize].next = index;
                entry.get_mut().1 = index;
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert((index, index));
            }
        }
    }

    /// The resident occurrences of `layer`, in node-tree order.
    pub(crate) fn Occurrences<'s>(
        &'s self,
        layer: &PaintLayer,
    ) -> impl Iterator<Item = PaintLayerPaintOrderEntry<'s, 'n, 'f>> + 's {
        let mut index = self
            .layers
            .get(&(layer as *const PaintLayer))
            .map_or(kNoOccurrence, |&(head, _)| head);
        std::iter::from_fn(move || {
            if index == kNoOccurrence {
                return None;
            }
            let occurrence = &self.occurrences[index as usize];
            index = occurrence.next;
            let (start, end) = occurrence.ancestors;
            Some(PaintLayerPaintOrderEntry {
                layer: occurrence.node,
                property_ancestors: &self.ancestors[start as usize..end as usize],
            })
        })
    }
}

// ---------------------------------------------------------------------------
// Hit-test-only projection. Paint no longer reads these node-sorted lists; the
// resident hit-test cursor (prepaint::resident_hit_test) still borrows them
// through EnsureResidentHitTestLayerOrder until it iterates Layout's
// PaintLayerStackingNode, as interaction_hit_test's translated iterator does.
// ---------------------------------------------------------------------------

/// A collected entry while RebuildZOrderLists runs; flattened into the
/// retained ResidentPaintOrderLists when the rebuild publishes.
#[derive(Clone)]
struct CollectedEntry<'n, 'f> {
    layer: &'n PaintTreeNode<'f>,
    property_ancestors: Vec<&'n PaintTreeNode<'f>>,
}

fn NodePointer(node: &PaintTreeNode<'_>) -> *const () {
    node as *const PaintTreeNode<'_> as *const ()
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

    fn Id(pointer: *const ()) -> u64 {
        unsafe { &*pointer.cast::<PaintTreeNode<'static>>() }
            .fragment
            .as_deref()
            .unwrap()
            .node_id
    }

    /// The published hit-test lists of `root`, as node ids.
    fn Order(root: &PaintTreeNode<'static>) -> Vec<u64> {
        EnsureResidentHitTestLayerOrder(root);
        let view = root.ResidentPaintOrder().unwrap();
        view.Layers().iter().map(|&pointer| Id(pointer)).collect()
    }

    /// Node ids of the scrollers whose overlay controls follow `target`.
    fn ControlsAfter(root: &PaintTreeNode<'static>, target: u64) -> Vec<u64> {
        EnsureResidentHitTestLayerOrder(root);
        let view = root.ResidentPaintOrder().unwrap();
        let target = *view
            .Layers()
            .iter()
            .find(|&&pointer| Id(pointer) == target)
            .unwrap();
        view.ControlsAfter(target)
            .iter()
            .map(|&(scroller, _)| Id(scroller))
            .collect()
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
        assert_eq!(Order(&root), [1, 3, 2]);
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

        assert_eq!(ControlsAfter(&root, 3), [1]);
        assert!(root.children[0].needs_reorder_overlay_overflow_controls.get());
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

        for id in Order(&root) {
            assert!(ControlsAfter(&root, id).is_empty());
        }
        assert!(!root.children[0].needs_reorder_overlay_overflow_controls.get());
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

        assert_eq!(ControlsAfter(&root, 3), [2, 1]);
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
        assert_eq!(Order(&root), [1, 2]);

        root.is_root = false;
        root.DirtyZOrderLists();
        assert_eq!(Order(&root), [1]);
    }
}
