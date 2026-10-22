// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/property_tree.h:1-60,64-70,149,161,167-170;
//         cc/trees/property_tree.cc:1-43,63-72,154-158.
// Rust modules/imports replace includes, namespaces and export guards. The
// source non-owning owner pointer becomes a typed owner identity resolved only
// through explicit live sibling contexts, with no self-reference or snapshot.
use super::clip_node::ClipNode;
use super::effect_node::EffectNode;
use super::property_ids::{
    kContentsRootPropertyNodeId, kInvalidPropertyNodeId, kRootPropertyNodeId,
};
use super::scroll_node::ScrollNode;
use super::transform_node::TransformNode;
use crate::adapter::input_types::ElementId;
use crate::adapter::property_tree::PropertyTreesOwnerId;
use crate::adapter::property_types::{
    FilterOperations, PropertyPath, PropertyTrace, PropertyTransform, TracedValueJSON,
};
use std::collections::{BTreeMap, BTreeSet};

// Rust's generic node field projection replaces C++ template member lookup.
// These accessors operate on the canonical resident node types, not projections.
pub trait PropertyNode: Default + Clone + PartialEq {
    fn id(&self) -> i32;
    fn set_id(&mut self, id: i32);
    fn parent_id(&self) -> i32;
    fn set_parent_id(&mut self, id: i32);
    fn AsValueInto(&self, value: &mut impl PropertyTrace);
}
macro_rules! property_node_fields {
    () => {
        fn id(&self) -> i32 {
            self.id
        }
        fn set_id(&mut self, id: i32) {
            self.id = id;
        }
        fn parent_id(&self) -> i32 {
            self.parent_id
        }
        fn set_parent_id(&mut self, id: i32) {
            self.parent_id = id;
        }
    };
}
impl<T: PropertyTransform> PropertyNode for TransformNode<T> {
    property_node_fields!();
    fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        TransformNode::AsValueInto(self, value);
    }
}
impl PropertyNode for ClipNode {
    property_node_fields!();
    fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        ClipNode::AsValueInto(self, value);
    }
}
impl<F: FilterOperations, P: PropertyPath> PropertyNode for EffectNode<F, P> {
    property_node_fields!();
    fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        EffectNode::AsValueInto(self, value);
    }
}
impl PropertyNode for ScrollNode {
    property_node_fields!();
    fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        ScrollNode::AsValueInto(self, value);
    }
}

// Source: cc/trees/property_tree.h:66-75,150-159,162-169.
// C++ deletes copy construction while permitting assignment; Rust similarly
// exposes explicit AssignFrom without deriving Clone for this owner.
pub struct PropertyTree<T: PropertyNode> {
    nodes_: Vec<T>,
    element_id_to_node_index_: BTreeMap<ElementId, i32>,
    needs_update_: bool,
    property_trees_: Option<PropertyTreesOwnerId>,
}
impl<T: PropertyNode> PropertyTree<T> {
    // Source: cc/trees/property_tree.h:150; cc/trees/property_tree.cc:55-62.
    pub fn new(property_trees: Option<PropertyTreesOwnerId>) -> Self {
        let mut result = Self {
            nodes_: vec![T::default()],
            element_id_to_node_index_: BTreeMap::new(),
            needs_update_: false,
            property_trees_: property_trees,
        };
        result.MutableBack().unwrap().set_id(kRootPropertyNodeId);
        result
            .MutableBack()
            .unwrap()
            .set_parent_id(kInvalidPropertyNodeId);
        result
    }
    // Source: cc/trees/property_tree.h:75; cc/trees/property_tree.cc:70-72.
    pub fn AssignFrom(&mut self, other: &Self) {
        self.nodes_.clone_from(&other.nodes_);
        self.element_id_to_node_index_
            .clone_from(&other.element_id_to_node_index_);
        self.needs_update_ = other.needs_update_;
        self.property_trees_ = other.property_trees_;
    }
    // Source: cc/trees/property_tree.h:82; cc/trees/property_tree.cc:91-100.
    pub fn Insert(&mut self, tree_node: &T, parent_id: i32) -> i32 {
        debug_assert!(!self.nodes_.is_empty());
        self.nodes_.push(tree_node.clone());
        let node_id = self.nodes_.len() as i32 - 1;
        let node = self.nodes_.last_mut().unwrap();
        node.set_parent_id(parent_id);
        node.set_id(node_id);
        node_id
    }
    // Source: cc/trees/property_tree.h:84-85; cc/trees/property_tree.cc:101-111.
    pub fn RemoveNodes(&mut self, n: usize) {
        assert!(n <= self.nodes_.len());
        self.nodes_.truncate(self.nodes_.len() - n);
        let upper_bound = i32::try_from(self.nodes_.len()).expect("checked node count");
        self.element_id_to_node_index_
            .retain(|_, index| *index < upper_bound);
    }
    // Source: cc/trees/property_tree.h:87-91.
    pub fn Node(&self, i: i32) -> &T {
        assert!(i < self.nodes_.len() as i32);
        assert!(i > kInvalidPropertyNodeId);
        &self.nodes_[i as usize]
    }
    // Source: cc/trees/property_tree.h:93-97.
    pub fn MutableNode(&mut self, i: i32) -> &mut T {
        assert!(i < self.nodes_.len() as i32);
        assert!(i > kInvalidPropertyNodeId);
        &mut self.nodes_[i as usize]
    }
    // Source: cc/trees/property_tree.h:99-101.
    pub fn HasParent(&self, t: &T) -> bool {
        t.parent_id() != kInvalidPropertyNodeId
    }
    // Source: cc/trees/property_tree.h:103.
    pub fn parent(&self, t: &T) -> &T {
        self.Node(t.parent_id())
    }
    // Source: cc/trees/property_tree.h:105.
    pub fn MutableParent(&mut self, t: &T) -> &mut T {
        self.MutableNode(t.parent_id())
    }
    // Source: cc/trees/property_tree.h:107.
    pub fn MutableBack(&mut self) -> Option<&mut T> {
        self.nodes_.last_mut()
    }
    // Source: cc/trees/property_tree.h:108.
    pub fn back(&self) -> Option<&T> {
        self.nodes_.last()
    }
    // Source: cc/trees/property_tree.h:110-112.
    pub fn SetElementIdForNodeId(&mut self, node_id: i32, element_id: ElementId) {
        self.element_id_to_node_index_.insert(element_id, node_id);
    }
    // Source: cc/trees/property_tree.h:113-120.
    pub fn MutableFindNodeFromElementId(&mut self, id: ElementId) -> Option<&mut T> {
        let index = *self.element_id_to_node_index_.get(&id)?;
        if index == kInvalidPropertyNodeId {
            return None;
        }
        Some(self.MutableNode(index))
    }
    // Source: cc/trees/property_tree.h:121-128.
    pub fn FindNodeFromElementId(&self, id: ElementId) -> Option<&T> {
        let index = *self.element_id_to_node_index_.get(&id)?;
        if index == kInvalidPropertyNodeId {
            return None;
        }
        Some(self.Node(index))
    }
    // Source: cc/trees/property_tree.h:130; cc/trees/property_tree.cc:112-126.
    pub fn clear(&mut self) {
        self.needs_update_ = false;
        self.nodes_.clear();
        self.nodes_.push(T::default());
        self.MutableBack().unwrap().set_id(kRootPropertyNodeId);
        self.MutableBack()
            .unwrap()
            .set_parent_id(kInvalidPropertyNodeId);
        self.element_id_to_node_index_.clear();
        #[cfg(debug_assertions)]
        {
            let tree = Self::new(None);
            debug_assert!(tree == *self);
        }
    }
    // Source: cc/trees/property_tree.h:131.
    pub fn size(&self) -> usize {
        self.nodes_.len()
    }
    // Source: cc/trees/property_tree.h:133-134.
    pub fn set_needs_update(&mut self, value: bool) {
        self.needs_update_ = value;
    }
    pub fn needs_update(&self) -> bool {
        self.needs_update_
    }
    // Source: cc/trees/property_tree.h:136-137.
    pub fn nodes_mut(&mut self) -> &mut Vec<T> {
        &mut self.nodes_
    }
    pub fn nodes(&self) -> &Vec<T> {
        &self.nodes_
    }
    // Source: cc/trees/property_tree.h:139.
    pub fn next_available_id(&self) -> i32 {
        self.size() as i32
    }
    // Source: cc/trees/property_tree.h:141.
    pub fn property_trees(&self) -> Option<PropertyTreesOwnerId> {
        self.property_trees_
    }
    // Source: cc/trees/property_tree.h:143; cc/trees/property_tree.cc:142-153.
    pub fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        value.BeginArray("nodes");
        for node in &self.nodes_ {
            value.BeginDictionaryUnnamed();
            node.AsValueInto(value);
            value.EndDictionary();
        }
        value.EndArray();
        value.SetBoolean("needs_update", self.needs_update_);
    }
    // Source: cc/trees/property_tree.h:145-147.
    pub fn element_id_to_node_index(&self) -> &BTreeMap<ElementId, i32> {
        &self.element_id_to_node_index_
    }
    // Source: cc/trees/property_tree.h:162-164.
    fn SetPropertyTrees(&mut self, property_trees: Option<PropertyTreesOwnerId>) {
        self.property_trees_ = property_trees;
    }
    // Source: cc/trees/property_tree.h:77-80; cc/trees/property_tree.cc:134-140.
    #[cfg(debug_assertions)]
    pub fn ToString(&self) -> String {
        let mut value = TracedValueJSON::default();
        self.AsValueInto(&mut value);
        value.ToFormattedJSON()
    }
}
// Source: cc/trees/property_tree.h:77-80; cc/trees/property_tree.cc:127-133.
#[cfg(debug_assertions)]
impl<T: PropertyNode> PartialEq for PropertyTree<T> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes() == other.nodes()
            && self.needs_update() == other.needs_update()
            && self.element_id_to_node_index() == other.element_id_to_node_index()
    }
}

use super::transform_node::TransformCachedNodeData;
use crate::adapter::geometry::{PointF, Rect, RectF, Size, SizeF, Vector2d, Vector2dF};
use crate::adapter::property_tree::PropertyTreeTransform;
use std::cell::RefCell;
// Source: cc/trees/mutator_host.h:43.
const kInvalidScale: f32 = 0.0;
use super::damage_reason::DamageReason;
use super::sticky_position_constraint::StickyPositionConstraint;
use super::viewport_property_ids::ViewportPropertyIds;
use crate::cc::base::math_util::MathUtil;
use crate::cc::base::synced_property::{AdditionGroup, SyncedProperty};
// Source: cc/trees/property_tree.h:61-62.
pub type SyncedScrollOffset = SyncedProperty<AdditionGroup<PointF, Vector2dF>>;

// Source: cc/trees/property_tree.h:414-427; cc/trees/property_tree.cc:44-51.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnchorPositionScrollData {
    pub adjustment_container_ids: Vec<ElementId>,
    pub accumulated_scroll_origin: Vector2d,
    pub needs_scroll_adjustment_in_x: bool,
    pub needs_scroll_adjustment_in_y: bool,
}
// Source: cc/trees/property_tree.h:429-454; cc/trees/property_tree.cc:52-54.
#[derive(Clone, Debug, PartialEq)]
pub struct StickyPositionNodeData {
    pub x_scroll_ancestor: i32,
    pub y_scroll_ancestor: i32,
    pub constraints: StickyPositionConstraint,
    pub nearest_node_shifting_sticky_box: i32,
    pub nearest_node_shifting_containing_block: i32,
    pub total_sticky_box_sticky_offset: Vector2dF,
    pub total_containing_block_sticky_offset: Vector2dF,
}
impl Default for StickyPositionNodeData {
    fn default() -> Self {
        Self {
            x_scroll_ancestor: kInvalidPropertyNodeId,
            y_scroll_ancestor: kInvalidPropertyNodeId,
            constraints: StickyPositionConstraint::default(),
            nearest_node_shifting_sticky_box: kInvalidPropertyNodeId,
            nearest_node_shifting_containing_block: kInvalidPropertyNodeId,
            total_sticky_box_sticky_offset: Vector2dF::default(),
            total_containing_block_sticky_offset: Vector2dF::default(),
        }
    }
}
// Source: cc/trees/property_tree.h:211-222; cc/trees/property_tree.cc:271-273.
#[derive(Default)]
pub struct UpdateTransformsData {
    pub stale_forward_dependencies: BTreeSet<i32>,
}

// Source: cc/trees/property_tree.h:175-189,394-412.
pub struct TransformTree<T: PropertyTreeTransform> {
    base_: PropertyTree<TransformNode<T>>,
    page_scale_factor_: f32,
    device_scale_factor_: f32,
    device_transform_scale_factor_: f32,
    external_page_scale_factor_: f32,
    nodes_affected_by_outer_viewport_bounds_delta_: Vec<i32>,
    nodes_affected_by_safe_area_inset_bottom_: Vec<i32>,
    cached_data_: Vec<TransformCachedNodeData<T>>,
    sticky_position_data_: Vec<StickyPositionNodeData>,
    anchor_position_scroll_data_: Vec<AnchorPositionScrollData>,
    drawn_elastic_overscroll_: BTreeMap<ElementId, Vector2dF>,
}
impl<T: PropertyTreeTransform> std::ops::Deref for TransformTree<T> {
    type Target = PropertyTree<TransformNode<T>>;
    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}
impl<T: PropertyTreeTransform> std::ops::DerefMut for TransformTree<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

// Source: cc/trees/property_tree.h:1007-1031.
// Separate owner fields permit disjoint live borrows of the sibling trees.
// These are the source owner fields, not a duplicated state projection.
pub struct PropertyTreesState {
    needs_rebuild_: bool,
    changed_: bool,
    full_tree_damaged_: bool,
    is_main_thread_: bool,
    is_active_: bool,
    sequence_number_: i32,
    inner_viewport_container_bounds_delta_: Vector2dF,
    outer_viewport_container_bounds_delta_: Vector2dF,
    transform_delta_by_safe_area_inset_bottom_: f32,
}
impl PropertyTreesState {
    pub fn is_main_thread(&self) -> bool {
        self.is_main_thread_
    }
    pub fn is_active(&self) -> bool {
        self.is_active_
    }
    pub fn set_changed(&mut self, value: bool) {
        self.changed_ = value;
    }
    pub fn inner_viewport_container_bounds_delta(&self) -> Vector2dF {
        self.inner_viewport_container_bounds_delta_
    }
}
impl Default for PropertyTreesState {
    fn default() -> Self {
        Self {
            needs_rebuild_: true,
            changed_: false,
            full_tree_damaged_: false,
            is_main_thread_: true,
            is_active_: false,
            sequence_number_: 0,
            inner_viewport_container_bounds_delta_: Vector2dF::default(),
            outer_viewport_container_bounds_delta_: Vector2dF::default(),
            transform_delta_by_safe_area_inset_bottom_: 0.0,
        }
    }
}
// Source: cc/trees/property_tree.h:798-818.
pub const kInvalidUpdateNumber: i32 = -1;
#[derive(Clone, Debug)]
pub struct AnimationScaleData {
    pub update_number: i32,
    pub maximum_to_screen_scale: f32,
    pub affected_by_animation_scale: bool,
    pub affected_by_invalid_scale: bool,
}
impl Default for AnimationScaleData {
    fn default() -> Self {
        Self {
            update_number: kInvalidUpdateNumber,
            maximum_to_screen_scale: 0.0,
            affected_by_animation_scale: false,
            affected_by_invalid_scale: false,
        }
    }
}
// Source: cc/trees/property_tree.h:820-843.
#[derive(Clone, Debug)]
pub struct DrawTransforms<T: PropertyTreeTransform> {
    pub might_be_invertible: bool,
    pub from_valid: bool,
    pub to_valid: bool,
    pub from_target: T,
    pub to_target: T,
}
impl<T: PropertyTreeTransform> DrawTransforms<T> {
    pub fn new(from: T, to: T) -> Self {
        Self {
            might_be_invertible: true,
            from_valid: false,
            to_valid: false,
            from_target: from,
            to_target: to,
        }
    }
}
impl<T: PropertyTreeTransform> PartialEq for DrawTransforms<T> {
    fn eq(&self, other: &Self) -> bool {
        self.from_valid == other.from_valid
            && self.to_valid == other.to_valid
            && self.from_target == other.from_target
            && self.to_target == other.to_target
    }
}
// Source: cc/trees/property_tree.h:845-852.
#[derive(Clone, Debug)]
pub struct DrawTransformData<T: PropertyTreeTransform> {
    pub update_number: i32,
    pub effect_id: i32,
    pub transforms: DrawTransforms<T>,
}
impl<T: PropertyTreeTransform> Default for DrawTransformData<T> {
    fn default() -> Self {
        Self {
            update_number: kInvalidUpdateNumber,
            effect_id: kInvalidPropertyNodeId,
            transforms: DrawTransforms::new(T::default(), T::default()),
        }
    }
}
// Source: cc/trees/property_tree.h:854-861.
pub struct PropertyTreesCachedData<T: PropertyTreeTransform> {
    transform_tree_update_number: i32,
    animation_scales: Vec<AnimationScaleData>,
    draw_transforms: RefCell<Vec<Vec<DrawTransformData<T>>>>,
}

/// Borrow-only context replacing the source tree's owner backpointer. Every
/// field is the resident sibling tree/owner data; no snapshots are generated.
pub struct TransformTreeContext<'a, T: PropertyTreeTransform> {
    pub owner: PropertyTreesOwnerId,
    pub scroll_tree: &'a ScrollTree,
    pub clip_tree: &'a mut ClipTree,
    pub state: &'a mut PropertyTreesState,
    pub cached_data: &'a mut PropertyTreesCachedData<T>,
}

/// Live sibling borrows for scroll mutations. Reborrowed in source order,
/// including synchronous owner callbacks; no queues, copied state or snapshots.
pub struct ScrollTreeMutationContext<'a, T: PropertyTreeTransform> {
    pub owner: PropertyTreesOwnerId,
    pub transform_tree: &'a mut TransformTree<T>,
    pub clip_tree: &'a mut ClipTree,
    pub state: &'a mut PropertyTreesState,
    pub cached_data: &'a mut PropertyTreesCachedData<T>,
}
impl<T: PropertyTreeTransform> TransformTree<T> {
    // Source: cc/trees/property_tree.h:177; cc/trees/property_tree.cc:79-90.
    pub fn new(property_trees: Option<PropertyTreesOwnerId>) -> Self {
        Self {
            base_: PropertyTree::new(property_trees),
            page_scale_factor_: 1.0,
            device_scale_factor_: 1.0,
            device_transform_scale_factor_: 1.0,
            external_page_scale_factor_: 1.0,
            nodes_affected_by_outer_viewport_bounds_delta_: Vec::new(),
            nodes_affected_by_safe_area_inset_bottom_: Vec::new(),
            cached_data_: vec![TransformCachedNodeData::default()],
            sticky_position_data_: Vec::new(),
            anchor_position_scroll_data_: Vec::new(),
            drawn_elastic_overscroll_: BTreeMap::new(),
        }
    }
    // Source: cc/trees/property_tree.h:185; cc/trees/property_tree.cc:88-90.
    pub fn AssignFrom(&mut self, other: &Self) {
        self.base_.AssignFrom(&other.base_);
        self.page_scale_factor_ = other.page_scale_factor_;
        self.device_scale_factor_ = other.device_scale_factor_;
        self.device_transform_scale_factor_ = other.device_transform_scale_factor_;
        self.external_page_scale_factor_ = other.external_page_scale_factor_;
        self.nodes_affected_by_outer_viewport_bounds_delta_
            .clone_from(&other.nodes_affected_by_outer_viewport_bounds_delta_);
        self.nodes_affected_by_safe_area_inset_bottom_
            .clone_from(&other.nodes_affected_by_safe_area_inset_bottom_);
        self.cached_data_.clone_from(&other.cached_data_);
        self.sticky_position_data_
            .clone_from(&other.sticky_position_data_);
        self.anchor_position_scroll_data_
            .clone_from(&other.anchor_position_scroll_data_);
        self.drawn_elastic_overscroll_
            .clone_from(&other.drawn_elastic_overscroll_);
    }
    // Source: cc/trees/property_tree.h:191; cc/trees/property_tree.cc:159-166.
    pub fn Insert(&mut self, tree_node: &TransformNode<T>, parent_id: i32) -> i32 {
        let node_id = self.base_.Insert(tree_node, parent_id);
        debug_assert!(node_id == self.cached_data_.len() as i32);
        self.cached_data_.push(TransformCachedNodeData::default());
        node_id
    }
    // Source: cc/trees/property_tree.h:192; cc/trees/property_tree.cc:167-171.
    pub fn RemoveNodes(&mut self, n: usize) {
        self.base_.RemoveNodes(n);
        self.cached_data_.truncate(self.cached_data_.len() - n);
    }
    // Source: cc/trees/property_tree.h:194; cc/trees/property_tree.cc:172-191.
    pub fn clear(&mut self) {
        self.base_.clear();
        self.page_scale_factor_ = 1.0;
        self.device_scale_factor_ = 1.0;
        self.device_transform_scale_factor_ = 1.0;
        self.external_page_scale_factor_ = 1.0;
        self.nodes_affected_by_outer_viewport_bounds_delta_.clear();
        self.nodes_affected_by_safe_area_inset_bottom_.clear();
        self.cached_data_.clear();
        self.cached_data_.push(TransformCachedNodeData::default());
        self.sticky_position_data_.clear();
        self.anchor_position_scroll_data_.clear();
        self.drawn_elastic_overscroll_.clear();
        #[cfg(debug_assertions)]
        {
            debug_assert!(Self::new(None) == *self);
        }
    }
    // Source: cc/trees/property_tree.h:238; cc/trees/property_tree.cc:192-197.
    pub fn set_needs_update(
        &mut self,
        needs_update: bool,
        context: &mut TransformTreeContext<'_, T>,
    ) {
        if needs_update && !self.base_.needs_update() {
            context.cached_data.UpdateTransformTreeUpdateNumber();
        }
        self.base_.set_needs_update(needs_update);
    }
    // Source: cc/trees/property_tree.h:196-197; cc/trees/property_tree.cc:198-215.
    pub fn OnTransformAnimated(
        &mut self,
        element_id: ElementId,
        transform: &T,
        context: &mut TransformTreeContext<'_, T>,
    ) -> bool {
        let Some(node) = self.MutableFindNodeFromElementId(element_id) else {
            return false;
        };
        if node.local == *transform {
            return false;
        }
        node.local = transform.clone();
        node.needs_local_transform_update = true;
        node.SetTransformChanged(DamageReason::kUntracked);
        context.state.changed_ = true;
        self.set_needs_update(true, context);
        true
    }
    // Source: cc/trees/property_tree.h:198; cc/trees/property_tree.cc:216-223.
    pub fn ResetChangeTracking(&mut self) {
        for id in kContentsRootPropertyNodeId..self.size() as i32 {
            self.MutableNode(id).ClearTransformChanged();
        }
    }

