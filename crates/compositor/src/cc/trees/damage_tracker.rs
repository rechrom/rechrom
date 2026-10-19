// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/damage_tracker.h:1-218
// Source: cc/trees/damage_tracker.cc:1-705
use super::{
    damage_reason::{DamageReason, DamageReasonSet},
    property_ids::kContentsRootPropertyNodeId,
};
use crate::adapter::{
    damage_context::{DamageContext, DamageEffectNode, DamageTransform},
    geometry::{Rect, ToEnclosingRect},
    input_types::ElementId,
    property_types::ViewTransitionElementResourceId,
};
use crate::cc::base::math_util::MathUtil;
use std::{
    cell::{Cell, RefCell, RefMut},
    collections::BTreeMap,
};

// Source: cc/trees/damage_tracker.h:61-62
type ViewTransitionElementResourceIdToRenderSurfaceMap<S> =
    BTreeMap<ViewTransitionElementResourceId, S>;

// Rust template field-access contract, shared by Rect and DamageAccumulator.
trait DamageRect {
    fn IsEmpty(&self) -> bool;
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn right(&self) -> i32;
    fn bottom(&self) -> i32;
}
impl DamageRect for Rect {
    fn IsEmpty(&self) -> bool {
        (*self).IsEmpty()
    }
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn right(&self) -> i32 {
        (*self).right()
    }
    fn bottom(&self) -> i32 {
        (*self).bottom()
    }
}

// Source: cc/trees/damage_tracker.h:66-117
#[derive(Clone, Copy)]
struct DamageAccumulator {
    is_valid_rect_: bool,
    x_: i32,
    y_: i32,
    right_: i32,
    bottom_: i32,
    reasons_: DamageReasonSet,
}
impl Default for DamageAccumulator {
    // Source: cc/trees/damage_tracker.h:109-115
    fn default() -> Self {
        Self {
            is_valid_rect_: true,
            x_: 0,
            y_: 0,
            right_: 0,
            bottom_: 0,
            reasons_: DamageReasonSet::default(),
        }
    }
}
impl DamageAccumulator {
    // Source: cc/trees/damage_tracker.h:68-95
    fn Union<R: DamageRect>(&mut self, rect: &R, reasons: DamageReasonSet) {
        if rect.IsEmpty() {
            return;
        }
        self.reasons_.PutAll(&reasons);
        if !self.is_valid_rect_ {
            return;
        }
        if self.IsEmpty() {
            self.x_ = rect.x();
            self.y_ = rect.y();
            self.right_ = rect.right();
            self.bottom_ = rect.bottom();
            return;
        }
        self.x_ = self.x_.min(rect.x());
        self.y_ = self.y_.min(rect.y());
        self.right_ = self.right_.max(rect.right());
        self.bottom_ = self.bottom_.max(rect.bottom());
    }
    // Source: cc/trees/damage_tracker.h:97
    fn UnionReasons(&mut self, reasons: DamageReasonSet) {
        self.reasons_.PutAll(&reasons);
    }
    // Source: cc/trees/damage_tracker.h:107
    fn reasons(&self) -> DamageReasonSet {
        self.reasons_
    }
    // Source: cc/trees/damage_tracker.cc:575-593
    fn GetAsRect(&mut self, rect: &mut Rect) -> bool {
        if !self.is_valid_rect_ {
            return false;
        }
        let width = self.right_.checked_sub(self.x_);
        let height = self.bottom_.checked_sub(self.y_);
        if width.is_none() || height.is_none() {
            self.is_valid_rect_ = false;
            return false;
        }
        rect.set_x(self.x_);
        rect.set_y(self.y_);
        rect.set_width(width.unwrap());
        rect.set_height(height.unwrap());
        true
    }
}
impl DamageRect for DamageAccumulator {
    // Source: cc/trees/damage_tracker.h:99-103
    fn x(&self) -> i32 {
        self.x_
    }
    fn y(&self) -> i32 {
        self.y_
    }
    fn right(&self) -> i32 {
        self.right_
    }
    fn bottom(&self) -> i32 {
        self.bottom_
    }
    fn IsEmpty(&self) -> bool {
        self.x_ == self.right_ || self.y_ == self.bottom_
    }
}

