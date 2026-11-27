#![allow(non_snake_case)]

//! Chromium PaintLayerStackingNode: the z-order lists and the overlay
//! overflow-control reordering map cached on a stacking-context PaintLayer.
//!
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: third_party/blink/renderer/core/paint/paint_layer_stacking_node.h:57-192
//! Source: third_party/blink/renderer/core/paint/paint_layer_stacking_node.cc:57-446
//!
//! Like the source, the node relies on PaintLayer's tree structure
//! (paint_layer_stacking_node.h:76-78). Rechrom derives that structure from
//! the authoritative LayoutObject tree (PaintLayer::FirstChild/NextSibling),
//! so the lists name the live PaintLayers through traced `Member`s; no second
//! layer tree or snapshot is retained. Paint (PaintLayerPainter through
//! PaintLayerPaintOrderIterator) and hit testing read these same lists.

use foundation::{DynamicTo, EPosition, HeapHashMap, HeapVector, Member, Traceable, Visitor};
use layout_style::style::computed_style::ComputedStyle;

use super::super::layout_box_model_object::LayoutBoxModelObject;
use super::super::layout_object::LayoutObject;
use super::layout_fragment_paint_layer::PaintLayer;

// Source: paint_layer_stacking_node.h:105-106.
pub type PaintLayers = HeapVector<Member<PaintLayer>>;

// Source: paint_layer_stacking_node.h:90-192.
pub struct PaintLayerStackingNode {
    // Source: paint_layer_stacking_node.h:138-142. Holds a sorted list of all
    // the descendant nodes within that have z-indices of 0 (or is treated as
    // 0 for positioned objects) or greater, and the negative ones.
    pos_z_order_list_: PaintLayers,
    neg_z_order_list_: PaintLayers,

    // Source: paint_layer_stacking_node.h:144-186. The PaintLayers (values)
    // whose overlay overflow controls paint after the key PaintLayer. Chromium
    // stores each value behind Member<GCedPaintLayers> only so the HeapHashMap
    // can hold it; the vector is owned in place here with the same tracing.
    layer_to_overlay_overflow_controls_painting_after_:
        HeapHashMap<Member<PaintLayer>, PaintLayers>,

    // Source: paint_layer_stacking_node.h:188.
    layer_: Member<PaintLayer>,

    // Source: paint_layer_stacking_node.h:190-191.
    z_order_lists_dirty_: bool,
}

// Source: paint_layer_stacking_node.cc:85-90.
fn ZIndexLessThan(first: *const PaintLayer, second: *const PaintLayer) -> bool {
    let first = unsafe { &*first }.GetLayoutObject();
    let second = unsafe { &*second }.GetLayoutObject();
    debug_assert!(first.IsStacked());
    debug_assert!(second.IsStacked());
    first.StyleRef().EffectiveZIndex() < second.StyleRef().EffectiveZIndex()
}

// Source: paint_layer_stacking_node.cc:92-103.
fn SetIfHigher(first: &mut *const PaintLayer, second: *const PaintLayer) -> bool {
    if second.is_null() {
        return false;
    }
    debug_assert!(
        unsafe { &*second }
            .GetLayoutObject()
            .StyleRef()
            .EffectiveZIndex()
            >= 0
    );
    // |second| appears later in the tree, so it's higher than |first| if its
    // z-index >= |first|'s z-index.
    if first.is_null() || !ZIndexLessThan(second, *first) {
        *first = second;
        return true;
    }
    false
}

// Source: paint_layer_stacking_node.cc:110-115.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
enum LayerType {
    kAbsolutePosition,
    kFixedPosition,
    kInFlowStacked,
}

const kLayerTypeCount: usize = 3;

// Source: paint_layer_stacking_node.cc:105-185. For finding the proper
// z-order of reparented overlay overflow controls. The order vector has the
// source's inline capacity of kLayerTypeCount and never holds duplicates.
struct HighestLayers {
    highest_layers: [*const PaintLayer; kLayerTypeCount],
    highest_layers_order: [LayerType; kLayerTypeCount],
    highest_layers_order_len: usize,
}

