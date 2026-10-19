//! Live dependencies of cc::DamageTracker. Every tracker reference below is
//! the resident tracker owned by its render surface. No tree/surface/layer or
//! tracker state is copied into an adapter projection.
use super::{
    geometry::{Rect, RectF, Size, Transform},
    input_types::ElementId,
    layer_tree_impl::{LayerTreeTypes, PropertyTreesFor},
    occlusion::SkMatrix,
    property_types::ViewTransitionElementResourceId,
};
use crate::cc::trees::{damage_reason::DamageReasonSet, damage_tracker::DamageTracker};
use std::cmp::Ordering;

// Source dependency: components/viz/common/view_transition_element_resource_id.h:42-43.
// Its defaulted spaceship compares members in declaration order. Inputs to
// DamageTracker's resource map are valid resource ids, as checked by the source.
impl PartialOrd for ViewTransitionElementResourceId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ViewTransitionElementResourceId {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.transition_token,
            self.local_id,
            self.for_scope_snapshot,
        )
            .cmp(&(
                other.transition_token,
                other.local_id,
                other.for_scope_snapshot,
            ))
    }
}
pub trait DamageTransform: Transform + super::property_tree::PropertyTreeTransform {
    fn ToFlattenedSkMatrix(&self) -> SkMatrix;
}
pub trait DamageFilters {
    fn HasFilterThatMovesPixels(&self) -> bool;
    fn MapRect(&self, rect: Rect, transform: &SkMatrix) -> Rect;
}
pub trait DamageEffectNode {
    fn target_id(&self) -> i32;
    fn transform_id(&self) -> i32;
    fn view_transition_element_resource_id(&self) -> ViewTransitionElementResourceId;
}
pub trait DamageLayer<T: DamageTransform, S: Copy + Eq> {
    type Types: LayerTreeTypes<Transform = T>;
    fn id(&self) -> i32;
    fn contributes_to_drawn_render_surface(&self) -> bool;
    fn render_target_effect_tree_index(&self, trees: &PropertyTreesFor<Self::Types>) -> i32;
    fn render_target(&self, trees: &PropertyTreesFor<Self::Types>) -> S;
    fn GetEnclosingVisibleRectInTargetSpace(&self, trees: &PropertyTreesFor<Self::Types>) -> Rect;
    fn ViewTransitionResourceId(&self) -> ViewTransitionElementResourceId;
    fn LayerPropertyChanged(&self, trees: &PropertyTreesFor<Self::Types>) -> bool;
    fn GetDamageReasons(&self, trees: &PropertyTreesFor<Self::Types>) -> DamageReasonSet;
    fn update_rect(&self) -> Rect;
    fn GetDamageRect(&self) -> Rect;
    fn visible_layer_rect(&self) -> Rect;
    fn bounds(&self, trees: &PropertyTreesFor<Self::Types>) -> Size;
    fn DrawTransform(&self, trees: &PropertyTreesFor<Self::Types>) -> T;
    fn LayerPropertyChangedFromPropertyTrees(&self, trees: &PropertyTreesFor<Self::Types>) -> bool;
    fn effect_tree_index(&self) -> i32;
    fn transform_tree_index(&self) -> i32;
    fn LayerPropertyChangedNotFromPropertyTrees(&self) -> bool;
}
pub trait DamageSurface<T: DamageTransform, S: Copy + Eq> {
    type Types: LayerTreeTypes<Transform = T>;
    fn id(&self) -> ElementId;
    fn render_target(&self, trees: &PropertyTreesFor<Self::Types>) -> S;
    fn EffectTreeIndex(&self) -> i32;
    fn OwningEffectNode<'a>(
        &self,
        trees: &'a PropertyTreesFor<Self::Types>,
    ) -> &'a dyn DamageEffectNode;
    fn BackdropFilters<'a>(
        &self,
        trees: &'a PropertyTreesFor<Self::Types>,
    ) -> &'a dyn DamageFilters;
    fn Filters<'a>(&self, trees: &'a PropertyTreesFor<Self::Types>) -> &'a dyn DamageFilters;
    fn intersects_damage_under(&self) -> bool;
    fn set_intersects_damage_under(&self, intersects: bool);
    fn SurfacePropertyChanged(&self) -> bool;
    fn AncestorPropertyChanged(&self, trees: &PropertyTreesFor<Self::Types>) -> bool;
    fn SurfaceScale(&self, trees: &PropertyTreesFor<Self::Types>) -> T;
    fn content_rect(&self) -> Rect;
    fn DrawableContentRect(&self, trees: &PropertyTreesFor<Self::Types>) -> RectF;
    fn draw_transform(&self) -> &T;
}
pub trait DamageContext: Sized {
    type Types: LayerTreeTypes<Transform = Self::Transform>;
    type Transform: DamageTransform;
    type Surface: Copy + Eq;
    type Layer: Copy + Eq;
    type LayerIterator<'a>: Iterator<Item = Self::Layer>
    where
        Self: 'a;
    type SurfaceIterator<'a>: Iterator<Item = i32>
    where
        Self: 'a;
    fn layers(&self) -> Self::LayerIterator<'_>;
    fn property_trees(&self) -> &PropertyTreesFor<Self::Types>;
    fn hud_layer(&self) -> Option<Self::Layer>;
    fn GetRenderSurfaceList(&self) -> Self::SurfaceIterator<'_>;
    fn GetRenderSurface(&self, effect_id: i32) -> Option<Self::Surface>;
    fn layer(
        &self,
        identity: Self::Layer,
    ) -> &dyn DamageLayer<Self::Transform, Self::Surface, Types = Self::Types>;
    fn surface(
        &self,
        identity: Self::Surface,
    ) -> &dyn DamageSurface<Self::Transform, Self::Surface, Types = Self::Types>;
    fn damage_tracker(&self, identity: Self::Surface) -> &DamageTracker<Self::Surface>;
    // The transform operation belongs to the existing Viz geometry owner.
    fn GetViewTransitionTransform(
        &self,
        shared_element_rect: Rect,
        content_rect: Rect,
    ) -> Self::Transform;
}