// Source: cc/trees/damage_tracker.h:144-159
#[derive(Clone, Copy, Default)]
struct LayerRectMapData {
    layer_id_: i32,
    mailbox_id_: u32,
    rect_: Rect,
}
impl LayerRectMapData {
    // Source: cc/trees/damage_tracker.h:146
    fn new(layer_id: i32) -> Self {
        Self {
            layer_id_: layer_id,
            ..Self::default()
        }
    }
    // Source: cc/trees/damage_tracker.h:147-150
    fn Update(&mut self, rect: Rect, mailbox_id: u32) {
        self.mailbox_id_ = mailbox_id;
        self.rect_ = rect;
    }
    // Source: cc/trees/damage_tracker.h:152-154
    fn LessThan(&self, other: &Self) -> bool {
        self.layer_id_ < other.layer_id_
    }
}
// Source: cc/trees/damage_tracker.h:161-177
#[derive(Clone, Copy, Default)]
struct SurfaceRectMapData {
    surface_id_: ElementId,
    mailbox_id_: u32,
    rect_: Rect,
}
impl SurfaceRectMapData {
    // Source: cc/trees/damage_tracker.h:163-164
    fn new(surface_id: ElementId) -> Self {
        Self {
            surface_id_: surface_id,
            ..Self::default()
        }
    }
    // Source: cc/trees/damage_tracker.h:165-168
    fn Update(&mut self, rect: Rect, mailbox_id: u32) {
        self.mailbox_id_ = mailbox_id;
        self.rect_ = rect;
    }
    // Source: cc/trees/damage_tracker.h:170-172
    fn LessThan(&self, other: &Self) -> bool {
        self.surface_id_ < other.surface_id_
    }
}
// Source: cc/trees/damage_tracker.h:178-179
type SortedRectMapForLayers = Vec<LayerRectMapData>;
type SortedRectMapForSurfaces = Vec<SurfaceRectMapData>;
// Source: cc/trees/damage_tracker.h:196-201
struct SurfaceWithRect<S> {
    render_surface: S,
    rect_in_target_space: Rect,
}
impl<S> SurfaceWithRect<S> {
    // Source: cc/trees/damage_tracker.h:197-198
    fn new(rs: S, rect: Rect) -> Self {
        Self {
            render_surface: rs,
            rect_in_target_space: rect,
        }
    }
}

