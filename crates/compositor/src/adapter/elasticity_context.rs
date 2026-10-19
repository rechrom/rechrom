//! Borrowed live facets of the LayerTreeImpl/HostImpl owners. Tree algorithms
//! operate directly on canonical ScrollTree/TransformTree values. Mutation
//! contexts split sibling borrows; no projected node/tree storage is created.
use super::{
    geometry::PointF, input_types::ElementId, property_tree::PropertyTreeTransform,
    scroll_elasticity::LayerTreeHostImpl,
};
use crate::cc::trees::property_tree::{ScrollTree, ScrollTreeMutationContext};
pub trait ElasticityTreeEffects {
    fn set_needs_update_draw_properties(&mut self);
}
pub trait ElasticityHostEffects: LayerTreeHostImpl {
    fn SetNeedsRedraw(&mut self, animation_only: bool, skip_if_inside_draw: bool);
    fn SetFullViewportDamage(&mut self);
    fn RequestImplSideInvalidationForRasterInducingScroll(&mut self, id: ElementId);
}
pub struct ElasticityTreeMutation<'a, T: PropertyTreeTransform> {
    pub scroll_tree: &'a mut ScrollTree,
    pub context: ScrollTreeMutationContext<'a, T>,
    pub tree_effects: &'a mut dyn ElasticityTreeEffects,
    pub host_policy: &'a dyn LayerTreeHostImpl,
    pub host_effects: Option<&'a mut dyn ElasticityHostEffects>,
    pub is_active_tree: bool,
}
/// Non-owning source owner identity, validated at every helper invocation.
pub trait ElasticityOwner {
    fn identity(&self) -> super::layer_tree_impl::LayerTreeHostId;
}
pub trait ElasticityQueries: ElasticityOwner + LayerTreeHostImpl {
    fn GetScrollTree(&self) -> &ScrollTree;
    fn InnerViewportScrollNodeId(&self) -> Option<i32>;
    fn OuterViewportScrollNodeId(&self) -> Option<i32>;
    fn enable_elastic_overscroll_on_root(&self) -> bool;
    fn enable_elastic_overscroll_for_subscroll(&self) -> bool;
    fn active_scroll_tree(&self) -> &ScrollTree;
    fn TotalScrollOffset(&self, id: Option<ElementId>) -> PointF;
    fn TotalMaxScrollOffset(&self, id: Option<ElementId>) -> PointF;
}
pub trait ElasticityContext: ElasticityQueries {
    type Transform: PropertyTreeTransform;
    /// Source cc/input/scroll_elasticity_helper.cc:174-290. Callback-local
    /// parts allow effects/scroll siblings to be split only for the actual call.
    fn WithActiveTreeMutation<R>(
        &mut self,
        visit: impl FnOnce(ElasticityTreeMutation<'_, Self::Transform>) -> R,
    ) -> R;
    fn WithActiveAndPendingMutation<R>(
        &mut self,
        visit: impl FnOnce(&ScrollTree, ElasticityTreeMutation<'_, Self::Transform>) -> R,
    ) -> R;
    fn ScrollByNodeId(&mut self, node_id: i32, delta: &super::geometry::Vector2dF);
    fn SetNeedsOneBeginImplFrame(&mut self);
}
/// Source cc/input/scroll_elasticity_helper.cc:328-330 only requires this
/// callback capability; a readonly input facade never pretends to own InputHandler.
pub trait ElasticityAnimationContext: ElasticityOwner {
    fn ElasticOverscrollAnimationFinished(&mut self, id: ElementId);
}