    // Source: cc/trees/property_tree.h:200-202; cc/trees/property_tree.cc:224-253.
    pub fn UpdateAllTransforms(
        &mut self,
        viewport_property_ids: &ViewportPropertyIds,
        context: &mut TransformTreeContext<'_, T>,
    ) {
        if !self.needs_update() {
            #[cfg(debug_assertions)]
            {
                for i in kContentsRootPropertyNodeId..self.size() as i32 {
                    debug_assert!(!self.Node(i).needs_local_transform_update);
                }
            }
            return;
        }
        let mut update_data = UpdateTransformsData::default();
        loop {
            let last_num_stale_forward_dependencies = update_data.stale_forward_dependencies.len();
            for i in kContentsRootPropertyNodeId..self.size() as i32 {
                self.UpdateTransforms(
                    i,
                    Some(viewport_property_ids),
                    Some(&mut update_data),
                    context,
                );
            }
            assert!(
                last_num_stale_forward_dependencies == 0
                    || update_data.stale_forward_dependencies.len()
                        < last_num_stale_forward_dependencies
            );
            if update_data.stale_forward_dependencies.is_empty() {
                break;
            }
        }
        self.set_needs_update(false, context);
    }
    // Source: cc/trees/property_tree.h:204-207; cc/trees/property_tree.cc:254-270.
    pub fn CopyFromPreservingNodes(&mut self, other: &Self) {
        self.base_.set_needs_update(other.needs_update());
        self.base_
            .element_id_to_node_index_
            .clone_from(&other.base_.element_id_to_node_index_);
        self.page_scale_factor_ = other.page_scale_factor_;
        self.device_scale_factor_ = other.device_scale_factor_;
        self.device_transform_scale_factor_ = other.device_transform_scale_factor_;
        self.external_page_scale_factor_ = other.external_page_scale_factor_;
        self.nodes_affected_by_outer_viewport_bounds_delta_
            .clone_from(&other.nodes_affected_by_outer_viewport_bounds_delta_);
        self.nodes_affected_by_safe_area_inset_bottom_
            .clone_from(&other.nodes_affected_by_safe_area_inset_bottom_);
        self.sticky_position_data_
            .clone_from(&other.sticky_position_data_);
        self.anchor_position_scroll_data_
            .clone_from(&other.anchor_position_scroll_data_);
        self.drawn_elastic_overscroll_
            .clone_from(&other.drawn_elastic_overscroll_);
    }
    // Source: cc/trees/property_tree.h:224-228; cc/trees/property_tree.cc:274-309.
    pub fn UpdateTransforms(
        &mut self,
        id: i32,
        viewport_property_ids: Option<&ViewportPropertyIds>,
        mut update_data: Option<&mut UpdateTransformsData>,
        context: &mut TransformTreeContext<'_, T>,
    ) {
        let old_to_parent = self.Node(id).to_parent.clone();
        let old_snap_amount = self.Node(id).snap_amount;
        let node = self.Node(id);
        if node.sticky_position_constraint_id >= 0
            || node.anchor_position_scroll_data_id >= 0
            || node.needs_local_transform_update
            || node.should_undo_overscroll
        {
            self.UpdateLocalTransform(
                id,
                viewport_property_ids,
                update_data.as_deref_mut(),
                context,
            );
        } else {
            self.UndoSnapping(id);
        }
        self.UpdateScreenSpaceTransform(id);
        self.UpdateAnimationProperties(id);
        self.UpdateSnapping(id);
        self.UpdateTransformChanged(id);
        self.UpdateNodeAndAncestorsAreAnimatedOrInvertible(id);
        self.UpdateNodeOrAncestorsWillChangeTransform(id);
        if let Some(data) = update_data {
            if self.Node(id).to_parent == old_to_parent
                && self.Node(id).snap_amount == old_snap_amount
            {
                data.stale_forward_dependencies.remove(&id);
            }
        }
        debug_assert!(!self.Node(id).needs_local_transform_update);
    }
    // Source: cc/trees/property_tree.h:374-376; cc/trees/property_tree.cc:310-318.
    fn IsDescendant(&self, mut desc_id: i32, source_id: i32) -> bool {
        while desc_id != source_id {
            if desc_id == kInvalidPropertyNodeId {
                return false;
            }
            desc_id = self.Node(desc_id).parent_id;
        }
        true
    }
    // Source: cc/trees/property_tree.h:342-346; cc/trees/property_tree.cc:319-396.
    pub fn CombineTransformsBetween(&self, source_id: i32, dest_id: i32, transform: &mut T) {
        debug_assert!(source_id > dest_id);
        let current = self.Node(source_id);
        let dest = self.Node(dest_id);
        if dest.ancestors_are_invertible && dest.node_and_ancestors_are_flat {
            transform.PostConcat(self.ToScreen(current.id));
            transform.PostConcat(self.FromScreen(dest.id));
            return;
        }
        let mut source_to_destination = vec![current.id];
        let mut current_node = if self.HasParent(current) {
            Some(self.parent(current))
        } else {
            None
        };
        while let Some(node) = current_node.filter(|node| node.id > dest_id) {
            source_to_destination.push(node.id);
            current_node = if self.HasParent(node) {
                Some(self.parent(node))
            } else {
                None
            };
        }
        let mut combined_transform = T::default();
        let ancestor = current_node.expect("source/destination common ancestor");
        if ancestor.id < dest_id {
            debug_assert!(self.IsDescendant(dest_id, ancestor.id));
            self.CombineInversesBetween(ancestor.id, dest_id, &mut combined_transform);
        }
        let source_to_destination_size = source_to_destination.len();
        for i in 0..source_to_destination_size {
            let index = source_to_destination_size - 1 - i;
            let node = self.Node(source_to_destination[index]);
            if node.flattens_inherited_transform {
                combined_transform.Flatten();
            }
            combined_transform.PreConcat(&node.to_parent);
        }
        transform.PostConcat(&combined_transform);
    }
    // Source: cc/trees/property_tree.h:348-353; cc/trees/property_tree.cc:397-424.
    pub fn CombineInversesBetween(&self, source_id: i32, dest_id: i32, transform: &mut T) -> bool {
        debug_assert!(source_id < dest_id);
        let current = self.Node(dest_id);
        let dest = self.Node(source_id);
        if current.ancestors_are_invertible && current.node_and_ancestors_are_flat {
            transform.PreConcat(self.FromScreen(current.id));
            transform.PreConcat(self.ToScreen(dest.id));
            return true;
        }
        let mut dest_to_source = T::default();
        self.CombineTransformsBetween(dest_id, source_id, &mut dest_to_source);
        let mut source_to_dest = T::default();
        let all_are_invertible = dest_to_source.GetInverse(&mut source_to_dest);
        transform.PreConcat(&source_to_dest);
        all_are_invertible
    }
    // Source: cc/trees/property_tree.h:363-364; cc/trees/property_tree.cc:425-436.
    pub fn SetDrawnElasticOverscroll(
        &mut self,
        id: ElementId,
        elastic_overscroll: &Vector2dF,
    ) -> bool {
        if elastic_overscroll.IsZero() {
            return self.drawn_elastic_overscroll_.remove(&id).is_some();
        }
        let current_overscroll = self.drawn_elastic_overscroll_.entry(id).or_default();
        let changed = *current_overscroll != *elastic_overscroll;
        *current_overscroll = *elastic_overscroll;
        changed
    }
    // Source: cc/trees/property_tree.h:366; cc/trees/property_tree.cc:437-444.
    pub fn GetDrawnElasticOverscroll(&self, id: ElementId) -> Vector2dF {
        self.drawn_elastic_overscroll_
            .get(&id)
            .copied()
            .unwrap_or_default()
    }
    // Source: cc/trees/property_tree.h:368-371; cc/trees/property_tree.cc:445-479.
    pub fn FindDrawnElasticOverscrollFromTransformId(
        &self,
        transform_id: i32,
        viewport_property_ids: Option<&ViewportPropertyIds>,
        context: &TransformTreeContext<'_, T>,
    ) -> (ElementId, Vector2dF) {
        let scroll_tree = context.scroll_tree;
        if viewport_property_ids
            .is_some_and(|ids| transform_id == ids.overscroll_elasticity_transform)
        {
            let ids = viewport_property_ids.unwrap();
            if ids.inner_scroll != kInvalidPropertyNodeId {
                let scroll_node = scroll_tree.Node(ids.inner_scroll);
                if let Some(overscroll) =
                    self.drawn_elastic_overscroll_.get(&scroll_node.element_id)
                {
                    return (scroll_node.element_id, *overscroll);
                }
            }
        } else {
            for (element_id, stretch_amount) in &self.drawn_elastic_overscroll_ {
                if let Some(scroll_node) = scroll_tree.FindNodeFromElementId(*element_id) {
                    if scroll_node.transform_id == transform_id {
                        return (*element_id, *stretch_amount);
                    }
                }
            }
        }
        (ElementId::default(), Vector2dF::default())
    }

    // Source: cc/trees/property_tree.h:379; cc/trees/property_tree.cc:480-613.
    fn StickyPositionOffset(
        &mut self,
        node_id: i32,
        context: &TransformTreeContext<'_, T>,
    ) -> Vector2dF {
        let Some(sticky_data) = self.GetStickyPositionData(node_id) else {
            return Vector2dF::default();
        };
        let constraint = &sticky_data.constraints;
        let scroll_node_x = if sticky_data.x_scroll_ancestor != kInvalidPropertyNodeId {
            Some(context.scroll_tree.Node(sticky_data.x_scroll_ancestor))
        } else {
            None
        };
        let scroll_node_y = if sticky_data.y_scroll_ancestor != kInvalidPropertyNodeId {
            Some(context.scroll_tree.Node(sticky_data.y_scroll_ancestor))
        } else {
            None
        };
        let transform_node_x = scroll_node_x
            .filter(|node| node.transform_id != kInvalidPropertyNodeId)
            .map(|node| self.Node(node.transform_id));
        let transform_node_y = scroll_node_y
            .filter(|node| node.transform_id != kInvalidPropertyNodeId)
            .map(|node| self.Node(node.transform_id));
        debug_assert!(transform_node_x.is_some() || transform_node_y.is_some());
        let mut scroll_position = PointF::default();
        let mut snap_offset = Vector2dF::default();
        if let Some(node) = transform_node_x {
            scroll_position.set_x(node.scroll_offset().x());
            snap_offset.set_x(node.snap_amount.x());
        }
        if let Some(node) = transform_node_y {
            scroll_position.set_y(node.scroll_offset().y());
            snap_offset.set_y(node.snap_amount.y());
        }
        scroll_position = scroll_position - snap_offset;
        let mut constraint_box_expansion = Vector2dF::default();
        if let Some(node) = scroll_node_x {
            debug_assert!(!node.scrolls_inner_viewport);
            if node.scrolls_outer_viewport {
                constraint_box_expansion
                    .set_x(context.state.outer_viewport_container_bounds_delta_.x());
            }
        }
        if let Some(node) = scroll_node_y {
            debug_assert!(!node.scrolls_inner_viewport);
            if node.scrolls_outer_viewport {
                constraint_box_expansion
                    .set_y(context.state.outer_viewport_container_bounds_delta_.y());
            }
        }
        let get_ancestor = |transform_node_id: i32| -> Option<&StickyPositionNodeData> {
            if transform_node_id == kInvalidPropertyNodeId {
                return None;
            }
            if transform_node_id >= self.size() as i32 {
                return None;
            }
            let ancestor_data = self.GetStickyPositionData(transform_node_id);
            debug_assert!(ancestor_data.is_some());
            ancestor_data
        };
        let mut ancestor_sticky_box_offset = Vector2dF::default();
        if let Some(ancestor) = get_ancestor(sticky_data.nearest_node_shifting_sticky_box) {
            if ancestor.x_scroll_ancestor == sticky_data.x_scroll_ancestor {
                ancestor_sticky_box_offset.set_x(ancestor.total_sticky_box_sticky_offset.x());
            }
            if ancestor.y_scroll_ancestor == sticky_data.y_scroll_ancestor {
                ancestor_sticky_box_offset.set_y(ancestor.total_sticky_box_sticky_offset.y());
            }
        }
        let mut ancestor_containing_block_offset = Vector2dF::default();
        if let Some(ancestor) = get_ancestor(sticky_data.nearest_node_shifting_containing_block) {
            if ancestor.x_scroll_ancestor == sticky_data.x_scroll_ancestor {
                ancestor_containing_block_offset
                    .set_x(ancestor.total_containing_block_sticky_offset.x());
            }
            if ancestor.y_scroll_ancestor == sticky_data.y_scroll_ancestor {
                ancestor_containing_block_offset
                    .set_y(ancestor.total_containing_block_sticky_offset.y());
            }
        }
        let sticky_offset = constraint.StickyPositionOffset(
            scroll_position,
            constraint_box_expansion,
            ancestor_sticky_box_offset,
            ancestor_containing_block_offset,
        );
        // Read the independent immutable constraint field before the output
        // fields are mutably borrowed; no node/constraint snapshot is created.
        let pixel_snap_offset = constraint.pixel_snap_offset;
        let sticky_data = self.MutableStickyPositionData(node_id).unwrap();
        sticky_data.total_sticky_box_sticky_offset = ancestor_sticky_box_offset + sticky_offset;
        sticky_data.total_containing_block_sticky_offset =
            ancestor_sticky_box_offset + ancestor_containing_block_offset + sticky_offset;
        crate::adapter::property_tree::RoundedVector2dF(sticky_offset + pixel_snap_offset)
    }
    // Source: cc/trees/property_tree.h:340; cc/trees/property_tree.cc:614-625.
    pub fn EnsureAnchorPositionScrollData(
        &mut self,
        node_id: i32,
    ) -> &mut AnchorPositionScrollData {
        let index = self.Node(node_id).anchor_position_scroll_data_id;
        if index == -1 || index as usize >= self.anchor_position_scroll_data_.len() {
            let new_index = self.anchor_position_scroll_data_.len() as i32;
            self.MutableNode(node_id).anchor_position_scroll_data_id = new_index;
            self.anchor_position_scroll_data_
                .push(AnchorPositionScrollData::default());
        }
        {
            let index = self.Node(node_id).anchor_position_scroll_data_id as usize;
            &mut self.anchor_position_scroll_data_[index]
        }
    }
    // Source: cc/trees/property_tree.h:338-339; cc/trees/property_tree.cc:626-636.
    pub fn GetAnchorPositionScrollData(&self, node_id: i32) -> Option<&AnchorPositionScrollData> {
        let node = self.Node(node_id);
        if node.anchor_position_scroll_data_id == -1
            || node.anchor_position_scroll_data_id as usize
                >= self.anchor_position_scroll_data_.len()
        {
            return None;
        }
        Some(&self.anchor_position_scroll_data_[node.anchor_position_scroll_data_id as usize])
    }
    // Source: cc/trees/property_tree.cc:657-667.
    // Source lambda expressed as a member helper so its immutable borrow ends
    // before the loop recursively mutates resident sticky/anchor offsets.
    fn GetTransformedAnchorOffset(
        &self,
        offset: Vector2dF,
        container_transform_id: i32,
        parent_id: i32,
    ) -> Vector2dF {
        if offset.IsZero() {
            return offset;
        }
        let mut mapper = self.ToScreen(container_transform_id).clone();
        mapper.PostConcat(self.FromScreen(parent_id));
        let transformed_offset = mapper.MapPoint(PointF::new(offset.x(), offset.y()));
        transformed_offset - mapper.MapPoint(PointF::default())
    }
    // Source: cc/trees/property_tree.h:380-383; cc/trees/property_tree.cc:637-713.
    fn AnchorPositionOffset(
        &mut self,
        node_id: i32,
        max_updated_node_id: i32,
        mut update_data: Option<&mut UpdateTransformsData>,
        visited: &mut BTreeSet<i32>,
        context: &TransformTreeContext<'_, T>,
    ) -> Vector2dF {
        if visited.contains(&node_id) {
            return Vector2dF::default();
        }
        visited.insert(node_id);
        let Some(data) = self.GetAnchorPositionScrollData(node_id) else {
            return Vector2dF::default();
        };
        assert!(update_data.is_some());
        let count = data.adjustment_container_ids.len();
        let parent_id = self.Node(node_id).parent_id;
        let mut accumulated_offset = Vector2dF::default();
        // Each id is read directly from canonical data; do not clone the vector
        // to work around the recursive borrow.
        for index in 0..count {
            let container_id = self
                .GetAnchorPositionScrollData(node_id)
                .unwrap()
                .adjustment_container_ids[index];
            let mut container_transform_id = kInvalidPropertyNodeId;
            if let Some(scroll_node) = context.scroll_tree.FindNodeFromElementId(container_id) {
                container_transform_id = scroll_node.transform_id;
                let transform_node = self.Node(container_transform_id);
                debug_assert!(container_transform_id != kInvalidPropertyNodeId);
                accumulated_offset = accumulated_offset
                    + self.GetTransformedAnchorOffset(
                        transform_node.scroll_offset().OffsetFromOrigin(),
                        transform_node.parent_id,
                        parent_id,
                    );
            } else if let Some(container_transform) = self.FindNodeFromElementId(container_id) {
                container_transform_id = container_transform.id;
                let mut adjustment = self.StickyPositionOffset(container_transform_id, context);
                adjustment = adjustment
                    + self.AnchorPositionOffset(
                        container_transform_id,
                        max_updated_node_id,
                        update_data.as_deref_mut(),
                        visited,
                        context,
                    );
                accumulated_offset = accumulated_offset
                    - self.GetTransformedAnchorOffset(
                        adjustment,
                        container_transform_id,
                        parent_id,
                    );
            }
            if container_transform_id > max_updated_node_id {
                update_data
                    .as_deref_mut()
                    .unwrap()
                    .stale_forward_dependencies
                    .insert(container_transform_id);
            }
        }
        let data = self.GetAnchorPositionScrollData(node_id).unwrap();
        let mut result = Vector2dF::new(
            data.accumulated_scroll_origin.x as f32,
            data.accumulated_scroll_origin.y as f32,
        ) - accumulated_offset;
        if !data.needs_scroll_adjustment_in_x {
            result.set_x(0.0);
        }
        if !data.needs_scroll_adjustment_in_y {
            result.set_y(0.0);
        }
        result
    }

