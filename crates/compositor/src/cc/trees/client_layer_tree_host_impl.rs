// Translation status: complete
// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/client_layer_tree_host_impl.h:1-18.
// Source: cc/trees/client_layer_tree_host_impl.h:32-33.
// Source: cc/trees/client_layer_tree_host_impl.h:68-72.
// Source: cc/trees/client_layer_tree_host_impl.cc:1-28.
// This source subclass introduces no fields. Rust composition preserves its
// single inherited resident owner; all synchronous services are call-scoped.
use super::{
    begin_main_frame_trace_id::BeginMainFrameTraceId,
    commit_state::{CommitState, ThreadUnsafeCommitState},
    layer_tree_host_impl::LayerTreeHostImpl,
    layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
};
use crate::adapter::picture_layer_impl::PictureLayerImplTypes;
use crate::adapter::{
    client_layer_tree_host_impl::*,
    layer_tree_host_impl::*,
    layer_tree_impl::{LayerTreeHostId, LayerTreeTypes},
};
use std::ops::{Deref, DerefMut};

pub struct ClientLayerTreeHostImpl<B: HostImplTypes + PictureLayerImplTypes> {
    base: LayerTreeHostImpl<B>,
}
impl<B: HostImplTypes + PictureLayerImplTypes> Deref for ClientLayerTreeHostImpl<B> {
    type Target = LayerTreeHostImpl<B>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> DerefMut for ClientLayerTreeHostImpl<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> ClientLayerTreeHostImpl<B> {
    // Source: cc/trees/client_layer_tree_host_impl.h:20-30.
    // Source: cc/trees/client_layer_tree_host_impl.cc:43-59.
    #[allow(clippy::too_many_arguments)]
    pub fn Create(
        owner: LayerTreeHostId,
        settings: &super::layer_tree_settings::LayerTreeSettings,
        delegate: HostImplDelegateId,
        task_runner_provider: HostImplTaskRunnerProviderId,
        rendering_stats: HostImplRenderingStatsId,
        task_graph_runner: HostImplTaskGraphRunnerId,
        mutator_host: Box<<B as LayerTreeTypes>::MutatorHost>,
        dark_mode_filter: Option<HostImplDarkModeFilterId>,
        id: i32,
        image_worker_task_runner: B::ImageWorkerTaskRunner,
        scheduling_delegate: HostImplSchedulingDelegateId,
        context: &mut HostImplContext<'_, B>,
    ) -> Self {
        assert!(!settings.trees_in_viz_in_viz_process);
        Self {
            base: LayerTreeHostImpl::new(
                owner,
                settings,
                delegate,
                task_runner_provider,
                rendering_stats,
                task_graph_runner,
                mutator_host,
                dark_mode_filter,
                id,
                image_worker_task_runner,
                scheduling_delegate,
                context,
            ),
        }
    }
    pub(crate) fn Base(&self) -> &LayerTreeHostImpl<B> {
        &self.base
    }
    pub(crate) fn BaseMutable(&mut self) -> &mut LayerTreeHostImpl<B> {
        &mut self.base
    }

    // Source: cc/trees/client_layer_tree_host_impl.h:44-45.
    // Source: cc/trees/client_layer_tree_host_impl.cc:109-118.
    pub fn BeginCommit(
        &mut self,
        source_frame_number: i32,
        trace_id: BeginMainFrameTraceId,
        context: &mut ClientHostContext<'_, B>,
    ) {
        context
            .services
            .TraceHostImplCEvent("cc", "LayerTreeHostImpl::BeginCommit");
        if !self.base.CommitsToActiveTree() {
            self.CreatePendingTree(context);
        }
        let tree = self.base.sync_tree().expect("source sync tree");
        tree.set_source_frame_number(source_frame_number);
        tree.set_trace_id(trace_id);
    }
    // A source sync_tree raw pointer is a fresh scoped borrow of the same
    // pending/active owner. No tree is taken out of its resident slot.
    fn WithSyncTree<R>(
        &mut self,
        context: &mut ClientHostContext<'_, B>,
        callback: impl FnOnce(&mut LayerTreeImpl<B>, &mut LayerTreeContext<'_, B>) -> R,
    ) -> R {
        if self.base.CommitsToActiveTree() {
            self.base.WithActiveTree(&mut context.Base(), callback)
        } else {
            self.base.WithPendingTree(&mut context.Base(), callback)
        }
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:56-56.
    // Source: cc/trees/client_layer_tree_host_impl.cc:343-360.
    pub fn CreatePendingTree(&mut self, context: &mut ClientHostContext<'_, B>) {
        let host = &mut self.base;
        assert!(!host.CommitsToActiveTree());
        assert!(host.pending_tree_.is_none());
        if host.recycle_tree_.is_some() {
            std::mem::swap(&mut host.recycle_tree_, &mut host.pending_tree_);
        } else {
            let (tree_id, property_owner) = context.services.AllocateLayerTreeIdentity();
            let active = host.active_tree_.as_deref().expect("source active tree");
            host.pending_tree_ = Some(Box::new(LayerTreeImpl::new(
                tree_id,
                property_owner,
                host.owner_id_,
                host.CurrentBeginFrameArgs().clone(),
                active.SharedPageScaleFactor(),
                active.SharedTopControlsShownRatio(),
                active.SharedBottomControlsShownRatio(),
            )));
        }
        host.pending_tree_fully_painted_ = false;
        let can_draw = host.CanDraw(&mut context.Base());
        context
            .services
            .OnCanDrawStateChanged(host.delegate_, can_draw);
        context.services.TracePendingTreeWaiting(
            host.pending_tree_.as_ref().unwrap().id,
            host.active_tree_
                .as_ref()
                .unwrap()
                .local_surface_id_from_parent(),
        );
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:55-55.
    // Source: cc/trees/client_layer_tree_host_impl.cc:338-341.
    pub fn SetTreePriority(
        &mut self,
        priority: crate::cc::tiles::tile_priority::TreePriority,
        context: &mut ClientHostContext<'_, B>,
    ) {
        self.base.global_tile_state_.set_tree_priority(priority);
        self.base
            .DidModifyTilePriorities(false, &mut context.Base());
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:54-54.
    // Source: cc/trees/client_layer_tree_host_impl.cc:332-336.
    pub fn InvalidateLayerTreeFrameSink(
        &mut self,
        needs_redraw: bool,
        context: &mut ClientHostContext<'_, B>,
    ) {
        let sink = self
            .base
            .layer_tree_frame_sink()
            .expect("source bound frame sink");
        context
            .services
            .ClientInvalidateFrameSink(sink, needs_redraw);
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:59-59.
    // Source: cc/trees/client_layer_tree_host_impl.cc:175-182.
    pub fn RecordGpuRasterizationHistogram(&self, context: &ClientHostContext<'_, B>) {
        context.services.ClientHistogramBoolean(
            "Renderer4.GpuRasterizationEnabled",
            self.base.raster_caps_.use_gpu_rasterization,
        );
    }
}

impl<B: HostImplTypes + PictureLayerImplTypes> ClientLayerTreeHostImpl<B> {
    // Source: cc/trees/client_layer_tree_host_impl.h:38-43.
    // Source: cc/trees/client_layer_tree_host_impl.cc:77-107.
    pub fn BeginMainFrameAborted(
        &mut self,
        reason: crate::cc::scheduler::commit_earlyout_reason::CommitEarlyOutReason,
        swap_promises: Vec<Box<B::SwapPromise>>,
        _args: &crate::adapter::begin_frame::BeginFrameArgs,
        next_bmf: bool,
        scroll_and_viewport_changes_synced: bool,
        context: &mut ClientHostContext<'_, B>,
    ) {
        use crate::adapter::layer_tree_impl::{DidNotSwapAction, DidNotSwapReason, SwapPromise};
        let main_frame_applied_deltas =
            crate::cc::scheduler::commit_earlyout_reason::MainFrameAppliedDeltas(
                reason,
            );
        self.base.WithActiveTree(&mut context.Base(), |tree, c| {
            tree.ApplySentScrollAndScaleDeltasFromAbortedCommit(
                next_bmf,
                main_frame_applied_deltas,
                c,
            );
        });
        if main_frame_applied_deltas {
            if let Some(pending) = self.base.pending_tree_.as_mut() {
                pending.AppendSwapPromises(swap_promises);
            } else {
                let timestamp = context.services.MonotonicTimeNow();
                for mut promise in swap_promises {
                    let action = promise.DidNotSwap(DidNotSwapReason::COMMIT_NO_UPDATE, timestamp);
                    debug_assert_eq!(action, DidNotSwapAction::BREAK_PROMISE);
                }
            }
        }
        if scroll_and_viewport_changes_synced {
            if let Some(manager) = self.base.browser_controls_offset_manager_.as_mut() {
                manager.NotifyConstraintSyncedToMainThread();
            }
        }
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:49-52.
    // Source: cc/trees/client_layer_tree_host_impl.cc:279-302.
    pub fn ReadyToCommit(
        &mut self,
        scroll_and_viewport_changes_synced: bool,
        metrics: Option<&crate::adapter::scheduler_host::BeginMainFrameMetrics>,
        commit_timeout: bool,
        context: &mut ClientHostContext<'_, B>,
    ) {
        if ((metrics.is_some_and(|metrics| metrics.should_measure_smoothness)) || commit_timeout)
            && !context
                .services
                .ClientFirstContentfulPaintReceived(&self.base.frame_sorter_)
        {
            context
                .services
                .ClientOnFirstContentfulPaintReceived(&mut self.base.frame_sorter_);
        }
        if scroll_and_viewport_changes_synced {
            if let Some(manager) = self.base.browser_controls_offset_manager_.as_mut() {
                manager.NotifyConstraintSyncedToMainThread();
            }
        }
        if !scroll_and_viewport_changes_synced {
            self.base.WithActiveTree(&mut context.Base(), |tree, c| {
                tree.ApplySentScrollAndScaleDeltasFromAbortedCommit(false, false, c);
            });
        }
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:62-62.
    // Source: cc/trees/client_layer_tree_host_impl.cc:160-173.
    fn PullLayerTreeHostPropertiesFrom(
        &mut self,
        commit: &CommitState<B>,
        context: &mut ClientHostContext<'_, B>,
    ) {
        self.base
            .SetExternalPinchGestureActive(commit.is_external_pinch_gesture_active);
        if commit.needs_gpu_rasterization_histogram {
            self.RecordGpuRasterizationHistogram(context);
        }
        self.base
            .SetDebugState(&commit.debug_state, &mut context.Base());
        self.base
            .SetVisualDeviceViewportSize(&commit.visual_device_viewport_size);
        self.base
            .set_viewport_mobile_optimized(commit.is_viewport_mobile_optimized);
        self.base
            .SetPrefersReducedMotion(commit.prefers_reduced_motion);
        self.base
            .SetMayThrottleIfUndrawnFrames(commit.may_throttle_if_undrawn_frames);
    }
}
impl<B> ClientLayerTreeHostImpl<B>
where
    B: HostImplTypes
        + PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes
        + LayerTreeTypes<TileManager = crate::cc::tiles::tile_manager::TileManager<B>>,
{
    // Source: cc/trees/client_layer_tree_host_impl.h:35-35.
    // Source: cc/trees/client_layer_tree_host_impl.cc:63-75.
    pub fn SetActiveURL(
        &mut self,
        url: &str,
        _source_id: i64,
        context: &mut ClientHostContext<'_, B>,
    ) {
        self.base.tile_manager_.set_active_url(url);
        self.base.has_observed_first_scroll_delay_ = false;
        context
            .services
            .FrameSorterReset(&mut self.base.frame_sorter_, true);
    }
}
impl<B> ClientLayerTreeHostImpl<B>
where
    B: HostImplTypes
        + PictureLayerImplTypes
        + LayerTreeTypes<CommitState = CommitState<B>, UnsafeCommitState = ThreadUnsafeCommitState<B>>,
{
    // Source: cc/trees/client_layer_tree_host_impl.h:46-47.
    // Source: cc/trees/client_layer_tree_host_impl.cc:120-158.
    pub fn FinishCommit(
        &mut self,
        state: &mut CommitState<B>,
        unsafe_state: &ThreadUnsafeCommitState<B>,
        main_context:&mut <B::Synchronizer as crate::adapter::layer_tree_impl::LayerTreeSynchronizer<B>>::MainContext<'_, '_>,
        context: &mut ClientHostContext<'_, B>,
    ) {
        context
            .services
            .TraceHostImplCEvent("cc,benchmark", "LayerTreeHostImpl::FinishCommit");
        {
            let release = context.services.ClientScopedBatchResourceRelease(
                self.base
                    .resource_provider_
                    .as_deref_mut()
                    .expect("source resource provider"),
            );
            self.WithSyncTree(context, |tree, c| {
                tree.PullPropertiesFrom(state, unsafe_state, c, main_context)
            });
            drop(release);
        }
        let active = self.base.CommitsToActiveTree();
        self.base
            .HandleRemovedScrollAnimatingElements(active, &mut context.Base());
        self.PullLayerTreeHostPropertiesFrom(state, context);
        for (request, image, speculative) in &state.queued_image_decodes {
            self.base
                .QueueImageDecode(*request, image, *speculative, &mut context.Base());
        }
        for benchmark in &mut state.benchmarks {
            self.base.ScheduleMicroBenchmark(
                benchmark
                    .take()
                    .expect("source nonnull benchmark unique_ptr"),
            );
        }
        self.base.new_local_surface_id_expected_ = false;
        if VerboseLogEnabled(context) {
            let (properties, layers) = self.WithSyncTree(context, |tree, c| {
                (tree.property_trees().ToString(), tree.LayerListAsJson(c))
            });
            context.services.ClientVerboseLog(&format!("{}: After finishing commit on impl, the sync tree:\nproperty_trees:\n{}\ncc::LayerImpls:\n{}",ClientNameForVerboseLog(),properties,layers));
        }
    }
}

use crate::adapter::image_animation::{PaintImageId, PaintImageIdFlatSet};
use crate::adapter::{
    layer_tree_impl::{LayerTreeLayer, LayerTreeLayerList, LayerTreeLayerStorage},
    paint_worklet::{PaintWorkletInput, PropertyValue},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
impl<B> ClientLayerTreeHostImpl<B>
where
    B: HostImplTypes<EventMetricsList = Vec<<B as LayerTreeTypes>::EventMetrics>>
        + PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes<PaintImageId = PaintImageId>
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
            TileManager = crate::cc::tiles::tile_manager::TileManager<B>,
        >,
{
    fn SyncTreeRead(&self) -> &LayerTreeImpl<B> {
        if self.base.CommitsToActiveTree() {
            self.base
                .active_tree_
                .as_deref()
                .expect("source active sync tree")
        } else {
            self.base
                .pending_tree_
                .as_deref()
                .expect("source pending sync tree")
        }
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:58-58.
    // Source: cc/trees/client_layer_tree_host_impl.cc:536-547.
    pub fn AnimatePendingTreeAfterCommit(&mut self, context: &mut ClientHostContext<'_, B>) {
        debug_assert!(self.base.pending_tree_.is_some());
        let monotonic_time = self.base.CurrentBeginFrameArgs().frame_time;
        self.base
            .AnimateLayers(monotonic_time, false, &mut context.Base());
        if let Some(input) = self.base.input_delegate_.as_mut() {
            use crate::cc::input::compositor_input_interfaces::InputDelegateForCompositor;
            input.TickAnimations(monotonic_time);
        }
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:64-65.
    // Source: cc/trees/client_layer_tree_host_impl.cc:549-592.
    fn GatherDirtyPaintWorklets(
        &self,
        dirty_ids: &mut PaintImageIdFlatSet,
    ) -> PaintWorkletJobMap<B> {
        let mut dirty = BTreeMap::new();
        for layer in self.SyncTreeRead().picture_layers_with_paint_worklets() {
            for (input, (image_id, record)) in layer.GetPaintWorkletRecordMap().iter() {
                if record.is_some() {
                    continue;
                }
                dirty_ids.insert(*image_id);
                let worklet_id = input.WorkletId();
                let vector = dirty.entry(worklet_id).or_insert_with(Vec::new);
                let mut animated_values = BTreeMap::new();
                for key in input.GetPropertyKeys() {
                    debug_assert!(!animated_values.contains_key(key));
                    let value = self
                        .base
                        .paint_worklet_tracker_
                        .GetPropertyAnimationValue(key);
                    if value.has_value() {
                        animated_values.insert(key.clone(), value);
                    }
                }
                vector.push(PaintWorkletJob {
                    layer_id: layer.identity().layer,
                    input: Arc::clone(input),
                    animated_property_values: animated_values,
                    output: None,
                });
            }
        }
        dirty
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:67-67.
    // Source: cc/trees/client_layer_tree_host_impl.cc:509-534.
    fn NotifyPendingTreeFullyPainted(&mut self, context: &mut ClientHostContext<'_, B>) {
        debug_assert!(
            self.base.pending_tree_fully_painted_
                && !self.base.settings_.trees_in_viz_in_viz_process
        );
        debug_assert!(self
            .base
            .paint_worklet_painter_
            .as_deref()
            .is_none_or(|painter| !context.services.ClientHasOngoingWorkletDispatch(painter)));
        let did_prepare_tiles = self.base.PrepareTiles(&mut context.Base());
        if !did_prepare_tiles {
            self.base.NotifyReadyToActivate(&mut context.Base());
            if self.base.CommitsToActiveTree()
                || self.base.settings_.wait_for_all_pipeline_stages_before_draw
            {
                self.base.NotifyReadyToDraw(&mut context.Base());
            }
        }
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:66-66.
    // Source: cc/trees/client_layer_tree_host_impl.cc:467-507.
    pub fn OnPaintWorkletResultsReady(
        &mut self,
        results: PaintWorkletJobMap<B>,
        context: &mut ClientHostContext<'_, B>,
    ) {
        #[cfg(debug_assertions)]
        {
            let mut dirty_ids = PaintImageIdFlatSet::new();
            debug_assert_eq!(
                results.len(),
                self.GatherDirtyPaintWorklets(&mut dirty_ids).len()
            );
        }
        for jobs in results.into_values() {
            for job in jobs {
                self.base.WithPendingTree(&mut context.Base(), |tree, c| {
                    B::WithPictureLayersContext(tree, c.host, |layers, _picture_context| {
                        let layer = layers
                            .LayerByIdMut(job.layer_id)
                            .expect("source pending layer cannot change during worklet dispatch");
                        layer
                            .AsPictureLayerMut()
                            .expect("source static_cast PictureLayerImpl")
                            .SetPaintWorkletRecord(
                                job.input,
                                job.output.expect("source completed worklet output"),
                            );
                    });
                });
            }
        }
        self.base.tile_priorities_dirty_ = true;
        self.base.pending_tree_fully_painted_ = true;
        context.services.ClientNotifyPaintWorkletStateChange(
            self.base.delegate_,
            crate::cc::scheduler::scheduler_state_machine::PaintWorkletState::IDLE,
        );
        if self.base.pending_tree_.is_some() {
            self.NotifyPendingTreeFullyPainted(context);
        }
    }
}

impl<B> ClientLayerTreeHostImpl<B>
where
    B: HostImplTypes<EventMetricsList = Vec<<B as LayerTreeTypes>::EventMetrics>>
        + PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes<PaintImageId = PaintImageId>
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
            TileManager = crate::cc::tiles::tile_manager::TileManager<B>,
        >,
{
    // Source: cc/trees/client_layer_tree_host_impl.h:53-53.
    // Source: cc/trees/client_layer_tree_host_impl.cc:304-330.
    pub fn InvalidateContentOnImplSide(&mut self, context: &mut ClientHostContext<'_, B>) {
        debug_assert!(
            self.base.pending_tree_.is_none() && !self.base.settings_.trees_in_viz_in_viz_process
        );
        debug_assert!(
            self.base.impl_thread_phase_
                == super::layer_tree_host_impl::ImplThreadPhase::INSIDE_IMPL_FRAME
                || self.base.settings_.using_synchronous_renderer_compositor
        );
        if !self.base.CommitsToActiveTree() {
            self.CreatePendingTree(context);
            if context
                .services
                .ClientScrollingThreadIsRaster(&self.base.frame_trackers_)
            {
                let events = context
                    .services
                    .TakeSavedEventsMetrics(&mut self.base.events_metrics_manager_);
                self.base
                    .pending_tree_
                    .as_deref_mut()
                    .unwrap()
                    .AppendEventMetricsFromRasterThread(events);
            }
            self.AnimatePendingTreeAfterCommit(context);
        }
        self.base.WithInputHandler(&mut context.Base(),|input,host|{
            use crate::cc::input::compositor_input_interfaces::InputDelegateForCompositor;
            if let Some(input)=input{input.DidImplSideInvalidate(host);}
        });
        self.UpdateSyncTreeAfterCommitOrImplSideInvalidation(context);
    }
    // Source: cc/trees/client_layer_tree_host_impl.h:63-63.
    // Source: cc/trees/client_layer_tree_host_impl.cc:362-465.
    fn UpdateSyncTreeAfterCommitOrImplSideInvalidation(
        &mut self,
        context: &mut ClientHostContext<'_, B>,
    ) {
        debug_assert!(!self.base.settings_.trees_in_viz_in_viz_process);
        self.base
            .sync_tree()
            .unwrap()
            .set_needs_update_draw_properties();
        self.WithSyncTree(context, |tree, c| {
            tree.UpdateDrawProperties(true, false, None, c);
        });
        // This builder lends the source set separately from only the Picture
        // invalidation capabilities. It clears it after the native call returns.
        self.base
            .InvalidateSyncTreeRasterInducingScrolls(&mut context.Base());
        let animated_properties = self
            .base
            .paint_worklet_tracker_
            .TakeAndResetAnimatedProperties();
        let mut worklets_invalidated = false;
        let _ = self
            .base
            .sync_tree()
            .unwrap()
            .LayersForMutable()
            .ForEachPictureLayer(&mut |layer| {
                let count = layer.GetPaintWorkletRecordMap().len();
                for index in 0..count {
                    // Only a scoped_refptr to this actual immutable input is lent
                    // across mutation of record values in the same native map.
                    let input = Arc::clone(layer.GetPaintWorkletRecordMap().input_at(index));
                    for key in input.GetPropertyKeys() {
                        if let Some((previous, next)) = animated_properties.get(key) {
                            worklets_invalidated = true;
                            layer.InvalidatePaintWorklets(key, previous, next);
                        }
                    }
                }
                std::ops::ControlFlow::Continue(())
            });
        if worklets_invalidated {
            context
                .services
                .SetNeedsImplSideInvalidation(self.base.delegate_, true);
            if self.SyncTreeRead().property_change_forces_commit_criteria()
                == crate::adapter::layer_tree_impl::PropertyChangeForcesCommitCriteria::kAny
            {
                self.base.SetNeedsCommit(&mut context.Base());
            }
        }
        let mut dirty_ids = PaintImageIdFlatSet::new();
        let dirty_worklets = self.GatherDirtyPaintWorklets(&mut dirty_ids);
        let mut images_to_invalidate = context
            .services
            .ClientTakeImagesToInvalidate(self.base.tile_manager_.checker_image_tracker_id());
        assert!(!self.base.settings_.trees_in_viz_in_viz_process);
        assert!(self.base.image_animation_controller_.is_some());
        let args = self.base.CurrentBeginFrameArgs().clone();
        let drivers = self.base.GatherAnimatedImageDriverState();
        let advanced =
            self.base
                .WithImageAnimationController(&mut context.Base(), |controller, client| {
                    let controller = controller.expect("source image animation controller");
                    let animated = controller.AnimateForSyncTree(&args, &drivers, client);
                    images_to_invalidate.extend(animated.iter().copied());
                    controller.HasAdvancedAnimationClients()
                });
        if advanced {
            self.base.SetNeedsCommit(&mut context.Base());
        }
        images_to_invalidate.extend(dirty_ids);
        self.WithSyncTree(context, |tree, c| {
            tree.InvalidateRegionForImages(&images_to_invalidate, c)
        });
        if self.base.CommitsToActiveTree() {
            self.base.ActivateStateForImages(&mut context.Base());
        }
        self.base
            .sync_tree()
            .unwrap()
            .SetCreatedBeginFrameArgs(args);
        if self.base.paint_worklet_painter_.is_none() {
            debug_assert!(self
                .SyncTreeRead()
                .picture_layers_with_paint_worklets()
                .next()
                .is_none());
            self.base.pending_tree_fully_painted_ = true;
            self.NotifyPendingTreeFullyPainted(context);
            return;
        }
        if dirty_worklets.is_empty() {
            self.base.pending_tree_fully_painted_ = true;
            self.NotifyPendingTreeFullyPainted(context);
            return;
        }
        context.services.ClientNotifyPaintWorkletStateChange(self.base.delegate_,
            crate::cc::scheduler::scheduler_state_machine::PaintWorkletState::PROCESSING);
        let owner = self.base.AsWeakPtr();
        let completed = context.services.ClientDispatchWorklets(
            self.base.paint_worklet_painter_.as_deref_mut().unwrap(),
            dirty_worklets,
            owner,
        );
        if let Some(results) = completed {
            self.OnPaintWorkletResultsReady(results, context);
        }
    }
}
impl<B> ClientLayerTreeHostImpl<B>
where
    B: HostImplTypes<EventMetricsList = Vec<<B as LayerTreeTypes>::EventMetrics>>
        + PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes<PaintImageId = PaintImageId>
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
            TileManager = crate::cc::tiles::tile_manager::TileManager<B>,
        >,
    for<'a> B::MicroBenchmarkImpl:
        crate::adapter::micro_benchmark::MicroBenchmarkImpl<BenchmarkContext<'a, B>>,
{
    // Source: cc/trees/client_layer_tree_host_impl.h:48-48.
    // Source: cc/trees/client_layer_tree_host_impl.cc:184-277.
    pub fn CommitComplete(&mut self, context: &mut ClientHostContext<'_, B>) {
        debug_assert!(!self.base.settings_.trees_in_viz_in_viz_process);
        context
            .services
            .ClientTraceCommitComplete(self.SyncTreeRead().trace_id());
        self.base.WithInputHandler(&mut context.Base(),|input,host|{
            use crate::cc::input::compositor_input_interfaces::InputDelegateForCompositor;
            if let Some(input)=input{input.DidCommit(host);}
        });
        if self.base.CommitsToActiveTree() {
            self.base.WithActiveTree(&mut context.Base(), |tree, c| {
                tree.HandleScrollbarShowRequests(c)
            });
            self.base.ActivateAnimations(&mut context.Base());
        }
        let mut used_properties = BTreeSet::new();
        for layer in self.SyncTreeRead().picture_layers_with_paint_worklets() {
            for (input, _) in layer.GetPaintWorkletRecordMap().iter() {
                used_properties.extend(input.GetPropertyKeys().iter().cloned());
            }
        }
        self.base
            .paint_worklet_tracker_
            .ClearUnusedInputProperties(used_properties);
        if self.base.CommitsToActiveTree() {
            self.base.Animate(&mut context.Base());
        } else {
            self.AnimatePendingTreeAfterCommit(context);
        }
        self.UpdateSyncTreeAfterCommitOrImplSideInvalidation(context);
        if !self.base.tile_priorities_dirty_ {
            let frame = self.SyncTreeRead().source_frame_number();
            B::SetResidentTileSyncFrame(&mut self.base.tile_manager_, frame, &mut context.Base());
        }
        {
            let host = &mut self.base;
            let mut benchmark_context = BenchmarkContext {
                owner: host.owner_id_,
                active: host.active_tree_.as_deref_mut().unwrap(),
                pending: host.pending_tree_.as_deref_mut(),
                recycle: host.recycle_tree_.as_deref_mut(),
                settings: &host.settings_,
                raster_caps: &host.raster_caps_,
                task_provider: host.task_runner_provider_,
                services: context.services,
            };
            host.micro_benchmark_controller_
                .DidCompleteCommit(&mut benchmark_context);
        }
        if context
            .services
            .ClientCurrentFrameHadRAF(&self.base.mutator_host_)
        {
            context
                .services
                .ClientStartSequence(&mut self.base.frame_trackers_, ClientSequence::RAF);
        }
        if context
            .services
            .MutatorHasCanvasInvalidation(&self.base.mutator_host_)
        {
            context.services.ClientStartSequence(
                &mut self.base.frame_trackers_,
                ClientSequence::CanvasAnimation,
            );
        }
        if context
            .services
            .ClientCurrentFrameHadRAF(&self.base.mutator_host_)
            || context
                .services
                .MutatorHasJSAnimation(&self.base.mutator_host_)
        {
            context
                .services
                .ClientStartSequence(&mut self.base.frame_trackers_, ClientSequence::JSAnimation);
        }
        if context
            .services
            .MutatorMainThreadAnimationsCount(&self.base.mutator_host_)
            > 0
            || context
                .services
                .MutatorHasSmilAnimation(&self.base.mutator_host_)
        {
            context.services.ClientStartSequence(
                &mut self.base.frame_trackers_,
                ClientSequence::MainThreadAnimation,
            );
            if context
                .services
                .ClientMutatorHasViewTransition(&self.base.mutator_host_)
            {
                context.services.ClientStartSequence(
                    &mut self.base.frame_trackers_,
                    ClientSequence::SETMainThreadAnimation,
                );
            }
        }
        for info in context
            .services
            .ClientTakePendingTrackerInfos(&mut self.base.mutator_host_)
        {
            if info.start {
                context
                    .services
                    .ClientStartCustomSequence(&mut self.base.frame_trackers_, info.id);
            } else {
                context
                    .services
                    .ClientStopCustomSequence(&mut self.base.frame_trackers_, info.id);
            }
        }
    }
}

// Source: cc/trees/client_layer_tree_host_impl.cc:30-32.
fn VerboseLogEnabled<B: HostImplTypes + PictureLayerImplTypes>(
    context: &ClientHostContext<'_, B>,
) -> bool {
    context.services.VlogIsOn(3)
}
// Source: cc/trees/client_layer_tree_host_impl.cc:34-36.
fn ClientNameForVerboseLog() -> &'static str {
    "ClientLTHI"
}
// Source: cc/trees/client_layer_tree_host_impl.cc:38-41.
// The source logging macro is the lazy VerboseLogEnabled conditional and
// ClientNameForVerboseLog prefix at FinishCommit's sole native callsite.
// Source: cc/trees/client_layer_tree_host_impl.cc:61-61.
// A defaulted subclass destructor drops this wrapper's sole base field.
// Source: cc/trees/client_layer_tree_host_impl.cc:593-594.
// Namespace closure is the Rust module boundary.

// Source: cc/trees/client_layer_tree_host_impl.h:19-19.
// Native public access is the Rust pub methods and the inherited base borrow.
// Source: cc/trees/client_layer_tree_host_impl.h:61-61.
// Native private helpers remain private Rust methods.

impl<B: HostImplTypes + PictureLayerImplTypes> ClientLayerTreeHostImpl<B> {
    // Source: cc/trees/client_layer_tree_host_impl.cc:61-61.
    // Rust inherits the base destructor through an explicit current-call context.
    pub(crate) fn Destroy(self, context: &mut HostImplContext<'_, B>) {
        self.base.Destroy(context);
    }
}
