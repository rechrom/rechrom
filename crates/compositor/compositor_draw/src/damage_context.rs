//! Live dependencies of `DamageTracker`. Every tracker reference is the
//! resident tracker owned by its render surface; no tree, surface, layer, or
//! tracker state is copied into an adapter projection.

use crate::{damage_tracker::DamageTracker, occlusion::SkMatrix};
use compositor_tree::{
    damage_reason::DamageReasonSet,
    geometry::{Rect, RectF, Size, Transform},
    property_types::ViewTransitionElementResourceId,
    transform::PropertyTreeTransform,
    ElementId,
};

pub trait DamageTransform: Transform + PropertyTreeTransform {
    fn ToFlattenedSkMatrix(&self) -> SkMatrix;
}

pub trait DamageLayer<T: DamageTransform, S: Copy + Eq> {
    type Trees;
    fn id(&self) -> i32;
    fn contributes_to_drawn_render_surface(&self) -> bool;
    fn render_target_effect_tree_index(&self, trees: &Self::Trees) -> i32;
    fn render_target(&self, trees: &Self::Trees) -> S;
    fn GetEnclosingVisibleRectInTargetSpace(&self, trees: &Self::Trees) -> Rect;
    fn ViewTransitionResourceId(&self) -> ViewTransitionElementResourceId;
    fn LayerPropertyChanged(&self, trees: &Self::Trees) -> bool;
    fn GetDamageReasons(&self, trees: &Self::Trees) -> DamageReasonSet;
    fn update_rect(&self) -> Rect;
    fn GetDamageRect(&self) -> Rect;
    fn visible_layer_rect(&self) -> Rect;
    fn bounds(&self, trees: &Self::Trees) -> Size;
    fn DrawTransform(&self, trees: &Self::Trees) -> T;
    fn LayerPropertyChangedFromPropertyTrees(&self, trees: &Self::Trees) -> bool;
    fn effect_tree_index(&self) -> i32;
    fn transform_tree_index(&self) -> i32;
    fn LayerPropertyChangedNotFromPropertyTrees(&self) -> bool;
}

pub trait DamageSurface<T: DamageTransform, S: Copy + Eq> {
    type Trees;
    fn id(&self) -> ElementId;
    fn render_target(&self, trees: &Self::Trees) -> S;
    fn EffectTreeIndex(&self) -> i32;
    fn ViewTransitionElementResourceId(
        &self,
        trees: &Self::Trees,
    ) -> ViewTransitionElementResourceId;
    fn BackdropFiltersMovePixels(&self, trees: &Self::Trees) -> bool;
    fn MapFiltersRect(&self, rect: Rect, transform: &SkMatrix, trees: &Self::Trees) -> Rect;
    fn intersects_damage_under(&self) -> bool;
    fn set_intersects_damage_under(&self, intersects: bool);
    fn SurfacePropertyChanged(&self) -> bool;
    fn AncestorPropertyChanged(&self, trees: &Self::Trees) -> bool;
    fn SurfaceScale(&self, trees: &Self::Trees) -> T;
    fn content_rect(&self) -> Rect;
    fn DrawableContentRect(&self, trees: &Self::Trees) -> RectF;
    fn draw_transform(&self) -> &T;
}

pub trait DamageContext: Sized {
    type Trees;
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
    fn property_trees(&self) -> &Self::Trees;
    fn hud_layer(&self) -> Option<Self::Layer>;
    fn GetRenderSurfaceList(&self) -> Self::SurfaceIterator<'_>;
    fn GetRenderSurface(&self, effect_id: i32) -> Option<Self::Surface>;
    fn LowestCommonAncestorWithRenderSurface(&self, first: i32, second: i32) -> i32;
    fn EffectTargetId(&self, effect_id: i32) -> i32;
    fn EffectTransformId(&self, effect_id: i32) -> i32;
    fn layer(
        &self,
        identity: Self::Layer,
    ) -> &dyn DamageLayer<Self::Transform, Self::Surface, Trees = Self::Trees>;
    fn surface(
        &self,
        identity: Self::Surface,
    ) -> &dyn DamageSurface<Self::Transform, Self::Surface, Trees = Self::Trees>;
    fn damage_tracker(&self, identity: Self::Surface) -> &DamageTracker<Self::Surface>;
    fn GetViewTransitionTransform(
        &self,
        shared_element_rect: Rect,
        content_rect: Rect,
    ) -> Self::Transform;
}
