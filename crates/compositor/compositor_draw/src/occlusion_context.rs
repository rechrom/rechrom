//! Live dependencies of the translated occlusion walk. Identities point into
//! retained owner storage and all queries are synchronous; no layer, property
//! tree, or surface snapshot is materialized here.

use crate::{
    occlusion::{OcclusionTransform as BaseOcclusionTransform, SkMatrix},
    simple_enclosed_region::SimpleEnclosedRegion,
};
use compositor_tree::{
    geometry::{Rect, Transform},
    transform::PropertyTreeTransform,
};

pub trait OcclusionTransform: BaseOcclusionTransform + Transform + PropertyTreeTransform {
    fn NonDegeneratePreserves2dAxisAlignment(&self) -> bool;
    fn Multiply(&self, other: &Self) -> Self;
    fn ToString(&self) -> String;
}

pub use compositor_tree::property_types::BlendMode as SkBlendMode;

pub trait OcclusionLayerTree<S: Copy + Eq> {
    fn RootRenderSurface(&self) -> Option<S>;
}

pub trait OcclusionLayer<T: OcclusionTransform, S: Copy + Eq> {
    type Trees;
    fn render_target(&self, trees: &Self::Trees) -> S;
    fn draw_opacity(&self) -> f32;
    fn Is3dSorted(&self, trees: &Self::Trees) -> bool;
    fn MaskFilterInfoIsEmpty(&self) -> bool;
    fn VisibleOpaqueRegion(&self) -> SimpleEnclosedRegion;
    fn effect_tree_index(&self) -> i32;
    fn visible_layer_rect(&self) -> Rect;
    fn DrawTransform(&self, trees: &Self::Trees) -> T;
    fn is_clipped(&self) -> bool;
    fn clip_rect(&self) -> Rect;
}

pub trait OcclusionRenderSurface<T: OcclusionTransform, S: Copy + Eq> {
    type Trees;
    fn screen_space_transform(&self) -> &T;
    fn content_rect(&self) -> Rect;
    fn nearest_occlusion_immune_ancestor(&self) -> Option<S>;
    fn render_target(&self, trees: &Self::Trees) -> S;
    fn screen_space_opacity(&self, trees: &Self::Trees) -> f32;
    fn view_transition_element_resource_id_is_valid(&self, trees: &Self::Trees) -> bool;
    fn CopyOfOutputRequired(&self, trees: &Self::Trees) -> bool;
    fn HasMaskingContributingSurface(&self, trees: &Self::Trees) -> bool;
    fn draw_opacity(&self) -> f32;
    fn BlendMode(&self, trees: &Self::Trees) -> SkBlendMode;
    fn FiltersAffectOpacity(&self, trees: &Self::Trees) -> bool;
    fn BackdropFiltersMovePixels(&self, trees: &Self::Trees) -> bool;
    fn MapBackdropFilterRectReverse(
        &self,
        rect: Rect,
        transform: &SkMatrix,
        trees: &Self::Trees,
    ) -> Rect;
    fn is_clipped(&self) -> bool;
    fn clip_rect(&self) -> Rect;
    fn draw_transform(&self) -> &T;
}

pub trait OcclusionContext {
    type Trees;
    fn property_trees(&self) -> &Self::Trees;
    type Transform: OcclusionTransform;
    type Surface: Copy + Eq;
    type Layer: Copy + Eq;
    fn surface(
        &self,
        identity: Self::Surface,
    ) -> &dyn OcclusionRenderSurface<Self::Transform, Self::Surface, Trees = Self::Trees>;
    fn layer(
        &self,
        identity: Self::Layer,
    ) -> &dyn OcclusionLayer<Self::Transform, Self::Surface, Trees = Self::Trees>;
    fn EffectHasRenderSurface(&self, identity: i32) -> bool;
    fn EffectBlendMode(&self, identity: i32) -> SkBlendMode;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectTreeLayerListIteratorState {
    kLayer,
    kTargetSurface,
    kContributingSurface,
}

#[derive(Clone, Copy, Debug)]
pub struct EffectTreeLayerListIteratorPosition<S, L> {
    pub state: EffectTreeLayerListIteratorState,
    pub target_render_surface: S,
    pub current_layer: Option<L>,
}
