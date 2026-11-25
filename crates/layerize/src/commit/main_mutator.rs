//! The main-thread cc::AnimationHost as the main half sees it.
//!
//! Chromium creates two AnimationHost instances per compositor: Blink's
//! LayerTreeView owns the main one (`AnimationHost::CreateMainInstance`,
//! cc/animation/animation_host.cc:55-57) and hands a raw pointer to
//! LayerTreeHost (`LayerTreeHost::InitParams::mutator_host`), which stores it
//! in `ThreadUnsafeCommitState::mutator_host` (cc/trees/commit_state.h:205).
//! The impl instance is created by `AnimationHost::CreateImplInstance`
//! (:74-79) for LayerTreeHostImpl. At commit, LayerTreeImpl reads the main
//! instance only through the unsafe commit state
//! (`unsafe_state.mutator_host->PushPropertiesTo(host_impl->mutator_host(),
//! property_trees)`, cc/trees/layer_tree_impl.cc:814-817), i.e. inside the
//! protected sequence of the commit lease.
//!
//! The AnimationHost itself is the shared cc owner
//! `super::animation_host::AnimationHostOwner` (its timelines, keyframe
//! effects and scroll-offset animations are compositor_animation values); main
//! code generic over the family names it only through this trait.
//! `LayerizeEngine` owns the main instance (the LayerTreeView role) and the
//! main LayerTreeHost stores the same owner as
//! `ThreadUnsafeCommitState::mutator_host`.
use super::input_types::ElementId;
use compositor_animation::compositor_mutation::PropertyAnimationMutation;
use scroll_animation::compositor_mutation::{ScrollAnimationCompletion, ScrollAnimationMutation};

/// The main AnimationHost API the main half calls. Clones share one
/// instance (the owner handle and the unsafe-state pointer of Chromium).
/// `Send + Sync`: the impl thread reads it inside the protected sequence.
pub trait MainMutatorHost: Clone + Send + Sync + 'static {
    /// cc::MutatorEvents, produced by the impl instance
    /// (LayerTreeHostImpl::TakeMutatorEvents, cc/trees/proxy_impl.cc:820) and
    /// carried to main by BeginMainFrameAndCommitState::mutator_events.
    type Events: Send;
    /// Blink's scroll-offset animation updates (ScrollOffsetAnimations /
    /// AddImplOnlyScrollAnimation counterparts on the main instance). Returns
    /// the scroll element whose next compositor delta PaintArtifactCompositor
    /// must drop (PAC::DropCompositorScrollDeltaNextCommit).
    fn ApplySourceScrollAnimation(&self, mutation: ScrollAnimationMutation) -> Option<ElementId>;
    /// Blink CompositorAnimation timeline/keyframe-model updates.
    fn ApplySourcePropertyAnimation(&self, mutation: PropertyAnimationMutation);
    /// AnimationHost::SetAnimationEvents (cc/animation/animation_host.cc:
    /// 795-822), from LayerTreeHost::ApplyMutatorEvents.
    fn SetAnimationEvents(&self, events: Self::Events);
    /// The AnimationDelegate notifications of finished source scroll
    /// animations, delivered to the embedder after the events above.
    fn TakeSourceScrollCompletions(&self) -> Vec<ScrollAnimationCompletion>;
    /// MutatorHostClient::SetMutatorsNeedCommit, latched since the last call.
    fn TakeNeedsCommit(&self) -> bool;
    /// AnimationHost::IsElementAnimating (LayerTreeHost::IsElementAnimating
    /// forwards to mutator_host_, cc/trees/layer_tree_host.cc).
    fn IsElementAnimating(&self, id: ElementId) -> bool;
}