impl HighestLayers {
    fn new() -> Self {
        Self {
            highest_layers: [std::ptr::null(); kLayerTypeCount],
            highest_layers_order: [LayerType::kAbsolutePosition; kLayerTypeCount],
            highest_layers_order_len: 0,
        }
    }

    fn Order(&self) -> &[LayerType] {
        &self.highest_layers_order[..self.highest_layers_order_len]
    }

    // Source: paint_layer_stacking_node.cc:120-135.
    fn UpdateOrderForSubtreeHighestLayers(
        &mut self,
        layer_type: LayerType,
        layer: *const PaintLayer,
    ) {
        if SetIfHigher(&mut self.highest_layers[layer_type as usize], layer) {
            let len = self.highest_layers_order_len;
            if let Some(position) = self.highest_layers_order[..len]
                .iter()
                .position(|existing| *existing == layer_type)
            {
                // |highest_layers_order| doesn't have duplicate elements, so
                // std::remove moves the later entries forward by one and the
                // freed last slot receives |type| (cc:123-130).
                self.highest_layers_order
                    .copy_within(position + 1..len, position);
                self.highest_layers_order[len - 1] = layer_type;
            } else {
                self.highest_layers_order[len] = layer_type;
                self.highest_layers_order_len += 1;
            }
        }
    }

    // Source: paint_layer_stacking_node.cc:137-145.
    fn GetLayerType(layer: &PaintLayer) -> LayerType {
        debug_assert!(layer.GetLayoutObject().IsStacked());
        let style = layer.GetLayoutObject().StyleRef();
        if style.GetPosition() == EPosition::kAbsolute {
            return LayerType::kAbsolutePosition;
        }
        if style.GetPosition() == EPosition::kFixed {
            return LayerType::kFixedPosition;
        }
        LayerType::kInFlowStacked
    }

    // Source: paint_layer_stacking_node.cc:147-164.
    fn Update(&mut self, layer: &PaintLayer) {
        let style = layer.GetLayoutObject().StyleRef();
        // We only need to consider zero or positive z-index stacked child for
        // candidates of causing reparent of overlay scrollbars of ancestors.
        // A negative z-index child will not cause reparent of overlay
        // scrollbars because the ancestor scroller either has auto z-index
        // which is above the child or has negative z-index which is a
        // stacking context.
        if !layer.GetLayoutObject().IsStacked() || style.EffectiveZIndex() < 0 {
            return;
        }
        // We should not consider layers that have been omitted from z-order
        // lists.
        if !layer.IsZOrderListVisible() {
            return;
        }
        self.UpdateOrderForSubtreeHighestLayers(Self::GetLayerType(layer), layer);
    }

    // Source: paint_layer_stacking_node.cc:166-184.
    fn Merge(&mut self, child: &HighestLayers, current_layer: &PaintLayer) {
        let object = current_layer.GetLayoutObject();
        for &layer_type in child.Order() {
            let mut layer_type_for_propagation = layer_type;
            if object.IsStacked()
                && ((layer_type == LayerType::kAbsolutePosition
                    && object.CanContainAbsolutePositionObjects())
                    || (layer_type == LayerType::kFixedPosition
                        && object.CanContainFixedPositionObjects())
                    || layer_type == LayerType::kInFlowStacked)
            {
                // If the child is contained by the current layer, then use the
                // current layer's type for propagation to ancestors.
                layer_type_for_propagation = Self::GetLayerType(current_layer);
            }
            self.UpdateOrderForSubtreeHighestLayers(
                layer_type_for_propagation,
                child.highest_layers[layer_type as usize],
            );
        }
    }
}

