//! Live dependencies of the translated occlusion walk. Identities point into
//! the retained owner; every field/filter/node query borrows that owner's live
//! object. No mirrored layer/property/surface state is materialized here.
use super::{
    geometry::{Rect, Transform},
    layer_tree_impl::{LayerTreeTypes, PropertyTreesFor},
};
use crate::cc::base::simple_enclosed_region::SimpleEnclosedRegion;

pub trait OcclusionTransform:
    Transform + super::property_tree::PropertyTreeTransform + Clone + Default + PartialEq
{
    fn NonDegeneratePreserves2dAxisAlignment(&self) -> bool;
    fn Multiply(&self, other: &Self) -> Self;
    fn ToString(&self) -> String;
}
// Both names denote the same external SkBlendMode value.
pub use super::property_types::BlendMode as SkBlendMode;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkMatrix {
    pub values: [f32; 9],
}
impl SkMatrix {
    pub const fn I() -> Self {
        Self {
            values: [1., 0., 0., 0., 1., 0., 0., 0., 1.],
        }
    }
}
pub trait OcclusionFilters {
    fn HasFilterThatAffectsOpacity(&self) -> bool;
    fn HasFilterThatMovesPixels(&self) -> bool;
    fn MapRectReverse(&self, rect: Rect, transform: &SkMatrix) -> Rect;
}
pub trait OcclusionEffectNode {
    fn screen_space_opacity(&self) -> f32;
    fn view_transition_element_resource_id_is_valid(&self) -> bool;
    fn HasRenderSurface(&self) -> bool;
    fn blend_mode(&self) -> SkBlendMode;
}
pub trait OcclusionLayerTree<S: Copy + Eq> {
    fn RootRenderSurface(&self) -> Option<S>;
}
pub trait OcclusionMaskFilterInfo {
    fn IsEmpty(&self) -> bool;
}
pub trait OcclusionDrawProperties {
    fn mask_filter_info(&self) -> &dyn OcclusionMaskFilterInfo;
}
pub trait OcclusionLayer<T: OcclusionTransform, S: Copy + Eq> {
    type Types: LayerTreeTypes<Transform = T>;
    fn render_target(&self, trees: &PropertyTreesFor<Self::Types>) -> S;
    fn draw_opacity(&self) -> f32;
    fn Is3dSorted(&self, trees: &PropertyTreesFor<Self::Types>) -> bool;
    fn draw_properties(&self) -> &dyn OcclusionDrawProperties;
    fn VisibleOpaqueRegion(&self) -> SimpleEnclosedRegion;
    fn effect_tree_index(&self) -> i32;
    fn visible_layer_rect(&self) -> Rect;
    fn DrawTransform(&self, trees: &PropertyTreesFor<Self::Types>) -> T;
    fn is_clipped(&self) -> bool;
    fn clip_rect(&self) -> Rect;
}
pub trait OcclusionRenderSurface<T: OcclusionTransform, S: Copy + Eq> {
    type Types: LayerTreeTypes<Transform = T>;
    fn screen_space_transform(&self) -> &T;
    fn content_rect(&self) -> Rect;
    fn nearest_occlusion_immune_ancestor(&self) -> Option<S>;
    fn render_target(&self, trees: &PropertyTreesFor<Self::Types>) -> S;
    fn OwningEffectNode<'a>(
        &self,
        trees: &'a PropertyTreesFor<Self::Types>,
    ) -> &'a dyn OcclusionEffectNode;
    fn CopyOfOutputRequired(&self, trees: &PropertyTreesFor<Self::Types>) -> bool;
    fn HasMaskingContributingSurface(&self, trees: &PropertyTreesFor<Self::Types>) -> bool;
    fn draw_opacity(&self) -> f32;
    fn BlendMode(&self, trees: &PropertyTreesFor<Self::Types>) -> SkBlendMode;
    fn Filters<'a>(&self, trees: &'a PropertyTreesFor<Self::Types>) -> &'a dyn OcclusionFilters;
    fn BackdropFilters<'a>(
        &self,
        trees: &'a PropertyTreesFor<Self::Types>,
    ) -> &'a dyn OcclusionFilters;
    fn is_clipped(&self) -> bool;
    fn clip_rect(&self) -> Rect;
    fn draw_transform(&self) -> &T;
}
pub trait OcclusionContext {
    type Types: LayerTreeTypes<Transform = Self::Transform>;
    fn property_trees(&self) -> &PropertyTreesFor<Self::Types>;
    type Transform: OcclusionTransform;
    type Surface: Copy + Eq;
    type Layer: Copy + Eq;
    fn surface(
        &self,
        identity: Self::Surface,
    ) -> &dyn OcclusionRenderSurface<Self::Transform, Self::Surface, Types = Self::Types>;
    fn layer(
        &self,
        identity: Self::Layer,
    ) -> &dyn OcclusionLayer<Self::Transform, Self::Surface, Types = Self::Types>;
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
