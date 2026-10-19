//! Call-scoped AnimationHost delegate capabilities. All reads borrow the
//! current resident trees; all writes synchronously enter canonical Host code.
//! No animation callback retains a Host, tree or this borrowed facade.
use super::{
    geometry::{PointF, Vector2dF},
    input_types::ElementId,
    layer_tree_host_impl::AnimationWorkletMutationState,
    layer_tree_impl::{ElementListType, LayerTreeHostId},
    time::{TimeDelta, TimeTicks},
};
use crate::cc::trees::property_tree::ScrollTree;

pub trait AnimationHostQueries {
    fn AnimationHostOwner(&self) -> LayerTreeHostId;
    fn IsElementInPropertyTrees(&self, element: ElementId, list: ElementListType) -> bool;
    fn CurrentScrollOffset(&self, element: ElementId, list: ElementListType) -> PointF;
    /// Each callback ends before a regular animation mutates an offset. Worklet
    /// input collection occurs after those mutations and observes the same tree.
    fn WithScrollTree(&self, list: ElementListType, callback: &mut dyn FnMut(&ScrollTree));
}
pub trait AnimationMutationTarget: AnimationHostQueries {
    // Source: cc/trees/mutator_host_delegate.h:39-40,55-59,71-75.
    fn SetMutatorsNeedCommit(&mut self);
    fn SetMutatorsNeedRebuildPropertyTrees(&mut self);
    fn SetElementScrollOffsetMutated(
        &mut self,
        element: ElementId,
        list: ElementListType,
        value: &PointF,
    );
    fn ScrollOffsetAnimationFinished(
        &mut self,
        element: ElementId,
        selected: Option<&mut dyn ScrollAnimationReentry>,
    );
    fn NotifyAnimationWorkletStateChange(
        &mut self,
        state: AnimationWorkletMutationState,
        list: ElementListType,
    );
}

/// Same-stack access to the actual Animation currently dispatching Finished.
/// The capability is borrowed only by that synchronous Host/Input call. It is
/// never stored in an owner, task, Engine output, or thread-local registry.
pub trait ScrollAnimationReentry {
    fn MatchesElement(&self, element: ElementId) -> bool;
    fn CreateAutoScrollAnimation(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        velocity: f32,
        start: TimeDelta,
    );
    fn CreateMouseWheelScrollAnimation(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        delayed: TimeDelta,
        start: TimeDelta,
    );
    fn AbortScrollAnimation(&mut self, element: ElementId, needs_completion: bool);
    fn UpdateScrollAnimationTarget(
        &mut self,
        delta: Vector2dF,
        max: PointF,
        time: TimeTicks,
        delayed: TimeDelta,
        element: ElementId,
    ) -> Option<PointF>;
    fn ElementHasImplOnlyScrollAnimation(&self, element: ElementId) -> bool;
}
