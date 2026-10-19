//! Native H and scrollbar capabilities. Animation methods borrow the same
//! AnimationHost allocation as G; metrics and effects forward native owners.
use super::{
    host_owners::MutatorEvents,
    input::MutatorHost,
    native_animation::NativeAnimationOwner,
    native_resource_events::{NativeResourceEffect, NativeResourceEffects},
    native_services::NativeEventSink,
};
use crate::{
    adapter::{
        animation_host::AnimationMutationTarget, geometry::TracedValue, layer_tree_host_impl::*,
        layer_tree_impl::LayerTreeHostId, property_types::PropertyTrace, time::TimeTicks,
    },
    NativeCompositorTypes,
};
use std::{collections::BTreeSet, sync::Arc};
pub(crate) struct NativeAnimationServices {
    animations: NativeAnimationOwner,
    effects: NativeResourceEffects,
    trace_categories: BTreeSet<String>,
    events: Arc<dyn NativeEventSink>,
}
impl NativeAnimationServices {
    pub(crate) fn new(
        animations: NativeAnimationOwner,
        effects: NativeResourceEffects,
        trace_categories: BTreeSet<String>,
        events: Arc<dyn NativeEventSink>,
    ) -> Self {
        Self {
            animations,
            effects,
            trace_categories,
            events,
        }
    }
    fn Check(&self, host: &MutatorHost) {
        assert_eq!(
            host.0.identity(),
            self.animations.identity(),
            "H and MutatorHost must share one native animation allocation"
        )
    }
}
impl HostImplHServices<NativeCompositorTypes> for NativeAnimationServices {
    fn MutatorHasImplOnlyScrollAnimatingElement(&self, host: &MutatorHost) -> bool {
        self.Check(host);
        self.animations.HasImplOnlyScrollAnimatingElement()
    }
    fn MutatorHasImplOnlyAutoScrollAnimatingElement(&self, host: &MutatorHost) -> bool {
        self.Check(host);
        self.animations.HasImplOnlyAutoScrollAnimatingElement()
    }
    fn MutatorTickAnimations(
        &mut self,
        owner: LayerTreeHostId,
        time: TimeTicks,
        active: bool,
        events: Option<&mut MutatorEvents>,
        target: &mut dyn AnimationMutationTarget,
    ) -> HostAnimationTickResult {
        assert_eq!(owner, target.AnimationHostOwner());
        self.animations.TickAnimations(time, active, events, target)
    }
    fn MutatorHasInvalidationAnimation(&self, host: &MutatorHost) -> bool {
        self.Check(host);
        self.animations.HasInvalidationAnimation()
    }
    fn MutatorHasNativePropertyAnimation(&self, host: &MutatorHost) -> bool {
        self.Check(host);
        self.animations.HasNativePropertyAnimation()
    }
    fn MutatorHasViewTransition(&self, host: &MutatorHost) -> bool {
        self.Check(host);
        self.animations.HasViewTransition()
    }
    fn MutatorUpdateAnimationState(
        &mut self,
        owner: LayerTreeHostId,
        start: bool,
        events: Option<&mut MutatorEvents>,
        target: &mut dyn AnimationMutationTarget,
    ) -> bool {
        assert_eq!(owner, target.AnimationHostOwner());
        self.animations.UpdateAnimationState(start, events, target)
    }
    fn MutatorActivateAnimations(
        &mut self,
        owner: LayerTreeHostId,
        events: Option<&mut MutatorEvents>,
        target: &mut dyn AnimationMutationTarget,
    ) -> bool {
        assert_eq!(owner, target.AnimationHostOwner());
        self.animations.ActivateAnimations(events, target)
    }
    fn MutatorEventsIsEmpty(&self, events: &MutatorEvents) -> bool {
        events.0.IsEmpty()
    }
    fn TrackersStartSequence(
        &mut self,
        trackers: &mut super::host_metrics::FrameSequenceTrackers<NativeCompositorTypes>,
        kind: HostAnimationSequence,
    ) {
        trackers.0.TrackersStartSequence(kind)
    }
    fn TrackersStopSequence(
        &mut self,
        trackers: &mut super::host_metrics::FrameSequenceTrackers<NativeCompositorTypes>,
        kind: HostAnimationSequence,
    ) {
        trackers.0.TrackersStopSequence(kind)
    }
    fn DidCompletePageScaleAnimationOnImplThread(&mut self, delegate: HostImplDelegateId) {
        self.effects
            .Push(NativeResourceEffect::PageScaleAnimationCompleted { delegate })
    }
    fn VideoFrameControllerOnBeginFrame(
        &mut self,
        controller: HostImplVideoFrameControllerId,
        args: &crate::adapter::begin_frame::BeginFrameArgs,
    ) {
        self.effects.Push(NativeResourceEffect::VideoBeginFrame {
            controller,
            args: args.clone(),
        })
    }
    fn SetVideoNeedsBeginFrames(&mut self, delegate: HostImplDelegateId, needs: bool) {
        self.effects
            .Push(NativeResourceEffect::VideoNeedsBeginFrames { delegate, needs })
    }
    fn GlobalTileStateTreePriority(
        &self,
        global: &crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority,
    ) -> crate::adapter::scheduler::TreePriority {
        global.tree_priority
    }
    fn NewHostTraceValue(&self) -> Box<super::host::TraceValue> {
        Box::new(super::native_host_picture::boxed_trace_value())
    }
    fn NewHostTraceValueJSON(&self) -> super::host::TraceValue {
        super::native_host_picture::boxed_trace_value()
    }
    fn HostTraceCategoryEnabled(&self, category: &str) -> bool {
        self.trace_categories.contains(category)
    }
    // Source dependency: components/viz/common/traced_value.cc:20-46.
    fn TraceTileSnapshot(
        &self,
        category: &str,
        value: &mut dyn PropertyTrace,
        name: &str,
        id: crate::adapter::tile::TileTraceId,
    ) {
        value.SetString("cat", category);
        value.SetString("id", &format!("{name}/0x{:x}", id.0))
    }
    fn TraceRasterSourceIdRef(
        &self,
        id: crate::adapter::prioritized_tile::RasterSourceTraceId,
        value: &mut dyn PropertyTrace,
        name: &str,
    ) {
        value.BeginDictionary(name);
        value.SetString("id_ref", &format!("0x{:x}", id.0));
        value.EndDictionary()
    }
    fn TraceTileDeletion(
        &self,
        category: &str,
        event: &str,
        id: crate::adapter::tile::TileTraceId,
        flow: &str,
    ) {
        self.events.Emit(
            "cc",
            format!("category={category} event={event} id={} flow={flow}", id.0),
        )
    }
    fn TileManagerBasicStateAsValueInto(
        &self,
        manager: &crate::cc::tiles::tile_manager::TileManager<NativeCompositorTypes>,
        value: &mut super::host::TraceValue,
    ) {
        manager.BasicStateAsValueInto(value)
    }
    fn TraceSetHostIdRef(
        &self,
        id: LayerTreeHostId,
        value: &mut super::host::TraceValue,
        name: &str,
    ) {
        {
            value.BeginDictionary(name);
            value.SetString("id_ref", &format!("0x{:x}", id.0));
            value.EndDictionary()
        }
    }
}
/// Real task/trace primitives for source scrollbar callbacks. The executor owns
/// only a weak typed token and the transport lease; delivery finds the current
/// canonical Host using its live Engine call stack.
pub(crate) struct NativeScrollbarRegistrationServices {
    pub(crate) animations: NativeAnimationOwner,
    pub(crate) executor: Arc<dyn crate::engine::CompositorExecutor>,
    pub(crate) commands: super::execution::EngineTaskQueue,
    pub(crate) events: Arc<dyn NativeEventSink>,
}
impl crate::adapter::host_picture::HostPictureRegistrationServices<NativeCompositorTypes>
    for NativeScrollbarRegistrationServices
{
    fn PostDelayedAutoscrollTask(
        &mut self,
        owner: HostImplWeak,
        task: crate::adapter::scrollbar::AutoscrollTask,
        delay: crate::adapter::time::TimeDelta,
    ) {
        let queue = self.commands.clone();
        self.executor.ExecuteAfter(
            delay,
            Box::new(move || queue.Push(super::execution::EngineTask::Autoscroll { owner, task })),
        )
    }
    fn ScrollAnimationAbort(
        &mut self,
        host: &mut MutatorHost,
        id: crate::adapter::input_types::ElementId,
    ) {
        assert_eq!(host.0.identity(), self.animations.identity());
        self.animations.AbortScrollAnimation(id, false)
    }
}
impl HostImplScrollbarRegistrationServices<NativeCompositorTypes>
    for NativeScrollbarRegistrationServices
{
    fn PostDelayedAutoscrollTask(
        &mut self,
        owner: HostImplWeak,
        task: crate::adapter::scrollbar::AutoscrollTask,
        delay: crate::adapter::time::TimeDelta,
    ) {
        crate::adapter::host_picture::HostPictureRegistrationServices::PostDelayedAutoscrollTask(
            self, owner, task, delay,
        )
    }
    fn TraceScrollbarNeedsAnimate(&self) {
        self.events.Emit("cc", "ScrollbarNeedsAnimate".into())
    }
    fn PostDelayedAnimationTaskOnImplThread(
        &mut self,
        delegate: HostImplDelegateId,
        task: crate::adapter::scrollbar_animation::ScrollbarAnimationTask,
        delay: crate::adapter::time::TimeDelta,
    ) {
        let queue = self.commands.clone();
        self.executor.ExecuteAfter(
            delay,
            Box::new(move || {
                queue.Push(super::execution::EngineTask::ScrollbarAnimation { delegate, task })
            }),
        )
    }
}