// Source: cc/trees/damage_tracker.h:32-64,181-214
// Rust generic tracks only the native stable surface identity. Short-lived
// context types are method parameters; no borrowed owner is retained.
pub struct DamageTracker<S: Copy + Eq> {
    // Rust interior borrows express the source's resident mutable ownership
    // without holding a whole-tracker borrow across live sibling queries.
    rect_history_for_layers_: RefCell<SortedRectMapForLayers>,
    rect_history_for_surfaces_: RefCell<SortedRectMapForSurfaces>,
    mailbox_id_: Cell<u32>,
    current_damage_: RefCell<DamageAccumulator>,
    has_damage_from_contributing_content_: Cell<bool>,
    damage_for_this_update_: RefCell<DamageAccumulator>,
    contributing_surfaces_: RefCell<Vec<SurfaceWithRect<S>>>,
    previous_view_transition_content_surfaces_by_id_: RefCell<Vec<ViewTransitionElementResourceId>>,
    current_view_transition_content_surfaces_by_id_: RefCell<Vec<ViewTransitionElementResourceId>>,
}
impl<S: Copy + Eq> Default for DamageTracker<S> {
    // Source: cc/trees/damage_tracker.cc:29-30
    // Source: cc/trees/damage_tracker.h:181-214
    fn default() -> Self {
        Self {
            rect_history_for_layers_: RefCell::default(),
            rect_history_for_surfaces_: RefCell::default(),
            mailbox_id_: Cell::new(0),
            current_damage_: RefCell::default(),
            has_damage_from_contributing_content_: Cell::new(false),
            damage_for_this_update_: RefCell::default(),
            contributing_surfaces_: RefCell::default(),
            previous_view_transition_content_surfaces_by_id_: RefCell::default(),
            current_view_transition_content_surfaces_by_id_: RefCell::default(),
        }
    }
}
impl<S: Copy + Eq> DamageTracker<S> {
    // Source: cc/trees/damage_tracker.cc:25-27
    pub fn Create() -> Box<Self> {
        Box::new(Self::default())
    }
    // Source: cc/trees/damage_tracker.h:45-48
    pub fn DidDrawDamagedArea(&self) {
        *self.current_damage_.borrow_mut() = DamageAccumulator::default();
        self.has_damage_from_contributing_content_.set(false);
    }
    // Source: cc/trees/damage_tracker.h:49-51
    pub fn AddDamageNextUpdate(&self, dmg: Rect) {
        self.current_damage_
            .borrow_mut()
            .Union(&dmg, DamageReasonSet::new([DamageReason::kUntracked]));
    }
    // Source: cc/trees/damage_tracker.h:56-58
    pub fn has_damage_from_contributing_content(&self) -> bool {
        self.has_damage_from_contributing_content_.get()
    }

    // Source: cc/trees/damage_tracker.cc:32-163
    pub fn UpdateDamageTracking<C: DamageContext<Surface = S>>(layer_tree_impl: &C) {
        let mut id_to_render_surface_map = ViewTransitionElementResourceIdToRenderSurfaceMap::new();
        Self::InitializeUpdateDamageTracking(layer_tree_impl, &mut id_to_render_surface_map);
        let effect_tree = layer_tree_impl.property_trees().effect_tree();
        let mut current_target_effect_id = kContentsRootPropertyNodeId;
        debug_assert!(effect_tree
            .GetRenderSurface(current_target_effect_id)
            .is_some());
        for layer_id in layer_tree_impl.layers() {
            let layer = layer_tree_impl.layer(layer_id);
            if !layer.contributes_to_drawn_render_surface() {
                continue;
            }
            let next_target_effect_id =
                layer.render_target_effect_tree_index(layer_tree_impl.property_trees());
            if next_target_effect_id != current_target_effect_id {
                let lowest_common_ancestor_id = effect_tree.LowestCommonAncestorWithRenderSurface(
                    current_target_effect_id,
                    next_target_effect_id,
                );
                while current_target_effect_id != lowest_common_ancestor_id {
                    let current_target = layer_tree_impl
                        .GetRenderSurface(current_target_effect_id)
                        .unwrap();
                    layer_tree_impl
                        .damage_tracker(current_target)
                        .ComputeSurfaceDamage(current_target, layer_tree_impl);
                    let parent_target = layer_tree_impl
                        .surface(current_target)
                        .render_target(layer_tree_impl.property_trees());
                    layer_tree_impl
                        .damage_tracker(parent_target)
                        .AccumulateDamageFromRenderSurface(current_target, layer_tree_impl);
                    current_target_effect_id =
                        effect_tree.Node(current_target_effect_id).target_id();
                }
                current_target_effect_id = next_target_effect_id;
            }
            let target_surface = layer.render_target(layer_tree_impl.property_trees());
            if Some(layer_id) != layer_tree_impl.hud_layer() {
                layer_tree_impl
                    .damage_tracker(target_surface)
                    .AccumulateDamageFromLayer(
                        layer_id,
                        &mut id_to_render_surface_map,
                        layer_tree_impl,
                    );
            }
        }
        debug_assert!(current_target_effect_id >= kContentsRootPropertyNodeId);
        let mut current_target = layer_tree_impl
            .GetRenderSurface(current_target_effect_id)
            .unwrap();
        loop {
            layer_tree_impl
                .damage_tracker(current_target)
                .ComputeSurfaceDamage(current_target, layer_tree_impl);
            if layer_tree_impl.surface(current_target).EffectTreeIndex()
                == kContentsRootPropertyNodeId
            {
                break;
            }
            let next_target = layer_tree_impl
                .surface(current_target)
                .render_target(layer_tree_impl.property_trees());
            layer_tree_impl
                .damage_tracker(next_target)
                .AccumulateDamageFromRenderSurface(current_target, layer_tree_impl);
            current_target = next_target;
        }
    }