// Source: paint_layer_stacking_node.cc:187-205.
fn ChildOfFlexboxOrGridOrGridLanesParentOrGrandparent(
    layer: *const PaintLayer,
) -> *const LayoutObject {
    let object =
        unsafe { &*layer }.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject;
    let parent = unsafe { &*object }.Parent();
    if parent.is_null() {
        return std::ptr::null();
    }
    let parent_ref = unsafe { &*parent };
    if parent_ref.IsFlexibleBox() || parent_ref.IsLayoutGridOrGridLanes() {
        return object;
    }
    let grandparent = parent_ref.Parent();
    if grandparent.is_null() {
        return std::ptr::null();
    }
    let grandparent = unsafe { &*grandparent };
    if grandparent.IsFlexibleBox() || grandparent.IsLayoutGridOrGridLanes() {
        return parent;
    }
    std::ptr::null()
}

// Source: paint_layer_stacking_node.cc:207-237.
fn OrderLessThan(first: *const PaintLayer, second: *const PaintLayer) -> bool {
    // TODO(chrishtr): make this work for arbitrary ancestors, not just parent
    // and grandparent.
    let first_ancestor = ChildOfFlexboxOrGridOrGridLanesParentOrGrandparent(first);
    let second_ancestor = ChildOfFlexboxOrGridOrGridLanesParentOrGrandparent(second);
    if first_ancestor.is_null() || second_ancestor.is_null() {
        return false;
    }
    let first_ancestor_ref = unsafe { &*first_ancestor };
    let second_ancestor_ref = unsafe { &*second_ancestor };
    if first_ancestor_ref.Parent() != second_ancestor_ref.Parent() {
        return false;
    }

    let first_object =
        unsafe { &*first }.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject;
    let second_object =
        unsafe { &*second }.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject;
    let mut first_order = 0;
    let mut second_order = 0;
    // Out of flow flexbox direct children paint as if order was 0:
    // https://drafts.csswg.org/css-display-4/#order-modified-document-order
    if first_ancestor != first_object || !first_ancestor_ref.IsOutOfFlowPositioned() {
        first_order = first_ancestor_ref.StyleRef().Order();
    }
    if second_ancestor != second_object || !second_ancestor_ref.IsOutOfFlowPositioned() {
        second_order = second_ancestor_ref.StyleRef().Order();
    }
    first_order < second_order
}

// Source: paint_layer_stacking_node.cc:239-254. Returns true if the children
// of `layer` may need ordering by `OrderLessThan`.
fn ChildrenMayBeAffectedByOrder(layer: &PaintLayer) -> bool {
    let root = layer.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject;
    let mut child = layer.FirstChild();
    if child.is_null() || unsafe { &*child }.NextSiblingWithin(root).is_null() {
        // Not enough children to need re-ordering.
        return false;
    }
    while !child.is_null() {
        let ancestor = ChildOfFlexboxOrGridOrGridLanesParentOrGrandparent(child);
        // This is the only case where `OrderLessThan` can return true.
        if !ancestor.is_null() && unsafe { &*ancestor }.StyleRef().Order() != 0 {
            return true;
        }
        child = unsafe { &*child }.NextSiblingWithin(root);
    }
    false
}

/// std::stable_sort over a less-than predicate (paint_layer_stacking_node.cc:
/// 276-277). OrderLessThan is not a total order across different flex/grid
/// parents, so a std comparator-based sort (which may reject inconsistent
/// orders) is not used; this merge keeps equal (incomparable) elements in
/// their original order, as std::stable_sort does.
fn StableSortByLess(
    items: &mut [*mut PaintLayer],
    less: fn(*const PaintLayer, *const PaintLayer) -> bool,
) {
    let len = items.len();
    if len < 2 {
        return;
    }
    if len <= 16 {
        for index in 1..len {
            let value = items[index];
            let mut hole = index;
            while hole > 0 && less(value, items[hole - 1]) {
                items[hole] = items[hole - 1];
                hole -= 1;
            }
            items[hole] = value;
        }
        return;
    }
    let middle = len / 2;
    StableSortByLess(&mut items[..middle], less);
    StableSortByLess(&mut items[middle..], less);
    let mut merged = Vec::with_capacity(len);
    let (mut left, mut right) = (0, middle);
    while left < middle && right < len {
        // Take from the right run only when it is strictly less, keeping the
        // left (earlier) element first on ties.
        if less(items[right], items[left]) {
            merged.push(items[right]);
            right += 1;
        } else {
            merged.push(items[left]);
            left += 1;
        }
    }
    merged.extend_from_slice(&items[left..middle]);
    merged.extend_from_slice(&items[right..len]);
    items.copy_from_slice(&merged);
}