    // Source: cc/trees/property_tree.h:329-331; cc/trees/property_tree.cc:714-754.
    pub fn UndoOverscroll(
        &mut self,
        node_id: i32,
        position_adjustment: &mut Vector2dF,
        viewport_property_ids: Option<&ViewportPropertyIds>,
        context: &mut TransformTreeContext<'_, T>,
    ) {
        debug_assert!(self.Node(node_id).should_undo_overscroll);
        let transform_id = viewport_property_ids.map_or(kInvalidPropertyNodeId, |ids| {
            ids.overscroll_elasticity_transform
        });
        if transform_id == kInvalidPropertyNodeId {
            return;
        }
        let clip_id = viewport_property_ids.map_or(kInvalidPropertyNodeId, |ids| ids.outer_clip);
        if clip_id == kInvalidPropertyNodeId {
            return;
        }
        let overscroll_offset = self
            .FindDrawnElasticOverscrollFromTransformId(transform_id, viewport_property_ids, context)
            .1;
        if overscroll_offset.IsZero() {
            return;
        }
        *position_adjustment = *position_adjustment
            + crate::cc::base::math_util::MathUtil::ScaleVectorByInverse(
                overscroll_offset,
                self.page_scale_factor(),
            );
        let clip_node = context.clip_tree.MutableNode(clip_id);
        let mut outsets = crate::adapter::property_tree::OutsetsF::default();
        if position_adjustment.x() < 0.0 {
            outsets.left = -position_adjustment.x();
        } else {
            outsets.right = position_adjustment.x();
        }
        if position_adjustment.y() < 0.0 {
            outsets.top = -position_adjustment.y();
        } else {
            outsets.bottom = position_adjustment.y();
        }
        clip_node.clip.Outset(outsets);
        context.clip_tree.set_needs_update(true);
    }
    // Source: cc/trees/property_tree.cc:755-800.
    fn ApplyElasticOverscrollStretch(
        scroll_tree: &ScrollTree,
        page_scale_factor: f32,
        elastic_overscroll: &(ElementId, Vector2dF),
        transform: &mut T,
        state: &PropertyTreesState,
    ) {
        let Some(scroll_node) = scroll_tree.FindNodeFromElementId(elastic_overscroll.0) else {
            return;
        };
        if scroll_tree
            .container_bounds(scroll_node.id, state)
            .IsEmpty()
            || elastic_overscroll.1.IsZero()
        {
            return;
        }
        let scroller_size = scroll_tree.container_bounds(scroll_node.id, state);
        let scale_factor = Vector2dF::new(
            1.0 + elastic_overscroll.1.x().abs() / scroller_size.width() as f32,
            1.0 + elastic_overscroll.1.y().abs() / scroller_size.height() as f32,
        );
        let mut pivot = PointF::default();
        if elastic_overscroll.1.x() > 0.0 {
            pivot.set_x(scroller_size.width() as f32);
        }
        if elastic_overscroll.1.y() > 0.0 {
            pivot.set_y(scroller_size.height() as f32);
        }
        if scroll_node.scrolls_inner_viewport {
            pivot =
                crate::cc::base::math_util::MathUtil::ScalePointByInverse(pivot, page_scale_factor);
        }
        transform.Translate(pivot.x(), pivot.y());
        transform.Scale(scale_factor.x(), scale_factor.y());
        transform.Translate(-pivot.x(), -pivot.y());
    }
    // Source: cc/trees/property_tree.cc:801-810.
    fn ApplyElasticOverscrollTranslate(
        elastic_overscroll: &(ElementId, Vector2dF),
        transform: &mut T,
    ) {
        transform.Translate(-elastic_overscroll.1.x(), -elastic_overscroll.1.y());
    }
    // Source: cc/trees/property_tree.h:384-386; cc/trees/property_tree.cc:811-875.
    fn UpdateLocalTransform(
        &mut self,
        node_id: i32,
        viewport_property_ids: Option<&ViewportPropertyIds>,
        mut update_data: Option<&mut UpdateTransformsData>,
        context: &mut TransformTreeContext<'_, T>,
    ) {
        let mut transform = T::default();
        let node = self.Node(node_id);
        transform.Translate3d(
            node.post_translation.x() + node.origin.x(),
            node.post_translation.y() + node.origin.y(),
            node.origin.z(),
        );
        let mut y_adjustment = 0.0;
        if node.moved_by_outer_viewport_bounds_delta_y {
            y_adjustment += context.state.outer_viewport_container_bounds_delta_.y();
        }
        if node.moved_by_safe_area_bottom {
            y_adjustment += context.state.transform_delta_by_safe_area_inset_bottom_;
        }
        let mut position_adjustment = Vector2dF::new(0.0, y_adjustment);
        #[cfg(not(target_os = "android"))]
        {
            if self.Node(node_id).should_undo_overscroll {
                self.UndoOverscroll(
                    node_id,
                    &mut position_adjustment,
                    viewport_property_ids,
                    context,
                );
            }
        }
        transform.Translate(position_adjustment.x(), position_adjustment.y());
        let elastic_overscroll =
            self.FindDrawnElasticOverscrollFromTransformId(node_id, viewport_property_ids, context);
        if !elastic_overscroll.1.IsZero() {
            #[cfg(target_os = "android")]
            Self::ApplyElasticOverscrollStretch(
                context.scroll_tree,
                self.page_scale_factor(),
                &elastic_overscroll,
                &mut transform,
                context.state,
            );
            #[cfg(not(target_os = "android"))]
            Self::ApplyElasticOverscrollTranslate(&elastic_overscroll, &mut transform);
        }
        let scroll_offset = self.Node(node_id).scroll_offset();
        transform.Translate(-scroll_offset.x(), -scroll_offset.y());
        let sticky_offset = self.StickyPositionOffset(node_id, context);
        transform.Translate(sticky_offset.x(), sticky_offset.y());
        if self.Node(node_id).anchor_position_scroll_data_id >= 0 {
            let mut visited = BTreeSet::new();
            let anchor_offset = self.AnchorPositionOffset(
                node_id,
                node_id - 1,
                update_data.as_deref_mut(),
                &mut visited,
                context,
            );
            transform.Translate(anchor_offset.x(), anchor_offset.y());
            self.MutableNode(node_id)
                .SetTransformChanged(DamageReason::kUntracked);
        }
        transform.PreConcat(&self.Node(node_id).local);
        let origin = self.Node(node_id).origin;
        transform.Translate3d(-origin.x(), -origin.y(), -origin.z());
        let node = self.MutableNode(node_id);
        node.set_to_parent(&transform);
        node.needs_local_transform_update = false;
    }
    // Rust disjoint-field borrowing of the same resident parent/current nodes.
    // Tree construction gives every parent a lower id than its child.
    fn NodeAndParentMut(&mut self, id: i32) -> (&mut TransformNode<T>, &TransformNode<T>) {
        let parent_id = self.Node(id).parent_id;
        assert!(parent_id >= 0 && parent_id < id);
        let (ancestors, current) = self.base_.nodes_.split_at_mut(id as usize);
        (&mut current[0], &ancestors[parent_id as usize])
    }
    // Source: cc/trees/property_tree.h:387-388; cc/trees/property_tree.cc:876-895.
    fn UpdateScreenSpaceTransform(&mut self, id: i32) {
        let parent_id = self.Node(id).parent_id;
        let mut to_screen_space_transform = self.ToScreen(parent_id).clone();
        if self.Node(id).flattens_inherited_transform {
            to_screen_space_transform.Flatten();
        }
        to_screen_space_transform.PreConcat(&self.Node(id).to_parent);
        let (node, parent_node) = self.NodeAndParentMut(id);
        node.ancestors_are_invertible = parent_node.ancestors_are_invertible;
        node.node_and_ancestors_are_flat =
            parent_node.node_and_ancestors_are_flat && node.to_parent.IsFlat();
        self.SetToScreen(id, &to_screen_space_transform);
        let mut from_screen = T::default();
        if !self.ToScreen(id).GetInverse(&mut from_screen) {
            self.MutableNode(id).ancestors_are_invertible = false;
        }
        self.SetFromScreen(id, &from_screen);
    }
    // Source: cc/trees/property_tree.h:389-390; cc/trees/property_tree.cc:896-904.
    fn UpdateAnimationProperties(&mut self, id: i32) {
        let (node, parent_node) = self.NodeAndParentMut(id);
        let ancestor_is_animating = parent_node.to_screen_is_potentially_animated;
        node.to_screen_is_potentially_animated =
            node.has_potential_animation || ancestor_is_animating;
    }
    // Source: cc/trees/property_tree.h:391; cc/trees/property_tree.cc:905-912.
    fn UndoSnapping(&mut self, id: i32) {
        let node = self.MutableNode(id);
        node.to_parent
            .Translate(-node.snap_amount.x(), -node.snap_amount.y());
        node.snap_amount = Vector2dF::default();
    }
    // Source: cc/trees/property_tree.h:392; cc/trees/property_tree.cc:913-950.
    fn UpdateSnapping(&mut self, id: i32) {
        let node = self.Node(id);
        if !node.should_be_snapped
            || node.to_screen_is_potentially_animated
            || !self.ToScreen(id).IsScaleOrTranslation()
            || !node.ancestors_are_invertible
        {
            return;
        }
        debug_assert!(id < self.cached_data_.len() as i32);
        let cached = &mut self.cached_data_[id as usize];
        cached.to_screen.Round2dTranslationComponents();
        let mut delta = cached.from_screen.clone();
        delta.MultiplyAssign(&cached.to_screen);
        const kTolerance: f32 = 1e-4;
        debug_assert!(delta.IsApproximatelyIdentityOrTranslation(kTolerance));
        let translation = delta.To2dTranslation();
        self.MutableNode(id).snap_amount = translation;
        if translation.IsZero() {
            return;
        }
        self.cached_data_[id as usize]
            .from_screen
            .PostTranslate(Vector2dF::new(-translation.x(), -translation.y()));
        let node = self.MutableNode(id);
        node.to_parent.Translate(translation.x(), translation.y());
        if node
            .to_parent
            .IsApproximatelyIdentityOrIntegerTranslation(kTolerance)
        {
            node.to_parent.RoundToIdentityOrIntegerTranslation();
        }
    }
    // Source: cc/trees/property_tree.h:229-230; cc/trees/property_tree.cc:951-957.
    pub fn UpdateTransformChanged(&mut self, id: i32) {
        let (node, parent_node) = self.NodeAndParentMut(id);
        if parent_node.transform_changed() {
            node.CopyTransformChangedFrom(parent_node);
        }
    }
    // Source: cc/trees/property_tree.h:231-233; cc/trees/property_tree.cc:958-975.
    pub fn UpdateNodeAndAncestorsAreAnimatedOrInvertible(&mut self, id: i32) {
        let (node, parent_node) = self.NodeAndParentMut(id);
        if !parent_node.node_and_ancestors_are_animated_or_invertible {
            node.node_and_ancestors_are_animated_or_invertible = false;
            return;
        }
        let mut is_invertible = node.is_invertible;
        if !node.ancestors_are_invertible && parent_node.ancestors_are_invertible {
            is_invertible = false;
        }
        node.node_and_ancestors_are_animated_or_invertible =
            node.has_potential_animation || is_invertible;
    }
    // Source: cc/trees/property_tree.h:234-236; cc/trees/property_tree.cc:976-983.
    pub fn UpdateNodeOrAncestorsWillChangeTransform(&mut self, id: i32) {
        let (node, parent_node) = self.NodeAndParentMut(id);
        node.node_or_ancestors_will_change_transform = node.will_change_transform;
        node.node_or_ancestors_will_change_transform |=
            parent_node.node_or_ancestors_will_change_transform;
    }

    // Source: cc/trees/property_tree.h:261-262; cc/trees/property_tree.cc:984-1025.
    pub fn SetRootScaleAndTransform(
        &mut self,
        device_scale_factor: f32,
        device_transform: &T,
        context: &mut TransformTreeContext<'_, T>,
    ) {
        self.device_scale_factor_ = device_scale_factor;
        let components = device_transform.ComputeTransform2dScaleComponents(1.0);
        self.device_transform_scale_factor_ = if components.x() < components.y() {
            components.y()
        } else {
            components.x()
        };
        let mut transform = device_transform.clone();
        transform.Scale(device_scale_factor, device_scale_factor);
        let screen_space_scale = transform.ComputeTransform2dScaleComponents(device_scale_factor);
        debug_assert!(screen_space_scale.x() != 0.0);
        debug_assert!(screen_space_scale.y() != 0.0);
        let mut root_to_screen = T::default();
        root_to_screen.Scale(screen_space_scale.x(), screen_space_scale.y());
        let root_from_screen = root_to_screen.GetCheckedInverse();
        if root_to_screen != *self.ToScreen(kRootPropertyNodeId) {
            self.SetToScreen(kRootPropertyNodeId, &root_to_screen);
            self.SetFromScreen(kRootPropertyNodeId, &root_from_screen);
            self.set_needs_update(true, context);
        }
        transform.PostConcat(&root_from_screen);
        let node = self.MutableNode(kContentsRootPropertyNodeId);
        if node.local != transform {
            node.local = transform;
            node.needs_local_transform_update = true;
            self.set_needs_update(true, context);
        }
    }
    // Source: cc/trees/property_tree.h:270; cc/trees/property_tree.cc:1026-1034.
    pub fn UpdateOuterViewportContainerBoundsDelta(
        &mut self,
        context: &mut TransformTreeContext<'_, T>,
    ) {
        if self
            .nodes_affected_by_outer_viewport_bounds_delta_
            .is_empty()
        {
            return;
        }
        self.set_needs_update(true, context);
        for index in 0..self.nodes_affected_by_outer_viewport_bounds_delta_.len() {
            let id = self.nodes_affected_by_outer_viewport_bounds_delta_[index];
            self.MutableNode(id).needs_local_transform_update = true;
        }
    }
    // Source: cc/trees/property_tree.h:272; cc/trees/property_tree.cc:1035-1038.
    pub fn AddNodeAffectedByOuterViewportBoundsDelta(&mut self, id: i32) {
        self.nodes_affected_by_outer_viewport_bounds_delta_.push(id);
    }
    // Source: cc/trees/property_tree.h:274; cc/trees/property_tree.cc:1039-1042.
    pub fn HasNodesAffectedByOuterViewportBoundsDelta(&self) -> bool {
        !self
            .nodes_affected_by_outer_viewport_bounds_delta_
            .is_empty()
    }
    // Source: cc/trees/property_tree.h:285; cc/trees/property_tree.cc:1043-1053.
    pub fn NeedTransformUpdateForSafeAreaInsetBottom(
        &mut self,
        context: &mut TransformTreeContext<'_, T>,
    ) {
        if self.nodes_affected_by_safe_area_inset_bottom_.is_empty() {
            return;
        }
        self.set_needs_update(true, context);
        for index in 0..self.nodes_affected_by_safe_area_inset_bottom_.len() {
            let id = self.nodes_affected_by_safe_area_inset_bottom_[index];
            self.MutableNode(id).needs_local_transform_update = true;
        }
    }
    // Source: cc/trees/property_tree.h:287; cc/trees/property_tree.cc:1054-1057.
    pub fn AddNodeAffectedBySafeAreaInsetBottom(&mut self, id: i32) {
        self.nodes_affected_by_safe_area_inset_bottom_.push(id);
    }
    // Source: cc/trees/property_tree.h:289; cc/trees/property_tree.cc:1058-1061.
    pub fn HasNodesAffectedBySafeAreaBottom(&self) -> bool {
        !self.nodes_affected_by_safe_area_inset_bottom_.is_empty()
    }
    // Source: cc/trees/property_tree.h:313; cc/trees/property_tree.cc:1062-1067.
    pub fn FromScreen(&self, id: i32) -> &T {
        debug_assert!(self.cached_data_.len() as i32 > id && id != kInvalidPropertyNodeId);
        &self.cached_data_[id as usize].from_screen
    }
    // Source: cc/trees/property_tree.h:314; cc/trees/property_tree.cc:1068-1074.
    pub fn SetFromScreen(&mut self, id: i32, transform: &T) {
        debug_assert!(self.cached_data_.len() as i32 > id && id != kInvalidPropertyNodeId);
        self.cached_data_[id as usize].from_screen = transform.clone();
    }
    // Source: cc/trees/property_tree.h:316; cc/trees/property_tree.cc:1075-1080.
    pub fn ToScreen(&self, id: i32) -> &T {
        debug_assert!(self.cached_data_.len() as i32 > id && id != kInvalidPropertyNodeId);
        &self.cached_data_[id as usize].to_screen
    }
    // Source: cc/trees/property_tree.h:317; cc/trees/property_tree.cc:1081-1087.
    pub fn SetToScreen(&mut self, id: i32, transform: &T) {
        debug_assert!(self.cached_data_.len() as i32 > id && id != kInvalidPropertyNodeId);
        self.cached_data_[id as usize].to_screen = transform.clone();
        self.cached_data_[id as usize].is_showing_backface = transform.IsBackFaceVisible();
    }
    // Source: cc/trees/property_tree.h:378; cc/trees/property_tree.cc:1103-1112.
    fn MutableStickyPositionData(&mut self, id: i32) -> Option<&mut StickyPositionNodeData> {
        let index = self.Node(id).sticky_position_constraint_id;
        if index == -1 || index as usize >= self.sticky_position_data_.len() {
            return None;
        }
        Some(&mut self.sticky_position_data_[index as usize])
    }
    // Source: cc/trees/property_tree.h:333-335.
    pub fn GetStickyPositionData(&self, id: i32) -> Option<&StickyPositionNodeData> {
        let index = self.Node(id).sticky_position_constraint_id;
        if index == -1 || index as usize >= self.sticky_position_data_.len() {
            return None;
        }
        Some(&self.sticky_position_data_[index as usize])
    }
    // Source: cc/trees/property_tree.h:336; cc/trees/property_tree.cc:1113-1123.
    pub fn EnsureStickyPositionData(&mut self, id: i32) -> &mut StickyPositionNodeData {
        let index = self.Node(id).sticky_position_constraint_id;
        if index == -1 || index as usize >= self.sticky_position_data_.len() {
            let new_index = self.sticky_position_data_.len() as i32;
            self.MutableNode(id).sticky_position_constraint_id = new_index;
            self.sticky_position_data_
                .push(StickyPositionNodeData::default());
        }
        let index = self.Node(id).sticky_position_constraint_id as usize;
        &mut self.sticky_position_data_[index]
    }

    // Source: cc/trees/property_tree.h:240-245.
    pub fn set_page_scale_factor(&mut self, value: f32) {
        self.page_scale_factor_ = value;
    }
    pub fn page_scale_factor(&self) -> f32 {
        self.page_scale_factor_
    }

    // Source: cc/trees/property_tree.h:247-250.
    pub fn set_device_scale_factor(&mut self, value: f32) {
        self.device_scale_factor_ = value;
    }
    pub fn device_scale_factor(&self) -> f32 {
        self.device_scale_factor_
    }

    // Source: cc/trees/property_tree.h:252-259.
    pub fn set_external_page_scale_factor(&mut self, value: f32) {
        self.external_page_scale_factor_ = value;
    }
    pub fn external_page_scale_factor(&self) -> f32 {
        self.external_page_scale_factor_
    }

    // Source: cc/trees/property_tree.h:263-268.
    pub fn set_device_transform_scale_factor(&mut self, value: f32) {
        self.device_transform_scale_factor_ = value;
    }
    pub fn device_transform_scale_factor(&self) -> f32 {
        self.device_transform_scale_factor_
    }

    // Source: cc/trees/property_tree.h:276-283.
    pub fn nodes_affected_by_outer_viewport_bounds_delta(&self) -> &Vec<i32> {
        &self.nodes_affected_by_outer_viewport_bounds_delta_
    }
    pub fn set_nodes_affected_by_outer_viewport_bounds_delta(&mut self, nodes: Vec<i32>) {
        self.nodes_affected_by_outer_viewport_bounds_delta_ = nodes;
    }