    // Source: cc/trees/damage_tracker.cc:165-184
    fn InitializeUpdateDamageTracking<C: DamageContext<Surface = S>>(
        layer_tree_impl: &C,
        id_to_render_surface_map: &mut ViewTransitionElementResourceIdToRenderSurfaceMap<S>,
    ) {
        for effect_id in layer_tree_impl.GetRenderSurfaceList() {
            let render_surface = layer_tree_impl.GetRenderSurface(effect_id).unwrap();
            layer_tree_impl
                .damage_tracker(render_surface)
                .PrepareForUpdate();
            let resource_id = layer_tree_impl
                .surface(render_surface)
                .OwningEffectNode(layer_tree_impl.property_trees())
                .view_transition_element_resource_id();
            if resource_id.IsValid() {
                debug_assert!(!id_to_render_surface_map.contains_key(&resource_id));
                id_to_render_surface_map.insert(resource_id, render_surface);
            }
        }
    }

    // Source: cc/trees/damage_tracker.cc:186-269
    fn ComputeSurfaceDamage<C: DamageContext<Surface = S>>(&self, render_surface: S, context: &C) {
        let mut damage_from_leftover_rects = self.TrackDamageFromLeftoverRects();
        self.has_damage_from_contributing_content_.set(
            self.has_damage_from_contributing_content_.get()
                | !damage_from_leftover_rects.IsEmpty(),
        );
        let mut expanded_damage_rect = Rect::default();
        let valid = damage_from_leftover_rects.GetAsRect(&mut expanded_damage_rect);
        let mut expanded = false;
        for contributing_surface in self.contributing_surfaces_.borrow().iter() {
            let surface = context.surface(contributing_surface.render_surface);
            let has_pixel_moving_backdrop_filters = surface
                .BackdropFilters(context.property_trees())
                .HasFilterThatMovesPixels();
            if !surface.intersects_damage_under() || has_pixel_moving_backdrop_filters {
                if !valid
                    || contributing_surface
                        .rect_in_target_space
                        .Intersects(expanded_damage_rect)
                {
                    surface.set_intersects_damage_under(true);
                    if has_pixel_moving_backdrop_filters {
                        expanded_damage_rect.Union(contributing_surface.rect_in_target_space);
                        expanded = true;
                    }
                }
            }
        }
        if expanded {
            self.damage_for_this_update_
                .borrow_mut()
                .Union(&expanded_damage_rect, damage_from_leftover_rects.reasons());
        }
        self.contributing_surfaces_.borrow_mut().clear();
        let mut reasons = self.damage_for_this_update_.borrow().reasons();
        if context.surface(render_surface).SurfacePropertyChanged() {
            reasons.Put(DamageReason::kUntracked);
        }
        if !damage_from_leftover_rects.IsEmpty() {
            reasons.PutAll(&damage_from_leftover_rects.reasons());
        }
        {
            self.damage_for_this_update_
                .borrow_mut()
                .Union(&damage_from_leftover_rects, DamageReasonSet::default());
            let mut damage_rect = Rect::default();
            let is_rect_valid = self
                .damage_for_this_update_
                .borrow_mut()
                .GetAsRect(&mut damage_rect);
            if is_rect_valid && !damage_rect.IsEmpty() {
                damage_rect = context
                    .surface(render_surface)
                    .Filters(context.property_trees())
                    .MapRect(
                        damage_rect,
                        &context
                            .surface(render_surface)
                            .SurfaceScale(context.property_trees())
                            .ToFlattenedSkMatrix(),
                    );
                *self.damage_for_this_update_.borrow_mut() = DamageAccumulator::default();
                self.damage_for_this_update_
                    .borrow_mut()
                    .Union(&damage_rect, DamageReasonSet::default());
            }
        }
        self.damage_for_this_update_
            .borrow_mut()
            .UnionReasons(reasons);
        if context.surface(render_surface).SurfacePropertyChanged() {
            self.damage_for_this_update_.borrow_mut().Union(
                &context.surface(render_surface).content_rect(),
                DamageReasonSet::default(),
            );
            self.has_damage_from_contributing_content_.set(
                self.has_damage_from_contributing_content_.get()
                    | !self.damage_for_this_update_.borrow().IsEmpty(),
            );
        }
        self.current_damage_.borrow_mut().Union(
            &*self.damage_for_this_update_.borrow(),
            self.damage_for_this_update_.borrow().reasons(),
        );
    }