// Source: paint_layer_stacking_node.cc:256-281. Calls `function` for the
// children of `layer`, sorted by the order CSS property if they are the child
// of a flexbox. See: https://www.w3.org/TR/css-flexbox-1/#painting
fn ForAllChildrenSortedByOrder(layer: *mut PaintLayer, function: &mut dyn FnMut(*mut PaintLayer)) {
    let layer_ref = unsafe { &*layer };
    let root = layer_ref.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject;
    // Optimization: `order` is relatively rare and we can avoid needing to
    // create and sort the vector of children in most cases.
    if !ChildrenMayBeAffectedByOrder(layer_ref) {
        let mut child = layer_ref.FirstChild();
        while !child.is_null() {
            let next = unsafe { &*child }.NextSiblingWithin(root);
            function(child);
            child = next;
        }
        return;
    }

    let mut sorted_children = Vec::new();
    let mut child = layer_ref.FirstChild();
    while !child.is_null() {
        sorted_children.push(child);
        child = unsafe { &*child }.NextSiblingWithin(root);
    }
    StableSortByLess(&mut sorted_children, OrderLessThan);
    for child in sorted_children {
        function(child);
    }
}

impl PaintLayerStackingNode {
    // Source: paint_layer_stacking_node.cc:59-65. FIXME: This should not
    // require PaintLayer. There is currently a cycle where in order to
    // determine if we isStacked() we have to ask the paint layer about some of
    // its state.
    pub(crate) fn new(layer: *mut PaintLayer) -> Self {
        debug_assert!(unsafe { &*layer }.GetLayoutObject().IsStackingContext());
        Self {
            pos_z_order_list_: PaintLayers::new(),
            neg_z_order_list_: PaintLayers::new(),
            layer_to_overlay_overflow_controls_painting_after_: HeapHashMap::default(),
            layer_: Member::from_ptr(layer),
            z_order_lists_dirty_: true,
        }
    }

    // Source: paint_layer_stacking_node.cc:67-83.
    pub(crate) fn DirtyZOrderLists(&mut self) {
        #[cfg(debug_assertions)]
        debug_assert!(unsafe { &*self.layer_.Get() }.LayerListMutationAllowed());

        self.pos_z_order_list_.clear();
        self.neg_z_order_list_.clear();

        for entry in self
            .layer_to_overlay_overflow_controls_painting_after_
            .values()
        {
            for layer in entry.iter() {
                let layer = unsafe { &mut *layer.Get() };
                // Chromium's PaintLayer::RemoveChild dirties these lists
                // before a layer leaves the tree (paint_layer.cc:774-775).
                // Rechrom derives layer links from LayoutObjects and a layer
                // can be destroyed after its LayoutObject was detached, so a
                // traced but destroyed entry is skipped instead of being
                // treated as live.
                if layer.IsDestroyed() {
                    continue;
                }
                layer.SetNeedsReorderOverlayOverflowControls(false);
            }
        }
        self.layer_to_overlay_overflow_controls_painting_after_
            .clear();

        self.z_order_lists_dirty_ = true;
    }