    // Source: cc/trees/property_tree.h:291-296.
    pub fn nodes_affected_by_safe_area_bottom(&self) -> &Vec<i32> {
        &self.nodes_affected_by_safe_area_inset_bottom_
    }
    pub fn set_nodes_affected_by_safe_area_bottom(&mut self, nodes: Vec<i32>) {
        self.nodes_affected_by_safe_area_inset_bottom_ = nodes;
    }

    // Source: cc/trees/property_tree.h:298-303.
    pub fn sticky_position_data(&self) -> &Vec<StickyPositionNodeData> {
        &self.sticky_position_data_
    }
    pub fn sticky_position_data_mut(&mut self) -> &mut Vec<StickyPositionNodeData> {
        &mut self.sticky_position_data_
    }

    // Source: cc/trees/property_tree.h:305-311.
    pub fn anchor_position_scroll_data(&self) -> &Vec<AnchorPositionScrollData> {
        &self.anchor_position_scroll_data_
    }
    pub fn anchor_position_scroll_data_mut(&mut self) -> &mut Vec<AnchorPositionScrollData> {
        &mut self.anchor_position_scroll_data_
    }

    // Source: cc/trees/property_tree.h:325-327.
    pub fn cached_data(&self) -> &Vec<TransformCachedNodeData<T>> {
        &self.cached_data_
    }
    // Source: cc/trees/property_tree.h:355-361.
    pub fn drawn_elastic_overscroll(&self) -> &BTreeMap<ElementId, Vector2dF> {
        &self.drawn_elastic_overscroll_
    }
    pub fn drawn_elastic_overscroll_mut(&mut self) -> &mut BTreeMap<ElementId, Vector2dF> {
        &mut self.drawn_elastic_overscroll_
    }
}
// Source: cc/trees/property_tree.h:187-189; cc/trees/property_tree.cc:1088-1102.
#[cfg(debug_assertions)]
impl<T: PropertyTreeTransform> PartialEq for TransformTree<T> {
    fn eq(&self, other: &Self) -> bool {
        self.base_ == other.base_
            && self.page_scale_factor_ == other.page_scale_factor()
            && self.device_scale_factor_ == other.device_scale_factor()
            && self.device_transform_scale_factor_ == other.device_transform_scale_factor()
            && self.external_page_scale_factor_ == other.external_page_scale_factor()
            && self.nodes_affected_by_outer_viewport_bounds_delta_
                == *other.nodes_affected_by_outer_viewport_bounds_delta()
            && self.cached_data_ == *other.cached_data()
            && self.drawn_elastic_overscroll_ == *other.drawn_elastic_overscroll()
    }
}
// Source: cc/trees/property_tree.h:319-323.
// These declarations have no definitions in the supplied Chromium .cc. This
// declaration-only Rust trait retains the contract without inventing behavior.
pub trait TransformTreeTargetAssociations {
    fn TargetId(&self, node_id: i32) -> i32;
    fn SetTargetId(&mut self, node_id: i32, target_id: i32);
    fn ContentTargetId(&self, node_id: i32) -> i32;
    fn SetContentTargetId(&mut self, node_id: i32, content_target_id: i32);
}

use super::effect_node::RenderSurfaceReason;
use super::property_ids::kViewportPropertyNodeId;
use crate::adapter::property_tree::{
    PropertyTreeCopyOutputRequest, PropertyTreeFeatures, PropertyTreeFilters,
    PropertyTreeRenderSurface, PropertyTreeRenderSurfaceFactory, RenderSurfacePropertyChangedFlags,
};
use crate::adapter::property_types::BlendMode;

// Source: cc/trees/property_tree.h:472-603.
// BTreeMap + per-key Vec is the ordered native multimap (including insertion
// order among equivalent keys). Box retains unique owned request/surface lifetime.
pub type CopyRequestMap<C> = BTreeMap<i32, Vec<Option<Box<C>>>>;
pub struct EffectTree<
    F: PropertyTreeFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
> {
    // Rust drops fields in declaration order. C++ destroys these members in
    // reverse order, then its PropertyTree base; abort callbacks must observe
    // that same surface/request/base lifetime order.
    // Source: cc/trees/property_tree.h:577-578; cc/trees/property_tree.cc:1129.
    render_surfaces_: Vec<Option<Box<R>>>,
    // Source: cc/trees/property_tree.h:572-575; cc/trees/property_tree.cc:1129.
    copy_requests_: CopyRequestMap<C>,
    // Source: cc/trees/property_tree.h:467; cc/trees/property_tree.cc:1124-1129.
    base_: PropertyTree<EffectNode<F, P>>,
}
impl<
        F: PropertyTreeFilters,
        P: PropertyPath,
        R: PropertyTreeRenderSurface,
        C: PropertyTreeCopyOutputRequest,
    > std::ops::Deref for EffectTree<F, P, R, C>
{
    type Target = PropertyTree<EffectNode<F, P>>;
    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}
impl<
        F: PropertyTreeFilters,
        P: PropertyPath,
        R: PropertyTreeRenderSurface,
        C: PropertyTreeCopyOutputRequest,
    > std::ops::DerefMut for EffectTree<F, P, R, C>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}
/// Call-scoped projection of the same EffectTree while its selected surface
/// fields are borrowed uniquely. Reserved ids are private existence proofs
/// constructed only after the corresponding resident slot was verified Some.
/// They expose no surface state and permit no structural tree mutation.
pub(crate) struct EffectTreeSurfaceQueries<
    'a,
    F: PropertyTreeFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
> {
    nodes: &'a PropertyTree<EffectNode<F, P>>,
    slices: [&'a [Option<Box<R>>]; 3],
    starts: [usize; 3],
    reserved: [Option<usize>; 2],
    total_len: usize,
}
impl<F: PropertyTreeFilters, P: PropertyPath, R: PropertyTreeRenderSurface>
    EffectTreeSurfaceQueries<'_, F, P, R>
{
    pub(crate) fn Node(&self, id: i32) -> &EffectNode<F, P> {
        self.nodes.Node(id)
    }
    pub(crate) fn HasRenderSurface(&self, id: i32) -> bool {
        let index = id as usize;
        assert!(index < self.total_len, "same native surface-vector bounds");
        if self.reserved.contains(&Some(index)) {
            return true;
        }
        for (slice, start) in self.slices.iter().zip(self.starts) {
            if index >= start && index - start < slice.len() {
                return slice[index - start].is_some();
            }
        }
        unreachable!("surface slot must be resident or uniquely reserved")
    }
}

/// Live owner siblings used only while executing the source backpointer access.
pub struct EffectTreeContext<'a, T: PropertyTreeTransform> {
    pub transform_tree: &'a TransformTree<T>,
    pub clip_tree: &'a mut ClipTree,
    pub state: &'a mut PropertyTreesState,
    pub cached_data: &'a mut PropertyTreesCachedData<T>,
    pub features: &'a dyn PropertyTreeFeatures,
}
impl<
        F: PropertyTreeFilters,
        P: PropertyPath,
        R: PropertyTreeRenderSurface,
        C: PropertyTreeCopyOutputRequest,
    > EffectTree<F, P, R, C>
{
    // Source: cc/trees/property_tree.cc:1124-1153.
    pub fn new(owner: Option<PropertyTreesOwnerId>) -> Self {
        Self {
            base_: PropertyTree::new(owner),
            copy_requests_: BTreeMap::new(),
            render_surfaces_: vec![None],
        }
    }
    pub fn Insert(&mut self, node: &EffectNode<F, P>, parent_id: i32) -> i32 {
        let id = self.base_.Insert(node, parent_id);
        debug_assert_eq!(id, self.render_surfaces_.len() as i32);
        self.render_surfaces_.push(None);
        id
    }
    pub fn RemoveNodes(&mut self, n: usize) {
        self.base_.RemoveNodes(n);
        self.render_surfaces_
            .truncate(self.render_surfaces_.len() - n);
    }
    pub fn clear(&mut self) {
        self.base_.clear();
        self.render_surfaces_.clear();
        self.render_surfaces_.push(None);
        #[cfg(debug_assertions)]
        {
            debug_assert!(Self::new(None) == *self);
        }
    }
    // Source: cc/trees/property_tree.cc:1155-1173.
    pub fn EffectiveOpacity(&self, node: &EffectNode<F, P>) -> f32 {
        Self::EffectiveOpacityValue(node)
    }
    fn EffectiveOpacityValue(node: &EffectNode<F, P>) -> f32 {
        if node.subtree_hidden {
            0.0
        } else {
            node.opacity
        }
    }
    fn UpdateOpacities(node: &mut EffectNode<F, P>, parent: Option<&EffectNode<F, P>>) {
        node.screen_space_opacity = Self::EffectiveOpacityValue(node);
        if let Some(parent) = parent {
            node.screen_space_opacity *= parent.screen_space_opacity;
        }
    }
    fn UpdateSubtreeHidden(node: &mut EffectNode<F, P>, parent: Option<&EffectNode<F, P>>) {
        if let Some(parent) = parent {
            node.subtree_hidden |= parent.subtree_hidden;
        }
    }
    // Source: cc/trees/property_tree.cc:1175-1200.
    fn UpdateIsDrawn(
        node: &mut EffectNode<F, P>,
        parent: Option<&EffectNode<F, P>>,
        state: &PropertyTreesState,
    ) {
        if node.has_copy_request || node.cache_render_surface || node.subtree_capture_id.is_valid()
        {
            node.is_drawn = true;
        } else if Self::EffectiveOpacityValue(node) == 0.0
            && (!node.has_potential_opacity_animation || state.is_active_)
            && node.backdrop_filters.IsEmpty()
        {
            node.is_drawn = false;
        } else if let Some(parent) = parent {
            node.is_drawn = parent.is_drawn;
        } else {
            node.is_drawn = true;
        }
    }
    // Source: cc/trees/property_tree.cc:1201-1225.
    pub fn UpdateEffectChanged(node: &mut EffectNode<F, P>, parent: Option<&EffectNode<F, P>>) {
        if parent.is_some_and(|parent| parent.effect_changed) {
            node.effect_changed = true;
        }
    }
    pub fn UpdateHasFilters(node: &mut EffectNode<F, P>, parent: Option<&EffectNode<F, P>>) {
        node.lcd_text_disallowed_by_filter =
            node.has_potential_filter_animation || !node.filters.AllowsLCDText();
        if let Some(parent) = parent {
            node.lcd_text_disallowed_by_filter |= parent.lcd_text_disallowed_by_filter;
        }
    }
    pub fn UpdateHasFastRoundedCorner(
        node: &mut EffectNode<F, P>,
        parent: Option<&EffectNode<F, P>>,
    ) {
        node.node_or_ancestor_has_fast_rounded_corner = node.is_fast_rounded_corner;
        if let Some(parent) = parent {
            node.node_or_ancestor_has_fast_rounded_corner |=
                parent.node_or_ancestor_has_fast_rounded_corner;
        }
    }
    // Source: cc/trees/property_tree.cc:1227-1241.
    fn UpdateBackfaceVisibility<T: PropertyTreeTransform>(
        node: &mut EffectNode<F, P>,
        parent: Option<&EffectNode<F, P>>,
        transforms: &TransformTree<T>,
    ) {
        if parent.is_some_and(|parent| parent.hidden_by_backface_visibility) {
            node.hidden_by_backface_visibility = true;
            return;
        }
        if node.double_sided {
            node.hidden_by_backface_visibility = false;
            return;
        }
        node.hidden_by_backface_visibility =
            transforms.cached_data_[node.transform_id as usize].is_showing_backface;
    }
    // Source: cc/trees/property_tree.cc:1243-1251.
    fn UpdateHasMaskingChild(node: &mut EffectNode<F, P>, parent: Option<&mut EffectNode<F, P>>) {
        node.has_masking_child = false;
        if node.blend_mode == BlendMode::kDstIn {
            if let Some(parent) = parent {
                parent.has_masking_child = true;
            }
        }
    }
    // Source: cc/trees/property_tree.cc:1253-1265.
    fn UpdateOnlyDrawsVisibleContent(
        node: &mut EffectNode<F, P>,
        parent: Option<&EffectNode<F, P>>,
    ) {
        node.only_draws_visible_content = !node.has_copy_request
            && !node.subtree_capture_id.is_valid()
            && !node.view_transition_element_resource_id.IsValid();
        if let Some(parent) = parent {
            node.only_draws_visible_content &= parent.only_draws_visible_content;
        }
        if !node.backdrop_filters.IsEmpty() {
            node.only_draws_visible_content &= !node.backdrop_filters.HasZoomFilter();
        }
    }
    // Source: cc/trees/property_tree.cc:1267-1322.
    pub fn UpdateSurfaceContentsScale<T: PropertyTreeTransform>(
        node: &mut EffectNode<F, P>,
        context: &mut EffectTreeContext<'_, T>,
    ) {
        if !node.HasRenderSurface() {
            node.surface_contents_scale = Vector2dF::new(1.0, 1.0);
            return;
        }
        let transforms = context.transform_tree;
        let mut layer_scale_factor =
            transforms.device_scale_factor() * transforms.device_transform_scale_factor();
        let transform_node = transforms.Node(node.transform_id);
        if transform_node.in_subtree_of_page_scale_layer {
            layer_scale_factor *= transforms.page_scale_factor();
        }
        let old_scale = node.surface_contents_scale;
        node.surface_contents_scale = transforms
            .ToScreen(transform_node.id)
            .ComputeTransform2dScaleComponents(layer_scale_factor);
        if node.id != kContentsRootPropertyNodeId
            && context.features.SizeOopifEffectSurfacesAtExternalScale()
        {
            node.surface_contents_scale
                .Scale(transforms.external_page_scale_factor());
        }
        if node.render_surface_reason
            == RenderSurfaceReason::k2DScaleTransformWithCompositedDescendants
        {
            node.surface_contents_scale
                .set_x(node.surface_contents_scale.x().abs().ceil());
            node.surface_contents_scale
                .set_y(node.surface_contents_scale.y().abs().ceil());
        }
        if old_scale != node.surface_contents_scale {
            context.clip_tree.set_needs_update(true);
            context.cached_data.UpdateTransformTreeUpdateNumber();
        }
    }
    // Source: cc/trees/property_tree.cc:1324-1338.
    pub fn OnOpacityAnimated(
        &mut self,
        id: ElementId,
        opacity: f32,
        state: &mut PropertyTreesState,
    ) -> bool {
        let Some(node) = self.MutableFindNodeFromElementId(id) else {
            return false;
        };
        if node.opacity == opacity {
            return false;
        }
        node.opacity = opacity;
        node.effect_changed = true;
        state.changed_ = true;
        self.set_needs_update(true);
        true
    }
    // Source: cc/trees/property_tree.cc:1340-1355.
    pub fn OnFilterAnimated(
        &mut self,
        id: ElementId,
        filters: &F,
        state: &mut PropertyTreesState,
    ) -> bool {
        let Some(node) = self.MutableFindNodeFromElementId(id) else {
            return false;
        };
        if node.filters == *filters {
            return false;
        }
        node.filters = filters.clone();
        node.effect_changed = true;
        state.changed_ = true;
        self.set_needs_update(true);
        true
    }
    // Source: cc/trees/property_tree.cc:1357-1373.
    pub fn OnBackdropFilterAnimated(
        &mut self,
        id: ElementId,
        filters: &F,
        state: &mut PropertyTreesState,
    ) -> bool {
        let Some(node) = self.MutableFindNodeFromElementId(id) else {
            return false;
        };
        if node.backdrop_filters == *filters {
            return false;
        }
        node.backdrop_filters = filters.clone();
        node.effect_changed = true;
        state.changed_ = true;
        self.set_needs_update(true);
        true
    }
    // Source: cc/trees/property_tree.cc:1375-1390.
    pub fn UpdateEffects<T: PropertyTreeTransform>(
        &mut self,
        id: i32,
        context: &mut EffectTreeContext<'_, T>,
    ) {
        let parent_id = self.Node(id).parent_id;
        let (node, mut parent) = if parent_id == kInvalidPropertyNodeId {
            (&mut self.base_.nodes_[id as usize], None)
        } else {
            debug_assert!(parent_id < id);
            let (parents, nodes) = self.base_.nodes_.split_at_mut(id as usize);
            (&mut nodes[0], Some(&mut parents[parent_id as usize]))
        };
        Self::UpdateOpacities(node, parent.as_deref());
        Self::UpdateSubtreeHidden(node, parent.as_deref());
        Self::UpdateIsDrawn(node, parent.as_deref(), context.state);
        Self::UpdateEffectChanged(node, parent.as_deref());
        Self::UpdateHasFilters(node, parent.as_deref());
        Self::UpdateHasFastRoundedCorner(node, parent.as_deref());
        Self::UpdateBackfaceVisibility(node, parent.as_deref(), context.transform_tree);
        Self::UpdateHasMaskingChild(node, parent.as_deref_mut());
        Self::UpdateOnlyDrawsVisibleContent(node, parent.as_deref());
        Self::UpdateClosestAncestorSharedElement(node, parent.as_deref());
        Self::UpdateSurfaceContentsScale(node, context);
    }
    // Source: cc/trees/property_tree.cc:1392-1401.
    fn UpdateClosestAncestorSharedElement(
        node: &mut EffectNode<F, P>,
        parent: Option<&EffectNode<F, P>>,
    ) {
        if node.view_transition_element_resource_id.IsValid() {
            node.closest_ancestor_with_shared_element_id = node.id;
        } else if let Some(parent) = parent {
            node.closest_ancestor_with_shared_element_id =
                parent.closest_ancestor_with_shared_element_id;
        }
    }
    // Source: cc/trees/property_tree.cc:1403-1413.
    pub fn AddCopyRequest(&mut self, id: i32, request: Box<C>) {
        self.MutableNode(id).has_copy_request = true;
        self.copy_requests_
            .entry(id)
            .or_default()
            .push(Some(request));
    }
    pub fn PullCopyRequestsFrom(&mut self, requests: &mut CopyRequestMap<C>) {
        self.copy_requests_ = std::mem::take(requests);
    }
    // Source: cc/trees/property_tree.cc:1415-1501.
    pub fn TakeCopyRequestsAndTransformToSurface<T: PropertyTreeTransform>(
        &mut self,
        node_id: i32,
        requests: &mut Vec<Box<C>>,
        transform_tree: &TransformTree<T>,
        cache: &PropertyTreesCachedData<T>,
    ) {
        let node = self.Node(node_id);
        debug_assert!(node.HasRenderSurface());
        debug_assert!(node.has_copy_request);
        let destination_id = node.transform_id;
        let source_id = if node.parent_id != kInvalidPropertyNodeId {
            destination_id
        } else {
            debug_assert_eq!(kRootPropertyNodeId, destination_id);
            kContentsRootPropertyNodeId
        };
        let mut transform = T::default();
        GetToTargetCore(
            transform_tree,
            self,
            cache,
            source_id,
            node_id,
            &mut transform,
        );
        // Erasing the equivalent-key range always drops rejected requests, whose
        // owning Viz implementation supplies Chromium's abort response on Drop.

        if transform.IsPositiveScaleOrTranslation() {
            let content_vector = Vector2d { x: 1024, y: 1024 };
            let surface_rect = transform.MapRect(RectF::new(
                0.0,
                0.0,
                content_vector.x as f32,
                content_vector.y as f32,
            ));
            if let Some(range) = self.copy_requests_.get_mut(&node_id) {
                for slot in range {
                    let request = slot.as_mut().expect("non-null copy request");
                    if request.has_area() {
                        request.set_area(MathUtil::MapEnclosingClippedRectIgnoringError(
                            &transform,
                            request.area(),
                            0.001,
                        ));
                    }
                    if request.is_scaled() || request.has_result_selection() {
                        let from = request.scale_from();
                        let scale_from_x_f = from.x as f32 * surface_rect.width();
                        let scale_from_y_f = from.y as f32 * surface_rect.height();
                        // IsValueInRangeForNumericType<int>(float) excludes 2^31,
                        // which rounds above INT_MAX in f32; do not use f32(INT_MAX).
                        if scale_from_x_f.is_nan()
                            || !(scale_from_x_f >= -2147483648.0 && scale_from_x_f < 2147483648.0)
                            || scale_from_y_f.is_nan()
                            || !(scale_from_y_f >= -2147483648.0 && scale_from_y_f < 2147483648.0)
                        {
                            continue;
                        }
                        let to = request.scale_to();
                        let Some(scale_to_x) = to.x.checked_mul(content_vector.x) else {
                            continue;
                        };
                        let Some(scale_to_y) = to.y.checked_mul(content_vector.y) else {
                            continue;
                        };
                        let scale_from_x = scale_from_x_f.round() as i32;
                        let scale_from_y = scale_from_y_f.round() as i32;
                        if scale_from_x <= 0
                            || scale_from_y <= 0
                            || scale_to_x <= 0
                            || scale_to_y <= 0
                        {
                            continue;
                        }
                        request.SetScaleRatio(
                            Vector2d {
                                x: scale_from_x,
                                y: scale_from_y,
                            },
                            Vector2d {
                                x: scale_to_x,
                                y: scale_to_y,
                            },
                        );
                    }
                    requests.push(slot.take().unwrap());
                }
            }
        }
        self.copy_requests_.remove(&node_id);
    }
    // Source: cc/trees/property_tree.cc:1503-1518.
    pub fn HasCopyRequests(&self) -> bool {
        !self.copy_requests_.is_empty()
    }
    pub fn ClearCopyRequests(&mut self) {
        for node in &mut self.base_.nodes_ {
            node.subtree_has_copy_request = false;
            node.has_copy_request = false;
            node.closest_ancestor_with_copy_request_id = kInvalidPropertyNodeId;
        }
        self.copy_requests_.clear();
        self.set_needs_update(true);
    }
    // Source: cc/trees/property_tree.cc:1520-1543.
    pub fn GetRenderSurfaceChangedFlags(&self, flags: &mut Vec<RenderSurfacePropertyChangedFlags>) {
        flags.resize(self.size(), RenderSurfacePropertyChangedFlags::default());
        for id in kContentsRootPropertyNodeId as usize..self.size() {
            flags[id] = self.render_surfaces_[id]
                .as_ref()
                .map_or(RenderSurfacePropertyChangedFlags::default(), |surface| {
                    surface.GetPropertyChangeFlags()
                });
        }
    }
    pub fn ApplyRenderSurfaceChangedFlags(&mut self, flags: &[RenderSurfacePropertyChangedFlags]) {
        if flags.is_empty() {
            return;
        }
        debug_assert_eq!(flags.len(), self.size());
        for id in kContentsRootPropertyNodeId as usize..self.size() {
            if let Some(surface) = self.render_surfaces_[id].as_mut() {
                surface.ApplyPropertyChangeFlags(flags[id]);
            }
        }
    }
    // Source: cc/trees/property_tree.cc:1545-1557.
    pub fn LowestCommonAncestorWithRenderSurface(&self, mut id_1: i32, mut id_2: i32) -> i32 {
        debug_assert!(self.GetRenderSurface(id_1).is_some());
        debug_assert!(self.GetRenderSurface(id_2).is_some());
        while id_1 != id_2 {
            if id_1 < id_2 {
                id_2 = self.Node(id_2).target_id;
            } else {
                id_1 = self.Node(id_1).target_id;
            }
        }
        id_1
    }
    // Source: cc/trees/property_tree.h:532-542.
    pub fn num_render_surfaces(&self) -> usize {
        self.render_surfaces_.len()
    }
    pub fn GetRenderSurface(&self, id: i32) -> Option<&R> {
        self.render_surfaces_[id as usize].as_deref()
    }
    pub fn GetRenderSurfaceMutable(&mut self, id: i32) -> Option<&mut R> {
        self.render_surfaces_[id as usize].as_deref_mut()
    }
    /// Rust field split of cc/property_tree.cc's resident surface access. The
    /// caller can mutate only the selected surface; nodes and all other slots
    /// remain borrowed read-only for exactly this current operation.
    pub(crate) fn GetRenderSurfaceMutableWithQueries(
        &mut self,
        id: i32,
    ) -> Option<(&mut R, EffectTreeSurfaceQueries<'_, F, P, R>)> {
        let index = id as usize;
        let total_len = self.render_surfaces_.len();
        let (before, tail) = self.render_surfaces_.split_at_mut(index);
        let (slot, after) = tail.split_first_mut().expect("native surface index");
        let surface = slot.as_deref_mut()?;
        let queries = EffectTreeSurfaceQueries {
            nodes: &self.base_,
            slices: [before, after, &[]],
            starts: [0, index + 1, total_len],
            reserved: [Some(index), None],
            total_len,
        };
        Some((surface, queries))
    }
    pub(crate) fn GetRenderSurfaceMutableAndOtherWithQueries(
        &mut self,
        mutable_id: i32,
        other_id: i32,
    ) -> Option<(&mut R, &R, EffectTreeSurfaceQueries<'_, F, P, R>)> {
        assert_ne!(mutable_id, other_id, "distinct native render surfaces");
        let mutable_index = mutable_id as usize;
        let other_index = other_id as usize;
        let low = mutable_index.min(other_index);
        let high = mutable_index.max(other_index);
        let total_len = self.render_surfaces_.len();
        let (left, high_tail) = self.render_surfaces_.split_at_mut(high);
        let (high_slot, after) = high_tail.split_first_mut().expect("native surface index");
        let (before, low_tail) = left.split_at_mut(low);
        let (low_slot, middle) = low_tail.split_first_mut().expect("native surface index");
        let low_surface = low_slot.as_deref_mut()?;
        let high_surface = high_slot.as_deref_mut()?;
        let queries = EffectTreeSurfaceQueries {
            nodes: &self.base_,
            slices: [before, middle, after],
            starts: [0, low + 1, high + 1],
            reserved: [Some(low), Some(high)],
            total_len,
        };
        if mutable_index < other_index {
            Some((low_surface, high_surface, queries))
        } else {
            Some((high_surface, low_surface, queries))
        }
    }
    /// Rust borrow projection for native code that mutates one resident
    /// RenderSurface while reading a distinct contributing surface. Both
    /// references point into the canonical EffectTree surface vector.
    pub fn GetRenderSurfaceMutableAndOther(
        &mut self,
        mutable_id: i32,
        other_id: i32,
    ) -> Option<(&mut R, &R)> {
        let (surface, other, _) =
            self.GetRenderSurfaceMutableAndOtherWithQueries(mutable_id, other_id)?;
        Some((surface, other))
    }
    // Source: cc/trees/property_tree.cc:1559-1565.
    pub fn ContributesToDrawnSurface(&self, id: i32) -> bool {
        let node = self.Node(id);
        node.is_drawn && (!self.HasParent(node) || self.parent(node).is_drawn)
    }
    // Source: cc/trees/property_tree.cc:1567-1578.
    pub fn ResetChangeTracking(&mut self) {
        for id in kContentsRootPropertyNodeId as usize..self.size() {
            self.base_.nodes_[id].effect_changed = false;
            if id < self.render_surfaces_.len() {
                if let Some(surface) = self.render_surfaces_[id].as_mut() {
                    surface.ResetPropertyChangedFlags();
                }
            }
        }
    }
    // Source: cc/trees/property_tree.cc:1580-1588.
    pub fn TakeRenderSurfaces(&mut self, surfaces: &mut Vec<Option<Box<R>>>) {
        for id in kContentsRootPropertyNodeId as usize..self.size() {
            if self.render_surfaces_[id].is_some() {
                surfaces.push(self.render_surfaces_[id].take());
            }
        }
    }
    // Source: cc/trees/property_tree.cc:1590-1653.
    pub fn CreateOrReuseRenderSurfaces(
        &mut self,
        old_surfaces: &mut Vec<Option<Box<R>>>,
        factory: &mut impl PropertyTreeRenderSurfaceFactory<R>,
    ) -> bool {
        let mut stable_id_node_id_list = Vec::new();
        for id in kContentsRootPropertyNodeId as usize..self.size() {
            let node = &self.base_.nodes_[id];
            if node.HasRenderSurface() {
                stable_id_node_id_list.push((node.element_id, node.id));
            }
        }
        stable_id_node_id_list.sort();
        old_surfaces.sort_by_key(|surface| {
            surface
                .as_ref()
                .expect("source requires non-null old surface")
                .id()
        });
        let mut changed = false;
        let mut surface_index = 0;
        let mut id_index = 0;
        while surface_index < old_surfaces.len() && id_index < stable_id_node_id_list.len() {
            let old_id = old_surfaces[surface_index].as_ref().unwrap().id();
            let (stable_id, new_node_id) = stable_id_node_id_list[id_index];
            if old_id == stable_id {
                self.render_surfaces_[new_node_id as usize] = old_surfaces[surface_index].take();
                self.render_surfaces_[new_node_id as usize]
                    .as_mut()
                    .unwrap()
                    .set_effect_tree_index(new_node_id);
                surface_index += 1;
                id_index += 1;
                continue;
            }
            changed = true;
            if stable_id < old_id {
                self.render_surfaces_[new_node_id as usize] =
                    Some(factory.CreateRenderSurface(stable_id));
                self.render_surfaces_[new_node_id as usize]
                    .as_mut()
                    .unwrap()
                    .set_effect_tree_index(new_node_id);
                id_index += 1;
            } else {
                surface_index += 1;
            }
        }
        if surface_index != old_surfaces.len() || id_index != stable_id_node_id_list.len() {
            changed = true;
        }
        while id_index < stable_id_node_id_list.len() {
            let (stable_id, new_node_id) = stable_id_node_id_list[id_index];
            self.render_surfaces_[new_node_id as usize] =
                Some(factory.CreateRenderSurface(stable_id));
            self.render_surfaces_[new_node_id as usize]
                .as_mut()
                .unwrap()
                .set_effect_tree_index(new_node_id);
            id_index += 1;
        }
        changed
    }
    // Source: cc/trees/property_tree.cc:1655-1667.
    pub fn ClippedHitTestRegionIsRectangle<T: PropertyTreeTransform>(
        &self,
        mut id: i32,
        transforms: &TransformTree<T>,
        cache: &PropertyTreesCachedData<T>,
    ) -> bool {
        while id != kContentsRootPropertyNodeId {
            let node = self.Node(id);
            let mut to_target = T::default();
            if !GetToTargetCore(
                transforms,
                self,
                cache,
                node.transform_id,
                node.target_id,
                &mut to_target,
            ) || !to_target.Preserves2dAxisAlignment()
            {
                return false;
            }
            id = node.target_id;
        }
        true
    }
    // Source: cc/trees/property_tree.cc:1669-1679.
    pub fn HitTestMayBeAffectedByMask(&self, mut id: i32) -> bool {
        while id != kContentsRootPropertyNodeId {
            let node = self.Node(id);
            if !node.mask_filter_info.IsEmpty() || node.has_masking_child {
                return true;
            }
            id = node.parent_id;
        }
        false
    }
    // Source: cc/trees/property_tree.cc:1681-1688.
    pub fn TakeCopyRequests(&mut self, state: &mut PropertyTreesState) -> CopyRequestMap<C> {
        if state.is_main_thread_ && !self.copy_requests_.is_empty() {
            state.needs_rebuild_ = true;
        }
        std::mem::take(&mut self.copy_requests_)
    }
    // Source: cc/trees/property_tree.cc:1716-1723.
    pub fn AssignFrom(&mut self, from: &Self) -> &mut Self {
        self.base_.AssignFrom(&from.base_);
        self.render_surfaces_.resize_with(self.size(), || None);
        self
    }
}
// Source: cc/trees/property_tree.cc:1725-1729.
#[cfg(debug_assertions)]
impl<
        F: PropertyTreeFilters,
        P: PropertyPath,
        R: PropertyTreeRenderSurface,
        C: PropertyTreeCopyOutputRequest,
    > PartialEq for EffectTree<F, P, R, C>
{
    fn eq(&self, other: &Self) -> bool {
        self.base_ == other.base_
    }
}

