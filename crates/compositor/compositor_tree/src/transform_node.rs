// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{
    damage_reason::{DamageReason, DamageReasonSet},
    geometry::{Point3F, PointF, Vector2dF},
    property_ids::kInvalidPropertyNodeId,
    property_types::{
        AddElementIdToTracedValue, AddPoint3FToTracedValue, AddPointFToTracedValue,
        AddTransformToTracedValue, AddVector2dFToTracedValue, PropertyTrace, PropertyTransform,
    },
    ElementId,
};

// Source: cc/trees/mutator_host.h:43
const kInvalidScale: f32 = 0.;

// Source: cc/trees/transform_node.h:26-181
// Source: cc/trees/transform_node.cc:18-20
#[derive(Clone, Debug, PartialEq)]
pub struct TransformNode<T: PropertyTransform> {
    // Source: cc/trees/transform_node.h:31
    pub element_id: ElementId,
    // Source: cc/trees/transform_node.h:37
    pub local: T,
    // Source: cc/trees/transform_node.h:38
    pub origin: Point3F,
    // Source: cc/trees/transform_node.h:41
    pub post_translation: Vector2dF,
    // Source: cc/trees/transform_node.h:35
    pub to_parent: T,
    // Source: cc/trees/transform_node.h:46
    pub id: i32,
    // Source: cc/trees/transform_node.h:48
    pub parent_id: i32,
    // Source: cc/trees/transform_node.h:51
    pub parent_frame_id: i32,
    // Source: cc/trees/transform_node.h:55
    pub sticky_position_constraint_id: i32,
    // Source: cc/trees/transform_node.h:59
    pub anchor_position_scroll_data_id: i32,
    // Source: cc/trees/transform_node.h:64
    pub sorting_context_id: i32,
    // Source: cc/trees/transform_node.h:68
    pub needs_local_transform_update: bool,
    // Source: cc/trees/transform_node.h:73
    pub node_and_ancestors_are_animated_or_invertible: bool,
    // Source: cc/trees/transform_node.h:75
    pub is_invertible: bool,
    // Source: cc/trees/transform_node.h:78
    pub ancestors_are_invertible: bool,
    // Source: cc/trees/transform_node.h:82
    pub has_potential_animation: bool,
    // Source: cc/trees/transform_node.h:84
    pub is_currently_animating: bool,
    // Source: cc/trees/transform_node.h:88
    pub to_screen_is_potentially_animated: bool,
    // Source: cc/trees/transform_node.h:92
    pub flattens_inherited_transform: bool,
    // Source: cc/trees/transform_node.h:96
    pub node_and_ancestors_are_flat: bool,
    // Source: cc/trees/transform_node.h:98
    pub scrolls: bool,
    // Source: cc/trees/transform_node.h:100
    pub should_undo_overscroll: bool,
    // Source: cc/trees/transform_node.h:102
    pub should_be_snapped: bool,
    // Source: cc/trees/transform_node.h:107
    pub moved_by_outer_viewport_bounds_delta_y: bool,
    // Source: cc/trees/transform_node.h:112
    pub moved_by_safe_area_bottom: bool,
    // Source: cc/trees/transform_node.h:118
    pub in_subtree_of_page_scale_layer: bool,
    // Source: cc/trees/transform_node.h:122
    pub delegates_to_parent_for_backface: bool,
    // Source: cc/trees/transform_node.h:126
    pub will_change_transform: bool,
    // Source: cc/trees/transform_node.h:129
    pub node_or_ancestors_will_change_transform: bool,
    // Source: cc/trees/transform_node.h:132
    transform_changed_: bool,
    // Source: cc/trees/transform_node.h:134
    scroll_offset_: PointF,
    // Source: cc/trees/transform_node.h:154
    pub snap_amount: Vector2dF,
    // Source: cc/trees/transform_node.h:159
    pub maximum_animation_scale: f32,
    // Source: cc/trees/transform_node.h:163
    pub visible_frame_element_id: ElementId,
    // Source: cc/trees/transform_node.h:166
    damage_reasons_: DamageReasonSet,
}
impl<T: PropertyTransform> Default for TransformNode<T> {
    // Source: cc/trees/transform_node.h:31-166
    fn default() -> Self {
        Self {
            element_id: ElementId::default(),
            local: T::default(),
            origin: Point3F::default(),
            post_translation: Vector2dF::default(),
            to_parent: T::default(),
            id: kInvalidPropertyNodeId,
            parent_id: kInvalidPropertyNodeId,
            parent_frame_id: kInvalidPropertyNodeId,
            sticky_position_constraint_id: kInvalidPropertyNodeId,
            anchor_position_scroll_data_id: kInvalidPropertyNodeId,
            sorting_context_id: 0,
            needs_local_transform_update: true,
            node_and_ancestors_are_animated_or_invertible: true,
            is_invertible: true,
            ancestors_are_invertible: true,
            has_potential_animation: false,
            is_currently_animating: false,
            to_screen_is_potentially_animated: false,
            flattens_inherited_transform: true,
            node_and_ancestors_are_flat: true,
            scrolls: false,
            should_undo_overscroll: false,
            should_be_snapped: false,
            moved_by_outer_viewport_bounds_delta_y: false,
            moved_by_safe_area_bottom: false,
            in_subtree_of_page_scale_layer: false,
            delegates_to_parent_for_backface: false,
            will_change_transform: false,
            node_or_ancestors_will_change_transform: false,
            transform_changed_: false,
            scroll_offset_: PointF::default(),
            snap_amount: Vector2dF::default(),
            maximum_animation_scale: kInvalidScale,
            visible_frame_element_id: ElementId::default(),
            damage_reasons_: DamageReasonSet::default(),
        }
    }
}
impl<T: PropertyTransform> TransformNode<T> {
    // Source: cc/trees/transform_node.cc:18-20
    pub fn new() -> Self {
        Self::default()
    }
    // Source: cc/trees/transform_node.cc:22-29
    pub fn SetScrollOffset(&mut self, offset: &PointF, damage_reason: DamageReason) {
        if !offset.x().is_finite() || !offset.y().is_finite() {
            return;
        }
        self.scroll_offset_ = *offset;
        self.damage_reasons_.Put(damage_reason);
    }
    // Source: cc/trees/transform_node.h:139
    pub fn scroll_offset(&self) -> &PointF {
        &self.scroll_offset_
    }
    // Source: cc/trees/transform_node.cc:31-34
    pub fn SetTransformChanged(&mut self, damage_reason: DamageReason) {
        self.transform_changed_ = true;
        self.damage_reasons_.Put(damage_reason);
    }
    // Source: cc/trees/transform_node.cc:36-39
    pub fn ClearTransformChanged(&mut self) {
        self.transform_changed_ = false;
        self.damage_reasons_.Clear();
    }
    // Source: cc/trees/transform_node.cc:41-44
    pub fn CopyTransformChangedFrom(&mut self, other: &Self) {
        self.transform_changed_ = other.transform_changed_;
        self.damage_reasons_.PutAll(&other.damage_reasons_);
    }
    // Source: cc/trees/transform_node.h:149
    pub fn transform_changed(&self) -> bool {
        self.transform_changed_
    }
    // Source: cc/trees/transform_node.cc:46-55
    pub fn SetDamageReasonsForDeserialization(&mut self, reasons: DamageReasonSet) -> bool {
        for reason in &reasons {
            if reason as i32 > DamageReason::kMaxValue as i32 {
                return false;
            }
        }
        self.damage_reasons_ = reasons;
        true
    }
    // Source: cc/trees/transform_node.h:171
    pub fn damage_reasons(&self) -> DamageReasonSet {
        self.damage_reasons_.clone()
    }
    // Source: cc/trees/transform_node.h:175-178
    pub fn set_to_parent(&mut self, transform: &T) {
        self.to_parent = transform.clone();
        self.is_invertible = self.to_parent.IsInvertible();
    }
    // Source: cc/trees/transform_node.cc:57-71
    pub fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        value.SetInteger("id", self.id);
        value.SetInteger("parent_id", self.parent_id);
        AddElementIdToTracedValue(self.element_id, value);
        AddTransformToTracedValue("local", &self.local, value);
        AddPoint3FToTracedValue("origin", &self.origin, value);
        AddVector2dFToTracedValue("post_translation", &self.post_translation, value);
        value.SetInteger("sorting_context_id", self.sorting_context_id);
        value.SetBoolean(
            "flattens_inherited_transform",
            self.flattens_inherited_transform,
        );
        value.SetBoolean("will_change_transform", self.will_change_transform);
        AddPointFToTracedValue("scroll_offset", &self.scroll_offset_, value);
        AddVector2dFToTracedValue("snap_amount", &self.snap_amount, value);
        value.SetInteger(
            "damage_reasons",
            self.damage_reasons_.ToEnumBitmask() as i32,
        );
    }
}

// Source: cc/trees/transform_node.h:184-195
// Source: cc/trees/transform_node.cc:73-78
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransformCachedNodeData<T: PropertyTransform> {
    // Source: cc/trees/transform_node.h:189
    pub from_screen: T,
    // Source: cc/trees/transform_node.h:190
    pub to_screen: T,
    // Source: cc/trees/transform_node.h:192
    pub is_showing_backface: bool,
}
impl<T: PropertyTransform> TransformCachedNodeData<T> {
    pub fn new() -> Self {
        Self::default()
    }
}