    // Source: paint_layer_stacking_node.cc:283-326.
    fn RebuildZOrderLists(&mut self) {
        #[cfg(debug_assertions)]
        debug_assert!(unsafe { &*self.layer_.Get() }.LayerListMutationAllowed());
        debug_assert!(self.z_order_lists_dirty_);

        let layer = self.layer_.Get();
        unsafe { &mut *layer }.SetNeedsReorderOverlayOverflowControls(false);

        let mut overscroll_area_parents = PaintLayers::new();
        ForAllChildrenSortedByOrder(layer, &mut |child| {
            self.CollectLayers(child, None, &mut overscroll_area_parents);
        });

        // Sort the two lists. Effective z-index is a total order, so the std
        // stable sort is exactly std::stable_sort(ZIndexLessThan) (cc:297-301).
        let z_index = |layer: &Member<PaintLayer>| {
            debug_assert!(unsafe { &*layer.Get() }.GetLayoutObject().IsStacked());
            unsafe { &*layer.Get() }
                .GetLayoutObject()
                .StyleRef()
                .EffectiveZIndex()
        };
        self.pos_z_order_list_.sort_by_key(z_index);
        self.neg_z_order_list_.sort_by_key(z_index);

        // Append overscroll area parent layers after all normal layers in this
        // stacking context. Note that these are appended in reverse order to
        // match the overscroll chaining order (last one is one overscrolls
        // first and is on the bottom of a stack).
        for overscroll_area_parent in overscroll_area_parents.iter().rev() {
            self.pos_z_order_list_.push(*overscroll_area_parent);
        }

        // Append layers for top layer elements after normal layer collection,
        // to ensure they are on top regardless of z-indexes. The layoutObjects
        // of top layer elements are children of the view, sorted in top layer
        // stacking order.
        let layer_ref = unsafe { &*layer };
        if layer_ref.IsRootLayer() {
            let root_block = layer_ref.GetLayoutObject().View();
            debug_assert!(!root_block.is_null());
            let mut child = unsafe { &*root_block.cast::<LayoutObject>() }.SlowFirstChild();
            while !child.is_null() {
                let child_ref = unsafe { &*child };
                if child_ref.IsInTopOrViewTransitionLayer() && child_ref.IsStacked() {
                    let box_model = DynamicTo::<LayoutBoxModelObject>(child);
                    assert!(!box_model.is_null());
                    self.pos_z_order_list_
                        .push(Member::from_ptr(unsafe { &*box_model }.Layer()));
                }
                child = child_ref.NextSibling();
            }
        }

        self.z_order_lists_dirty_ = false;
    }

    // Source: paint_layer_stacking_node.cc:328-401.
    fn CollectLayers(
        &mut self,
        paint_layer: *mut PaintLayer,
        mut highest_layers: Option<&mut HighestLayers>,
        overscroll_area_parents: &mut PaintLayers,
    ) {
        unsafe { &mut *paint_layer }.SetNeedsReorderOverlayOverflowControls(false);

        let paint_layer_ref = unsafe { &*paint_layer };
        if paint_layer_ref.IsInTopOrViewTransitionLayer() {
            return;
        }

        let object = paint_layer_ref.GetLayoutObject();
        if object.IsOverscrollAreaParent() {
            assert!(object.IsStacked());
            overscroll_area_parents.push(Member::from_ptr(paint_layer));
            return;
        }

        if let Some(highest_layers) = highest_layers.as_deref_mut() {
            highest_layers.Update(paint_layer_ref);
        }

        let style = object.StyleRef();

        if object.IsStacked() && paint_layer_ref.IsZOrderListVisible() {
            let list = if style.EffectiveZIndex() >= 0 {
                &mut self.pos_z_order_list_
            } else {
                &mut self.neg_z_order_list_
            };
            list.push(Member::from_ptr(paint_layer));
        }

        if object.IsStackingContext() {
            return;
        }

        let scrollable_area = object.GetScrollableArea();
        let has_overlay_overflow_controls =
            !scrollable_area.is_null() && unsafe { &*scrollable_area }.HasOverlayOverflowControls();
        let mut subtree_highest_layers =
            if has_overlay_overflow_controls || highest_layers.is_some() {
                Some(HighestLayers::new())
            } else {
                None
            };

        ForAllChildrenSortedByOrder(paint_layer, &mut |child| {
            self.CollectLayers(
                child,
                subtree_highest_layers.as_mut(),
                overscroll_area_parents,
            );
        });

        if has_overlay_overflow_controls {
            let subtree = subtree_highest_layers
                .as_ref()
                .expect("overlay overflow controls collect subtree highest layers");
            let mut layer_to_paint_overlay_overflow_controls_after: *const PaintLayer =
                std::ptr::null();
            for &layer_type in subtree.Order() {
                if layer_type == LayerType::kFixedPosition
                    && !object.CanContainFixedPositionObjects()
                {
                    continue;
                }
                if layer_type == LayerType::kAbsolutePosition
                    && !object.CanContainAbsolutePositionObjects()
                {
                    continue;
                }
                SetIfHigher(
                    &mut layer_to_paint_overlay_overflow_controls_after,
                    subtree.highest_layers[layer_type as usize],
                );
            }

            if !layer_to_paint_overlay_overflow_controls_after.is_null() {
                self.layer_to_overlay_overflow_controls_painting_after_
                    .entry(Member::from_ptr(
                        layer_to_paint_overlay_overflow_controls_after as *mut PaintLayer,
                    ))
                    .or_default()
                    .push(Member::from_ptr(paint_layer));
            }
            unsafe { &mut *paint_layer }.SetNeedsReorderOverlayOverflowControls(
                !layer_to_paint_overlay_overflow_controls_after.is_null(),
            );
        }

        if let Some(highest_layers) = highest_layers {
            highest_layers.Merge(
                subtree_highest_layers
                    .as_ref()
                    .expect("an ancestor collector requests subtree highest layers"),
                unsafe { &*paint_layer },
            );
        }
    }