// Source: cc/trees/property_tree.h:456-470; cc/trees/property_tree.cc:1690-1714.
pub struct ClipTree {
    base_: PropertyTree<ClipNode>,
}
impl std::ops::Deref for ClipTree {
    type Target = PropertyTree<ClipNode>;
    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}
impl std::ops::DerefMut for ClipTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}
impl ClipTree {
    pub fn new(owner: Option<PropertyTreesOwnerId>) -> Self {
        Self {
            base_: PropertyTree::new(owner),
        }
    }
    pub fn AssignFrom(&mut self, from: &Self) -> &mut Self {
        self.base_.AssignFrom(&from.base_);
        self
    }
    pub fn SetViewportClip(&mut self, viewport_rect: RectF) {
        if self.size() < 2 {
            return;
        }
        let node = self.MutableNode(1);
        if viewport_rect == node.clip {
            return;
        }
        node.clip = viewport_rect;
        self.set_needs_update(true);
    }
    pub fn ViewportClip(&self) -> RectF {
        debug_assert!(self.size() > 1);
        self.Node(kViewportPropertyNodeId).clip
    }
}
#[cfg(debug_assertions)]
impl PartialEq for ClipTree {
    fn eq(&self, other: &Self) -> bool {
        self.base_ == other.base_
    }
}

use super::compositor_commit_data::{CompositorCommitData, ScrollUpdateInfo};
use super::scroll_source_type::ScrollSourceType;
use crate::adapter::property_tree::{PropertyTreeMutatorHost, PropertyTreeScrollLayerTree};
use crate::cc::input::scroll_snap_data::TargetSnapAreaElementIds;
use std::cell::{Ref, RefMut};
use std::rc::{Rc, Weak};

// Source: cc/trees/property_tree.h:605-626.
pub trait ScrollCallbacks {
    fn DidCompositorScroll(
        &self,
        element: ElementId,
        offset: &PointF,
        source: ScrollSourceType,
        snap: &Option<TargetSnapAreaElementIds>,
    );
    fn DidChangeScrollbarsHidden(&self, element: ElementId, hidden: bool);
}
// Source: cc/trees/property_tree.h:628-795.
// Rc<RefCell> is solely the source scoped_refptr<SyncedProperty> value shared by
// active/pending trees. Trees and all their nodes remain independently resident.
pub type SyncedScrollOffsetMap = BTreeMap<ElementId, Rc<RefCell<SyncedScrollOffset>>>;
pub struct ScrollTree {
    base_: PropertyTree<ScrollNode>,
    currently_scrolling_node_id_: i32,
    scroll_offset_map_: BTreeMap<ElementId, PointF>,
    synced_scroll_offset_map_: SyncedScrollOffsetMap,
    elastic_overscroll_: BTreeMap<ElementId, Vector2dF>,
    scrolling_contents_cull_rects_: BTreeMap<ElementId, Rect>,
    callbacks_: Option<Weak<dyn ScrollCallbacks>>,
}
impl std::ops::Deref for ScrollTree {
    type Target = PropertyTree<ScrollNode>;
    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}
