//! The cc::MutatorHost interface (cc/trees/mutator_host.h) and its client
//! (cc/trees/mutator_host_client.h, mutator_host_delegate.h), shared by the
//! main LayerTreeHost and the impl LayerTreeHostImpl. The AnimationHost
//! (`super::animation_host`) implements the host side; each tree host
//! implements the client side. Moved from compositor/src/adapter
//! (animation_host.rs, layer_tree_impl.rs, layer_tree_host_impl.rs,
//! client_layer_tree_host_impl.rs) and compositor/src/runtime (input.rs,
//! host_owners.rs, types.rs), which re-export them.
use super::{
    geometry::{PointF, Vector2dF},
    input_types::ElementId,
    property_tree::ScrollTree,
    property_tree_services::PropertyTreeMutatorHost,
};
use compositor_animation::{
    animation_events::AnimationEvents, keyframe_model::CompositorAnimationCurve,
};
pub use compositor_animation::ElementListType;
pub use compositor_tree::LayerTreeHostId;
use compositor_tree::target_property::Type as TargetProperty;
use compositor_values::layer_tree_mutator::LayerTreeMutator;
use scroll_animation::{TimeDelta, TimeTicks};

// Source: cc/trees/mutator_host_delegate.h:27-32.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AnimationWorkletMutationState {
    STARTED = 0,
    COMPLETED_WITH_UPDATE = 1,
    COMPLETED_NO_UPDATE = 2,
    CANCELED = 3,
}

// Source: cc/trees/mutator_host.h:29-38.
#[derive(Clone, Copy, Debug, Default)]
pub struct HostAnimationTickResult {
    pub animated: bool,
    pub needs_next_frame: bool,
}

/// A FrameSequenceTracker start/stop request the AnimationHost queues for
/// its tree host (AnimationHost::pending_compositor_metrics_tracker_infos_,
/// cc/animation/animation_host.h).
pub struct TrackedAnimationSequenceInfo {
    pub id: usize,
    pub start: bool,
}

/// Call-scoped MutatorHostClient capabilities (cc/trees/mutator_host_client.h).
/// All reads borrow the current resident trees; all writes synchronously
/// enter canonical Host code. No animation callback retains a Host, tree or
/// this borrowed facade.
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
    fn SetElementOpacityMutated(&mut self, element: ElementId, list: ElementListType, value: f32);
    fn SetElementTransformMutated(
        &mut self,
        element: ElementId,
        list: ElementListType,
        matrix: &[f64; 16],
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

/// The cc::MutatorHost queries LayerTreeImpl makes (cc/trees/mutator_host.h).
pub trait LayerTreeMutatorHost {
    fn HasAnyAnimationTargetingProperty(&self, id: ElementId, property: TargetProperty) -> bool;
    fn HasPotentiallyRunningAnimationForProperty(
        &self,
        id: ElementId,
        list: ElementListType,
        property: TargetProperty,
    ) -> bool;
    fn MaximumScale(&self, id: ElementId, list: ElementListType) -> f32;
}

/// Native ThreadUnsafeCommitState's non-owning MutatorHost* identity. The
/// actual animation owner remains in its existing subsystem and is never
/// resolved through a global registry or retained by a Layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainMutatorHostIdentity(pub u64);

/// The std::unique_ptr<MutatorHost> LayerTreeHostImpl owns
/// (cc/trees/layer_tree_host_impl.h mutator_host_): the exact read interface
/// consumed by cc trees. Animation state remains owned by the AnimationHost.
pub trait MutatorHostBackend:
    LayerTreeMutatorHost + PropertyTreeMutatorHost + std::any::Any
{
    fn AsAnyMut(&mut self) -> &mut dyn std::any::Any;
    fn SetLayerTreeMutator(&mut self, mutator: Option<Box<LayerTreeMutator>>);
    fn identity(&self) -> MainMutatorHostIdentity;
    fn NextFrameHasPendingRAF(&self) -> bool;
    fn HasCanvasInvalidation(&self) -> bool;
    fn HasJSAnimation(&self) -> bool;
    fn MainThreadAnimationsCount(&self) -> usize;
    fn HasSmilAnimation(&self) -> bool;
}
pub struct MutatorHost(pub Box<dyn MutatorHostBackend>);
impl MutatorHost {
    pub fn identity(&self) -> MainMutatorHostIdentity {
        self.0.identity()
    }
    pub fn SetLayerTreeMutator(&mut self, mutator: Option<Box<LayerTreeMutator>>) {
        self.0.SetLayerTreeMutator(mutator)
    }
    pub fn NextFrameHasPendingRAF(&self) -> bool {
        self.0.NextFrameHasPendingRAF()
    }
    pub fn HasCanvasInvalidation(&self) -> bool {
        self.0.HasCanvasInvalidation()
    }
    pub fn HasJSAnimation(&self) -> bool {
        self.0.HasJSAnimation()
    }
    pub fn MainThreadAnimationsCount(&self) -> usize {
        self.0.MainThreadAnimationsCount()
    }
    pub fn HasSmilAnimation(&self) -> bool {
        self.0.HasSmilAnimation()
    }
}
impl LayerTreeMutatorHost for MutatorHost {
    fn HasAnyAnimationTargetingProperty(&self, id: ElementId, property: TargetProperty) -> bool {
        self.0.HasAnyAnimationTargetingProperty(id, property)
    }
    fn HasPotentiallyRunningAnimationForProperty(
        &self,
        id: ElementId,
        list: ElementListType,
        property: TargetProperty,
    ) -> bool {
        self.0
            .HasPotentiallyRunningAnimationForProperty(id, list, property)
    }
    fn MaximumScale(&self, id: ElementId, list: ElementListType) -> f32 {
        self.0.MaximumScale(id, list)
    }
}
impl PropertyTreeMutatorHost for MutatorHost {
    fn ScrollOffsetAnimationWasInterrupted(&self, id: ElementId) -> bool {
        self.0.ScrollOffsetAnimationWasInterrupted(id)
    }
}

/// cc::MutatorEvents (cc/trees/mutator_host.h): the AnimationHost's events
/// stay in their native animation allocation. The host only tests emptiness
/// and transfers the same owner to the mutator.
/// `Send`: ProxyImpl::ScheduledActionSendBeginMainFrame moves the events to
/// ProxyMain::BeginMainFrame inside BeginMainFrameAndCommitState
/// (cc/trees/proxy_impl.cc:820, proxy_main.cc:351-352).
pub trait MutatorEventsBackend: Send {
    fn IsEmpty(&self) -> bool;
    fn WithScrollAnimationEvents(
        &mut self,
        callback: &mut dyn FnMut(&mut AnimationEvents<CompositorAnimationCurve>),
    );
}
pub struct MutatorEvents(pub Box<dyn MutatorEventsBackend>);
impl MutatorEvents {
    pub fn WithScrollAnimationEvents(
        &mut self,
        callback: &mut dyn FnMut(&mut AnimationEvents<CompositorAnimationCurve>),
    ) {
        self.0.WithScrollAnimationEvents(callback)
    }
    pub fn IsEmpty(&self) -> bool {
        self.0.IsEmpty()
    }
}