    // Source: paint_layer_stacking_node.cc:403-434. Returns whether a relevant
    // style changed.
    pub(crate) fn StyleDidChange(
        paint_layer: &mut PaintLayer,
        old_style: *const ComputedStyle,
    ) -> bool {
        let mut was_stacking_context = false;
        let mut was_stacked = false;
        let mut old_z_index = 0;
        let mut old_order = 0;
        let object = paint_layer.GetLayoutObject();
        if let Some(old_style) = unsafe { old_style.as_ref() } {
            was_stacking_context = object.IsStackingContextWithStyle(old_style);
            old_z_index = old_style.EffectiveZIndex();
            old_order = old_style.Order();
            was_stacked = object.IsStackedWithStyle(old_style);
        }

        let new_style = object.StyleRef();

        let should_be_stacking_context = object.IsStackingContext();
        let should_be_stacked = object.IsStacked();
        if should_be_stacking_context == was_stacking_context
            && was_stacked == should_be_stacked
            && old_z_index == new_style.EffectiveZIndex()
            && old_order == new_style.Order()
        {
            return false;
        }

        paint_layer.DirtyStackingContextZOrderLists();

        let node = paint_layer.StackingNodePointer();
        if !node.is_null() {
            unsafe { &mut *node }.DirtyZOrderLists();
        }
        true
    }

    // Source: paint_layer_stacking_node.cc:436-439.
    pub(crate) fn UpdateZOrderLists(&mut self) {
        if self.z_order_lists_dirty_ {
            self.RebuildZOrderLists();
        }
    }

    // Source: paint_layer_stacking_node.h:108-111.
    pub fn PosZOrderList(&self) -> &PaintLayers {
        debug_assert!(!self.z_order_lists_dirty_);
        &self.pos_z_order_list_
    }

    // Source: paint_layer_stacking_node.h:112-115.
    pub fn NegZOrderList(&self) -> &PaintLayers {
        debug_assert!(!self.z_order_lists_dirty_);
        &self.neg_z_order_list_
    }

    // Source: paint_layer_stacking_node.h:117-124.
    pub fn LayersPaintingOverlayOverflowControlsAfter(
        &self,
        layer: *const PaintLayer,
    ) -> Option<&PaintLayers> {
        debug_assert!(!self.z_order_lists_dirty_);
        self.layer_to_overlay_overflow_controls_painting_after_
            .get(&Member::from_ptr(layer as *mut PaintLayer))
    }
}

// Source: paint_layer_stacking_node.cc:441-446.
impl Traceable for PaintLayerStackingNode {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.layer_);
        visitor.Trace(&self.pos_z_order_list_);
        visitor.Trace(&self.neg_z_order_list_);
        visitor.Trace(&self.layer_to_overlay_overflow_controls_painting_after_);
    }
}