impl std::ops::DerefMut for ScrollTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}
// Rust borrow bridge: a source pointer can refer to this ScrollTree's own
// resident node. Keep either the external reference or its stable node id;
// resolving the id borrows the live node, never a clone or copied projection.
#[derive(Clone, Copy)]
enum ScrollNodeAccess<'a> {
    External(&'a ScrollNode),
    Resident(i32),
}
impl ScrollNodeAccess<'_> {
    fn Resolve<'a>(&'a self, tree: &'a ScrollTree) -> &'a ScrollNode {
        match self {
            Self::External(node) => node,
            Self::Resident(id) => tree.Node(*id),
        }
    }
}
impl ScrollTree {
    // Source: cc/trees/property_tree.cc:1731-1734.
    pub fn new(owner: Option<PropertyTreesOwnerId>) -> Self {
        Self {
            base_: PropertyTree::new(owner),
            currently_scrolling_node_id_: kInvalidPropertyNodeId,
            scroll_offset_map_: BTreeMap::new(),
            synced_scroll_offset_map_: BTreeMap::new(),
            elastic_overscroll_: BTreeMap::new(),
            scrolling_contents_cull_rects_: BTreeMap::new(),
            callbacks_: None,
        }
    }
    // Source: cc/trees/property_tree.cc:1736-1765.
    pub fn AssignFrom(&mut self, from: &Self, from_state: &PropertyTreesState) -> &mut Self {
        self.base_.AssignFrom(&from.base_);
        self.scrolling_contents_cull_rects_ = from.scrolling_contents_cull_rects_.clone();
        if from_state.is_main_thread_ {
            self.scroll_offset_map_ = from.scroll_offset_map_.clone();
        }
        self.currently_scrolling_node_id_ = kInvalidPropertyNodeId;
        let mut overscroll = std::mem::take(&mut self.elastic_overscroll_);
        overscroll.retain(|id, _| {
            self.FindNodeFromElementId(*id).is_some_and(|node| {
                crate::cc::trees::scroll_elasticity_utils::ShouldAllowOverscrollEffect(
                    node, self, None,
                )
            })
        });
        self.elastic_overscroll_ = overscroll;
        self
    }
    // Source: cc/trees/property_tree.cc:1782-1787.
    #[cfg(debug_assertions)]
    pub fn CopyCompleteTreeState(&mut self, other: &Self) {
        self.currently_scrolling_node_id_ = other.currently_scrolling_node_id_;
        self.scroll_offset_map_ = other.scroll_offset_map_.clone();
        self.synced_scroll_offset_map_ = other.synced_scroll_offset_map_.clone();
        self.callbacks_ = other.callbacks_.clone();
    }
    // Source: cc/trees/property_tree.cc:1790-1803.
    pub fn CanRealizeScrollsOnActiveTree(&self, node: &ScrollNode) -> bool {
        node.transform_id != kInvalidPropertyNodeId
            && node.is_composited
            && node.main_thread_repaint_reasons.empty()
    }
    pub fn CanRealizeScrollsOnPendingTree(&self, node: &ScrollNode) -> bool {
        node.transform_id != kInvalidPropertyNodeId
            && !node.is_composited
            && node.main_thread_repaint_reasons.empty()
    }
    pub fn ShouldRealizeScrollsOnMain(&self, node: &ScrollNode) -> bool {
        node.transform_id != kInvalidPropertyNodeId && !node.main_thread_repaint_reasons.empty()
    }
    // Source: cc/trees/property_tree.cc:1805-1826.
    pub fn clear(&mut self, state: &PropertyTreesState) {
        self.base_.clear();
        if state.is_main_thread_ {
            self.currently_scrolling_node_id_ = kInvalidPropertyNodeId;
            self.scroll_offset_map_.clear();
        }
        self.scrolling_contents_cull_rects_.clear();
        #[cfg(debug_assertions)]
        {
            let mut tree = Self::new(None);
            if state.is_main_thread_ {
                tree.callbacks_ = self.callbacks_.clone();
            } else {
                debug_assert!(self.scroll_offset_map_.is_empty());
                tree.currently_scrolling_node_id_ = self.currently_scrolling_node_id_;
                tree.synced_scroll_offset_map_ = self.synced_scroll_offset_map_.clone();
            }
            debug_assert!(tree == *self);
        }
    }
    // Source: cc/trees/property_tree.cc:1828-1855.
    pub fn MaxScrollOffset<T: PropertyTreeTransform>(
        &self,
        id: i32,
        transforms: &TransformTree<T>,
        state: &PropertyTreesState,
    ) -> PointF {
        let node = self.Node(id);
        let scroll_bounds = self.scroll_bounds(id, state);
        if scroll_bounds.IsEmpty() {
            return PointF::default();
        }
        let scale_factor = if node.max_scroll_offset_affected_by_page_scale {
            transforms.page_scale_factor()
        } else {
            1.0
        };
        let scaled_width = (scroll_bounds.width() * scale_factor).floor();
        let scaled_height = (scroll_bounds.height() * scale_factor).floor();
        let clip_bounds = self.container_bounds(node.id, state);
        let mut max_offset = PointF::new(
            scaled_width - clip_bounds.width() as f32,
            scaled_height - clip_bounds.height() as f32,
        );
        max_offset.Scale(1.0 / scale_factor);
        max_offset.SetToMax(PointF::default());
        max_offset
    }
    // Source: cc/trees/property_tree.cc:1857-1865.
    pub fn scroll_bounds(&self, id: i32, state: &PropertyTreesState) -> SizeF {
        let node = self.Node(id);
        let mut bounds = SizeF::new(node.bounds.width() as f32, node.bounds.height() as f32);
        if node.scrolls_inner_viewport {
            let delta = state.outer_viewport_container_bounds_delta_;
            bounds.Enlarge(delta.x(), delta.y());
        }
        bounds
    }
    // Source: cc/trees/property_tree.cc:1867-1885.
    pub fn OnScrollOffsetAnimated<T: PropertyTreeTransform>(
        &mut self,
        id: ElementId,
        index: i32,
        offset: &PointF,
        layer_tree: &mut impl PropertyTreeScrollLayerTree<T>,
        context: &mut ScrollTreeMutationContext<'_, T>,
    ) {
        if !context.state.is_active_ {
            return;
        }
        let offset = self.ClampScrollOffsetToLimits(
            *offset,
            self.Node(index),
            context.transform_tree,
            context.state,
        );
        if self.SetScrollOffset(id, &offset, context.state) {
            layer_tree.DidUpdateScrollOffset(id, false, self, context);
        }
        layer_tree.DidAnimateScrollOffset();
    }
    // Source: cc/trees/property_tree.cc:1887-1904.
    pub fn container_bounds(&self, id: i32, state: &PropertyTreesState) -> Size {
        let node = self.Node(id);
        let mut bounds = node.container_bounds;
        let mut delta = Vector2dF::default();
        if node.scrolls_inner_viewport {
            delta += state.inner_viewport_container_bounds_delta_;
        } else if node.scrolls_outer_viewport {
            delta += state.outer_viewport_container_bounds_delta_;
        }
        bounds.Enlarge(delta.x().ceil() as i32, delta.y().ceil() as i32);
        bounds
    }
    // Source: cc/trees/property_tree.cc:1906-1928; cc/trees/property_tree.h:653.
    pub fn CurrentlyScrollingNode(&self) -> Option<&ScrollNode> {
        if self.currently_scrolling_node_id_ == kInvalidPropertyNodeId {
            None
        } else {
            Some(self.Node(self.currently_scrolling_node_id_))
        }
    }
    pub fn CurrentlyScrollingNodeMutable(&mut self) -> Option<&mut ScrollNode> {
        if self.currently_scrolling_node_id_ == kInvalidPropertyNodeId {
            None
        } else {
            Some({
                let id = self.currently_scrolling_node_id_;
                self.MutableNode(id)
            })
        }
    }
    #[cfg(debug_assertions)]
    pub fn CurrentlyScrollingNodeId(&self) -> i32 {
        self.currently_scrolling_node_id_
    }
    pub fn set_currently_scrolling_node(&mut self, id: i32) {
        self.currently_scrolling_node_id_ = id;
    }
    pub fn currently_scrolling_node(&self) -> i32 {
        self.currently_scrolling_node_id_
    }
    // Source: cc/trees/property_tree.cc:1930-1933.
    pub fn ScreenSpaceTransform<T: PropertyTreeTransform>(
        &self,
        id: i32,
        transforms: &TransformTree<T>,
    ) -> T {
        transforms.ToScreen(self.Node(id).transform_id).clone()
    }
    // Source: cc/trees/property_tree.cc:1935-1944.
    pub fn GetSyncedScrollOffset(
        &self,
        id: ElementId,
        state: &PropertyTreesState,
    ) -> Option<Ref<'_, SyncedScrollOffset>> {
        debug_assert!(!state.is_main_thread_);
        self.synced_scroll_offset_map_
            .get(&id)
            .map(|offset| offset.borrow())
    }
    pub fn GetSyncedScrollOffsetMutable(
        &self,
        id: ElementId,
        state: &PropertyTreesState,
    ) -> Option<RefMut<'_, SyncedScrollOffset>> {
        debug_assert!(!state.is_main_thread_);
        self.synced_scroll_offset_map_
            .get(&id)
            .map(|offset| offset.borrow_mut())
    }
    // Source: cc/trees/property_tree.cc:1946-1955.
    pub fn ClampScrollToMaxScrollOffset<T: PropertyTreeTransform>(
        &mut self,
        node: &ScrollNode,
        layer_tree: &mut impl PropertyTreeScrollLayerTree<T>,
        context: &mut ScrollTreeMutationContext<'_, T>,
    ) -> Vector2dF {
        self.ClampScrollToMaxScrollOffsetWithAccess(
            ScrollNodeAccess::External(node),
            layer_tree,
            context,
        )
    }
    pub fn ClampScrollToMaxScrollOffsetById<T: PropertyTreeTransform>(
        &mut self,
        id: i32,
        layer_tree: &mut impl PropertyTreeScrollLayerTree<T>,
        context: &mut ScrollTreeMutationContext<'_, T>,
    ) -> Vector2dF {
        self.ClampScrollToMaxScrollOffsetWithAccess(
            ScrollNodeAccess::Resident(id),
            layer_tree,
            context,
        )
    }
    // Source: cc/trees/property_tree.cc:1946-1955 (shared original algorithm).
    fn ClampScrollToMaxScrollOffsetWithAccess<T: PropertyTreeTransform>(
        &mut self,
        access: ScrollNodeAccess<'_>,
        layer_tree: &mut impl PropertyTreeScrollLayerTree<T>,
        context: &mut ScrollTreeMutationContext<'_, T>,
    ) -> Vector2dF {
        let node = access.Resolve(self);
        let old = self.current_scroll_offset(node.element_id, context.state);
        let clamped =
            self.ClampScrollOffsetToLimits(old, node, context.transform_tree, context.state);
        let delta = clamped - old;
        if !delta.IsZero() {
            self.ScrollByWithAccess(access, &delta, layer_tree, context);
        }
        delta
    }
    // Source: cc/trees/property_tree.cc:1957-1965.
    pub fn current_scroll_offset(&self, id: ElementId, state: &PropertyTreesState) -> PointF {
        if state.is_main_thread_ {
            return self
                .scroll_offset_map_
                .get(&id)
                .copied()
                .unwrap_or_default();
        }
        self.GetSyncedScrollOffset(id, state)
            .map_or(PointF::default(), |offset| offset.Current(state.is_active_))
    }
    // Source: cc/trees/property_tree.cc:1967-2019.
    pub fn GetScrollOffsetForScrollTimeline<T: PropertyTreeTransform>(
        &self,
        node: &ScrollNode,
        transforms: &mut TransformTree<T>,
        context: &mut TransformTreeContext<'_, T>,
    ) -> PointF {
        let mut offset = self.current_scroll_offset(node.element_id, context.state);
        if !context.state.is_main_thread_ {
            if let Some(synced) = self.GetSyncedScrollOffset(node.element_id, context.state) {
                if context.state.is_active_ {
                    if !self.CanRealizeScrollsOnActiveTree(node) {
                        offset = synced.ActiveBase();
                    }
                } else if !self.CanRealizeScrollsOnActiveTree(node)
                    && !self.CanRealizeScrollsOnPendingTree(node)
                {
                    offset = synced.PendingBase();
                }
            }
        }
        if node.transform_id == kInvalidPropertyNodeId {
            return offset;
        }
        if transforms.Node(node.transform_id).scrolls {
            if transforms
                .Node(node.transform_id)
                .needs_local_transform_update
            {
                transforms.UpdateTransforms(node.transform_id, None, None, context);
            }
            offset = self.ClampScrollOffsetToLimits(
                offset - transforms.Node(node.transform_id).snap_amount,
                node,
                transforms,
                context.state,
            );
        }
        offset
    }
    // Source: cc/trees/property_tree.cc:2021-2049.
    fn PullDeltaForMainThread(
        offset: &mut SyncedScrollOffset,
        fractional: bool,
        next_bmf: bool,
        state: &PropertyTreesState,
    ) -> Vector2dF {
        debug_assert!(state.is_active_);
        if fractional {
            return offset.PullDeltaForMainThread(next_bmf);
        }
        let current = offset.Current(true);
        let rounded = PointF::new(current.x().round(), current.y().round());
        let active = offset.ActiveBase();
        let diff = active - PointF::new(active.x().round(), active.y().round());
        offset.SetCurrent(rounded + diff);
        let delta = offset.PullDeltaForMainThread(next_bmf);
        offset.SetCurrent(current);
        delta
    }
    // Source: cc/trees/property_tree.cc:2051-2100.
    pub fn CollectScrollDeltas(
        &self,
        commit: &mut CompositorCommitData,
        inner: ElementId,
        fractional: bool,
        snapped: &BTreeMap<ElementId, TargetSnapAreaElementIds>,
        mutator: Option<&dyn PropertyTreeMutatorHost>,
        next_bmf: bool,
        state: &PropertyTreesState,
    ) {
        debug_assert!(!state.is_main_thread_);
        for (id, offset) in &self.synced_scroll_offset_map_ {
            let mut delta = Vector2dF::default();
            let clobber = mutator.is_some_and(|host| host.ScrollOffsetAnimationWasInterrupted(*id));
            if !clobber {
                delta = Self::PullDeltaForMainThread(
                    &mut offset.borrow_mut(),
                    fractional,
                    next_bmf,
                    state,
                );
            }
            let snap = snapped.get(id).cloned();
            if !delta.IsZero() || snap.is_some() {
                let update = ScrollUpdateInfo::new(*id, delta, snap);
                if *id == inner {
                    commit.inner_viewport_scroll = update;
                } else {
                    commit.scrolls.push(update);
                }
            }
        }
    }
    // Source: cc/trees/property_tree.cc:2102-2107.
    pub fn CollectScrollDeltasForTesting(&self, fractional: bool, state: &PropertyTreesState) {
        for offset in self.synced_scroll_offset_map_.values() {
            Self::PullDeltaForMainThread(&mut offset.borrow_mut(), fractional, false, state);
        }
    }
    // Source: cc/trees/property_tree.cc:2109-2172.
    pub fn PushScrollUpdatesFromMainThread<T: PropertyTreeTransform>(
        &mut self,
        main: &ScrollTree,
        layer_tree: &mut impl PropertyTreeScrollLayerTree<T>,
        fractional: bool,
        context: &mut ScrollTreeMutationContext<'_, T>,
    ) {
        debug_assert!(!context.state.is_main_thread_);
        self.synced_scroll_offset_map_.retain(|id, offset| {
            if !main.scroll_offset_map_.contains_key(id) {
                let mut offset = offset.borrow_mut();
                let current = offset.Current(true);
                offset.PushMainToPending(current);
                false
            } else {
                true
            }
        });
        for (id, main_offset) in &main.scroll_offset_map_ {
            let offset = self
                .synced_scroll_offset_map_
                .entry(*id)
                .or_insert_with(|| Rc::new(RefCell::new(SyncedScrollOffset::default())));
            let mut offset = offset.borrow_mut();
            let mut needs_update = offset.PushMainToPending(*main_offset);
            if !fractional {
                let pending = offset.Current(false);
                let rounded = PointF::new(pending.x().round(), pending.y().round());
                needs_update = *main_offset != rounded;
            }
            if context.state.is_active_ {
                needs_update |= offset.PushPendingToActive();
            }
            drop(offset);
            if needs_update {
                layer_tree.DidUpdateScrollOffset(*id, true, self, context);
            }
        }
    }
    // Source: cc/trees/property_tree.cc:2174-2193.
    pub fn PushScrollUpdatesFromPendingTree<T: PropertyTreeTransform>(
        &mut self,
        pending: &ScrollTree,
        pending_state: &PropertyTreesState,
        layer_tree: &mut impl PropertyTreeScrollLayerTree<T>,
        context: &mut ScrollTreeMutationContext<'_, T>,
    ) {
        debug_assert!(context.state.is_active_);
        debug_assert!(!pending_state.is_main_thread_);
        debug_assert!(!pending_state.is_active_);
        self.synced_scroll_offset_map_.clear();
        for (id, offset) in &pending.synced_scroll_offset_map_ {
            self.synced_scroll_offset_map_
                .insert(*id, Rc::clone(offset));
            let changed = offset.borrow_mut().PushPendingToActive();
            if changed {
                layer_tree.DidUpdateScrollOffset(*id, true, self, context);
            }
        }
    }
    // Source: cc/trees/property_tree.cc:2195-2201.
    pub fn ApplySentScrollDeltasFromAbortedCommit(
        &self,
        next_bmf: bool,
        applied: bool,
        state: &PropertyTreesState,
    ) {
        debug_assert!(state.is_active_);
        for offset in self.synced_scroll_offset_map_.values() {
            offset.borrow_mut().AbortCommit(next_bmf, applied);
        }
    }
    // Source: cc/trees/property_tree.cc:2203-2215.
    pub fn SetBaseScrollOffset(
        &mut self,
        id: ElementId,
        offset: &PointF,
        state: &PropertyTreesState,
    ) {
        if !offset.x().is_finite() || !offset.y().is_finite() {
            return;
        }
        if state.is_main_thread_ {
            self.scroll_offset_map_.insert(id, *offset);
            return;
        }
        debug_assert!(self.GetSyncedScrollOffset(id, state).is_some());
        self.GetSyncedScrollOffsetMutable(id, state)
            .unwrap()
            .PushMainToPending(*offset);
    }
    // Source: cc/trees/property_tree.cc:2217-2239.
    pub fn SetScrollOffset(
        &mut self,
        id: ElementId,
        offset: &PointF,
        state: &PropertyTreesState,
    ) -> bool {
        if !offset.x().is_finite() || !offset.y().is_finite() {
            return false;
        }
        if state.is_main_thread_ {
            let current = self.scroll_offset_map_.entry(id).or_default();
            if *current == *offset {
                return false;
            }
            *current = *offset;
            return true;
        }
        if state.is_active_ {
            if let Some(mut synced) = self.GetSyncedScrollOffsetMutable(id, state) {
                return synced.SetCurrent(*offset);
            }
        }
        false
    }
    // Source: cc/trees/property_tree.h:694-698.
    pub fn SetScrollOffsetClobberActiveValue(&self, id: ElementId, state: &PropertyTreesState) {
        if let Some(mut offset) = self.GetSyncedScrollOffsetMutable(id, state) {
            offset.set_clobber_active_value();
        }
    }
    // Source: cc/trees/property_tree.cc:2241-2266.
    pub fn SetElasticOverscroll(&mut self, node: &ScrollNode, overscroll: &Vector2dF) -> bool {
        self.SetElasticOverscrollForElementId(node.element_id, overscroll)
    }
    pub fn SetElasticOverscrollById(&mut self, id: i32, overscroll: &Vector2dF) -> bool {
        self.SetElasticOverscrollForElementId(self.Node(id).element_id, overscroll)
    }
    // Source: cc/trees/property_tree.cc:2241-2252 (shared original algorithm).
    fn SetElasticOverscrollForElementId(
        &mut self,
        element_id: ElementId,
        overscroll: &Vector2dF,
    ) -> bool {
        if overscroll.IsZero() {
            return self.elastic_overscroll_.remove(&element_id).is_some();
        }
        let current = self.elastic_overscroll_.entry(element_id).or_default();
        let changed = *current != *overscroll;
        *current = *overscroll;
        changed
    }
    pub fn GetElasticOverscroll(&self, node: &ScrollNode) -> Vector2dF {
        self.GetElasticOverscrollFromElementId(node.element_id)
    }
    pub fn GetElasticOverscrollFromElementId(&self, id: ElementId) -> Vector2dF {
        self.elastic_overscroll_
            .get(&id)
            .copied()
            .unwrap_or_default()
    }
    // Source: cc/trees/property_tree.cc:2268-2301.
    pub fn FindElasticOverscrollFromTransformId(
        &self,
        id: i32,
        viewport: Option<&ViewportPropertyIds>,
    ) -> (ElementId, Vector2dF) {
        if viewport.is_some_and(|viewport| id == viewport.overscroll_elasticity_transform) {
            let viewport = viewport.unwrap();
            if viewport.inner_scroll != kInvalidPropertyNodeId {
                let node = self.Node(viewport.inner_scroll);
                if let Some(amount) = self.elastic_overscroll_.get(&node.element_id) {
                    return (node.element_id, *amount);
                }
            }
        } else {
            for (element, amount) in &self.elastic_overscroll_ {
                if self
                    .FindNodeFromElementId(*element)
                    .is_some_and(|node| node.transform_id == id)
                {
                    return (*element, *amount);
                }
            }
        }
        (ElementId::default(), Vector2dF::default())
    }
    // Source: cc/trees/property_tree.cc:2303-2318.
    pub fn SetScrollingContentsCullRect(&mut self, id: ElementId, rect: &Rect) {
        self.scrolling_contents_cull_rects_.insert(id, *rect);
    }
    pub fn ClearScrollingContentsCullRect(&mut self, id: ElementId) {
        self.scrolling_contents_cull_rects_.remove(&id);
    }
    pub fn ScrollingContentsCullRect(&self, id: ElementId) -> Option<&Rect> {
        self.scrolling_contents_cull_rects_.get(&id)
    }
    // Source: cc/trees/property_tree.cc:2320-2328.
    pub fn GetOrCreateSyncedScrollOffsetForTesting(
        &mut self,
        id: ElementId,
    ) -> RefMut<'_, SyncedScrollOffset> {
        self.synced_scroll_offset_map_
            .entry(id)
            .or_insert_with(|| Rc::new(RefCell::new(SyncedScrollOffset::default())))
            .borrow_mut()
    }
    // Source: cc/trees/property_tree.cc:2330-2339.
    pub fn UpdateScrollOffsetBaseForTesting(
        &mut self,
        id: ElementId,
        offset: &PointF,
        state: &PropertyTreesState,
    ) -> bool {
        debug_assert!(!state.is_main_thread_);
        let mut synced = self.GetOrCreateSyncedScrollOffsetForTesting(id);
        let mut changed = synced.PushMainToPending(*offset);
        if state.is_active_ {
            changed |= synced.PushPendingToActive();
        }
        changed
    }
    // Source: cc/trees/property_tree.cc:2341-2347.
    pub fn SetScrollOffsetDeltaForTesting(&mut self, id: ElementId, delta: &Vector2dF) -> bool {
        let mut synced = self.GetOrCreateSyncedScrollOffsetForTesting(id);
        let base = synced.ActiveBase();
        synced.SetCurrent(base + *delta)
    }
    // Source: cc/trees/property_tree.cc:2349-2369.
    pub fn GetScrollOffsetBaseForTesting(
        &self,
        id: ElementId,
        state: &PropertyTreesState,
    ) -> PointF {
        debug_assert!(!state.is_main_thread_);
        self.GetSyncedScrollOffset(id, state)
            .map_or(PointF::default(), |synced| {
                if state.is_active_ {
                    synced.ActiveBase()
                } else {
                    synced.PendingBase()
                }
            })
    }
    pub fn GetScrollOffsetDeltaForTesting(
        &self,
        id: ElementId,
        state: &PropertyTreesState,
    ) -> Vector2dF {
        debug_assert!(!state.is_main_thread_);
        self.GetSyncedScrollOffset(id, state)
            .map_or(Vector2dF::default(), |synced| {
                if state.is_active_ {
                    synced.Delta()
                } else {
                    synced.PendingDelta()
                }
            })
    }
    // Source: cc/trees/property_tree.cc:2371-2395.
    pub fn ScrollBy<T: PropertyTreeTransform>(
        &mut self,
        node: &ScrollNode,
        scroll: &Vector2dF,
        layer_tree: &mut impl PropertyTreeScrollLayerTree<T>,
        context: &mut ScrollTreeMutationContext<'_, T>,
    ) -> Vector2dF {
        self.ScrollByWithAccess(
            ScrollNodeAccess::External(node),
            scroll,
            layer_tree,
            context,
        )
    }
    pub fn ScrollByNodeId<T: PropertyTreeTransform>(
        &mut self,
        id: i32,
        scroll: &Vector2dF,
        layer_tree: &mut impl PropertyTreeScrollLayerTree<T>,
        context: &mut ScrollTreeMutationContext<'_, T>,
    ) -> Vector2dF {
        self.ScrollByWithAccess(ScrollNodeAccess::Resident(id), scroll, layer_tree, context)
    }
    // Source: cc/trees/property_tree.cc:2371-2395 (shared original algorithm).
    fn ScrollByWithAccess<T: PropertyTreeTransform>(
        &mut self,
        access: ScrollNodeAccess<'_>,
        scroll: &Vector2dF,
        layer_tree: &mut impl PropertyTreeScrollLayerTree<T>,
        context: &mut ScrollTreeMutationContext<'_, T>,
    ) -> Vector2dF {
        let node = access.Resolve(self);
        let mut adjusted = *scroll;
        if !node.user_scrollable_horizontal {
            adjusted.set_x(0.0);
        }
        if !node.user_scrollable_vertical {
            adjusted.set_y(0.0);
        }
        let old = self.current_scroll_offset(node.element_id, context.state);
        let new = self.ClampScrollOffsetToLimits(
            old + adjusted,
            node,
            context.transform_tree,
            context.state,
        );
        let element_id = node.element_id;
        if self.SetScrollOffset(element_id, &new, context.state) {
            layer_tree.DidUpdateScrollOffset(element_id, false, self, context);
        }
        old + *scroll - new
    }
    // Source: cc/trees/property_tree.cc:2397-2403.
    pub fn ClampScrollOffsetToLimits<T: PropertyTreeTransform>(
        &self,
        mut offset: PointF,
        node: &ScrollNode,
        transforms: &TransformTree<T>,
        state: &PropertyTreesState,
    ) -> PointF {
        offset.SetToMin(self.MaxScrollOffset(node.id, transforms, state));
        offset.SetToMax(PointF::default());
        offset
    }
    // Source: cc/trees/property_tree.cc:2405-2427.
    pub fn SetScrollCallbacks(
        &mut self,
        callbacks: Weak<dyn ScrollCallbacks>,
        state: &PropertyTreesState,
    ) {
        debug_assert!(state.is_main_thread_);
        self.callbacks_ = Some(callbacks);
    }
    pub fn NotifyDidCompositorScroll(
        &self,
        id: ElementId,
        offset: &PointF,
        source: ScrollSourceType,
        snap: &Option<TargetSnapAreaElementIds>,
        state: &PropertyTreesState,
    ) {
        debug_assert!(state.is_main_thread_);
        if let Some(callback) = self.callbacks_.as_ref().and_then(Weak::upgrade) {
            callback.DidCompositorScroll(id, offset, source, snap);
        }
    }
    pub fn NotifyDidChangeScrollbarsHidden(
        &self,
        id: ElementId,
        hidden: bool,
        state: &PropertyTreesState,
    ) {
        debug_assert!(state.is_main_thread_);
        if let Some(callback) = self.callbacks_.as_ref().and_then(Weak::upgrade) {
            callback.DidChangeScrollbarsHidden(id, hidden);
        }
    }
    // Source: cc/trees/property_tree.h:752-772.
    pub fn synced_scroll_offset_map(&self) -> &SyncedScrollOffsetMap {
        &self.synced_scroll_offset_map_
    }
    pub fn synced_scroll_offset_map_mut(&mut self) -> &mut SyncedScrollOffsetMap {
        &mut self.synced_scroll_offset_map_
    }
    pub fn scrolling_contents_cull_rects(&self) -> &BTreeMap<ElementId, Rect> {
        &self.scrolling_contents_cull_rects_
    }
    pub fn scrolling_contents_cull_rects_mut(&mut self) -> &mut BTreeMap<ElementId, Rect> {
        &mut self.scrolling_contents_cull_rects_
    }
    pub fn elastic_overscroll(&self) -> &BTreeMap<ElementId, Vector2dF> {
        &self.elastic_overscroll_
    }
    pub fn elastic_overscroll_mut(&mut self) -> &mut BTreeMap<ElementId, Vector2dF> {
        &mut self.elastic_overscroll_
    }
}
// Source: cc/trees/property_tree.cc:1767-1780.
#[cfg(debug_assertions)]
impl PartialEq for ScrollTree {
    fn eq(&self, other: &Self) -> bool {
        if self.scroll_offset_map_ != other.scroll_offset_map_ {
            return false;
        }
        if self.synced_scroll_offset_map_.len() != other.synced_scroll_offset_map_.len()
            || !self.synced_scroll_offset_map_.iter().all(|(id, value)| {
                other
                    .synced_scroll_offset_map_
                    .get(id)
                    .is_some_and(|other| Rc::ptr_eq(value, other))
            })
        {
            return false;
        }
        let callback_eq = match (&self.callbacks_, &other.callbacks_) {
            (None, None) => true,
            (Some(a), Some(b)) => Weak::ptr_eq(a, b),
            _ => false,
        };
        if !callback_eq {
            return false;
        }
        self.base_ == other.base_
            && self.currently_scrolling_node_id_ == other.currently_scrolling_node_id_
    }
}
impl crate::adapter::scroll_elasticity::ScrollTree for ScrollTree {
    fn CanRealizeScrollsOnActiveTree(&self, node: &ScrollNode) -> bool {
        ScrollTree::CanRealizeScrollsOnActiveTree(self, node)
    }
    fn CanRealizeScrollsOnPendingTree(&self, node: &ScrollNode) -> bool {
        ScrollTree::CanRealizeScrollsOnPendingTree(self, node)
    }
}