    // Source: cc/trees/damage_tracker.cc:271-273
    pub fn GetDamageRectIfValid(&self, rect: &mut Rect) -> bool {
        self.current_damage_.borrow_mut().GetAsRect(rect)
    }
    // Source: cc/trees/damage_tracker.cc:275-277
    pub fn GetDamageReasons(&self) -> DamageReasonSet {
        self.current_damage_.borrow().reasons()
    }
    // Source: cc/trees/damage_tracker.cc:279-293
    fn RectDataForLayer(
        &self,
        layer_id: i32,
        layer_is_new: &mut bool,
    ) -> RefMut<'_, LayerRectMapData> {
        let data = LayerRectMapData::new(layer_id);
        let mut history = self.rect_history_for_layers_.borrow_mut();
        let it = history.partition_point(|entry| entry.LessThan(&data));
        if it == history.len() || history[it].layer_id_ != layer_id {
            *layer_is_new = true;
            history.insert(it, data);
        }
        RefMut::map(history, |history| &mut history[it])
    }
    // Source: cc/trees/damage_tracker.cc:295-309
    fn RectDataForSurface(
        &self,
        surface_id: ElementId,
        surface_is_new: &mut bool,
    ) -> RefMut<'_, SurfaceRectMapData> {
        let data = SurfaceRectMapData::new(surface_id);
        let mut history = self.rect_history_for_surfaces_.borrow_mut();
        let it = history.partition_point(|entry| entry.LessThan(&data));
        if it == history.len() || history[it].surface_id_ != surface_id {
            *surface_is_new = true;
            history.insert(it, data);
        }
        RefMut::map(history, |history| &mut history[it])
    }
    // Source: cc/trees/damage_tracker.cc:311-319
    fn PrepareForUpdate(&self) {
        self.mailbox_id_.set(self.mailbox_id_.get().wrapping_add(1));
        *self.damage_for_this_update_.borrow_mut() = DamageAccumulator::default();
        self.has_damage_from_contributing_content_.set(false);
        self.contributing_surfaces_.borrow_mut().clear();
        self.current_view_transition_content_surfaces_by_id_
            .swap(&self.previous_view_transition_content_surfaces_by_id_);
        self.current_view_transition_content_surfaces_by_id_
            .borrow_mut()
            .clear();
    }

    // Source: cc/trees/damage_tracker.cc:321-383
    fn TrackDamageFromLeftoverRects(&self) -> DamageAccumulator {
        let mut damage = DamageAccumulator::default();
        let mut layers = self.rect_history_for_layers_.borrow_mut();
        let mut surfaces = self.rect_history_for_surfaces_.borrow_mut();
        let mut layer_cur_pos = 0;
        let mut layer_copy_pos = layer_cur_pos;
        let mut surface_cur_pos = 0;
        let mut surface_copy_pos = surface_cur_pos;
        while layer_cur_pos < layers.len() {
            if layers[layer_cur_pos].mailbox_id_ == self.mailbox_id_.get() {
                if layer_cur_pos != layer_copy_pos {
                    layers[layer_copy_pos] = layers[layer_cur_pos];
                }
                layer_copy_pos += 1;
            } else {
                damage.Union(
                    &layers[layer_cur_pos].rect_,
                    DamageReasonSet::new([DamageReason::kUntracked]),
                );
            }
            layer_cur_pos += 1;
        }
        while surface_cur_pos < surfaces.len() {
            if surfaces[surface_cur_pos].mailbox_id_ == self.mailbox_id_.get() {
                if surface_cur_pos != surface_copy_pos {
                    surfaces[surface_copy_pos] = surfaces[surface_cur_pos];
                }
                surface_copy_pos += 1;
            } else {
                damage.Union(
                    &surfaces[surface_cur_pos].rect_,
                    DamageReasonSet::new([DamageReason::kUntracked]),
                );
            }
            surface_cur_pos += 1;
        }
        if layer_copy_pos != layers.len() {
            layers.truncate(layer_copy_pos);
        }
        if surface_copy_pos != surfaces.len() {
            surfaces.truncate(surface_copy_pos);
        }
        if layers.capacity() > layers.len() * 4 {
            *layers = layers.clone();
        }
        if surfaces.capacity() > surfaces.len() * 4 {
            *surfaces = surfaces.clone();
        }
        damage
    }

    // Source: cc/trees/damage_tracker.cc:385-490
    fn AccumulateDamageFromLayer<C: DamageContext<Surface = S>>(
        &self,
        layer_id: C::Layer,
        id_to_render_surface_map: &mut ViewTransitionElementResourceIdToRenderSurfaceMap<S>,
        context: &C,
    ) {
        let layer = context.layer(layer_id);
        let mut layer_is_new = false;
        let mut data = self.RectDataForLayer(layer.id(), &mut layer_is_new);
        let old_visible_rect_in_target_space = data.rect_;
        let visible_rect_in_target_space =
            layer.GetEnclosingVisibleRectInTargetSpace(context.property_trees());
        data.Update(visible_rect_in_target_space, self.mailbox_id_.get());
        drop(data);
        let mut view_transition_content_surface_damage_rect = Rect::default();
        if layer.ViewTransitionResourceId().IsValid() {
            view_transition_content_surface_damage_rect = self
                .GetViewTransitionContentSurfaceDamageInSharedElementLayerSpace(
                    layer_id,
                    id_to_render_surface_map,
                    context,
                );
        }
        if layer_is_new || layer.LayerPropertyChanged(context.property_trees()) {
            let mut reasons = layer.GetDamageReasons(context.property_trees());
            if layer_is_new {
                reasons.Put(DamageReason::kUntracked);
            }
            self.damage_for_this_update_
                .borrow_mut()
                .Union(&visible_rect_in_target_space, reasons);
            self.damage_for_this_update_.borrow_mut().Union(
                &old_visible_rect_in_target_space,
                DamageReasonSet::default(),
            );
        } else {
            let mut reasons = layer.GetDamageReasons(context.property_trees());
            let mut damage_rect = layer.update_rect();
            damage_rect.Union(layer.GetDamageRect());
            damage_rect.Intersect(layer.visible_layer_rect());
            damage_rect.Union(view_transition_content_surface_damage_rect);
            let bounds = layer.bounds(context.property_trees());
            if view_transition_content_surface_damage_rect.Intersects(Rect::new(
                0,
                0,
                bounds.width(),
                bounds.height(),
            )) {
                reasons.Put(DamageReason::kUntracked);
            }
            let bounds = layer.bounds(context.property_trees());
            damage_rect.Intersect(Rect::new(0, 0, bounds.width(), bounds.height()));
            if !damage_rect.IsEmpty() {
                let damage_visible_rect_in_target_space = MathUtil::MapEnclosingClippedRect(
                    &layer.DrawTransform(context.property_trees()),
                    damage_rect,
                );
                self.damage_for_this_update_.borrow_mut().Union(
                    &damage_visible_rect_in_target_space,
                    DamageReasonSet::default(),
                );
            }
            self.damage_for_this_update_
                .borrow_mut()
                .UnionReasons(reasons);
        }
        let mut property_change_on_non_target_node = false;
        if layer.LayerPropertyChangedFromPropertyTrees(context.property_trees()) {
            let effect_id = context
                .surface(layer.render_target(context.property_trees()))
                .EffectTreeIndex();
            let effect_node = context.property_trees().effect_tree().Node(effect_id);
            let transform_id = effect_node.transform_id();
            property_change_on_non_target_node = layer.effect_tree_index() != effect_id
                || layer.transform_tree_index() != transform_id;
        }
        if layer_is_new
            || !layer.update_rect().IsEmpty()
            || layer.LayerPropertyChangedNotFromPropertyTrees()
            || !layer.GetDamageRect().IsEmpty()
            || property_change_on_non_target_node
            || !view_transition_content_surface_damage_rect.IsEmpty()
        {
            self.has_damage_from_contributing_content_.set(
                self.has_damage_from_contributing_content_.get()
                    | !self.damage_for_this_update_.borrow().IsEmpty(),
            );
        }
    }

    // Source: cc/trees/damage_tracker.cc:492-573
    fn AccumulateDamageFromRenderSurface<C: DamageContext<Surface = S>>(
        &self,
        render_surface: S,
        context: &C,
    ) {
        let surface = context.surface(render_surface);
        let mut surface_is_new = false;
        let mut data = self.RectDataForSurface(surface.id(), &mut surface_is_new);
        let old_surface_rect = data.rect_;
        let surface_rect_in_target_space =
            ToEnclosingRect(surface.DrawableContentRect(context.property_trees()));
        data.Update(surface_rect_in_target_space, self.mailbox_id_.get());
        drop(data);
        self.contributing_surfaces_
            .borrow_mut()
            .push(SurfaceWithRect::new(
                render_surface,
                surface_rect_in_target_space,
            ));
        let mut damage_on_target = Rect::default();
        let valid = self
            .damage_for_this_update_
            .borrow_mut()
            .GetAsRect(&mut damage_on_target);
        let mut intersects_damage_under =
            !valid || damage_on_target.Intersects(surface_rect_in_target_space);
        if surface
            .BackdropFilters(context.property_trees())
            .HasFilterThatMovesPixels()
            && intersects_damage_under
        {
            self.damage_for_this_update_.borrow_mut().Union(
                &surface_rect_in_target_space,
                DamageReasonSet::new([DamageReason::kUntracked]),
            );
        }
        if surface_is_new
            || surface.SurfacePropertyChanged()
            || surface.AncestorPropertyChanged(context.property_trees())
        {
            let mut reasons = context.damage_tracker(render_surface).GetDamageReasons();
            if surface_is_new {
                reasons.Put(DamageReason::kUntracked);
            }
            self.damage_for_this_update_
                .borrow_mut()
                .Union(&surface_rect_in_target_space, reasons);
            self.damage_for_this_update_
                .borrow_mut()
                .Union(&old_surface_rect, DamageReasonSet::default());
            intersects_damage_under = true;
        } else {
            let mut damage_rect_in_local_space = Rect::default();
            let is_valid_rect = context
                .damage_tracker(render_surface)
                .GetDamageRectIfValid(&mut damage_rect_in_local_space);
            if is_valid_rect && !damage_rect_in_local_space.IsEmpty() {
                let draw_transform = surface.draw_transform();
                let mut damage_rect_in_target_space =
                    MathUtil::MapEnclosingClippedRect(draw_transform, damage_rect_in_local_space);
                damage_rect_in_target_space.Intersect(surface_rect_in_target_space);
                self.damage_for_this_update_.borrow_mut().Union(
                    &damage_rect_in_target_space,
                    context.damage_tracker(render_surface).GetDamageReasons(),
                );
            } else if !is_valid_rect {
                self.damage_for_this_update_.borrow_mut().Union(
                    &surface_rect_in_target_space,
                    context.damage_tracker(render_surface).GetDamageReasons(),
                );
            }
        }
        surface.set_intersects_damage_under(intersects_damage_under);
        self.has_damage_from_contributing_content_.set(
            self.has_damage_from_contributing_content_.get()
                | !self.damage_for_this_update_.borrow().IsEmpty(),
        );
    }

    // Source: cc/trees/damage_tracker.cc:595-703
    fn GetViewTransitionContentSurfaceDamageInSharedElementLayerSpace<
        C: DamageContext<Surface = S>,
    >(
        &self,
        layer_id: C::Layer,
        id_to_render_surface_map: &mut ViewTransitionElementResourceIdToRenderSurfaceMap<S>,
        context: &C,
    ) -> Rect {
        let layer = context.layer(layer_id);
        debug_assert!(layer.ViewTransitionResourceId().IsValid());
        let vt_resource_id = layer.ViewTransitionResourceId();
        let mut view_transition_content_surface = None;
        if let Some(shared_surface) = id_to_render_surface_map.get(&vt_resource_id) {
            view_transition_content_surface = Some(*shared_surface);
            debug_assert!(!self
                .current_view_transition_content_surfaces_by_id_
                .borrow()
                .contains(&vt_resource_id));
            self.current_view_transition_content_surfaces_by_id_
                .borrow_mut()
                .push(vt_resource_id);
        }
        let bounds = layer.bounds(context.property_trees());
        let layer_drawable_bounds = Rect::new(0, 0, bounds.width(), bounds.height());
        if let Some(view_transition_content_surface) = view_transition_content_surface {
            let mut surface_is_new = false;
            let mut data = self.RectDataForSurface(
                context.surface(view_transition_content_surface).id(),
                &mut surface_is_new,
            );
            data.Update(layer_drawable_bounds, self.mailbox_id_.get());
            drop(data);
            if surface_is_new
                || context
                    .surface(view_transition_content_surface)
                    .SurfacePropertyChanged()
                || context
                    .surface(view_transition_content_surface)
                    .AncestorPropertyChanged(context.property_trees())
            {
                return layer_drawable_bounds;
            }
        }
        if self
            .previous_view_transition_content_surfaces_by_id_
            .borrow()
            .contains(&vt_resource_id)
            != self
                .current_view_transition_content_surfaces_by_id_
                .borrow()
                .contains(&vt_resource_id)
        {
            return layer_drawable_bounds;
        }
        let view_transition_content_surface = match view_transition_content_surface {
            Some(surface) => surface,
            None => return Rect::default(),
        };
        let mut damage_rect_in_local_space = Rect::default();
        let is_valid_rect = context
            .damage_tracker(view_transition_content_surface)
            .GetDamageRectIfValid(&mut damage_rect_in_local_space);
        if !is_valid_rect {
            return layer_drawable_bounds;
        } else if damage_rect_in_local_space.IsEmpty() {
            return Rect::default();
        } else {
            let render_surface_content_rect = context
                .surface(view_transition_content_surface)
                .content_rect();
            let view_transition_transform = context
                .GetViewTransitionTransform(layer_drawable_bounds, render_surface_content_rect);
            return MathUtil::MapEnclosingClippedRect(
                &view_transition_transform,
                damage_rect_in_local_space,
            );
        }
    }
}

// Source: cc/trees/damage_tracker.h:136-137
// ExpandDamageInsideRectWithFilters is declared privately upstream, but has no
// definition or call site in the checked-in source. No replacement body is
// invented for that unused declaration; the actual filter-expansion algorithm
// is translated in ComputeSurfaceDamage above (damage_tracker.cc:186-269).