use super::clip_node::ClipRectData;
use super::property_animation_state::PropertyAnimationState;
use super::target_property::{
    PropertyToElementIdMap, Type as TargetProperty, ALL_TARGET_PROPERTIES,
};
// Source: cc/trees/property_tree.cc:2429-2434.
impl<T: PropertyTreeTransform> Default for PropertyTreesCachedData<T> {
    fn default() -> Self {
        Self {
            transform_tree_update_number: 0,
            animation_scales: Vec::new(),
            draw_transforms: RefCell::new(Vec::new()),
        }
    }
}
impl<T: PropertyTreeTransform> PropertyTreesCachedData<T> {
    // Source: cc/trees/property_tree.cc:3054-3056.
    pub fn UpdateTransformTreeUpdateNumber(&mut self) {
        self.transform_tree_update_number += 1;
    }
}
// Source: cc/trees/property_tree.h:864-1040.
pub struct PropertyTrees<
    T: PropertyTreeTransform,
    F: PropertyTreeFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
> {
    owner_: PropertyTreesOwnerId,
    transform_tree_: TransformTree<T>,
    effect_tree_: EffectTree<F, P, R, C>,
    clip_tree_: ClipTree,
    scroll_tree_: ScrollTree,
    state_: PropertyTreesState,
    changed_effect_nodes_: Vec<i32>,
    changed_transform_nodes_: Vec<i32>,
    surface_property_changed_flags_: Vec<RenderSurfacePropertyChangedFlags>,
    cached_data_: PropertyTreesCachedData<T>,
}
impl<
        T: PropertyTreeTransform,
        F: PropertyTreeFilters,
        P: PropertyPath,
        R: PropertyTreeRenderSurface,
        C: PropertyTreeCopyOutputRequest,
    > PropertyTrees<T, F, P, R, C>
{
    // Source: cc/trees/property_tree.cc:2436-2442.
    // Caller supplies a stable identity in place of the implicit source `this`.
    pub fn new(owner: PropertyTreesOwnerId) -> Self {
        Self {
            owner_: owner,
            transform_tree_: TransformTree::new(Some(owner)),
            effect_tree_: EffectTree::new(Some(owner)),
            clip_tree_: ClipTree::new(Some(owner)),
            scroll_tree_: ScrollTree::new(Some(owner)),
            state_: PropertyTreesState::default(),
            changed_effect_nodes_: Vec::new(),
            changed_transform_nodes_: Vec::new(),
            surface_property_changed_flags_: Vec::new(),
            cached_data_: PropertyTreesCachedData::default(),
        }
    }
    // Source: cc/trees/property_tree.h:884-927.
    pub fn clip_tree(&self) -> &ClipTree {
        &self.clip_tree_
    }
    pub fn clip_tree_mutable(&mut self) -> &mut ClipTree {
        &mut self.clip_tree_
    }
    pub fn effect_tree(&self) -> &EffectTree<F, P, R, C> {
        &self.effect_tree_
    }
    pub fn effect_tree_mutable(&mut self) -> &mut EffectTree<F, P, R, C> {
        &mut self.effect_tree_
    }
    pub fn scroll_tree(&self) -> &ScrollTree {
        &self.scroll_tree_
    }
    pub fn scroll_tree_mutable(&mut self) -> &mut ScrollTree {
        &mut self.scroll_tree_
    }
    pub fn transform_tree(&self) -> &TransformTree<T> {
        &self.transform_tree_
    }
    pub fn transform_tree_mutable(&mut self) -> &mut TransformTree<T> {
        &mut self.transform_tree_
    }
    // Source algorithm: cc/trees/property_tree.cc:1655-1667.
    // Rust borrow bridge: pass the resident sibling tree/cache directly to the
    // canonical EffectTree method without exposing or copying private state.
    pub(crate) fn ClippedHitTestRegionIsRectangle(&self, effect_id: i32) -> bool {
        self.effect_tree_.ClippedHitTestRegionIsRectangle(
            effect_id,
            &self.transform_tree_,
            &self.cached_data_,
        )
    }
    // Source algorithm: cc/trees/property_tree.cc:1415-1501.
    // Rust borrow bridge: pass the resident sibling tree/cache directly to the
    // canonical EffectTree method without exposing or copying private state.
    pub(crate) fn TakeCopyRequestsAndTransformToSurface(
        &mut self,
        effect_id: i32,
        requests: &mut Vec<Box<C>>,
    ) {
        self.effect_tree_.TakeCopyRequestsAndTransformToSurface(
            effect_id,
            requests,
            &self.transform_tree_,
            &self.cached_data_,
        );
    }
    pub fn set_needs_rebuild(&mut self, value: bool) {
        self.state_.needs_rebuild_ = value;
    }
    pub fn needs_rebuild(&self) -> bool {
        self.state_.needs_rebuild_
    }
    pub fn set_changed(&mut self, value: bool) {
        self.state_.changed_ = value;
    }
    pub fn changed(&self) -> bool {
        self.state_.changed_
    }
    pub fn set_full_tree_damaged(&mut self, value: bool) {
        self.state_.full_tree_damaged_ = value;
    }
    pub fn full_tree_damaged(&self) -> bool {
        self.state_.full_tree_damaged_
    }
    pub fn set_is_main_thread(&mut self, value: bool) {
        self.state_.is_main_thread_ = value;
    }
    pub fn is_main_thread(&self) -> bool {
        self.state_.is_main_thread_
    }
    pub fn set_is_active(&mut self, value: bool) {
        self.state_.is_active_ = value;
    }
    pub fn is_active(&self) -> bool {
        self.state_.is_active_
    }
    pub fn set_sequence_number(&mut self, value: i32) {
        self.state_.sequence_number_ = value;
    }
    pub fn sequence_number(&self) -> i32 {
        self.state_.sequence_number_
    }
    pub fn increment_sequence_number(&mut self) {
        self.state_.sequence_number_ += 1;
    }
    pub fn changed_effect_nodes(&self) -> &Vec<i32> {
        &self.changed_effect_nodes_
    }
    pub fn changed_transform_nodes(&self) -> &Vec<i32> {
        &self.changed_transform_nodes_
    }
    pub fn surface_property_changed_flags(&self) -> &Vec<RenderSurfacePropertyChangedFlags> {
        &self.surface_property_changed_flags_
    }
    // Rust-only split borrows resolve the non-owning backpointer to resident trees.
    pub fn transform_tree_with_context(
        &mut self,
    ) -> (&mut TransformTree<T>, TransformTreeContext<'_, T>) {
        (
            &mut self.transform_tree_,
            TransformTreeContext {
                owner: self.owner_,
                scroll_tree: &self.scroll_tree_,
                clip_tree: &mut self.clip_tree_,
                state: &mut self.state_,
                cached_data: &mut self.cached_data_,
            },
        )
    }
    pub fn effect_tree_with_context<'a>(
        &'a mut self,
        features: &'a dyn PropertyTreeFeatures,
    ) -> (&'a mut EffectTree<F, P, R, C>, EffectTreeContext<'a, T>) {
        (
            &mut self.effect_tree_,
            EffectTreeContext {
                transform_tree: &self.transform_tree_,
                clip_tree: &mut self.clip_tree_,
                state: &mut self.state_,
                cached_data: &mut self.cached_data_,
                features,
            },
        )
    }
    pub fn scroll_tree_with_context(
        &mut self,
    ) -> (&mut ScrollTree, &TransformTree<T>, &PropertyTreesState) {
        (&mut self.scroll_tree_, &self.transform_tree_, &self.state_)
    }
    pub fn scroll_tree_mutation_context(
        &mut self,
    ) -> (&mut ScrollTree, ScrollTreeMutationContext<'_, T>) {
        (
            &mut self.scroll_tree_,
            ScrollTreeMutationContext {
                owner: self.owner_,
                transform_tree: &mut self.transform_tree_,
                clip_tree: &mut self.clip_tree_,
                state: &mut self.state_,
                cached_data: &mut self.cached_data_,
            },
        )
    }
    /// Ownership bridge for the source owner's effect-tree backpointer.
    pub fn TakeCopyRequests(&mut self) -> CopyRequestMap<C> {
        self.effect_tree_.TakeCopyRequests(&mut self.state_)
    }
    pub fn effect_tree_with_state(
        &mut self,
    ) -> (&mut EffectTree<F, P, R, C>, &mut PropertyTreesState) {
        (&mut self.effect_tree_, &mut self.state_)
    }
    /// Rust ownership bridge: borrows resident state; no data is copied.
    pub fn state(&self) -> &PropertyTreesState {
        &self.state_
    }
    pub fn scroll_tree_read_context(
        &self,
    ) -> (&ScrollTree, &TransformTree<T>, &PropertyTreesState) {
        (&self.scroll_tree_, &self.transform_tree_, &self.state_)
    }
    // Source: cc/trees/property_tree.cc:2465-2491.
    pub fn AssignFrom(&mut self, from: &Self) -> &mut Self {
        self.transform_tree_.AssignFrom(&from.transform_tree_);
        self.effect_tree_.AssignFrom(&from.effect_tree_);
        self.clip_tree_.AssignFrom(&from.clip_tree_);
        self.scroll_tree_
            .AssignFrom(&from.scroll_tree_, &from.state_);
        self.set_needs_rebuild(from.needs_rebuild());
        self.set_changed(from.changed());
        self.set_full_tree_damaged(from.full_tree_damaged());
        self.set_sequence_number(from.sequence_number());
        self.set_is_main_thread(from.is_main_thread());
        self.set_is_active(from.is_active());
        self.SetInnerViewportContainerBoundsDelta(from.inner_viewport_container_bounds_delta());
        self.SetOuterViewportContainerBoundsDelta(from.outer_viewport_container_bounds_delta());
        self.SetTransformDeltaBySafeAreaInsetBottom(
            from.transform_delta_by_safe_area_inset_bottom(),
        );
        self.changed_effect_nodes_ = from.changed_effect_nodes_.clone();
        self.changed_transform_nodes_ = from.changed_transform_nodes_.clone();
        self.surface_property_changed_flags_ = from.surface_property_changed_flags_.clone();
        self.transform_tree_.SetPropertyTrees(Some(self.owner_));
        self.effect_tree_.SetPropertyTrees(Some(self.owner_));
        self.clip_tree_.SetPropertyTrees(Some(self.owner_));
        self.scroll_tree_.SetPropertyTrees(Some(self.owner_));
        self.ResetCachedData();
        self
    }
    // Source: cc/trees/property_tree.cc:2493-2521.
    pub fn clear(&mut self) {
        self.transform_tree_.clear();
        self.clip_tree_.clear();
        self.effect_tree_.clear();
        self.scroll_tree_.clear(&self.state_);
        self.set_needs_rebuild(true);
        self.set_full_tree_damaged(false);
        self.set_changed(false);
        self.increment_sequence_number();
        self.changed_effect_nodes_.clear();
        self.changed_transform_nodes_.clear();
        self.surface_property_changed_flags_.clear();
        #[cfg(debug_assertions)]
        {
            let mut tree = Self::new(self.owner_);
            tree.transform_tree_.AssignFrom(&self.transform_tree_);
            tree.effect_tree_.AssignFrom(&self.effect_tree_);
            tree.clip_tree_.AssignFrom(&self.clip_tree_);
            tree.scroll_tree_
                .AssignFrom(&self.scroll_tree_, &self.state_);
            tree.scroll_tree_.CopyCompleteTreeState(&self.scroll_tree_);
            tree.set_sequence_number(self.sequence_number());
            tree.set_is_main_thread(self.is_main_thread());
            tree.set_is_active(self.is_active());
            debug_assert!(tree == *self);
        }
    }
    // Source: cc/trees/property_tree.cc:2523-2547; cc/trees/property_tree.h:961-977.
    pub fn SetInnerViewportContainerBoundsDelta(&mut self, value: Vector2dF) {
        if self.inner_viewport_container_bounds_delta() == value {
            return;
        }
        self.state_.inner_viewport_container_bounds_delta_ = value;
    }
    pub fn SetOuterViewportContainerBoundsDelta(&mut self, value: Vector2dF) {
        if self.outer_viewport_container_bounds_delta() == value {
            return;
        }
        self.state_.outer_viewport_container_bounds_delta_ = value;
        let (tree, mut context) = self.transform_tree_with_context();
        tree.UpdateOuterViewportContainerBoundsDelta(&mut context);
    }
    pub fn SetTransformDeltaBySafeAreaInsetBottom(&mut self, value: f32) {
        if self.transform_delta_by_safe_area_inset_bottom() == value {
            return;
        }
        self.state_.transform_delta_by_safe_area_inset_bottom_ = value;
        let (tree, mut context) = self.transform_tree_with_context();
        tree.NeedTransformUpdateForSafeAreaInsetBottom(&mut context);
    }
    pub fn inner_viewport_container_bounds_delta(&self) -> Vector2dF {
        self.state_.inner_viewport_container_bounds_delta_
    }
    pub fn outer_viewport_container_bounds_delta(&self) -> Vector2dF {
        self.state_.outer_viewport_container_bounds_delta_
    }
    pub fn inner_viewport_scroll_bounds_delta(&self) -> Vector2dF {
        self.outer_viewport_container_bounds_delta()
    }
    pub fn transform_delta_by_safe_area_inset_bottom(&self) -> f32 {
        self.state_.transform_delta_by_safe_area_inset_bottom_
    }
    // Source: cc/trees/property_tree.cc:2549-2654.
    pub fn ElementIsAnimatingChanged(
        &mut self,
        map: &PropertyToElementIdMap,
        mask: &PropertyAnimationState,
        state: &PropertyAnimationState,
        check_node_existence: bool,
    ) -> bool {
        let mut updated_transform = false;
        for property in ALL_TARGET_PROPERTIES {
            let index = property as usize;
            if !mask.currently_running.test(index) && !mask.potentially_animating.test(index) {
                continue;
            }
            let Some(element) = map.get(&property) else {
                continue;
            };
            match property {
                TargetProperty::TRANSFORM
                | TargetProperty::SCALE
                | TargetProperty::ROTATE
                | TargetProperty::TRANSLATE => {
                    if let Some(node) = self.transform_tree_.MutableFindNodeFromElementId(*element)
                    {
                        if mask.currently_running.test(index) {
                            node.is_currently_animating = state.currently_running.test(index);
                        }
                        if mask.potentially_animating.test(index) {
                            node.has_potential_animation = state.potentially_animating.test(index);
                            let (tree, mut context) = self.transform_tree_with_context();
                            tree.set_needs_update(true, &mut context);
                            updated_transform = true;
                        }
                    } else {
                        debug_assert!(
                            !check_node_existence
                                || (!state.currently_running.test(index)
                                    && !state.potentially_animating.test(index))
                                || self.needs_rebuild(),
                            "Attempting to animate non existent transform node"
                        );
                    }
                }
                TargetProperty::OPACITY => {
                    if let Some(node) = self.effect_tree_.MutableFindNodeFromElementId(*element) {
                        if mask.potentially_animating.test(index) {
                            node.has_potential_opacity_animation =
                                state.potentially_animating.test(index);
                            self.effect_tree_.set_needs_update(true);
                        }
                    } else {
                        debug_assert!(
                            !check_node_existence
                                || (!state.currently_running.test(index)
                                    && !state.potentially_animating.test(index))
                                || self.needs_rebuild(),
                            "Attempting to animate opacity on non existent effect node"
                        );
                    }
                }
                TargetProperty::FILTER => {
                    if let Some(node) = self.effect_tree_.MutableFindNodeFromElementId(*element) {
                        if mask.potentially_animating.test(index) {
                            node.has_potential_filter_animation =
                                state.potentially_animating.test(index);
                        }
                    } else {
                        debug_assert!(
                            !check_node_existence
                                || (!state.currently_running.test(index)
                                    && !state.potentially_animating.test(index))
                                || self.needs_rebuild(),
                            "Attempting to animate filter on non existent effect node"
                        );
                    }
                }
                TargetProperty::BACKDROP_FILTER => {
                    if let Some(node) = self.effect_tree_.MutableFindNodeFromElementId(*element) {
                        if mask.potentially_animating.test(index) {
                            node.has_potential_backdrop_filter_animation =
                                state.potentially_animating.test(index);
                        }
                    } else {
                        debug_assert!(
                            !check_node_existence
                                || (!state.currently_running.test(index)
                                    && !state.potentially_animating.test(index))
                                || self.needs_rebuild(),
                            "Attempting to animate filter on non existent effect node"
                        );
                    }
                }
                _ => {}
            }
        }
        updated_transform
    }
    // Source: cc/trees/property_tree.cc:2656-2663.
    pub fn MaximumAnimationScaleChanged(&mut self, id: ElementId, scale: f32) {
        if let Some(node) = self.transform_tree_.MutableFindNodeFromElementId(id) {
            node.maximum_animation_scale = scale;
            self.UpdateTransformTreeUpdateNumber();
        }
    }
    // Source: cc/trees/property_tree.cc:2665-2682.
    pub fn UpdateChangeTracking(&mut self) {
        for id in kContentsRootPropertyNodeId..self.effect_tree_.size() as i32 {
            let parent_id = self.effect_tree_.Node(id).parent_id;
            let (node, parent) = if parent_id == kInvalidPropertyNodeId {
                (&mut self.effect_tree_.base_.nodes_[id as usize], None)
            } else {
                let (parents, nodes) = self.effect_tree_.base_.nodes_.split_at_mut(id as usize);
                (&mut nodes[0], Some(&parents[parent_id as usize]))
            };
            EffectTree::<F, P, R, C>::UpdateEffectChanged(node, parent);
        }
        for id in kContentsRootPropertyNodeId..self.transform_tree_.size() as i32 {
            self.transform_tree_.UpdateTransformChanged(id);
        }
    }
    // Source: cc/trees/property_tree.cc:2684-2698.
    pub fn GetChangedNodes(&self, effects: &mut Vec<i32>, transforms: &mut Vec<i32>) {
        for id in kContentsRootPropertyNodeId..self.effect_tree_.size() as i32 {
            if self.effect_tree_.Node(id).effect_changed {
                effects.push(id);
            }
        }
        for id in kContentsRootPropertyNodeId..self.transform_tree_.size() as i32 {
            if self.transform_tree_.Node(id).transform_changed() {
                transforms.push(id);
            }
        }
    }
    // Source: cc/trees/property_tree.cc:2700-2713.
    pub fn ApplyChangedNodes(&mut self, effects: &[i32], transforms: &[i32]) {
        if !effects.is_empty() || !transforms.is_empty() {
            for id in effects {
                self.effect_tree_.MutableNode(*id).effect_changed = true;
            }
            for id in transforms {
                self.transform_tree_
                    .MutableNode(*id)
                    .SetTransformChanged(DamageReason::kUntracked);
            }
            self.UpdateChangeTracking();
        }
    }
    // Source: cc/trees/property_tree.cc:2715-2718.
    pub fn CollectChangeState(&mut self) {
        // Split disjoint resident fields; append directly to native tracking Vecs.
        for id in kContentsRootPropertyNodeId..self.effect_tree_.size() as i32 {
            if self.effect_tree_.Node(id).effect_changed {
                self.changed_effect_nodes_.push(id);
            }
        }
        for id in kContentsRootPropertyNodeId..self.transform_tree_.size() as i32 {
            if self.transform_tree_.Node(id).transform_changed() {
                self.changed_transform_nodes_.push(id);
            }
        }
        self.effect_tree_
            .GetRenderSurfaceChangedFlags(&mut self.surface_property_changed_flags_);
    }
    // Source: cc/trees/property_tree.cc:2720-2727.
    pub fn TakeChangeStateFrom(&mut self, source: &mut Self) {
        let mut requests = source.effect_tree_.TakeCopyRequests(&mut source.state_);
        self.effect_tree_.PullCopyRequestsFrom(&mut requests);
        self.CollectChangeState();
    }
    // Source: cc/trees/property_tree.cc:2729-2740.
    pub fn ApplyChangeStateFrom(&mut self, source: &mut Self) {
        self.state_.changed_ |= source.changed();
        self.state_.needs_rebuild_ |= source.needs_rebuild();
        self.state_.full_tree_damaged_ |= source.full_tree_damaged();
        let mut requests = source.effect_tree_.TakeCopyRequests(&mut source.state_);
        for (key, mut values) in self.effect_tree_.TakeCopyRequests(&mut self.state_) {
            requests.entry(key).or_default().append(&mut values);
        }
        self.effect_tree_.PullCopyRequestsFrom(&mut requests);
        self.ApplyChangedNodes(
            &source.changed_effect_nodes_,
            &source.changed_transform_nodes_,
        );
    }
    // Source: cc/trees/property_tree.cc:2742-2750.
    pub fn ResetAllChangeTracking(&mut self) {
        self.transform_tree_.ResetChangeTracking();
        self.effect_tree_.ResetChangeTracking();
        self.set_changed(false);
        self.set_full_tree_damaged(false);
        self.changed_effect_nodes_.clear();
        self.changed_transform_nodes_.clear();
        self.surface_property_changed_flags_.clear();
    }
    // Source: cc/trees/property_tree.cc:2752-2783.
    pub fn AsTracedValue(&self) -> TracedValueJSON {
        let mut value = TracedValueJSON::default();
        self.AsValueInto(&mut value);
        value
    }
    pub fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        value.SetInteger("sequence_number", self.sequence_number());
        value.BeginDictionary("transform_tree");
        self.transform_tree_.AsValueInto(value);
        value.EndDictionary();
        value.BeginDictionary("effect_tree");
        self.effect_tree_.AsValueInto(value);
        value.EndDictionary();
        value.BeginDictionary("clip_tree");
        self.clip_tree_.AsValueInto(value);
        value.EndDictionary();
        value.BeginDictionary("scroll_tree");
        self.scroll_tree_.AsValueInto(value);
        value.EndDictionary();
    }
    pub fn ToString(&self) -> String {
        self.AsTracedValue().ToFormattedJSON()
    }
    // Source: cc/trees/property_tree.cc:2785-2804.
    pub fn AnimationScaleCacheIsInvalid(&self, id: i32) -> bool {
        debug_assert!(!self.is_main_thread());
        assert!(id >= 0 && (id as usize) < self.cached_data_.animation_scales.len());
        self.cached_data_.animation_scales[id as usize].update_number == kInvalidUpdateNumber
    }
    pub fn MaximumAnimationToScreenScale(&mut self, id: i32) -> f32 {
        self.GetAnimationScaleData(id).maximum_to_screen_scale
    }
    pub fn AnimationAffectedByInvalidScale(&mut self, id: i32) -> bool {
        self.GetAnimationScaleData(id).affected_by_invalid_scale
    }
    // Source: cc/trees/property_tree.cc:2806-2878.
    fn GetAnimationScaleData(&mut self, id: i32) -> &AnimationScaleData {
        debug_assert!(!self.is_main_thread());
        assert!(id >= 0 && (id as usize) < self.cached_data_.animation_scales.len());
        if self.cached_data_.animation_scales[id as usize].update_number
            == self.cached_data_.transform_tree_update_number
        {
            return &self.cached_data_.animation_scales[id as usize];
        }
        self.cached_data_.animation_scales[id as usize].update_number =
            self.cached_data_.transform_tree_update_number;
        let parent_id = self.transform_tree_.Node(id).parent_id;
        if parent_id != kInvalidPropertyNodeId {
            self.GetAnimationScaleData(parent_id);
        }
        // Source recursive parent scale lookup is complete before taking these
        // disjoint resident cache references. No copied animation cache records.
        let (current, parent) = if parent_id == kInvalidPropertyNodeId {
            (&mut self.cached_data_.animation_scales[id as usize], None)
        } else {
            let (parents, nodes) = self.cached_data_.animation_scales.split_at_mut(id as usize);
            (&mut nodes[0], Some(&parents[parent_id as usize]))
        };
        let node = self.transform_tree_.Node(id);
        let ancestor_affected = parent.is_some_and(|scale| scale.affected_by_animation_scale);
        let node_affected = node.has_potential_animation && node.maximum_animation_scale != 1.0;
        current.affected_by_animation_scale = node_affected || ancestor_affected;
        current.affected_by_invalid_scale = parent
            .is_some_and(|scale| scale.affected_by_invalid_scale)
            || node.to_parent.HasPerspective()
            || (node.has_potential_animation && node.maximum_animation_scale == kInvalidScale);
        let failed_for_multiple = ancestor_affected && node_affected;
        let mut local_maximum = 1.0;
        if current.affected_by_invalid_scale || failed_for_multiple {
        } else if !node.to_screen_is_potentially_animated {
            let scales = self
                .transform_tree_
                .ToScreen(id)
                .ComputeTransform2dScaleComponents(kInvalidScale);
            current.maximum_to_screen_scale = scales.x().max(scales.y());
            return current;
        } else if !node.has_potential_animation {
            let scales = node.local.ComputeTransform2dScaleComponents(1.0);
            local_maximum = scales.x().max(scales.y());
        } else {
            debug_assert_ne!(node.maximum_animation_scale, kInvalidScale);
            local_maximum = node.maximum_animation_scale;
        }
        current.maximum_to_screen_scale = local_maximum;
        if let Some(parent) = parent {
            current.maximum_to_screen_scale *= parent.maximum_to_screen_scale;
        }
        current
    }
    // Source: cc/trees/property_tree.cc:2880-2890.
    pub fn SetMaximumAnimationToScreenScaleForTesting(
        &mut self,
        id: i32,
        scale: f32,
        invalid: bool,
    ) {
        assert!(id >= 0 && (id as usize) < self.cached_data_.animation_scales.len());
        let current = &mut self.cached_data_.animation_scales[id as usize];
        current.maximum_to_screen_scale = scale;
        current.affected_by_invalid_scale = invalid;
        current.update_number = self.cached_data_.transform_tree_update_number;
    }
    // Source: cc/trees/property_tree.cc:2892-2912.
    pub fn GetToTarget(&self, transform_id: i32, effect_id: i32, out: &mut T) -> bool {
        GetToTargetCore(
            &self.transform_tree_,
            &self.effect_tree_,
            &self.cached_data_,
            transform_id,
            effect_id,
            out,
        )
    }
    // Source: cc/trees/property_tree.cc:2914-2936.
    pub fn GetFromTarget(&self, transform_id: i32, effect_id: i32, out: &mut T) -> bool {
        let node = self.transform_tree_.Node(transform_id);
        if node.ancestors_are_invertible && effect_id == kContentsRootPropertyNodeId {
            *out = self.transform_tree_.FromScreen(transform_id).clone();
            return true;
        }
        let mut transforms = GetDrawTransformsCore(
            &self.transform_tree_,
            &self.effect_tree_,
            &self.cached_data_,
            transform_id,
            effect_id,
        );
        if transforms.from_valid {
            *out = transforms.from_target.clone();
            true
        } else if !transforms.might_be_invertible {
            false
        } else {
            transforms.might_be_invertible = transforms.to_target.GetInverse(out);
            transforms.from_valid = transforms.might_be_invertible;
            transforms.from_target = out.clone();
            transforms.from_valid
        }
    }
    // Source: cc/trees/property_tree.h:1033-1036; cc/trees/property_tree.cc:2938-2958.
    fn FetchDrawTransformsDataFromCache(
        &self,
        transform_id: i32,
        effect_id: i32,
    ) -> RefMut<'_, DrawTransformData<T>> {
        FetchDrawTransformsDataFromCacheCore(&self.cached_data_, transform_id, effect_id)
    }
    // Source: cc/trees/property_tree.h:1031-1032; cc/trees/property_tree.cc:2981-3036.
    fn GetDrawTransforms(
        &self,
        transform_id: i32,
        effect_id: i32,
    ) -> RefMut<'_, DrawTransforms<T>> {
        GetDrawTransformsCore(
            &self.transform_tree_,
            &self.effect_tree_,
            &self.cached_data_,
            transform_id,
            effect_id,
        )
    }
    // Source: cc/trees/property_tree.cc:2960-2970.
    pub fn FetchClipRectFromCache(&self, clip_id: i32, target_id: i32) -> RefMut<'_, ClipRectData> {
        let mut entries = self.clip_tree_.Node(clip_id).cached_clip_rects.borrow_mut();
        let index = if let Some(index) = entries.iter().position(|data| {
            data.target_id == target_id || data.target_id == kInvalidPropertyNodeId
        }) {
            index
        } else {
            entries.push(ClipRectData::default());
            let index = entries.len() - 1;
            entries[index].target_id = kInvalidPropertyNodeId;
            index
        };
        RefMut::map(entries, |entries| &mut entries[index])
    }
    // Source: cc/trees/property_tree.cc:2972-2979.
    pub fn HasElement(&self, id: ElementId) -> bool {
        id != ElementId::default()
            && (self.clip_tree_.FindNodeFromElementId(id).is_some()
                || self.effect_tree_.FindNodeFromElementId(id).is_some()
                || self.scroll_tree_.FindNodeFromElementId(id).is_some()
                || self.transform_tree_.FindNodeFromElementId(id).is_some())
    }
    // Source: cc/trees/property_tree.cc:3038-3052.
    pub fn ResetCachedData(&mut self) {
        self.cached_data_.transform_tree_update_number = 0;
        let count = self.transform_tree_.size();
        self.cached_data_
            .animation_scales
            .resize_with(count, AnimationScaleData::default);
        for scale in &mut self.cached_data_.animation_scales {
            scale.update_number = kInvalidUpdateNumber;
        }
        let entries = self.cached_data_.draw_transforms.get_mut();
        entries.resize_with(count, || vec![DrawTransformData::default()]);
        for values in entries {
            values.resize_with(1, DrawTransformData::default);
            values[0].update_number = kInvalidUpdateNumber;
            values[0].effect_id = kInvalidPropertyNodeId;
        }
    }
    // Source: cc/trees/property_tree.cc:3054-3056.
    pub fn UpdateTransformTreeUpdateNumber(&mut self) {
        self.cached_data_.UpdateTransformTreeUpdateNumber();
    }
    // Source: cc/trees/property_tree.cc:3058-3074.
    pub fn ToScreenSpaceTransformWithoutSurfaceContentsScale(
        &self,
        transform_id: i32,
        effect_id: i32,
    ) -> T {
        if transform_id == kRootPropertyNodeId {
            return T::default();
        }
        let mut transform = self.transform_tree_.ToScreen(transform_id).clone();
        let node = self.effect_tree_.Node(effect_id);
        if node.surface_contents_scale.x() != 0.0 && node.surface_contents_scale.y() != 0.0 {
            transform.Scale(
                1.0 / node.surface_contents_scale.x(),
                1.0 / node.surface_contents_scale.y(),
            );
        }
        transform
    }
}
// Source: cc/trees/property_tree.cc:2444-2464.
#[cfg(debug_assertions)]
impl<
        T: PropertyTreeTransform,
        F: PropertyTreeFilters,
        P: PropertyPath,
        R: PropertyTreeRenderSurface,
        C: PropertyTreeCopyOutputRequest,
    > PartialEq for PropertyTrees<T, F, P, R, C>
{
    fn eq(&self, other: &Self) -> bool {
        self.transform_tree_ == other.transform_tree_
            && self.effect_tree_ == other.effect_tree_
            && self.clip_tree_ == other.clip_tree_
            && self.scroll_tree_ == other.scroll_tree_
            && self.needs_rebuild() == other.needs_rebuild()
            && self.changed() == other.changed()
            && self.full_tree_damaged() == other.full_tree_damaged()
            && self.is_main_thread() == other.is_main_thread()
            && self.is_active() == other.is_active()
            && self.sequence_number() == other.sequence_number()
            && self.inner_viewport_container_bounds_delta()
                == other.inner_viewport_container_bounds_delta()
            && self.outer_viewport_container_bounds_delta()
                == other.outer_viewport_container_bounds_delta()
            && self.changed_effect_nodes_ == other.changed_effect_nodes_
            && self.changed_transform_nodes_ == other.changed_transform_nodes_
    }
}

// Source: cc/trees/property_tree.cc:2892-2912.
// Same translated algorithm, factored so the owner's disjoint live borrows can
// call it from EffectTree as well as PropertyTrees without self-reference.
fn GetToTargetCore<
    T: PropertyTreeTransform,
    F: PropertyTreeFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    tree: &TransformTree<T>,
    effects: &EffectTree<F, P, R, C>,
    cache: &PropertyTreesCachedData<T>,
    transform_id: i32,
    effect_id: i32,
    out: &mut T,
) -> bool {
    if effect_id == kContentsRootPropertyNodeId {
        *out = tree.ToScreen(transform_id).clone();
        return true;
    }
    let mut transforms = GetDrawTransformsCore(tree, effects, cache, transform_id, effect_id);
    if transforms.to_valid {
        *out = transforms.to_target.clone();
        true
    } else if !transforms.might_be_invertible {
        false
    } else {
        transforms.might_be_invertible = transforms.from_target.GetInverse(out);
        transforms.to_valid = transforms.might_be_invertible;
        transforms.to_target = out.clone();
        transforms.to_valid
    }
}
// Source: cc/trees/property_tree.cc:2938-2958.
fn FetchDrawTransformsDataFromCacheCore<T: PropertyTreeTransform>(
    cache: &PropertyTreesCachedData<T>,
    transform_id: i32,
    effect_id: i32,
) -> RefMut<'_, DrawTransformData<T>> {
    let mut all = cache.draw_transforms.borrow_mut();
    assert!(transform_id >= 0 && (transform_id as usize) < all.len());
    let entries = &mut all[transform_id as usize];
    let index = if let Some(index) = entries
        .iter()
        .position(|data| data.effect_id == effect_id || data.effect_id == kInvalidPropertyNodeId)
    {
        index
    } else {
        entries.push(DrawTransformData::default());
        let index = entries.len() - 1;
        entries[index].update_number = kInvalidUpdateNumber;
        entries[index].effect_id = effect_id;
        index
    };
    RefMut::map(all, |all| &mut all[transform_id as usize][index])
}
// Source: cc/trees/property_tree.cc:2981-3036.
fn GetDrawTransformsCore<
    'a,
    T: PropertyTreeTransform,
    F: PropertyTreeFilters,
    P: PropertyPath,
    R: PropertyTreeRenderSurface,
    C: PropertyTreeCopyOutputRequest,
>(
    tree: &TransformTree<T>,
    effects: &EffectTree<F, P, R, C>,
    cache: &'a PropertyTreesCachedData<T>,
    transform_id: i32,
    effect_id: i32,
) -> RefMut<'a, DrawTransforms<T>> {
    let node = effects.Node(effect_id);
    let dest = node.transform_id;
    let mut data = FetchDrawTransformsDataFromCacheCore(cache, transform_id, effect_id);
    debug_assert!(
        data.update_number != cache.transform_tree_update_number
            || data.effect_id != kInvalidPropertyNodeId
    );
    if data.update_number == cache.transform_tree_update_number {
        return RefMut::map(data, |data| &mut data.transforms);
    }
    let mut target = T::default();
    let mut from = T::default();
    let mut already_computed_inverse = false;
    if transform_id == dest {
        target.Scale(
            node.surface_contents_scale.x(),
            node.surface_contents_scale.y(),
        );
        data.transforms.to_valid = true;
        data.transforms.from_valid = false;
    } else if transform_id > dest {
        tree.CombineTransformsBetween(transform_id, dest, &mut target);
        target.PostScale(
            node.surface_contents_scale.x(),
            node.surface_contents_scale.y(),
        );
        data.transforms.to_valid = true;
        data.transforms.from_valid = false;
        data.transforms.might_be_invertible = true;
    } else {
        let mut combined = T::default();
        tree.CombineTransformsBetween(dest, transform_id, &mut combined);
        if node.surface_contents_scale.x() != 0.0 && node.surface_contents_scale.y() != 0.0 {
            combined.Scale(
                1.0 / node.surface_contents_scale.x(),
                1.0 / node.surface_contents_scale.y(),
            );
        }
        let invertible = combined.GetInverse(&mut target);
        data.transforms.might_be_invertible = invertible;
        data.transforms.to_valid = invertible;
        data.transforms.from_valid = true;
        from = combined;
        already_computed_inverse = true;
    }
    if !already_computed_inverse {
        data.transforms.to_valid = true;
    }
    data.update_number = cache.transform_tree_update_number;
    data.effect_id = effect_id;
    data.transforms.from_target = from;
    data.transforms.to_target = target;
    RefMut::map(data, |data| &mut data.transforms)
}

// Source: cc/trees/property_tree.h:508.
// Declared, without a supplied upstream definition (as with TransformTree's
// target association declarations). Keep a contract instead of invented code.
pub trait EffectTreePushCopyRequests {
    fn PushCopyRequestsTo(&mut self, other: &mut Self);
}
