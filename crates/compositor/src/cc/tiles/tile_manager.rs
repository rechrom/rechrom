// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tile_manager.h:1-59
// Source: cc/tiles/tile_manager.cc:1-55
use crate::{
    adapter::{prioritized_tile::PrioritizedTileResidentExt, resource_pool::ResourcePoolTypes},
    cc::{resources::resource_pool::InUsePoolResource, tiles::tile::Tile},
};
use compositor_raster::completion_stats::{
    RasterCompletionToken as RasterCompletionTokenValue, RasterCompletionTokenError,
    RasterTaskCompletionStats, ValidateRasterCompletionToken,
};
use compositor_raster::completion_task::{
    DidFinishRunningAllTilesTask, TaskSetFinishedTask as TaskSetFinishedTaskImpl,
};
use compositor_tile::tile_manager_policy::{
    DecideEvictionStep, DecideNoRasterWorkAction, EvictionStep, MemoryUsage, NoRasterWorkAction,
    ShouldRasterOccludedTiles as ShouldRasterOccludedTilesForPolicy, Signals,
    TilePriorityViolatesMemoryPolicy as TilePriorityViolatesMemoryPolicyForPolicy,
};
use compositor_tile::tile_assignment::{
    AssignTileWork, TileAssignmentConfig, TileAssignmentContext,
};
use compositor_tile::prepaint_trim::{PrepaintTrimContext, TrimPrepaintTiles as TrimPrepaintPolicy};

// Source: cc/tiles/tile_manager.h:60-68
// Source: cc/tiles/tile_manager.cc:354-366
pub(crate) fn RasterTaskCompletionStatsAsValue(
    stats: &RasterTaskCompletionStats,
) -> TracedValueJSON {
    let mut state = TracedValueJSON::default();
    state.SetInteger(
        "completed_count",
        stats.completed_count.min(i32::MAX as usize) as i32,
    );
    state.SetInteger(
        "canceled_count",
        stats.canceled_count.min(i32::MAX as usize) as i32,
    );
    state
}

use crate::cc::raster::{
    task::TaskGraph,
    tile_task::{TileTaskImpl, TileTaskRef},
};
pub use compositor_raster::tile_task_graph::TileTaskGraphTrace as TileManagerGraphTrace;
use compositor_raster::tile_task_graph::{
    CompletionTasks, FinalizeCompletionGraph, InsertRasterTaskBatchWithCompletionEdges,
    InsertStandaloneDecodeTaskBatchWithCompletionEdge, RasterTaskGraphCandidate,
    RasterTaskGraphMembership, TILE_TASK_PRIORITY_BASE as kTileTaskPriorityBase,
};

use crate::adapter::{
    task_runner::{SequencedTaskRunner, TaskLocation},
    time::TimeDelta,
};
use crate::cc::raster::{
    task::{Task, TaskImpl},
    tile_task::{SupportsBackgroundThreadPriority, SupportsConcurrentExecution, TileTask},
};
use std::sync::{Arc, Mutex};

// External diagnostics and raster-execution capabilities only. They contain no
// manager/layer/tile queries, owner routes or scheduling/completion decisions.
pub(crate) trait TileManagerWorkerDiagnostics:
    compositor_raster::completion_task::RasterWorkerTaskDiagnostics
{
    fn CalledOnOriginThread(&self) -> bool;
    fn TraceRasterWorker(
        &self,
        source_prepare_tiles_id: u64,
    ) -> Box<dyn crate::adapter::tile_manager::TileManagerTraceScope>;
    fn ScopedRasterTask(
        &self,
        tile: crate::adapter::tile::TileTraceId,
        resolution: TileResolution,
        source_frame_number: i32,
        layer_id: i32,
    ) -> Box<dyn crate::adapter::tile_manager::TileManagerTraceScope>;
}
pub(crate) use compositor_raster::completion_task::{
    RasterCommandFlush as TileManagerRasterFlush, RasterQueryQueue as TileManagerRasterQueryQueue,
};

impl compositor_raster::completion_task::RasterOriginTaskRunner for dyn SequencedTaskRunner {
    fn PostRasterTask(
        &self,
        location: compositor_raster::completion_task::RasterTaskLocation,
        closure: compositor_raster::completion_task::OnceClosure,
    ) {
        self.PostDelayedTask(
            TaskLocation {
                function: location.function,
                file: location.file,
                line: location.line,
            },
            closure,
            TimeDelta::default(),
        );
    }
}

use super::{
    tile::Id, tile_manager_settings::TileManagerSettings,
    tile_priority::GlobalStateThatImpactsTilePriority, tile_task_manager::TileTaskManager,
};
use crate::adapter::{
    resource_pool::{PoolClockId, ResourcePoolId},
    tile::{TileId, TileIdAllocator, TileManagerId, TileTraceContext},
    tile_manager::{
        CheckerImageTrackerId, DecodedImageTrackerId, ImageControllerId, RasterBufferProviderId,
        RasterQueryQueueId, TileManagerClientId, TileManagerTypes,
    },
    time::TimeTicks,
};
use crate::cc::{base::unique_notifier::UniqueNotifier, resources::memory_history};
use std::collections::{HashMap, HashSet};

// Rust boundary of source cc158-170: source OnTaskCompleted's direct owner
// callback is returned as a move-only value. The origin TileTaskManager consumes
// it at that same source point, before DidComplete and dependent notification.
// No worker can invoke a manager. This package-owned token names only its
// producing owner and task generation.
pub(crate) type RasterCompletionToken = RasterCompletionTokenValue<TileManagerId, Id>;
pub struct RasterCompletion<B: ResourcePoolTypes> {
    pub token: RasterCompletionToken,
    pub resource: InUsePoolResource<B>,
    pub was_canceled: bool,
}
// Source: cc/tiles/tile_manager.h:70-98,398-475
// The only native owned compositor state is retained here. Source raw pointers
// are non-owning identities resolved through a per-call live context; no Page,
// Host, Pool or resident layer/tree reference is retained. Image capability IDs
// name externally-owned image services as required by the package boundary.
// Reverse member order retains native owned-field destruction order.
pub struct TileManager<B: TileManagerTypes> {
    ready_to_draw_callback_weak_ptr_factory_: Arc<()>,
    task_set_finished_weak_ptr_factory_: Arc<()>,
    check_pending_tile_queries_callback_: Option<Arc<()>>,
    has_pending_queries_: bool,
    metrics_sampling_rate_: f32,
    tick_clock_for_testing_: Option<PoolClockId>,
    task_runner_for_testing_: Option<Arc<dyn SequencedTaskRunner>>,
    has_pending_tile_trimming_task_: bool,
    has_pending_idle_task_: bool,
    last_active_time_: TimeTicks,
    active_url_: String,
    num_of_tiles_with_checker_images_: i32,
    locked_image_tasks_: Vec<TileTaskRef<RasterCompletion<B>>>,
    scheduled_draw_images_: HashMap<Id, Vec<B::DrawImage>>,
    pending_tile_requirements_dirty_: bool,
    pending_required_for_draw_callback_id_: u64,
    pending_required_for_activation_callback_id_: u64,
    pending_gpu_work_tiles_: HashSet<TileId>,
    next_tile_id_: u64,
    prepare_tiles_count_: u64,
    has_scheduled_tile_tasks_: bool,
    signals_check_notifier_: Option<Arc<UniqueNotifier>>,
    signals_: Signals,
    more_tiles_need_prepare_check_notifier_: Option<Arc<UniqueNotifier>>,
    graph_: TaskGraph<RasterCompletion<B>>,
    raster_task_completion_stats_: RasterTaskCompletionStats,
    checker_image_tracker_: CheckerImageTrackerId,
    decoded_image_tracker_: DecodedImageTrackerId,
    image_controller_: ImageControllerId,
    did_oom_on_last_assign_: bool,
    did_check_for_completed_tasks_since_last_schedule_tasks_: bool,
    memory_stats_from_last_assign_: memory_history::Entry,
    all_tiles_that_need_to_be_rasterized_are_scheduled_: bool,
    tiles_: HashMap<Id, TileId>,
    pending_raster_queries_: Option<RasterQueryQueueId>,
    use_gpu_rasterization_: bool,
    tile_manager_settings_: TileManagerSettings,
    running_on_renderer_process_: bool,
    scheduled_raster_task_limit_: usize,
    global_state_: GlobalStateThatImpactsTilePriority,
    raster_buffer_provider_: Option<RasterBufferProviderId>,
    tile_task_manager_: Option<Box<dyn TileTaskManager<RasterCompletion<B>>>>,
    resource_pool_: Option<ResourcePoolId>,
    task_runner_: Arc<dyn SequencedTaskRunner>,
    client_: TileManagerClientId,
    identity_: TileManagerId,
    owner_lifetime_: Arc<()>,
    destroyed_: bool,
}
// Source: cc/tiles/tile_manager.h:288-304
// Source: cc/tiles/tile_manager.cc:2400-2403
// No Clone: native copy is unavailable and move transfers the actual vectors.
struct PrioritizedWorkToSchedule<B: TileManagerTypes> {
    required_for_draw_tile_state_changed: bool,
    checker_image_decode_queue: crate::adapter::tile_manager::ImageDecodeQueue<B>,
    extra_prepaint_images: Vec<B::DrawImage>,
    tiles_to_process_for_images: Vec<super::prioritized_tile::PrioritizedTile>,
    tiles_to_raster: Vec<super::prioritized_tile::PrioritizedTile>,
}
impl<B: TileManagerTypes> Default for PrioritizedWorkToSchedule<B> {
    fn default() -> Self {
        Self {
            tiles_to_raster: Vec::new(),
            tiles_to_process_for_images: Vec::new(),
            extra_prepaint_images: Vec::new(),
            checker_image_decode_queue: Vec::new(),
            required_for_draw_tile_state_changed: false,
        }
    }
}

use crate::adapter::{
    geometry::TracedValue,
    property_types::{PropertyTrace, TilePriorityTrace, TracedValueJSON},
    tile_manager::TileManagerRuntime,
};
// Commands carry source callback identity only. Posting is an external
// enqueue-only endpoint; no closure may retain/dereference a compositor owner.
#[derive(Clone, Copy)]
pub(crate) enum TileManagerCommand {
    CheckIfMoreTilesNeedToBePrepared,
    CheckForCompletedTasksAndIssueSignals,
    ReduceTileMemoryWhenIdle,
    TrimPrepaintTiles,
    CheckRasterFinishedQueries,
    CheckPendingGpuWorkAndIssueSignals,
    DidFinishRunningTileTasksRequiredForActivation,
    DidFinishRunningTileTasksRequiredForDraw,
}
struct PrepaintTrimAdapter<'a, B: TileManagerTypes, C: TileManagerRuntime<B>> {
    manager: &'a mut TileManager<B>,
    context: &'a mut C,
    queue: Option<Box<EvictionTilePriorityQueue<B>>>,
}

impl<B: TileManagerTypes, C: TileManagerRuntime<B>> PrepaintTrimContext
    for PrepaintTrimAdapter<'_, B, C>
{
    fn queue_is_empty(&self) -> bool {
        self.queue.as_ref().expect("prepaint eviction queue").IsEmpty()
    }

    fn current_priority(&self) -> TilePriority {
        *self
            .queue
            .as_ref()
            .expect("prepaint eviction queue")
            .Top(self.context)
            .priority()
    }

    fn current_was_used(&self) -> bool {
        let tile = self
            .queue
            .as_ref()
            .expect("prepaint eviction queue")
            .Top(self.context)
            .tile_id()
            .expect("source non-null tile");
        self.context.TileById(tile).used()
    }

    fn free_current_resource(&mut self) -> bool {
        let tile = self
            .queue
            .as_ref()
            .expect("prepaint eviction queue")
            .Top(self.context)
            .tile_id()
            .expect("source non-null tile");
        self.manager
            .FreeResourcesForTileAndNotifyClientIfTileWasReadyToDraw(tile, self.context)
    }

    fn clear_current_used(&mut self) {
        let tile = self
            .queue
            .as_ref()
            .expect("prepaint eviction queue")
            .Top(self.context)
            .tile_id()
            .expect("source non-null tile");
        self.context.TileByIdMutable(tile).clear_used();
    }

    fn pop_current(&mut self) {
        let context = &*self.context;
        self.queue
            .as_mut()
            .expect("prepaint eviction queue")
            .Pop(context);
    }
}

struct PreparedRasterCandidate<B: TileManagerTypes> {
    prioritized_tile: super::prioritized_tile::PrioritizedTile,
    target_color_params: B::TargetColorParams,
}

struct TileAssignmentAdapter<'a, B: TileManagerTypes, C: TileManagerRuntime<B>> {
    manager: &'a mut TileManager<B>,
    context: &'a mut C,
    raster_priority_queue:
        Option<Box<dyn super::raster_tile_priority_queue::RasterTilePriorityQueue<B>>>,
    eviction_priority_queue: Option<Box<EvictionTilePriorityQueue<B>>>,
    work: PrioritizedWorkToSchedule<B>,
}

impl<B: TileManagerTypes, C: TileManagerRuntime<B>> TileAssignmentAdapter<'_, B, C> {
    fn queue(&self) -> &dyn super::raster_tile_priority_queue::RasterTilePriorityQueue<B> {
        self.raster_priority_queue
            .as_deref()
            .expect("assignment raster queue")
    }

    fn current(&self) -> super::prioritized_tile::PrioritizedTile {
        self.queue().Top().clone()
    }
}

impl<B: TileManagerTypes, C: TileManagerRuntime<B>> TileAssignmentContext
    for TileAssignmentAdapter<'_, B, C>
{
    type PreparedRasterCandidate = PreparedRasterCandidate<B>;

    fn queue_is_empty(&self) -> bool {
        self.queue().IsEmpty()
    }

    fn current_priority(&self) -> TilePriority {
        *self.queue().Top().priority()
    }

    fn current_is_occluded(&self) -> bool {
        self.queue().Top().is_occluded()
    }

    fn pop_current(&mut self) {
        let context = &*self.context;
        self.raster_priority_queue
            .as_deref_mut()
            .expect("assignment raster queue")
            .Pop(context);
    }

    fn analyze_current_solid_color(&mut self) -> bool {
        let prioritized_tile = self.current();
        let tile = prioritized_tile.tile_id().expect("source non-null tile");
        if self.context.TileById(tile).is_solid_color_analysis_performed()
            || !self.context.TileById(tile).use_picture_analysis()
            || !kUseColorEstimator
        {
            return false;
        }
        self.context
            .TileByIdMutable(tile)
            .set_solid_color_analysis_performed(true);
        let mut color = crate::adapter::layer_tree_impl::Color4f::default();
        const kMaxOpsToAnalyze: i32 = 5;
        let raster_source = prioritized_tile
            .raster_source::<super::picture_layer_tiling::PictureLayerTiling<B>>(self.context)
            .as_ref()
            .expect("source raster_source");
        let rect = self.context.TileById(tile).enclosing_layer_rect();
        let is_solid_color = raster_source.PerformSolidColorAnalysis(
            rect,
            &mut color,
            kMaxOpsToAnalyze,
        );
        if !is_solid_color {
            return false;
        }
        self.context
            .TileByIdMutable(tile)
            .draw_info_mut()
            .set_solid_color(&color);
        self.context.NotifyTileStateChangedById(
            self.manager.client_,
            tile,
            true,
            false,
        );
        self.work.required_for_draw_tile_state_changed |=
            self.context.TileById(tile).required_for_draw();
        true
    }

    fn current_is_prepaint_images_only(&self) -> bool {
        let prioritized_tile = self.queue().Top();
        let tile = prioritized_tile.tile_id().expect("source non-null tile");
        self.context.TileById(tile).is_prepaint()
            && prioritized_tile.is_process_for_images_only()
    }

    fn queue_current_for_image_processing(&mut self) {
        self.work.tiles_to_process_for_images.push(self.current());
    }

    fn prepare_current_raster_candidate(&mut self) -> Self::PreparedRasterCandidate {
        let prioritized_tile = self.current();
        let content_color_usage =
            GetContentColorUsageForPrioritizedTile(&prioritized_tile, self.context);
        let target_color_params = self
            .context
            .GetTargetColorParams(self.manager.client_, content_color_usage);
        PreparedRasterCandidate {
            prioritized_tile,
            target_color_params,
        }
    }

    fn current_needs_raster(&self) -> bool {
        let tile = self
            .queue()
            .Top()
            .tile_id()
            .expect("source non-null tile");
        self.context.TileById(tile).draw_info().NeedsRaster()
    }

    fn queue_checker_decode_only(&mut self, candidate: Self::PreparedRasterCandidate) {
        let tile = candidate
            .prioritized_tile
            .tile_id()
            .expect("source non-null tile");
        debug_assert!(self.context.TileById(tile).draw_info().is_checker_imaged());
        debug_assert!(
            candidate
                .prioritized_tile
                .should_decode_checkered_images_for_tile()
        );
        self.manager.AddCheckeredImagesToDecodeQueue(
            &candidate.prioritized_tile,
            &candidate.target_color_params,
            DecodeType::kRaster,
            &mut self.work.checker_image_decode_queue,
            self.context,
        );
    }

    fn current_memory_requirement(&self) -> MemoryUsage {
        let tile = self
            .queue()
            .Top()
            .tile_id()
            .expect("source non-null tile");
        if self.context.TileById(tile).raster_task().is_some() {
            return MemoryUsage::default();
        }
        let size = self.context.TileById(tile).desired_texture_size();
        let format = self.context.GetTileFormat(self.manager.client_);
        MemoryUsage::FromEstimatedBytes(B::EstimatedSizeInBytes(&format, &size))
    }

    fn free_lower_priority_resources(
        &mut self,
        limit: &MemoryUsage,
        protected_priority: &TilePriority,
        usage: &mut MemoryUsage,
    ) {
        self.eviction_priority_queue = self
            .manager
            .FreeTileResourcesWithLowerPriorityUntilUsageIsWithinLimit(
                self.eviction_priority_queue.take(),
                limit,
                protected_priority,
                usage,
                &mut self.work.required_for_draw_tile_state_changed,
                self.context,
            );
    }

    fn free_resources_to_hard_limit(
        &mut self,
        hard_limit: &MemoryUsage,
        usage: &mut MemoryUsage,
    ) {
        self.eviction_priority_queue = self.manager.FreeTileResourcesUntilUsageIsWithinLimit(
            self.eviction_priority_queue.take(),
            hard_limit,
            usage,
            &mut self.work.required_for_draw_tile_state_changed,
            self.context,
        );
    }

    fn schedule_current_raster(
        &mut self,
        candidate: Self::PreparedRasterCandidate,
        schedule_priority: u32,
    ) -> bool {
        let tile = candidate
            .prioritized_tile
            .tile_id()
            .expect("source non-null tile");
        debug_assert!(
            self.context.TileById(tile).draw_info().mode()
                == super::tile_draw_info::Mode::OOM_MODE
                || !self.context.TileById(tile).draw_info().IsReadyToDraw()
        );
        if self.context.TileById(tile).HasRasterTask() {
            if self
                .context
                .TileById(tile)
                .raster_task_scheduled_with_checker_images()
                && candidate
                    .prioritized_tile
                    .should_decode_checkered_images_for_tile()
            {
                self.manager.AddCheckeredImagesToDecodeQueue(
                    &candidate.prioritized_tile,
                    &candidate.target_color_params,
                    DecodeType::kRaster,
                    &mut self.work.checker_image_decode_queue,
                    self.context,
                );
            }
        } else {
            let Some(raster_task) = self.manager.CreateRasterTask(
                &candidate.prioritized_tile,
                &candidate.target_color_params,
                &mut self.work,
                self.context,
            ) else {
                return false;
            };
            self.context
                .TileByIdMutable(tile)
                .set_raster_task(Some(raster_task));
            self.context.TileByIdMutable(tile).mark_used();
        }
        self.context
            .TileByIdMutable(tile)
            .set_scheduled_priority(schedule_priority);
        self.work
            .tiles_to_raster
            .push(candidate.prioritized_tile);
        true
    }

    fn current_should_decode_checkered_images(&self) -> bool {
        self.queue()
            .Top()
            .should_decode_checkered_images_for_tile()
    }

    fn queue_current_oom_checker_decode(&mut self) {
        let prioritized_tile = self.current();
        let tile = prioritized_tile.tile_id().expect("source non-null tile");
        let content_color_usage =
            GetContentColorUsageForPrioritizedTile(&prioritized_tile, self.context);
        let target_color_params = self
            .context
            .GetTargetColorParams(self.manager.client_, content_color_usage);
        if !self.context.TileById(tile).draw_info().is_checker_imaged()
            && !self
                .context
                .TileById(tile)
                .raster_task_scheduled_with_checker_images()
        {
            return;
        }
        self.manager.AddCheckeredImagesToDecodeQueue(
            &prioritized_tile,
            &target_color_params,
            DecodeType::kRaster,
            &mut self.work.checker_image_decode_queue,
            self.context,
        );
    }

    fn trace_memory_policy_violation(&mut self) {
        self.context.TraceInstant(
            "TileManager::AssignGpuMemory tile violates memory policy",
            || None,
        );
    }

    fn report_hard_memory_limit_exceeded(&mut self) {
        self.context
            .LogError("WARNING: tile memory limits exceeded, some content may not draw");
    }
}

impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:93-103
    // Source: cc/tiles/tile_manager.cc:368-413
    pub(crate) fn new(
        identity: TileManagerId,
        client: TileManagerClientId,
        origin_task_runner: Arc<dyn SequencedTaskRunner>,
        image_worker_task_runner: Arc<dyn SequencedTaskRunner>,
        scheduled_raster_task_limit: usize,
        running_on_renderer_process: bool,
        tile_manager_settings: TileManagerSettings,
        context: &mut impl crate::adapter::tile_manager::TileManagerInitialization<B>,
    ) -> Self {
        // Tokens carry lifetime only, not a callable owner, and cannot provide
        // access to manager state. Source weak-factory invalidation stays local.
        let owner_lifetime = Arc::new(());
        let task_set_finished_lifetime = Arc::new(());
        let ready_to_draw_lifetime = Arc::new(());
        let image_controller = context.CreateImageController(
            origin_task_runner.clone(),
            image_worker_task_runner,
            identity,
        );
        let decoded_image_tracker =
            context.CreateDecodedImageTracker(image_controller, origin_task_runner.clone());
        let checker_image_tracker = context.CreateCheckerImageTracker(
            image_controller,
            identity,
            tile_manager_settings.enable_checker_imaging,
            tile_manager_settings.min_image_bytes_to_checker,
        );
        let more_tiles_notifier = UniqueNotifier::new(
            origin_task_runner.clone(),
            context.RepeatingCommand(
                identity,
                Arc::downgrade(&owner_lifetime),
                TileManagerCommand::CheckIfMoreTilesNeedToBePrepared,
            ),
        );
        let signals_notifier = UniqueNotifier::new(
            origin_task_runner.clone(),
            context.RepeatingCommand(
                identity,
                Arc::downgrade(&owner_lifetime),
                TileManagerCommand::CheckForCompletedTasksAndIssueSignals,
            ),
        );
        let result = Self {
            client_: client,
            task_runner_: origin_task_runner,
            resource_pool_: None,
            tile_task_manager_: None,
            raster_buffer_provider_: None,
            global_state_: GlobalStateThatImpactsTilePriority::default(),
            scheduled_raster_task_limit_: scheduled_raster_task_limit,
            running_on_renderer_process_: running_on_renderer_process,
            tile_manager_settings_: tile_manager_settings,
            use_gpu_rasterization_: false,
            pending_raster_queries_: None,
            tiles_: HashMap::new(),
            all_tiles_that_need_to_be_rasterized_are_scheduled_: true,
            memory_stats_from_last_assign_: memory_history::Entry::default(),
            did_check_for_completed_tasks_since_last_schedule_tasks_: true,
            did_oom_on_last_assign_: false,
            image_controller_: image_controller,
            decoded_image_tracker_: decoded_image_tracker,
            checker_image_tracker_: checker_image_tracker,
            raster_task_completion_stats_: RasterTaskCompletionStats::default(),
            graph_: TaskGraph::new(),
            more_tiles_need_prepare_check_notifier_: Some(more_tiles_notifier),
            signals_: Signals::default(),
            signals_check_notifier_: Some(signals_notifier),
            has_scheduled_tile_tasks_: false,
            prepare_tiles_count_: 0,
            next_tile_id_: 0,
            pending_gpu_work_tiles_: HashSet::new(),
            pending_required_for_activation_callback_id_: 0,
            pending_required_for_draw_callback_id_: 0,
            pending_tile_requirements_dirty_: false,
            scheduled_draw_images_: HashMap::new(),
            locked_image_tasks_: Vec::new(),
            num_of_tiles_with_checker_images_: 0,
            active_url_: String::new(),
            last_active_time_: TimeTicks::default(),
            has_pending_idle_task_: false,
            has_pending_tile_trimming_task_: false,
            task_runner_for_testing_: None,
            tick_clock_for_testing_: None,
            metrics_sampling_rate_: 0.01,
            has_pending_queries_: false,
            check_pending_tile_queries_callback_: None,
            task_set_finished_weak_ptr_factory_: task_set_finished_lifetime,
            ready_to_draw_callback_weak_ptr_factory_: ready_to_draw_lifetime,
            identity_: identity,
            owner_lifetime_: owner_lifetime,
            destroyed_: false,
        };
        if context.HasCurrentDefaultTaskRunner() {
            context.RegisterDumpProvider(identity, "TileManager");
        }
        result
    }
    // Source: cc/tiles/tile_manager.h:155-157
    pub(crate) fn memory_stats_from_last_assign(&self) -> &memory_history::Entry {
        &self.memory_stats_from_last_assign_
    }
    // Source: cc/tiles/tile_manager.h:251
    fn GetUniqueTileId(&mut self) -> Id {
        self.next_tile_id_ += 1;
        self.next_tile_id_
    }
    // Source: cc/tiles/tile_manager.h:137
    // Source: cc/tiles/tile_manager.cc:781-783
    pub(crate) fn DidModifyTilePriorities(&mut self) {
        self.pending_tile_requirements_dirty_ = true;
    }
    // Source: cc/tiles/tile_manager.h:199
    // Source: cc/tiles/tile_manager.cc:1702-1704
    pub(crate) fn ResetSignalsForTesting(&mut self) {
        self.signals_ = Signals::default();
    }
    // Source: cc/tiles/tile_manager.h:154
    // Source: cc/tiles/tile_manager.cc:793-800
    pub(crate) fn BasicStateAsValueInto(
        &self,
        state: &mut (impl PropertyTrace + TilePriorityTrace),
    ) {
        state.SetInteger(
            "tile_count",
            self.tiles_.len().min(i32::MAX as usize) as i32,
        );
        state.SetBoolean("did_oom_on_last_assign", self.did_oom_on_last_assign_);
        state.BeginDictionary("global_state");
        self.global_state_.AsValueInto(state);
        state.EndDictionary();
    }
    // Source: cc/tiles/tile_manager.h:163-166
    pub(crate) fn SetGlobalStateForTesting(&mut self, state: &GlobalStateThatImpactsTilePriority) {
        self.global_state_ = state.clone();
    }
    // Source: cc/tiles/tile_manager.h:171-178
    pub(crate) fn SetRasterBufferProviderForTesting(
        &mut self,
        provider: Option<RasterBufferProviderId>,
    ) {
        self.raster_buffer_provider_ = provider;
    }
    pub(crate) fn raster_buffer_provider_for_testing(&self) -> Option<RasterBufferProviderId> {
        self.raster_buffer_provider_
    }
    // Source: cc/tiles/tile_manager.h:180-183
    pub(crate) fn SetPendingRasterQueriesForTesting(
        &mut self,
        queries: Option<RasterQueryQueueId>,
    ) {
        self.pending_raster_queries_ = queries;
    }
    // Source: cc/tiles/tile_manager.h:187-189
    pub(crate) fn SetScheduledRasterTaskLimitForTesting(&mut self, limit: usize) {
        self.scheduled_raster_task_limit_ = limit;
    }
    // Source: cc/tiles/tile_manager.h:195-197
    pub(crate) fn SetMoreTilesNeedToBeRasterizedForTesting(&mut self) {
        self.all_tiles_that_need_to_be_rasterized_are_scheduled_ = false;
    }
    // Source: cc/tiles/tile_manager.h:201-203
    pub(crate) fn HasScheduledTileTasksForTesting(&self) -> bool {
        self.has_scheduled_tile_tasks_
    }
    // Source: cc/tiles/tile_manager.h:207
    pub(crate) fn DisbleMetricsSubsamplingForTesting(&mut self) {
        self.metrics_sampling_rate_ = 1.0;
    }
    // Source: cc/tiles/tile_manager.h:222-224
    pub(crate) fn num_of_tiles_with_checker_images(&self) -> i32 {
        self.num_of_tiles_with_checker_images_
    }
    // Source: cc/tiles/tile_manager.h:237
    pub(crate) fn set_active_url(&mut self, url: &str) {
        self.active_url_ = url.to_owned();
    }
    // Source: cc/tiles/tile_manager.h:383
    // Source: cc/tiles/tile_manager.cc:2328-2331
    fn NowWithOverride(&self, context: &impl TileManagerRuntime<B>) -> TimeTicks {
        if let Some(clock) = self.tick_clock_for_testing_ {
            context.NowWithClock(clock)
        } else {
            context.Now()
        }
    }
    // Source: cc/tiles/tile_manager.h:384
    // Source: cc/tiles/tile_manager.cc:2333-2336
    fn TaskRunnerWithOverride(&self) -> &Arc<dyn SequencedTaskRunner> {
        self.task_runner_for_testing_
            .as_ref()
            .unwrap_or(&self.task_runner_)
    }
    // Source: cc/tiles/tile_manager.h:239-241
    // Source: cc/tiles/tile_manager.cc:2266-2271
    pub(crate) fn SetOverridesForTesting(
        &mut self,
        runner: Option<Arc<dyn SequencedTaskRunner>>,
        clock: Option<PoolClockId>,
    ) {
        self.task_runner_for_testing_ = runner;
        self.tick_clock_for_testing_ = clock;
    }
    // Source: cc/tiles/tile_manager.h:168-169
    // Source: cc/tiles/tile_manager.cc:834-837
    pub(crate) fn SetTileTaskManagerForTesting(
        &mut self,
        manager: Option<Box<dyn TileTaskManager<RasterCompletion<B>>>>,
    ) {
        self.tile_task_manager_ = manager;
    }
}

// Source: cc/tiles/tile_manager.h:251
// Actual manager identity/allocator plus independent call-scoped trace service.
// Construction cannot release a tile, so it requests only TileIdAllocator.
struct TileAllocationContext<'m, 's, B: TileManagerTypes, C: TileTraceContext> {
    manager: &'m mut TileManager<B>,
    trace: &'s C,
}
impl<B: TileManagerTypes, C: TileTraceContext> TileTraceContext
    for TileAllocationContext<'_, '_, B, C>
{
    fn MakeDictIntoImplicitSnapshotWithCategory<V: PropertyTrace>(
        &self,
        category: &str,
        value: &mut V,
        name: &str,
        id: crate::adapter::tile::TileTraceId,
    ) {
        self.trace
            .MakeDictIntoImplicitSnapshotWithCategory(category, value, name, id);
    }
    fn TraceTileDeleted(
        &self,
        category: &str,
        event: &str,
        id: crate::adapter::tile::TileTraceId,
        flow_scope: &str,
    ) {
        self.trace.TraceTileDeleted(category, event, id, flow_scope);
    }
}
impl<B: TileManagerTypes, C: TileTraceContext> TileIdAllocator
    for TileAllocationContext<'_, '_, B, C>
{
    fn identity(&self) -> TileManagerId {
        self.manager.identity_
    }
    fn GetUniqueTileId(&mut self) -> Id {
        self.manager.GetUniqueTileId()
    }
}
impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:139-142
    // Source: cc/tiles/tile_manager.cc:1785-1798
    pub(crate) fn CreateTile(
        &mut self,
        info: &super::tile::CreateInfo<B::RasterTransform>,
        layer_id: i32,
        source_frame_number: i32,
        flags: i32,
        trace: &impl TileTraceContext,
    ) -> Box<Tile<B>> {
        debug_assert!(self.tile_task_manager_.is_some());
        let tile = Box::new(Tile::new(
            &mut TileAllocationContext {
                manager: self,
                trace,
            },
            info,
            layer_id,
            source_frame_number,
            flags,
        ));
        debug_assert!(!self.tiles_.contains_key(&tile.id()));
        self.tiles_.insert(tile.id(), tile.identity());
        tile
    }
    // Source: cc/tiles/tile_manager.h:185
    // Source: cc/tiles/tile_manager.cc:839-845
    // Source explicitly returns a vector of non-owning Tile*. Identity values
    // name those same resident objects; no Tile or mutable metadata is copied.
    pub(crate) fn AllTilesForTesting(&self) -> Vec<TileId> {
        let mut tiles = Vec::new();
        for tile in self.tiles_.values() {
            tiles.push(*tile);
        }
        tiles
    }
    // Source: cc/tiles/tile_manager.h:152-153
    // Source: cc/tiles/tile_manager.cc:785-791
    pub(crate) fn BasicStateAsValue(&self) -> TracedValueJSON {
        let mut value = TracedValueJSON::default();
        self.BasicStateAsValueInto(&mut value);
        value
    }
    // Source: cc/tiles/tile_manager.h:372-373
    // Source: cc/tiles/tile_manager.cc:2090-2103
    fn ScheduledTasksStateAsValue(&self) -> TracedValueJSON {
        let mut state = TracedValueJSON::default();
        state.BeginDictionary("tasks_pending");
        state.SetBoolean(
            "activate_tile_tasks_completed",
            self.signals_.activate_tile_tasks_completed,
        );
        state.SetBoolean(
            "draw_tile_tasks_completed",
            self.signals_.draw_tile_tasks_completed,
        );
        state.SetBoolean(
            "all_tile_tasks_completed",
            self.signals_.all_tile_tasks_completed,
        );
        state.EndDictionary();
        state
    }
    // Source: cc/tiles/tile_manager.h:338
    // Source: cc/tiles/tile_manager.cc:896-910
    fn TilePriorityViolatesMemoryPolicy(
        &self,
        priority: &super::tile_priority::TilePriority,
    ) -> bool {
        TilePriorityViolatesMemoryPolicyForPolicy(self.global_state_.memory_limit_policy, priority)
    }
    // Source: cc/tiles/tile_manager.h:382
    // Source: cc/tiles/tile_manager.cc:2323-2326
    fn ShouldRasterOccludedTiles(&self) -> bool {
        ShouldRasterOccludedTilesForPolicy(self.global_state_.memory_limit_policy)
    }
    // Source: cc/tiles/tile_manager.h:147
    // Source: cc/tiles/tile_manager.cc:2069-2071
    pub(crate) fn TakeImagesToInvalidateOnSyncTree<
        'c,
        C: crate::adapter::tile_manager::TileManagerImages<B>,
    >(
        &self,
        context: &'c mut C,
    ) -> &'c std::collections::BTreeSet<B::PaintImageId> {
        context
            .CheckerImageTracker(self.checker_image_tracker_)
            .TakeImagesToInvalidateOnSyncTree()
    }
    // Source: cc/tiles/tile_manager.h:148
    // Source: cc/tiles/tile_manager.cc:2073-2075
    pub(crate) fn DidActivateSyncTree(
        &mut self,
        context: &mut impl crate::adapter::tile_manager::TileManagerImages<B>,
    ) {
        context
            .CheckerImageTracker(self.checker_image_tracker_)
            .DidActivateSyncTree();
    }
    // Source: cc/tiles/tile_manager.h:149
    // Source: cc/tiles/tile_manager.cc:2077-2080
    pub(crate) fn ClearCheckerImageTracking(
        &mut self,
        can_clear_decode_policy_tracking: bool,
        context: &mut impl crate::adapter::tile_manager::TileManagerImages<B>,
    ) {
        context
            .CheckerImageTracker(self.checker_image_tracker_)
            .ClearTracker(can_clear_decode_policy_tracking);
    }
    // Source: cc/tiles/tile_manager.h:150
    // Source: cc/tiles/tile_manager.cc:2082-2084
    pub(crate) fn SetCheckerImagingForceDisabled(
        &mut self,
        force_disable: bool,
        context: &mut impl crate::adapter::tile_manager::TileManagerImages<B>,
    ) {
        context
            .CheckerImageTracker(self.checker_image_tracker_)
            .set_force_disabled(force_disable);
    }
    // Source: cc/tiles/tile_manager.h:205
    // Source: cc/tiles/tile_manager.cc:1781-1783
    pub(crate) fn FlushImageControllerTasksForTesting(
        &mut self,
        context: &mut impl crate::adapter::tile_manager::TileManagerImages<B>,
    ) {
        context
            .ImageController(self.image_controller_)
            .FlushDecodeTasksForTesting();
    }
    // Source: cc/tiles/tile_manager.h:233-235
    pub(crate) fn decode_tasks_for_testing(&mut self, id: Id) -> &Vec<B::DrawImage> {
        self.scheduled_draw_images_.entry(id).or_default()
    }
    // Source: cc/tiles/tile_manager.h:226-228
    pub(crate) fn checker_image_tracker<
        'c,
        C: crate::adapter::tile_manager::TileManagerImages<B>,
    >(
        &self,
        context: &'c mut C,
    ) -> &'c mut dyn crate::adapter::tile_manager::CheckerImageTracker<B> {
        context.CheckerImageTracker(self.checker_image_tracker_)
    }
    // Source: cc/tiles/tile_manager.h:229-231
    pub(crate) fn decoded_image_tracker<
        'c,
        C: crate::adapter::tile_manager::TileManagerImages<B>,
    >(
        &self,
        context: &'c mut C,
    ) -> &'c mut dyn crate::adapter::tile_manager::DecodedImageTracker<B> {
        context.DecodedImageTracker(self.decoded_image_tracker_)
    }
}

use crate::cc::{
    resources::resource_pool::ResourcePool, tiles::tile_task_manager::TileTaskCompletionConsumer,
};
// Source: cc/tiles/tile_manager.cc:1706-1779
// Borrow-only view of the actual owner fields needed by native completion. Both
// the public callback and task-manager consumer execute this one source body.
struct TileManagerCompletionParts<'m, 'c, B: TileManagerTypes, C: TileManagerRuntime<B>> {
    identity: TileManagerId,
    prepare_tiles_count: u64,
    tiles: &'m HashMap<Id, TileId>,
    scheduled_draw_images: &'m mut HashMap<Id, Vec<B::DrawImage>>,
    num_checker: &'m mut i32,
    completion_stats: &'m mut RasterTaskCompletionStats,
    pending_gpu_work_tiles: &'m mut HashSet<TileId>,
    global_state: &'m GlobalStateThatImpactsTilePriority,
    image_controller: ImageControllerId,
    pool: Option<ResourcePoolId>,
    provider: Option<RasterBufferProviderId>,
    client: TileManagerClientId,
    context: &'c mut C,
}
impl<B: TileManagerTypes, C: TileManagerRuntime<B>> TileManagerCompletionParts<'_, '_, B, C> {
    fn OnRasterTaskCompleted(
        &mut self,
        tile_id: Id,
        resource: InUsePoolResource<B>,
        was_canceled: bool,
    ) {
        let tile = self.tiles.get(&tile_id).copied();
        let mut raster_task_was_scheduled_with_checker_images = false;
        if let Some(tile) = tile {
            let tile = self.context.TileByIdMutable(tile);
            tile.set_raster_task(None);
            raster_task_was_scheduled_with_checker_images =
                tile.set_raster_task_scheduled_with_checker_images(false);
            if raster_task_was_scheduled_with_checker_images {
                *self.num_checker -= 1;
            }
        }
        let images = self
            .scheduled_draw_images
            .get(&tile_id)
            .expect("source CHECK(images_it != end)");
        self.context
            .ImageController(self.image_controller)
            .UnrefImages(images);
        self.scheduled_draw_images.remove(&tile_id);
        if was_canceled {
            self.completion_stats.canceled_count += 1;
            self.context
                .ResourcePoolMutable(self.pool.expect("source resource_pool_"))
                .ReleaseResource(resource);
            return;
        }
        self.context
            .ResourcePoolMutable(self.pool.expect("source resource_pool_"))
            .OnContentReplaced(&resource, tile_id);
        self.completion_stats.completed_count += 1;
        let Some(tile) = tile else {
            self.context
                .ResourcePoolMutable(self.pool.expect("source resource_pool_"))
                .ReleaseResource(resource);
            return;
        };
        self.context
            .RasterBufferProvider(self.provider.expect("source raster_buffer_provider_"))
            .NotifyWorkSubmitted();
        let exported = self
            .context
            .ResourcePoolMutable(self.pool.expect("source resource_pool_"))
            .PrepareForExport(&resource, B::TileRasterTaskResourceSource());
        let mut is_ready_for_draw = true;
        if self.global_state.tree_priority
            == super::tile_priority::TreePriority::SMOOTHNESS_TAKES_PRIORITY
        {
            is_ready_for_draw = self
                .context
                .RasterBufferProvider(self.provider.unwrap())
                .IsResourceReadyToDraw(&resource);
        }
        if exported {
            self.context
                .TileByIdMutable(tile)
                .draw_info_mut()
                .SetResource(resource, raster_task_was_scheduled_with_checker_images);
        } else {
            self.context
                .ResourcePoolMutable(self.pool.unwrap())
                .ReleaseResource(resource);
            self.context.TileByIdMutable(tile).draw_info_mut().set_oom();
        }
        if raster_task_was_scheduled_with_checker_images {
            *self.num_checker += 1;
        }
        if !is_ready_for_draw {
            self.pending_gpu_work_tiles.insert(tile);
        } else {
            self.context
                .TileByIdMutable(tile)
                .draw_info_mut()
                .set_resource_ready_for_draw();
            self.context
                .NotifyTileStateChangedById(self.client, tile, true, true);
        }
    }
}
impl<B: TileManagerTypes, C: TileManagerRuntime<B>> TileTaskCompletionConsumer<RasterCompletion<B>>
    for TileManagerCompletionParts<'_, '_, B, C>
{
    fn OnRasterCompletion(&mut self, completion: RasterCompletion<B>) {
        match ValidateRasterCompletionToken(
            &completion.token,
            &self.identity,
            self.prepare_tiles_count,
        ) {
            Ok(()) => {}
            Err(RasterCompletionTokenError::WrongManager) => {
                panic!("completion belongs to current owner")
            }
            Err(RasterCompletionTokenError::FuturePrepareGeneration) => {
                panic!("completion generation cannot be from a future prepare")
            }
        }
        if let Some(tile) = self.tiles.get(&completion.token.tile_id) {
            let tile = self.context.TileById(*tile);
            assert_eq!(
                tile.raster_task().map(|task| task.task().trace_task_id()),
                Some(completion.token.trace_task_id),
                "completion belongs to the current tile task"
            );
        }
        self.OnRasterTaskCompleted(
            completion.token.tile_id,
            completion.resource,
            completion.was_canceled,
        );
    }
    // Source: cc/tiles/tile_manager.cc:668-671
    // Invoked synchronously at TileTaskManager's original callback.Run point.
    fn ExternalDependencyCompleted(&mut self, dependent: TileTaskRef<RasterCompletion<B>>) {
        self.context
            .ImageController(self.image_controller)
            .ExternalDependencyCompletedForTask(dependent);
    }
}
// Source: cc/tiles/tile_manager.cc:1190-1202
// The owner method and iteration/split callers share exactly one source body.
fn FreeResourcesForTile<B: TileManagerTypes>(
    tile: &mut Tile<B>,
    pool: &mut ResourcePool<B>,
    num_checker: &mut i32,
    pending_gpu_work_tiles: &mut HashSet<TileId>,
) {
    let identity = tile.identity();
    let draw_info = tile.draw_info_mut();
    if draw_info.is_checker_imaged() {
        *num_checker -= 1;
    }
    debug_assert!(*num_checker >= 0);
    if draw_info.has_resource() {
        pool.ReleaseResource(draw_info.TakeResource());
        pending_gpu_work_tiles.remove(&identity);
    }
}
impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:209-211
    // Source: cc/tiles/tile_manager.cc:1706-1779
    pub(crate) fn OnRasterTaskCompleted(
        &mut self,
        tile: Id,
        resource: InUsePoolResource<B>,
        was_canceled: bool,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        self.CompletionParts(context)
            .OnRasterTaskCompleted(tile, resource, was_canceled);
    }
    fn CompletionParts<'m, 'c, C: TileManagerRuntime<B>>(
        &'m mut self,
        context: &'c mut C,
    ) -> TileManagerCompletionParts<'m, 'c, B, C> {
        TileManagerCompletionParts {
            identity: self.identity_,
            prepare_tiles_count: self.prepare_tiles_count_,
            tiles: &self.tiles_,
            scheduled_draw_images: &mut self.scheduled_draw_images_,
            num_checker: &mut self.num_of_tiles_with_checker_images_,
            completion_stats: &mut self.raster_task_completion_stats_,
            pending_gpu_work_tiles: &mut self.pending_gpu_work_tiles_,
            global_state: &self.global_state_,
            image_controller: self.image_controller_,
            pool: self.resource_pool_,
            provider: self.raster_buffer_provider_,
            client: self.client_,
            context,
        }
    }
    // Source: cc/tiles/tile_manager.h:315-316
    // Source: cc/tiles/tile_manager.cc:1190-1202
    fn FreeResourcesForTile(
        &mut self,
        tile: &mut Tile<B>,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        FreeResourcesForTile(
            tile,
            context.ResourcePoolMutable(self.resource_pool_.expect("source resource_pool_")),
            &mut self.num_of_tiles_with_checker_images_,
            &mut self.pending_gpu_work_tiles_,
        );
    }
    // Source: cc/tiles/tile_manager.h:317-319
    // Source: cc/tiles/tile_manager.cc:1204-1215
    fn FreeResourcesForTileAndNotifyClientIfTileWasReadyToDraw(
        &mut self,
        tile: TileId,
        context: &mut impl TileManagerRuntime<B>,
    ) -> bool {
        let _trace = context
            .TraceScope("TileManager::FreeResourcesForTileAndNotifyClientIfTileWasReadyToDraw");
        let was_ready_to_draw = context.TileById(tile).draw_info().IsReadyToDraw();
        {
            let (tile, pool) =
                context.TileAndPool(tile, self.resource_pool_.expect("source resource_pool_"));
            FreeResourcesForTile(
                tile,
                pool,
                &mut self.num_of_tiles_with_checker_images_,
                &mut self.pending_gpu_work_tiles_,
            );
        }
        context.NotifyTileStateChangedById(self.client_, tile, was_ready_to_draw, false);
        context.TileById(tile).required_for_draw()
    }
    // Source: cc/tiles/tile_manager.h:250
    // Source: cc/tiles/tile_manager.cc:603-613
    fn Release(
        &mut self,
        tile: &mut Tile<B>,
        context: &mut impl crate::adapter::tile_manager::TileReleaseRuntime<B>,
    ) {
        if tile.raster_task_scheduled_with_checker_images() {
            self.num_of_tiles_with_checker_images_ -= 1;
        }
        debug_assert!(self.num_of_tiles_with_checker_images_ >= 0);
        assert!(tile.deleted());
        FreeResourcesForTile(
            tile,
            context.ResourcePoolMutable(self.resource_pool_.expect("source resource_pool_")),
            &mut self.num_of_tiles_with_checker_images_,
            &mut self.pending_gpu_work_tiles_,
        );
        context.NotifyTileStateChanged(self.client_, tile, false, true);
        self.tiles_.remove(&tile.id());
    }
    // Source: cc/tiles/tile_manager.h:348-349
    // Source: cc/tiles/tile_manager.cc:673-683
    fn ExternalDependencyCompletedForRasterTask(
        &mut self,
        dependent: TileTaskRef<RasterCompletion<B>>,
    ) {
        assert!(
            self.tile_task_manager_.is_some()
                || dependent.task().WithState(|state| state.IsCanceled())
        );
        if let Some(manager) = self.tile_task_manager_.as_ref() {
            manager.ExternalDependencyCompletedForTask(dependent);
        }
    }
    // Source: cc/tiles/tile_manager.h:374
    // Source: cc/tiles/tile_manager.cc:2105-2112
    fn UsePartialRaster(
        &self,
        msaa_sample_count: i32,
        context: &mut impl TileManagerRuntime<B>,
    ) -> bool {
        self.tile_manager_settings_.use_partial_raster
            && context
                .RasterBufferProvider(
                    self.raster_buffer_provider_
                        .expect("source raster_buffer_provider_"),
                )
                .CanPartialRasterIntoProvidedResource()
            && msaa_sample_count == 0
    }
    // Source: cc/tiles/tile_manager.h:214
    // Source: cc/tiles/tile_manager.cc:2086-2088
    pub(crate) fn NeedsInvalidationForCheckerImagedTiles(
        &mut self,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        context.RequestImplSideInvalidationForCheckerImagedTiles(self.client_);
    }
}

impl<B: TileManagerTypes> TileManager<B> {
    // Borrow split only: the task-manager field stays resident while its sibling
    // source accounting fields and live services are lent to the completion
    // consumer. No owned object is taken out or replaced for borrowing.
    fn CheckForCompletedTasks(&mut self, context: &mut impl TileManagerRuntime<B>) {
        let services = context.TaskManagerServices();
        let mut parts = TileManagerCompletionParts {
            identity: self.identity_,
            prepare_tiles_count: self.prepare_tiles_count_,
            tiles: &self.tiles_,
            scheduled_draw_images: &mut self.scheduled_draw_images_,
            num_checker: &mut self.num_of_tiles_with_checker_images_,
            completion_stats: &mut self.raster_task_completion_stats_,
            pending_gpu_work_tiles: &mut self.pending_gpu_work_tiles_,
            global_state: &self.global_state_,
            image_controller: self.image_controller_,
            pool: self.resource_pool_,
            provider: self.raster_buffer_provider_,
            client: self.client_,
            context,
        };
        self.tile_task_manager_
            .as_ref()
            .expect("source tile_task_manager_")
            .CheckForCompletedTasks(services.as_ref(), &mut parts);
    }
    // Source: cc/tiles/tile_manager.h:115-118
    // Source: cc/tiles/tile_manager.cc:421-451
    pub(crate) fn FinishTasksAndCleanUp(&mut self, context: &mut impl TileManagerRuntime<B>) {
        if let Some(manager) = self.tile_task_manager_.as_ref() {
            self.global_state_ = GlobalStateThatImpactsTilePriority::default();
            let services = context.TaskManagerServices();
            manager.Shutdown(services.as_ref());
            context
                .RasterBufferProvider(
                    self.raster_buffer_provider_
                        .expect("source raster_buffer_provider_"),
                )
                .Shutdown();
            self.CheckForCompletedTasks(context);
            self.tile_task_manager_ = None;
        }
        self.resource_pool_ = None;
        self.pending_raster_queries_ = None;
        self.more_tiles_need_prepare_check_notifier_
            .as_ref()
            .expect("source live notifier")
            .Cancel();
        self.signals_check_notifier_
            .as_ref()
            .expect("source live notifier")
            .Cancel();
        self.task_set_finished_weak_ptr_factory_ = Arc::new(());
        self.ready_to_draw_callback_weak_ptr_factory_ = Arc::new(());
        self.check_pending_tile_queries_callback_ = None;
        self.raster_buffer_provider_ = None;
        let can_clear_decode_policy_tracking = false;
        context
            .CheckerImageTracker(self.checker_image_tracker_)
            .ClearTracker(can_clear_decode_policy_tracking);
        context
            .ImageController(self.image_controller_)
            .SetImageDecodeCache(None);
        self.locked_image_tasks_.clear();
    }
    // Source: cc/tiles/tile_manager.h:101
    // Source: cc/tiles/tile_manager.cc:415-419
    // Explicit live-context destruction preserves the native destructor body;
    // field order above preserves the subsequent member destruction. External
    // image objects are disposed in the same native reverse-member order.
    pub(crate) fn Destroy(&mut self, context: &mut impl TileManagerRuntime<B>) {
        assert!(!self.destroyed_, "TileManager destructor runs once");
        context.UnregisterDumpProvider(self.identity_);
        self.FinishTasksAndCleanUp(context);
        // Source h425-474 reverse-member destruction. These are true moves of
        // owned slots during teardown, not copied state or borrow workarounds.
        self.ready_to_draw_callback_weak_ptr_factory_ = Arc::new(());
        self.task_set_finished_weak_ptr_factory_ = Arc::new(());
        self.check_pending_tile_queries_callback_ = None;
        drop(self.task_runner_for_testing_.take());
        drop(std::mem::take(&mut self.active_url_));
        drop(std::mem::take(&mut self.locked_image_tasks_));
        drop(std::mem::take(&mut self.scheduled_draw_images_));
        drop(std::mem::take(&mut self.pending_gpu_work_tiles_));
        drop(self.signals_check_notifier_.take());
        drop(self.more_tiles_need_prepare_check_notifier_.take());
        drop(std::mem::take(&mut self.graph_));
        context.DestroyCheckerImageTracker(self.checker_image_tracker_);
        context.DestroyDecodedImageTracker(self.decoded_image_tracker_);
        context.DestroyImageController(self.image_controller_);
        self.owner_lifetime_ = Arc::new(());
        self.destroyed_ = true;
    }
    // Source: cc/tiles/tile_manager.h:123-128
    // Source: cc/tiles/tile_manager.cc:582-601
    pub(crate) fn SetResources(
        &mut self,
        resource_pool: Option<ResourcePoolId>,
        image_decode_cache: Option<crate::adapter::tile_manager::ImageDecodeCacheId>,
        task_graph_runner: Arc<
            dyn crate::cc::raster::task_graph_runner::TaskGraphRunner<RasterCompletion<B>>,
        >,
        raster_buffer_provider: Option<RasterBufferProviderId>,
        use_gpu_rasterization: bool,
        pending_raster_queries: Option<RasterQueryQueueId>,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        debug_assert!(self.tile_task_manager_.is_none());
        // A non-null reference expresses source DCHECK(task_graph_runner).
        self.use_gpu_rasterization_ = use_gpu_rasterization;
        self.pending_raster_queries_ = pending_raster_queries;
        self.resource_pool_ = resource_pool;
        context
            .ImageController(self.image_controller_)
            .SetImageDecodeCache(image_decode_cache);
        // The source bound ExternalDependencyCompletedForNonRasterTask is
        // explicit in the current call-scoped completion consumer, not stored
        // as a closure with a manager backreference.
        self.tile_task_manager_ = Some(super::tile_task_manager::TileTaskManagerImpl::Create(
            task_graph_runner,
        ));
        self.raster_buffer_provider_ = raster_buffer_provider;
    }
    // Source: cc/tiles/tile_manager.h:350-351
    // Source: cc/tiles/tile_manager.cc:668-671
    fn ExternalDependencyCompletedForNonRasterTask(
        &mut self,
        dependent: TileTaskRef<RasterCompletion<B>>,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        context
            .ImageController(self.image_controller_)
            .ExternalDependencyCompletedForTask(dependent);
    }
    // Source: cc/tiles/tile_manager.h:344
    // Source: cc/tiles/tile_manager.cc:615-624
    fn DidFinishRunningTileTasksRequiredForActivation(
        &mut self,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        let _trace =
            context.TraceScope("TileManager::DidFinishRunningTileTasksRequiredForActivation");
        context.TraceInstantOnTrack(
            "ScheduledTasksState",
            GetTracingTrack(self.identity_),
            || Some(self.ScheduledTasksStateAsValue()),
        );
        assert!(self.tile_task_manager_.is_some());
        self.signals_.activate_tile_tasks_completed = true;
        self.signals_check_notifier_
            .as_ref()
            .expect("source live notifier")
            .Schedule();
    }
    // Source: cc/tiles/tile_manager.h:345
    // Source: cc/tiles/tile_manager.cc:626-634
    fn DidFinishRunningTileTasksRequiredForDraw(
        &mut self,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        let _trace = context.TraceScope("TileManager::DidFinishRunningTileTasksRequiredForDraw");
        context.TraceInstantOnTrack(
            "ScheduledTasksState",
            GetTracingTrack(self.identity_),
            || Some(self.ScheduledTasksStateAsValue()),
        );
        assert!(self.tile_task_manager_.is_some());
        self.signals_.draw_tile_tasks_completed = true;
        self.signals_check_notifier_
            .as_ref()
            .expect("source live notifier")
            .Schedule();
    }
    // Source: cc/tiles/tile_manager.h:346-347
    // Source: cc/tiles/tile_manager.cc:636-666
    fn DidFinishRunningAllTileTasks(
        &mut self,
        start_time: TimeTicks,
        has_pending_queries: bool,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        let _trace = context.TraceScope("TileManager::DidFinishRunningAllTileTasks");
        context.TraceEndOnTrack(GetTracingTrack(self.identity_));
        debug_assert!(self.resource_pool_.is_some());
        debug_assert!(self.tile_task_manager_.is_some());
        if !start_time.is_null() {
            context.HistogramDuration(
                "Compositing.TileManager.RasterTasksDuration",
                context.Now() - start_time,
                TimeDelta::Microseconds(10),
                TimeDelta::Milliseconds(200),
                50,
            );
        }
        self.has_scheduled_tile_tasks_ = false;
        self.has_pending_queries_ = has_pending_queries;
        if self.all_tiles_that_need_to_be_rasterized_are_scheduled_
            && !context
                .ResourcePool(self.resource_pool_.unwrap())
                .ResourceUsageTooHigh()
        {
            assert!(self.tile_task_manager_.is_some());
            self.signals_.all_tile_tasks_completed = true;
            self.signals_check_notifier_
                .as_ref()
                .expect("source live notifier")
                .Schedule();
            return;
        }
        self.more_tiles_need_prepare_check_notifier_
            .as_ref()
            .expect("source live notifier")
            .Schedule();
    }
    // Source: cc/tiles/tile_manager.h:339
    // Source: cc/tiles/tile_manager.cc:1800-1825
    fn AreRequiredTilesReadyToDraw(
        &self,
        kind: super::raster_tile_priority_queue::Type,
        context: &mut impl TileManagerRuntime<B>,
    ) -> bool {
        let mut raster_priority_queue =
            context.BuildRasterQueue(self.client_, self.global_state_.tree_priority, kind);
        while !raster_priority_queue.IsEmpty() {
            let prioritized_tile = raster_priority_queue.Top();
            if !prioritized_tile
                .tile::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
                .expect("source non-null queue tile")
                .draw_info()
                .IsReadyToDraw()
            {
                return false;
            }
            raster_priority_queue.Pop(context);
        }
        #[cfg(debug_assertions)]
        {
            let mut all_queue =
                context.BuildRasterQueue(self.client_, self.global_state_.tree_priority, kind);
            while !all_queue.IsEmpty() {
                let tile = all_queue
                    .Top()
                    .tile::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
                    .expect("source non-null queue tile");
                debug_assert!(!tile.required_for_activation() || tile.draw_info().IsReadyToDraw());
                all_queue.Pop(context);
            }
        }
        true
    }
    // Source: cc/tiles/tile_manager.h:144
    // Source: cc/tiles/tile_manager.cc:1827-1832
    pub(crate) fn IsReadyToActivate(&self, context: &mut impl TileManagerRuntime<B>) -> bool {
        let _trace = context.TraceScope("TileManager::IsReadyToActivate");
        self.pending_required_for_activation_callback_id_ == 0
            && self.AreRequiredTilesReadyToDraw(
                super::raster_tile_priority_queue::Type::REQUIRED_FOR_ACTIVATION,
                context,
            )
    }
    // Source: cc/tiles/tile_manager.h:145
    // Source: cc/tiles/tile_manager.cc:1834-1839
    pub(crate) fn IsReadyToDraw(&self, context: &mut impl TileManagerRuntime<B>) -> bool {
        let _trace = context.TraceScope("TileManager::IsReadyToDraw");
        self.pending_required_for_draw_callback_id_ == 0
            && self.AreRequiredTilesReadyToDraw(
                super::raster_tile_priority_queue::Type::REQUIRED_FOR_DRAW,
                context,
            )
    }
}
impl<B: TileManagerTypes> Drop for TileManager<B> {
    // Source: cc/tiles/tile_manager.cc:415-419
    fn drop(&mut self) {
        debug_assert!(
            self.destroyed_,
            "Destroy must execute with live per-call services before dropping TileManager"
        );
        debug_assert!(self.tile_task_manager_.is_none());
    }
}

impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:379
    // Source: cc/tiles/tile_manager.cc:1841-1852
    fn ScheduleCheckRasterFinishedQueries(&mut self, context: &impl TileManagerRuntime<B>) {
        debug_assert!(self.has_pending_queries_);
        if self.check_pending_tile_queries_callback_.is_some() {
            return;
        }
        let lifetime = Arc::new(());
        let callback = context.OnceCommand(
            self.identity_,
            Arc::downgrade(&lifetime),
            TileManagerCommand::CheckRasterFinishedQueries,
        );
        self.check_pending_tile_queries_callback_ = Some(lifetime);
        self.task_runner_.PostDelayedTask(
            TaskLocation {
                function: "TileManager::ScheduleCheckRasterFinishedQueries",
                file: "cc/tiles/tile_manager.cc",
                line: 1849,
            },
            callback,
            TimeDelta::Milliseconds(100),
        );
    }
    // Source: cc/tiles/tile_manager.h:380
    // Source: cc/tiles/tile_manager.cc:1854-1871
    fn CheckRasterFinishedQueries(&mut self, context: &mut impl TileManagerRuntime<B>) {
        self.check_pending_tile_queries_callback_ = None;
        if !self.has_pending_queries_ {
            return;
        }
        if self.has_scheduled_tile_tasks_ || !self.signals_.all_tile_tasks_completed {
            return;
        }
        self.has_pending_queries_ = false;
        if let Some(queries) = self.pending_raster_queries_ {
            self.has_pending_queries_ = context.CheckRasterFinishedQueries(queries);
        }
        if self.has_pending_queries_ {
            self.ScheduleCheckRasterFinishedQueries(context);
        }
    }
    // Source: cc/tiles/tile_manager.h:378
    // Source: cc/tiles/tile_manager.cc:1881-1940
    fn IssueSignals(&mut self, context: &mut impl TileManagerRuntime<B>) {
        if self.signals_.activate_tile_tasks_completed
            && self.signals_.activate_gpu_work_completed
            && !self.signals_.did_notify_ready_to_activate
        {
            if context.HasPendingTree(self.client_) && self.IsReadyToActivate(context) {
                let _trace = context.TraceScope("TileManager::IssueSignals - ready to activate");
                self.signals_.did_notify_ready_to_activate = true;
                context.NotifyReadyToActivate(self.client_);
            }
        }
        if self.signals_.draw_tile_tasks_completed
            && self.signals_.draw_gpu_work_completed
            && !self.signals_.did_notify_ready_to_draw
        {
            if self.tile_manager_settings_.needs_notify_ready_to_draw && self.IsReadyToDraw(context)
            {
                let _trace = context.TraceScope("TileManager::IssueSignals - ready to draw");
                self.signals_.did_notify_ready_to_draw = true;
                context.NotifyReadyToDraw(self.client_);
            }
        }
        if self.signals_.all_tile_tasks_completed
            && !self.signals_.did_notify_all_tile_tasks_completed
        {
            if !self.has_scheduled_tile_tasks_ {
                let _trace =
                    context.TraceScope("TileManager::IssueSignals - all tile tasks completed");
                if self.has_pending_queries_ {
                    self.ScheduleCheckRasterFinishedQueries(context);
                }
                self.signals_.did_notify_all_tile_tasks_completed = true;
                context.NotifyAllTileTasksCompleted(self.client_);
            }
        }
        let notify_ready_to_activate_pending =
            context.HasPendingTree(self.client_) && !self.signals_.did_notify_ready_to_activate;
        let notify_ready_to_draw_pending = self.tile_manager_settings_.needs_notify_ready_to_draw
            && !self.signals_.did_notify_ready_to_draw;
        if self.signals_.did_notify_all_tile_tasks_completed {
            context
                .CheckerImageTracker(self.checker_image_tracker_)
                .SetMaxDecodePriorityAllowed(crate::adapter::tile_manager::DecodeType::kPreDecode);
        } else if !notify_ready_to_activate_pending && !notify_ready_to_draw_pending {
            context
                .CheckerImageTracker(self.checker_image_tracker_)
                .SetMaxDecodePriorityAllowed(crate::adapter::tile_manager::DecodeType::kRaster);
        }
    }
    // Source: cc/tiles/tile_manager.h:377
    // Source: cc/tiles/tile_manager.cc:2114-2189
    fn CheckPendingGpuWorkAndIssueSignals(&mut self, context: &mut impl TileManagerRuntime<B>) {
        let _trace =
            context.TraceScopeWithArgs("TileManager::CheckPendingGpuWorkAndIssueSignals", || {
                vec![
                    crate::adapter::tile_manager::TileManagerTraceArgument::Unsigned(
                        "pending_gpu_work_tiles",
                        self.pending_gpu_work_tiles_.len() as u64,
                    ),
                    crate::adapter::tile_manager::TileManagerTraceArgument::String(
                        "tree_priority",
                        super::tile_priority::TreePriorityToString(
                            self.global_state_.tree_priority,
                        ),
                    ),
                ]
            });
        // Native vectors contain only non-owning lease pointers. These values
        // preserve pointer identity and resolve resident leases at the same
        // final provider call; no resource or tile data is copied.
        let mut required_for_activation = Vec::new();
        let mut required_for_draw = Vec::new();
        let provider = self.raster_buffer_provider_;
        self.pending_gpu_work_tiles_.retain(|tile| {
            debug_assert!(context.TileById(*tile).draw_info().has_resource());
            if self.pending_tile_requirements_dirty_ {
                context
                    .TilingById(tile.tiling)
                    .UpdateRequiredStatesOnTile(*tile, context);
            }
            if self.global_state_.tree_priority
                != super::tile_priority::TreePriority::SMOOTHNESS_TAKES_PRIORITY
                || context.IsResourceReadyToDrawForTile(
                    provider.expect("source raster_buffer_provider_"),
                    *tile,
                )
            {
                context
                    .TileByIdMutable(*tile)
                    .draw_info_mut()
                    .set_resource_ready_for_draw();
                context.NotifyTileStateChangedById(self.client_, *tile, true, true);
                return false;
            }
            if context.TileById(*tile).required_for_activation() {
                required_for_activation.push(*tile);
            }
            if context.TileById(*tile).required_for_draw() {
                required_for_draw.push(*tile);
            }
            true
        });
        if required_for_activation.is_empty() {
            self.pending_required_for_activation_callback_id_ = 0;
        } else {
            let callback = context.OnceCommand(
                self.identity_,
                Arc::downgrade(&self.ready_to_draw_callback_weak_ptr_factory_),
                TileManagerCommand::CheckPendingGpuWorkAndIssueSignals,
            );
            self.pending_required_for_activation_callback_id_ = context
                .SetReadyToDrawCallbackForTiles(
                    self.raster_buffer_provider_.unwrap(),
                    &required_for_activation,
                    callback,
                    self.pending_required_for_activation_callback_id_,
                );
        }
        if required_for_draw.is_empty() {
            self.pending_required_for_draw_callback_id_ = 0;
        } else {
            let callback = context.OnceCommand(
                self.identity_,
                Arc::downgrade(&self.ready_to_draw_callback_weak_ptr_factory_),
                TileManagerCommand::CheckPendingGpuWorkAndIssueSignals,
            );
            self.pending_required_for_draw_callback_id_ = context.SetReadyToDrawCallbackForTiles(
                self.raster_buffer_provider_.unwrap(),
                &required_for_draw,
                callback,
                self.pending_required_for_draw_callback_id_,
            );
        }
        self.signals_.activate_gpu_work_completed =
            self.pending_required_for_activation_callback_id_ == 0;
        self.signals_.draw_gpu_work_completed = self.pending_required_for_draw_callback_id_ == 0;
        self.pending_tile_requirements_dirty_ = false;
        self.IssueSignals(context);
    }
    // Source: cc/tiles/tile_manager.h:376
    // Source: cc/tiles/tile_manager.cc:1873-1879
    fn CheckForCompletedTasksAndIssueSignals(&mut self, context: &mut impl TileManagerRuntime<B>) {
        let _trace = context.TraceScope("TileManager::CheckForCompletedTasksAndIssueSignals");
        self.CheckForCompletedTasks(context);
        self.did_check_for_completed_tasks_since_last_schedule_tasks_ = true;
        self.CheckPendingGpuWorkAndIssueSignals(context);
    }
    // Source: cc/tiles/tile_manager.h:133
    // Source: cc/tiles/tile_manager.cc:752-779
    pub(crate) fn PrepareToDraw(&mut self, context: &mut impl TileManagerRuntime<B>) -> bool {
        let _trace = context.TraceScope("TileManager::PrepareToDraw");
        if self.tile_task_manager_.is_none() {
            context.TraceInstant("TileManager::PrepareToDrawAborted", || None);
            return true;
        }
        self.CheckForCompletedTasks(context);
        self.did_check_for_completed_tasks_since_last_schedule_tasks_ = true;
        self.CheckPendingGpuWorkAndIssueSignals(context);
        let is_ready_to_draw = self.IsReadyToDraw(context);
        if is_ready_to_draw {
            context.SetIsLikelyToRequireADraw(self.client_, false);
        }
        context.TraceInstant("TileManager::PrepareToDrawFinished", || {
            Some(RasterTaskCompletionStatsAsValue(
                &self.raster_task_completion_stats_,
            ))
        });
        self.raster_task_completion_stats_ = RasterTaskCompletionStats::default();
        is_ready_to_draw
    }
}

use super::{
    eviction_tile_priority_queue::EvictionTilePriorityQueue,
    tile_priority::{PriorityBin, TilePriority, TileResolution},
};
use crate::adapter::tile_manager::TileManagerFeature;
// Source: cc/tiles/tile_manager.h:388
const kDelayBeforeTimeReclaim: TimeDelta = TimeDelta::Seconds(5 * 60);
impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:310
    // Source: cc/tiles/tile_manager.cc:453-468
    fn ScheduleReduceTileMemoryWhenIdle(
        &mut self,
        time_since_last_active: TimeDelta,
        context: &impl TileManagerRuntime<B>,
    ) {
        if !context.FeatureEnabled(TileManagerFeature::ReclaimPrepaintTilesWhenIdle)
            || self.has_pending_idle_task_
        {
            return;
        }
        self.has_pending_idle_task_ = true;
        let delay = kDelayBeforeTimeReclaim - time_since_last_active;
        self.TaskRunnerWithOverride().PostDelayedTask(
            TaskLocation {
                function: "TileManager::ScheduleReduceTileMemoryWhenIdle",
                file: "cc/tiles/tile_manager.cc",
                line: 463,
            },
            context.OnceCommand(
                self.identity_,
                Arc::downgrade(&self.ready_to_draw_callback_weak_ptr_factory_),
                TileManagerCommand::ReduceTileMemoryWhenIdle,
            ),
            delay,
        );
    }
    // Source: cc/tiles/tile_manager.h:387
    // Source: cc/tiles/tile_manager.cc:470-473
    pub(crate) fn GetTrimPrepaintTilesDelay(context: &impl TileManagerRuntime<B>) -> TimeDelta {
        TimeDelta::Seconds(context.ReclaimDelayInSeconds())
    }
    // Source: cc/tiles/tile_manager.h:311
    // Source: cc/tiles/tile_manager.cc:475-487
    fn ScheduleTrimPrepaintTiles(&mut self, context: &impl TileManagerRuntime<B>) {
        if !context.FeatureEnabled(TileManagerFeature::ReclaimOldPrepaintTiles)
            || self.has_pending_tile_trimming_task_
        {
            return;
        }
        self.has_pending_tile_trimming_task_ = true;
        self.TaskRunnerWithOverride().PostDelayedTask(
            TaskLocation {
                function: "TileManager::ScheduleTrimPrepaintTiles",
                file: "cc/tiles/tile_manager.cc",
                line: 482,
            },
            context.OnceCommand(
                self.identity_,
                Arc::downgrade(&self.ready_to_draw_callback_weak_ptr_factory_),
                TileManagerCommand::TrimPrepaintTiles,
            ),
            Self::GetTrimPrepaintTilesDelay(context),
        );
    }
    // Source: cc/tiles/tile_manager.h:312
    // Source: cc/tiles/tile_manager.cc:489-518
    fn ReduceTileMemoryWhenIdle(&mut self, context: &mut impl TileManagerRuntime<B>) {
        self.has_pending_idle_task_ = false;
        let time_since_last_active = self.NowWithOverride(context) - self.last_active_time_;
        if time_since_last_active < kDelayBeforeTimeReclaim {
            self.ScheduleReduceTileMemoryWhenIdle(time_since_last_active, context);
            return;
        }
        let limit = MemoryUsage::new(0, 0);
        let pool = context.ResourcePool(self.resource_pool_.expect("source resource_pool_"));
        let mut usage = MemoryUsage::new(pool.memory_usage_bytes(), pool.resource_count());
        const kVisiblePriority: TilePriority =
            TilePriority::new(TileResolution::HIGH_RESOLUTION, PriorityBin::NOW, 0.0);
        let mut freed_required_for_draw_tile = false;
        self.FreeTileResourcesWithLowerPriorityUntilUsageIsWithinLimit(
            None,
            &limit,
            &kVisiblePriority,
            &mut usage,
            &mut freed_required_for_draw_tile,
            context,
        );
        if freed_required_for_draw_tile {
            context.SetNeedsRedraw(self.client_, false, true);
        }
    }
    // Source: cc/tiles/tile_manager.h:313
    // Source: cc/tiles/tile_manager.cc:520-580
    fn TrimPrepaintTiles(&mut self, context: &mut impl TileManagerRuntime<B>) {
        self.has_pending_tile_trimming_task_ = false;
        let memory_limit_policy = self.global_state_.memory_limit_policy;
        let queue = context.BuildEvictionQueue(self.client_);
        let mut adapter = PrepaintTrimAdapter {
            manager: self,
            context,
            queue: Some(queue),
        };
        let outcome = TrimPrepaintPolicy(&mut adapter, memory_limit_policy);
        let queue = adapter.queue.take().expect("prepaint eviction queue");
        drop(adapter);
        drop(queue);
        if outcome.freed_required_for_draw_tile {
            context.SetNeedsRedraw(self.client_, false, true);
        }
        if outcome.retry_for_deferred_tiles {
            self.ScheduleTrimPrepaintTiles(context);
        }
    }
    // Source: cc/tiles/tile_manager.h:325-330
    // Source: cc/tiles/tile_manager.cc:847-867
    fn FreeTileResourcesUntilUsageIsWithinLimit(
        &mut self,
        mut eviction_priority_queue: Option<Box<EvictionTilePriorityQueue<B>>>,
        limit: &MemoryUsage,
        usage: &mut MemoryUsage,
        freed_required_for_draw_tile: &mut bool,
        context: &mut impl TileManagerRuntime<B>,
    ) -> Option<Box<EvictionTilePriorityQueue<B>>> {
        loop {
            if DecideEvictionStep(usage, limit, None, None) != EvictionStep::Evict {
                break;
            }
            if eviction_priority_queue.is_none() {
                eviction_priority_queue = Some(context.BuildEvictionQueue(self.client_));
            }
            let queue = eviction_priority_queue.as_mut().unwrap();
            if queue.IsEmpty() {
                break;
            }
            let tile = queue.Top(context).tile_id().expect("source non-null tile");
            *usage -= &MemoryUsage::FromTile(context.TileById(tile));
            *freed_required_for_draw_tile |=
                self.FreeResourcesForTileAndNotifyClientIfTileWasReadyToDraw(tile, context);
            queue.Pop(context);
        }
        eviction_priority_queue
    }
    // Source: cc/tiles/tile_manager.h:331-337
    // Source: cc/tiles/tile_manager.cc:869-894
    fn FreeTileResourcesWithLowerPriorityUntilUsageIsWithinLimit(
        &mut self,
        mut eviction_priority_queue: Option<Box<EvictionTilePriorityQueue<B>>>,
        limit: &MemoryUsage,
        other_priority: &TilePriority,
        usage: &mut MemoryUsage,
        freed_required_for_draw_tile: &mut bool,
        context: &mut impl TileManagerRuntime<B>,
    ) -> Option<Box<EvictionTilePriorityQueue<B>>> {
        loop {
            if DecideEvictionStep(usage, limit, None, None) != EvictionStep::Evict {
                break;
            }
            if eviction_priority_queue.is_none() {
                eviction_priority_queue = Some(context.BuildEvictionQueue(self.client_));
            }
            let queue = eviction_priority_queue.as_mut().unwrap();
            if queue.IsEmpty() {
                break;
            }
            let prioritized_tile = queue.Top(context);
            match DecideEvictionStep(
                usage,
                limit,
                Some(prioritized_tile.priority()),
                Some(other_priority),
            ) {
                EvictionStep::Evict => {}
                EvictionStep::StopAtPriorityBarrier | EvictionStep::StopWithinLimit => break,
            }
            let tile = prioritized_tile.tile_id().expect("source non-null tile");
            *usage -= &MemoryUsage::FromTile(context.TileById(tile));
            *freed_required_for_draw_tile |=
                self.FreeResourcesForTileAndNotifyClientIfTileWasReadyToDraw(tile, context);
            queue.Pop(context);
        }
        eviction_priority_queue
    }
    // Source: cc/tiles/tile_manager.h:341-342
    // Source: cc/tiles/tile_manager.cc:2046-2067
    fn MarkTilesOutOfMemory(
        &mut self,
        mut queue: Box<dyn super::raster_tile_priority_queue::RasterTilePriorityQueue<B>>,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        let mut tiles_required_for_draw = false;
        while !queue.IsEmpty() {
            let tile = queue.Top().tile_id().expect("source non-null tile");
            if context.TileById(tile).draw_info().IsReadyToDraw() {
                queue.Pop(context);
                continue;
            }
            context.TileByIdMutable(tile).draw_info_mut().set_oom();
            context.NotifyTileStateChangedById(self.client_, tile, true, false);
            tiles_required_for_draw |= context.TileById(tile).required_for_draw();
            queue.Pop(context);
        }
        if tiles_required_for_draw {
            context.SetNeedsRedraw(self.client_, false, true);
        }
    }
}

use crate::adapter::{
    geometry::{AxisTransform2d, Rect, RectF, ToEnclosingRect},
    tile_manager::{
        AnimatedImageFrameIndexMap, DecodeQueueItem, DecodeType, ImageDecodeQueue,
        TileManagerDrawImage, TileManagerPaintImage, TileManagerRasterSource,
    },
};
// Source: cc/tiles/tile_manager.cc:345-350
fn GetContentColorUsageForPrioritizedTile<B: TileManagerTypes>(
    prioritized_tile: &super::prioritized_tile::PrioritizedTile,
    context: &impl TileManagerRuntime<B>,
) -> B::ContentColorUsage {
    prioritized_tile
        .raster_source::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
        .as_ref()
        .expect("source raster_source")
        .content_color_usage()
}
impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:306-307
    // Source: cc/tiles/tile_manager.cc:1176-1188
    fn FreeResourcesForOccludedTiles(&mut self, context: &mut impl TileManagerRuntime<B>) {
        context.WithTilesWithResourceIterator(
            self.client_,
            self.resource_pool_.expect("source resource_pool_"),
            |mut parts| {
                let _ =
                    parts
                        .traversal
                        .ForEachResourceTile(&mut |tile, is_occluded, notification| {
                            if is_occluded {
                                FreeResourcesForTile(
                                    tile,
                                    parts.pool,
                                    &mut self.num_of_tiles_with_checker_images_,
                                    &mut self.pending_gpu_work_tiles_,
                                );
                                // Source GetCurrent again denotes this same live Tile
                                // after its resource release; the borrow never escapes.
                                notification.NotifyTileStateChanged(tile, false, true);
                            }
                            std::ops::ControlFlow::Continue(())
                        });
            },
        );
    }
    // Source: cc/tiles/tile_manager.h:358-366
    // Source: cc/tiles/tile_manager.cc:1217-1248
    fn PartitionImagesForCheckering(
        client: TileManagerClientId,
        checker: CheckerImageTrackerId,
        prioritized_tile: &super::prioritized_tile::PrioritizedTile,
        target_color_params: &B::TargetColorParams,
        sync_decoded_images: &mut Vec<B::DrawImage>,
        checkered_images: &mut Vec<B::PaintImage>,
        invalidated_rect: Option<&Rect>,
        mut image_to_frame_index: Option<&mut AnimatedImageFrameIndexMap<B>>,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        let tile = prioritized_tile
            .tile::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
            .expect("source non-null tile");
        let mut enclosing_rect = *tile.enclosing_layer_rect();
        if let Some(invalidated_rect) = invalidated_rect {
            enclosing_rect = ToEnclosingRect(
                tile.raster_transform()
                    .InverseMapRect(RectF::from(*invalidated_rect)),
            );
        }
        let source_tiling = prioritized_tile
            .source_tiling::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
            .expect("source non-null tiling")
            .identity();
        let tile_id = tile.identity();
        let parts =
            context.ImagePartitionParts(source_tiling, tile_id, enclosing_rect, client, checker);
        let tree = parts.tile_tiling.tree();
        for original_draw_image in parts.images {
            let image = original_draw_image.paint_image();
            let frame_index = parts.client.GetFrameIndexForImage(image, tree);
            if let Some(map) = image_to_frame_index.as_deref_mut() {
                map.insert(image.stable_id(), frame_index);
            }
            let draw_image = original_draw_image.WithRasterParameters(
                parts.tile.contents_scale_key(),
                frame_index,
                target_color_params,
            );
            if parts.checker.ShouldCheckerImage(&draw_image, tree) {
                checkered_images.push(draw_image.paint_image().clone());
            } else {
                sync_decoded_images.push(draw_image);
            }
        }
    }
    // Source: cc/tiles/tile_manager.h:367-371
    // Source: cc/tiles/tile_manager.cc:1250-1269
    fn AddCheckeredImagesToDecodeQueue(
        &mut self,
        prioritized_tile: &super::prioritized_tile::PrioritizedTile,
        target_color_params: &B::TargetColorParams,
        decode_type: DecodeType,
        image_decode_queue: &mut ImageDecodeQueue<B>,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        let tile = prioritized_tile
            .tile::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
            .expect("source non-null tile");
        let source_tiling = prioritized_tile
            .source_tiling::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
            .expect("source non-null tiling")
            .identity();
        let enclosing_rect = *tile.enclosing_layer_rect();
        let tile_id = tile.identity();
        let parts = context.ImagePartitionParts(
            source_tiling,
            tile_id,
            enclosing_rect,
            self.client_,
            self.checker_image_tracker_,
        );
        let tree = parts.tile_tiling.tree();
        for original_draw_image in parts.images {
            let frame_index = parts
                .client
                .GetFrameIndexForImage(original_draw_image.paint_image(), tree);
            let draw_image = original_draw_image.WithRasterParameters(
                parts.tile.contents_scale_key(),
                frame_index,
                target_color_params,
            );
            if parts.checker.ShouldCheckerImage(&draw_image, tree) {
                image_decode_queue.push(DecodeQueueItem {
                    image: draw_image.paint_image().clone(),
                    decode_type,
                });
            }
        }
    }
}

use crate::adapter::{
    prioritized_tile::ScrollOffsetMap,
    tile::TileTraceId,
    tile_manager::{
        RasterImageProvider, RasterPlaybackRequest, RasterPlaybackSettings,
        RasterPlaybackSettingsView, TileManagerRasterBuffer, WorkletImageProvider,
    },
};
// Source: cc/tiles/tile_manager.cc:64-68
const kUseColorEstimator: bool = true;
// Source: cc/tiles/tile_manager.cc:70-96
// Owned external image-provider values preserve the source dispatch wrapper.
// Actual content/record access and raster image lifetime remain in raster.
struct DispatchingImageProvider<B: TileManagerTypes> {
    paint_worklet_image_provider_: B::PaintWorkletImageProvider,
    playback_image_provider_: B::PlaybackImageProvider,
}
impl<B: TileManagerTypes> DispatchingImageProvider<B> {
    fn new(
        playback_image_provider: B::PlaybackImageProvider,
        paint_worklet_image_provider: B::PaintWorkletImageProvider,
    ) -> Self {
        Self {
            playback_image_provider_: playback_image_provider,
            paint_worklet_image_provider_: paint_worklet_image_provider,
        }
    }
}
impl<B: TileManagerTypes> RasterImageProvider<B> for DispatchingImageProvider<B> {
    fn GetRasterContent(&mut self, draw_image: &B::DrawImage) -> B::ScopedRasterResult {
        if draw_image.paint_image().IsPaintWorklet() {
            self.paint_worklet_image_provider_
                .GetPaintRecordResult(draw_image.paint_image().GetPaintWorkletInput())
        } else {
            self.playback_image_provider_.GetRasterContent(draw_image)
        }
    }
}
// Source: cc/tiles/tile_manager.cc:98-211
// Source captures these immutable worker inputs explicitly at construction.
// They are raster work, not a retained owner/tree snapshot. Metadata remains
// read from actual Tile fields at the constructor's source call positions.
// Private access cells synchronize only this one refcounted task's owned
// provider/buffer/lease. They are never exposed to scheduling algorithms.
struct RasterTaskImpl<B: TileManagerTypes> {
    raster_inducing_scroll_offsets_: ScrollOffsetMap,
    url_: String,
    image_provider_: Mutex<DispatchingImageProvider<B>>,
    raster_buffer_: Mutex<Option<B::RasterBuffer>>,
    source_frame_number_: i32,
    new_content_id_: u64,
    tile_tracing_id_: TileTraceId,
    source_prepare_tiles_id_: u64,
    layer_id_: i32,
    tile_resolution_: TileResolution,
    playback_settings_: RasterPlaybackSettings,
    raster_transform_: B::RasterTransform,
    invalid_content_rect_: Rect,
    content_rect_: Rect,
    raster_source_: Arc<B::RasterSource>,
    resource_: Mutex<InUsePoolResource<B>>,
    tile_id_: Id,
    manager_: TileManagerId,
    diagnostics_: Arc<dyn TileManagerWorkerDiagnostics>,
    base_: TileTask<RasterCompletion<B>>,
}
impl<B: TileManagerTypes> RasterTaskImpl<B> {
    fn new(
        manager: TileManagerId,
        tile: &Tile<B>,
        resource: InUsePoolResource<B>,
        raster_source: Arc<B::RasterSource>,
        playback_settings: RasterPlaybackSettings,
        tile_resolution: TileResolution,
        invalidated_rect: Rect,
        source_prepare_tiles_id: u64,
        raster_buffer: Option<B::RasterBuffer>,
        dependencies: &mut Vec<TileTaskRef<RasterCompletion<B>>>,
        is_gpu_rasterization: bool,
        image_provider: DispatchingImageProvider<B>,
        url: String,
        raster_inducing_scroll_offsets: ScrollOffsetMap,
        diagnostics: Arc<dyn TileManagerWorkerDiagnostics>,
    ) -> Self {
        let base = TileTask::new(
            diagnostics.as_ref(),
            if is_gpu_rasterization {
                SupportsConcurrentExecution::kNo
            } else {
                SupportsConcurrentExecution::kYes
            },
            if raster_buffer
                .as_ref()
                .is_some_and(|buffer| buffer.SupportsBackgroundThreadPriority())
            {
                SupportsBackgroundThreadPriority::kYes
            } else {
                SupportsBackgroundThreadPriority::kNo
            },
            Some(dependencies),
        );
        let result = Self {
            base_: base,
            manager_: manager,
            tile_id_: tile.id(),
            resource_: Mutex::new(resource),
            raster_source_: raster_source,
            content_rect_: *tile.content_rect(),
            invalid_content_rect_: invalidated_rect,
            raster_transform_: tile.raster_transform().clone(),
            playback_settings_: playback_settings,
            tile_resolution_: tile_resolution,
            layer_id_: tile.layer_id(),
            source_prepare_tiles_id_: source_prepare_tiles_id,
            tile_tracing_id_: TileTraceId(std::ptr::from_ref(tile) as usize),
            new_content_id_: tile.id(),
            source_frame_number_: tile.source_frame_number(),
            raster_buffer_: Mutex::new(raster_buffer),
            image_provider_: Mutex::new(image_provider),
            url_: url,
            raster_inducing_scroll_offsets_: raster_inducing_scroll_offsets,
            diagnostics_: diagnostics,
        };
        debug_assert!(result.diagnostics_.CalledOnOriginThread());
        // Source settings' two self-pointers are bound as short references in
        // RunOnWorkerThread; no self-reference or owner reference is retained.
        result
    }
}
impl<B: TileManagerTypes> TaskImpl<RasterCompletion<B>> for RasterTaskImpl<B> {
    fn task(&self) -> &Task {
        self.base_.task()
    }
    fn RunOnWorkerThread(&self) {
        let _trace = self
            .diagnostics_
            .TraceRasterWorker(self.source_prepare_tiles_id_);
        // Rust Arc makes source DCHECK(raster_source_.get()) structural.
        let mut buffer = self.raster_buffer_.lock().unwrap();
        debug_assert!(buffer.is_some());
        let _raster_task = self.diagnostics_.ScopedRasterTask(
            self.tile_tracing_id_,
            self.tile_resolution_,
            self.source_frame_number_,
            self.layer_id_,
        );
        let mut image_provider = self.image_provider_.lock().unwrap();
        let settings = RasterPlaybackSettingsView {
            values: &self.playback_settings_,
            image_provider: &mut *image_provider,
            raster_inducing_scroll_offsets: &self.raster_inducing_scroll_offsets_,
        };
        buffer
            .as_mut()
            .expect("source raster_buffer_")
            .Playback(RasterPlaybackRequest {
                source: &self.raster_source_,
                content_rect: &self.content_rect_,
                invalid_content_rect: &self.invalid_content_rect_,
                new_content_id: self.new_content_id_,
                raster_transform: &self.raster_transform_,
                settings,
                url: &self.url_,
            });
    }
    fn AsTileTask(&self) -> Option<&dyn TileTaskImpl<RasterCompletion<B>>> {
        Some(self)
    }
}
impl<B: TileManagerTypes> TileTaskImpl<RasterCompletion<B>> for RasterTaskImpl<B> {
    fn tile_task(&self) -> &TileTask<RasterCompletion<B>> {
        &self.base_
    }
    fn OnTaskCompleted(&self) -> Option<RasterCompletion<B>> {
        debug_assert!(self.diagnostics_.CalledOnOriginThread());
        // Release the actual raster buffer first, outside the access cell,
        // before transferring the original lease to the synchronous consumer.
        let buffer = self.raster_buffer_.lock().unwrap().take();
        drop(buffer);
        let resource = std::mem::take(&mut *self.resource_.lock().unwrap());
        Some(RasterCompletion {
            token: RasterCompletionToken {
                manager: self.manager_,
                tile_id: self.tile_id_,
                prepare_tiles_id: self.source_prepare_tiles_id_,
                trace_task_id: self.task().trace_task_id(),
            },
            resource,
            was_canceled: self.task().WithState(|state| state.IsCanceled()),
        })
    }
}
impl<B: TileManagerTypes> Drop for RasterTaskImpl<B> {
    fn drop(&mut self) {
        debug_assert!(self.diagnostics_.CalledOnOriginThread());
        debug_assert!(self.raster_buffer_.get_mut().unwrap().is_none());
        debug_assert!(!self.resource_.get_mut().unwrap().IsValid());
    }
}

use crate::adapter::tile_manager::{
    ImageTracingInfo, PlaybackImageProviderSettings, TileManagerTargetColorParams,
};
impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:320-323
    // Source: cc/tiles/tile_manager.cc:1482-1631
    fn CreateRasterTask(
        &mut self,
        prioritized_tile: &super::prioritized_tile::PrioritizedTile,
        target_color_params: &B::TargetColorParams,
        work_to_schedule: &mut PrioritizedWorkToSchedule<B>,
        context: &mut impl TileManagerRuntime<B>,
    ) -> Option<TileTaskRef<RasterCompletion<B>>> {
        let _trace = context.TraceScope("TileManager::CreateRasterTask");
        let tile = prioritized_tile.tile_id().expect("source non-null tile");
        let _tile_trace = context.TraceScopeWithArgs("TileManager::CreateRasterTask", || {
            vec![
                crate::adapter::tile_manager::TileManagerTraceArgument::Unsigned(
                    "Tile",
                    context.TileById(tile).id(),
                ),
            ]
        });
        let source_tiling = prioritized_tile
            .source_tiling::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
            .expect("source non-null source_tiling")
            .identity();
        let msaa_sample_count = context.GetMSAASampleCountForRaster(
            self.client_,
            prioritized_tile
                .raster_source::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
                .as_ref()
                .expect("source raster_source")
                .GetDisplayItemList(),
        );
        let mut format = context.GetTileFormat(self.client_);
        if B::IsHDR(target_color_params.color_space())
            && GetContentColorUsageForPrioritizedTile(prioritized_tile, context)
                == B::HdrContentColorUsage()
        {
            format = crate::adapter::resource_types::SinglePlaneFormat::RGBA_F16;
        }
        let mut resource = InUsePoolResource::<B>::default();
        let mut resource_content_id = 0;
        let mut invalidated_rect = *context.TileById(tile).invalidated_content_rect();
        if self.UsePartialRaster(msaa_sample_count, context)
            && context.TileById(tile).invalidated_id() != 0
        {
            let (tile, source, pool) = context.TileRasterSourceAndPool(
                tile,
                source_tiling,
                self.resource_pool_.expect("source resource_pool_"),
            );
            let debug_name = source.debug_name();
            resource = pool.TryAcquireResourceForPartialRaster(
                tile.id(),
                tile.invalidated_content_rect(),
                tile.invalidated_id(),
                &mut invalidated_rect,
                target_color_params.color_space(),
                &*debug_name,
            );
        }
        let mut partial_tile_decode = false;
        if resource.IsValid() {
            resource_content_id = context.TileById(tile).invalidated_id();
            debug_assert_eq!(format, *resource.format());
            partial_tile_decode = true;
        } else {
            let (tile, source, pool) = context.TileRasterSourceAndPool(
                tile,
                source_tiling,
                self.resource_pool_.expect("source resource_pool_"),
            );
            let debug_name = source.debug_name();
            resource = pool.AcquireResource(
                &tile.desired_texture_size(),
                format,
                target_color_params.color_space(),
                &*debug_name,
            );
            debug_assert!(resource.IsValid());
        }
        let mut playback_settings = RasterPlaybackSettings::default();
        playback_settings.use_lcd_text = context.TileById(tile).can_use_lcd_text();
        playback_settings.msaa_sample_count = msaa_sample_count;
        playback_settings.visible = context.TileById(tile).required_for_activation()
            || context.TileById(tile).required_for_draw();
        playback_settings.hdr_headroom = target_color_params.GetHdrHeadroom();
        let mut decode_tasks = Vec::new();
        let tile_id = context.TileById(tile).id();
        let sync_decoded_images = self.scheduled_draw_images_.entry(tile_id).or_default();
        sync_decoded_images.clear();
        let mut checkered_images = Vec::new();
        let mut image_id_to_current_frame_index = Arc::new(AnimatedImageFrameIndexMap::<B>::new());
        Self::PartitionImagesForCheckering(
            self.client_,
            self.checker_image_tracker_,
            prioritized_tile,
            target_color_params,
            sync_decoded_images,
            &mut checkered_images,
            if partial_tile_decode {
                Some(&invalidated_rect)
            } else {
                None
            },
            Some(
                Arc::get_mut(&mut image_id_to_current_frame_index)
                    .expect("new source map has unique owner"),
            ),
            context,
        );
        let tracing_info = ImageTracingInfo {
            prepare_tiles_id: self.prepare_tiles_count_,
            priority_bin: prioritized_tile.priority().priority_bin,
        };
        let mut has_at_raster_images = false;
        context
            .ImageController(self.image_controller_)
            .ConvertImagesToTasks(
                sync_decoded_images,
                &mut decode_tasks,
                &mut has_at_raster_images,
                tracing_info,
            );
        context
            .DecodedImageTracker(self.decoded_image_tracker_)
            .OnImagesUsedInDraw(sync_decoded_images);
        let has_checker_images = !checkered_images.is_empty();
        context
            .TileByIdMutable(tile)
            .set_raster_task_scheduled_with_checker_images(has_checker_images);
        if has_checker_images {
            self.num_of_tiles_with_checker_images_ += 1;
        }
        if has_at_raster_images && context.TileById(tile).is_prepaint() {
            work_to_schedule
                .extra_prepaint_images
                .extend_from_slice(sync_decoded_images);
            self.OnRasterTaskCompleted(tile_id, resource, true, context);
            return None;
        }
        let mut images_to_skip = std::collections::BTreeSet::new();
        for image in checkered_images {
            debug_assert!(!image.ShouldAnimate());
            images_to_skip.insert(image.stable_id());
            if prioritized_tile.should_decode_checkered_images_for_tile() {
                work_to_schedule
                    .checker_image_decode_queue
                    .push(DecodeQueueItem {
                        image,
                        decode_type: DecodeType::kRaster,
                    });
            }
        }
        let invalidated_id = context.TileById(tile).invalidated_id();
        // Rust borrow split of source provider_->AcquireBufferForRaster: its
        // resource.set_backing callback borrows this same resident pool.
        let raster_buffer = context.AcquireRasterBufferForResource(
            self.raster_buffer_provider_
                .expect("source raster_buffer_provider_"),
            self.resource_pool_.expect("source resource_pool_"),
            &resource,
            resource_content_id,
            invalidated_id,
        );
        let mut settings = PlaybackImageProviderSettings::<B> {
            images_to_skip,
            image_to_current_frame_index: image_id_to_current_frame_index,
            use_gpu_raster_mode: false,
        };
        if self.use_gpu_rasterization_ {
            settings.use_gpu_raster_mode = true;
        }
        let cache = context.ImageController(self.image_controller_).cache();
        let image_provider =
            context.CreatePlaybackImageProvider(cache, target_color_params, settings);
        // This is the source's deliberate immutable worklet-record copy for
        // worker lifetime (cc1615-1619), not an owner/tree snapshot API.
        let (paint_worklet_records, raster_source, raster_inducing_scroll_offsets) = {
            let source_tiling = context.TilingById(
                prioritized_tile
                    .source_tiling_id()
                    .expect("source nonnull tiling"),
            );
            (
                source_tiling.GetPaintWorkletRecords(context).clone(),
                source_tiling
                    .raster_source()
                    .as_ref()
                    .expect("source raster_source")
                    .clone(),
                source_tiling.GetRasterInducingScrollOffsets(context),
            )
        };
        let paint_worklet_image_provider =
            context.CreatePaintWorkletImageProvider(paint_worklet_records);
        let dispatching_image_provider =
            DispatchingImageProvider::new(image_provider, paint_worklet_image_provider);
        Some(Arc::new(RasterTaskImpl::new(
            self.identity_,
            context.TileById(tile),
            resource,
            raster_source,
            playback_settings,
            prioritized_tile.priority().resolution,
            invalidated_rect,
            self.prepare_tiles_count_,
            raster_buffer,
            &mut decode_tasks,
            self.use_gpu_rasterization_,
            dispatching_image_provider,
            self.active_url_.clone(),
            raster_inducing_scroll_offsets,
            context.WorkerDiagnostics(),
        )))
    }
}
impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:353-354
    // Source: cc/tiles/tile_manager.cc:2191-2199
    fn CreateTaskSetFinishedTask(
        &self,
        callback: TileManagerCommand,
        context: &impl TileManagerRuntime<B>,
    ) -> TileTaskRef<RasterCompletion<B>> {
        Arc::new(TaskSetFinishedTaskImpl::new(
            self.task_runner_.clone(),
            context.OnceCommand(
                self.identity_,
                Arc::downgrade(&self.task_set_finished_weak_ptr_factory_),
                callback,
            ),
            context.WorkerDiagnostics(),
        ))
    }
    // Source: cc/tiles/tile_manager.h:356-357
    // Source: cc/tiles/tile_manager.cc:1271-1480
    fn ScheduleTasks(
        &mut self,
        mut work_to_schedule: PrioritizedWorkToSchedule<B>,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        let start_time = if context.ShouldRecordSubsampledMetric(self.metrics_sampling_rate_) {
            context.Now()
        } else {
            TimeTicks::default()
        };
        let tiles_that_need_to_be_rasterized = &work_to_schedule.tiles_to_raster;
        let _trace = context.TraceScopeWithArgs("TileManager::ScheduleTasks", || {
            vec![
                crate::adapter::tile_manager::TileManagerTraceArgument::Unsigned(
                    "count",
                    tiles_that_need_to_be_rasterized.len() as u64,
                ),
            ]
        });
        debug_assert!(self.did_check_for_completed_tasks_since_last_schedule_tasks_);
        if !self.has_scheduled_tile_tasks_ {
            context.TraceBeginOnTrack("ScheduledTasks", GetTracingTrack(self.identity_));
        }
        self.task_set_finished_weak_ptr_factory_ = Arc::new(());
        self.has_scheduled_tile_tasks_ = true;
        let required_for_activation_done_task = self.CreateTaskSetFinishedTask(
            TileManagerCommand::DidFinishRunningTileTasksRequiredForActivation,
            context,
        );
        let required_for_draw_done_task = self.CreateTaskSetFinishedTask(
            TileManagerCommand::DidFinishRunningTileTasksRequiredForDraw,
            context,
        );
        let all_done_cb = context.AllTilesDoneCommand(
            self.identity_,
            Arc::downgrade(&self.task_set_finished_weak_ptr_factory_),
            start_time,
        );
        let pending_raster_queries = self
            .pending_raster_queries_
            .map(|id| context.RasterQueryCapability(id));
        let raster_buffer_provider = self
            .raster_buffer_provider_
            .map(|id| context.RasterFlushCapability(id));
        let all_done_task: TileTaskRef<RasterCompletion<B>> =
            Arc::new(DidFinishRunningAllTilesTask::new(
                self.task_runner_.clone(),
                pending_raster_queries,
                raster_buffer_provider,
                tiles_that_need_to_be_rasterized.len(),
                all_done_cb,
                context.WorkerDiagnostics(),
            ));
        let completion_tasks = CompletionTasks {
            required_for_activation: &required_for_activation_done_task,
            required_for_draw: &required_for_draw_done_task,
            all: &all_done_task,
        };
        let mut raster_candidates = Vec::with_capacity(tiles_that_need_to_be_rasterized.len());
        for prioritized_tile in tiles_that_need_to_be_rasterized {
            let tile = context.TileById(prioritized_tile.tile_id().expect("source non-null tile"));
            debug_assert!(tile.draw_info().requires_resource());
            debug_assert!(!tile.draw_info().has_resource());
            debug_assert!(tile.HasRasterTask());
            let task = tile.raster_task().unwrap().clone();
            task.task()
                .set_frame_number(i64::from(tile.source_frame_number()));
            debug_assert!(!task.tile_task().HasCompleted());
            raster_candidates.push(RasterTaskGraphCandidate {
                task,
                membership: RasterTaskGraphMembership {
                    required_for_activation: tile.required_for_activation(),
                    required_for_draw: tile.required_for_draw(),
                    needed_now: prioritized_tile.priority().priority_bin == PriorityBin::NOW,
                },
            });
        }
        let raster_batch = InsertRasterTaskBatchWithCompletionEdges(
            &mut self.graph_,
            raster_candidates,
            kTileTaskPriorityBase as usize,
            completion_tasks,
            context.GraphTrace(),
        );
        let mut completion_counts = raster_batch.completion_counts;
        let tiles_to_process_for_images = &work_to_schedule.tiles_to_process_for_images;
        let mut new_locked_images = Vec::new();
        for prioritized_tile in tiles_to_process_for_images {
            let content_color_usage =
                GetContentColorUsageForPrioritizedTile(prioritized_tile, context);
            let target_color_params =
                context.GetTargetColorParams(self.client_, content_color_usage);
            let mut sync_decoded_images = Vec::new();
            let mut checkered_images = Vec::new();
            Self::PartitionImagesForCheckering(
                self.client_,
                self.checker_image_tracker_,
                prioritized_tile,
                &target_color_params,
                &mut sync_decoded_images,
                &mut checkered_images,
                None,
                None,
                context,
            );
            new_locked_images.extend(sync_decoded_images);
            for image in checkered_images {
                work_to_schedule
                    .checker_image_decode_queue
                    .push(DecodeQueueItem {
                        image,
                        decode_type: DecodeType::kPreDecode,
                    });
            }
        }
        new_locked_images.extend_from_slice(&work_to_schedule.extra_prepaint_images);
        let tracing_info = ImageTracingInfo {
            prepare_tiles_id: self.prepare_tiles_count_,
            priority_bin: PriorityBin::SOON,
        };
        let mut new_locked_image_tasks = context
            .ImageController(self.image_controller_)
            .SetPredecodeImages(&new_locked_images, tracing_info);
        context
            .DecodedImageTracker(self.decoded_image_tracker_)
            .OnImagesUsedInDraw(&new_locked_images);
        work_to_schedule.extra_prepaint_images.clear();
        let decode_batch = InsertStandaloneDecodeTaskBatchWithCompletionEdge(
            &mut self.graph_,
            new_locked_image_tasks.iter().cloned(),
            raster_batch.next_priority,
            &all_done_task,
            context.GraphTrace(),
        );
        completion_counts.all += decode_batch.inserted_tasks;
        std::mem::swap(&mut self.locked_image_tasks_, &mut new_locked_image_tasks);
        context
            .ResourcePoolMutable(self.resource_pool_.expect("source resource_pool_"))
            .ReduceResourceUsage();
        context
            .ImageController(self.image_controller_)
            .ReduceMemoryUsage();
        FinalizeCompletionGraph(
            &mut self.graph_,
            completion_tasks,
            completion_counts,
            context.GraphTrace(),
        );
        let services = context.TaskManagerServices();
        self.tile_task_manager_
            .as_ref()
            .expect("source tile_task_manager_")
            .ScheduleTasks(&mut self.graph_, services.as_ref());
        context
            .CheckerImageTracker(self.checker_image_tracker_)
            .ScheduleImageDecodeQueue(work_to_schedule.checker_image_decode_queue);
        self.graph_.Reset();
        self.did_check_for_completed_tasks_since_last_schedule_tasks_ = false;
        context.TraceInstantOnTrack(
            "ScheduledTasksState",
            GetTracingTrack(self.identity_),
            || Some(self.ScheduledTasksStateAsValue()),
        );
        // Native old locked-image tasks remain owned until after scheduling.
        drop(new_locked_image_tasks);
    }
}
impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:355
    // Source: cc/tiles/tile_manager.cc:912-1174
    fn AssignGpuMemoryToTiles(
        &mut self,
        context: &mut impl TileManagerRuntime<B>,
    ) -> PrioritizedWorkToSchedule<B> {
        context.TraceBeginDefault("TileManager::AssignGpuMemoryToTiles");
        debug_assert!(self.resource_pool_.is_some());
        debug_assert!(self.tile_task_manager_.is_some());

        let hard_memory_limit = MemoryUsage::new(
            self.global_state_.hard_memory_limit_in_bytes,
            self.global_state_.num_resources_limit,
        );
        let soft_memory_limit = MemoryUsage::new(
            self.global_state_.soft_memory_limit_in_bytes,
            self.global_state_.num_resources_limit,
        );
        let initial_memory_usage = {
            let pool = context.ResourcePool(self.resource_pool_.unwrap());
            MemoryUsage::new(pool.memory_usage_bytes(), pool.resource_count())
        };
        let raster_priority_queue = context.BuildRasterQueue(
            self.client_,
            self.global_state_.tree_priority,
            super::raster_tile_priority_queue::Type::ALL,
        );
        let config = TileAssignmentConfig {
            memory_limit_policy: self.global_state_.memory_limit_policy,
            hard_memory_limit,
            soft_memory_limit,
            initial_memory_usage,
            raster_task_limit: self.scheduled_raster_task_limit_,
            has_checker_imaged_tiles: self.num_of_tiles_with_checker_images_ > 0,
        };
        let mut adapter = TileAssignmentAdapter {
            manager: self,
            context,
            raster_priority_queue: Some(raster_priority_queue),
            eviction_priority_queue: None,
            work: PrioritizedWorkToSchedule::default(),
        };
        let outcome = AssignTileWork(&mut adapter, config);
        let work_to_schedule = std::mem::take(&mut adapter.work);
        let raster_priority_queue = adapter
            .raster_priority_queue
            .take()
            .expect("assignment raster queue");
        let eviction_priority_queue = adapter.eviction_priority_queue.take();
        drop(adapter);

        self.all_tiles_that_need_to_be_rasterized_are_scheduled_ =
            outcome.all_tiles_scheduled;
        self.did_oom_on_last_assign_ = !outcome.had_enough_memory_for_needed_tiles;
        if context.ShouldRecordSubsampledMetric(self.metrics_sampling_rate_) {
            if !self.running_on_renderer_process_ {
                context.HistogramBoolean(
                    "Compositing.TileManager.EnoughMemory.Browser",
                    outcome.had_enough_memory_for_needed_tiles,
                );
                if outcome.had_enough_memory_for_needed_tiles {
                    context.HistogramMemoryMB(
                        "Compositing.TileManager.MemoryUsageWhenEnoughMemory.Browser",
                        outcome.final_memory_usage.memory_bytes() / (1024 * 1024),
                    );
                }
                if self.did_oom_on_last_assign_ {
                    context.HistogramMemoryMB(
                        "Compositing.TileManager.LimitWhenNotEnoughMemory.Browser",
                        hard_memory_limit.memory_bytes() / (1024 * 1024),
                    );
                }
            } else {
                context.HistogramBoolean(
                    "Compositing.TileManager.EnoughMemory.Renderer",
                    outcome.had_enough_memory_for_needed_tiles,
                );
                if outcome.had_enough_memory_for_needed_tiles {
                    context.HistogramMemoryMB(
                        "Compositing.TileManager.MemoryUsageWhenEnoughMemory.Renderer",
                        outcome.final_memory_usage.memory_bytes() / (1024 * 1024),
                    );
                }
                if self.did_oom_on_last_assign_ {
                    context.HistogramMemoryMB(
                        "Compositing.TileManager.LimitWhenNotEnoughMemory.Renderer",
                        hard_memory_limit.memory_bytes() / (1024 * 1024),
                    );
                }
            }
        }
        self.memory_stats_from_last_assign_.total_budget_in_bytes =
            self.global_state_.hard_memory_limit_in_bytes;
        self.memory_stats_from_last_assign_.total_bytes_used =
            outcome.final_memory_usage.memory_bytes();
        debug_assert!(self.memory_stats_from_last_assign_.total_bytes_used >= 0);
        self.memory_stats_from_last_assign_.had_enough_memory =
            outcome.had_enough_memory_for_needed_tiles;

        context.TraceEndWithState(|| {
            let mut trace_args = TracedValueJSON::default();
            trace_args.SetBoolean(
                "all_tiles_that_need_to_be_rasterized_are_scheduled",
                outcome.all_tiles_scheduled,
            );
            trace_args.SetBoolean(
                "had_enough_memory_to_schedule_tiles_needed_now",
                outcome.had_enough_memory_for_needed_tiles,
            );
            trace_args
        });
        context
            .ImageController(self.image_controller_)
            .RecordCacheStats();

        // Native queue owners stay alive through the final metrics/cache calls.
        // They release in reverse declaration order, matching the source locals.
        drop(eviction_priority_queue);
        drop(raster_priority_queue);
        work_to_schedule
    }
}
impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:113
    // Source: cc/tiles/tile_manager.cc:685-750
    pub(crate) fn PrepareTiles(
        &mut self,
        state: &GlobalStateThatImpactsTilePriority,
        context: &mut impl TileManagerRuntime<B>,
    ) -> bool {
        let _trace = context.TraceScope("TileManager::PrepareTiles");
        self.prepare_tiles_count_ += 1;
        self.last_active_time_ = self.NowWithOverride(context);
        self.ScheduleReduceTileMemoryWhenIdle(TimeDelta::default(), context);
        self.ScheduleTrimPrepaintTiles(context);
        let _prepare_trace = context.TraceScopeWithArgs("TileManager::PrepareTiles", || {
            vec![
                crate::adapter::tile_manager::TileManagerTraceArgument::Unsigned(
                    "prepare_tiles_id",
                    self.prepare_tiles_count_,
                ),
            ]
        });
        if self.tile_task_manager_.is_none() {
            context.TraceInstant("PrepareTiles aborted", || None);
            return false;
        }
        self.signals_ = Signals::default();
        if self.global_state_.viewport_size != state.viewport_size {
            context
                .ResourcePoolMutable(self.resource_pool_.expect("source resource_pool_"))
                .NotifyOfViewportSizeChange(self.global_state_.viewport_size, state.viewport_size);
        }
        self.global_state_ = state.clone();
        context
            .CheckerImageTracker(self.checker_image_tracker_)
            .SetNoDecodesAllowed();
        if !self.did_check_for_completed_tasks_since_last_schedule_tasks_ {
            self.CheckForCompletedTasks(context);
            self.did_check_for_completed_tasks_since_last_schedule_tasks_ = true;
        }
        if !self.ShouldRasterOccludedTiles() {
            self.FreeResourcesForOccludedTiles(context);
        }
        let prioritized_work = self.AssignGpuMemoryToTiles(context);
        context.SetIsLikelyToRequireADraw(
            self.client_,
            !prioritized_work.tiles_to_raster.is_empty()
                && context
                    .TileById(
                        prioritized_work
                            .tiles_to_raster
                            .first()
                            .unwrap()
                            .tile_id()
                            .expect("source non-null tile"),
                    )
                    .required_for_draw(),
        );
        let required_for_draw_tile_state_changed =
            prioritized_work.required_for_draw_tile_state_changed;
        self.ScheduleTasks(prioritized_work, context);
        if required_for_draw_tile_state_changed {
            context.SetNeedsRedraw(self.client_, false, true);
        }
        context.TraceInstant("DidPrepareTiles", || Some(self.BasicStateAsValue()));
        true
    }
    // Source: cc/tiles/tile_manager.h:340
    // Source: cc/tiles/tile_manager.cc:1942-2044
    fn CheckIfMoreTilesNeedToBePrepared(&mut self, context: &mut impl TileManagerRuntime<B>) {
        self.CheckForCompletedTasks(context);
        self.did_check_for_completed_tasks_since_last_schedule_tasks_ = true;
        let work_to_schedule = self.AssignGpuMemoryToTiles(context);
        context.SetIsLikelyToRequireADraw(
            self.client_,
            !work_to_schedule.tiles_to_raster.is_empty()
                && context
                    .TileById(
                        work_to_schedule
                            .tiles_to_raster
                            .first()
                            .unwrap()
                            .tile_id()
                            .expect("source non-null tile"),
                    )
                    .required_for_draw(),
        );
        let required_for_draw_tile_state_changed =
            work_to_schedule.required_for_draw_tile_state_changed;
        if !work_to_schedule.tiles_to_raster.is_empty() {
            self.ScheduleTasks(work_to_schedule, context);
            if required_for_draw_tile_state_changed {
                context.SetNeedsRedraw(self.client_, false, true);
            }
            return;
        }
        if required_for_draw_tile_state_changed {
            context.SetNeedsRedraw(self.client_, false, true);
        }
        if self.global_state_.tree_priority
            != super::tile_priority::TreePriority::SMOOTHNESS_TAKES_PRIORITY
        {
            context
                .ImageController(self.image_controller_)
                .SetPredecodeImages(
                    &[],
                    ImageTracingInfo {
                        prepare_tiles_id: 0,
                        priority_bin: PriorityBin::NOW,
                    },
                );
            self.locked_image_tasks_.clear();
        }
        context
            .ResourcePoolMutable(self.resource_pool_.expect("source resource_pool_"))
            .ReduceResourceUsage();
        context
            .ImageController(self.image_controller_)
            .ReduceMemoryUsage();
        assert!(self.tile_task_manager_.is_some());
        self.signals_.activate_tile_tasks_completed = true;
        self.signals_.draw_tile_tasks_completed = true;
        self.signals_.all_tile_tasks_completed = true;
        self.signals_check_notifier_
            .as_ref()
            .expect("source live notifier")
            .Schedule();
        match DecideNoRasterWorkAction(
            self.global_state_.tree_priority,
            self.global_state_.memory_limit_policy,
            context.FeatureEnabled(TileManagerFeature::TileOOMFreezeMitigation),
            self.did_oom_on_last_assign_,
            self.all_tiles_that_need_to_be_rasterized_are_scheduled_,
        ) {
            NoRasterWorkAction::WaitForRequiredTiles {
                schedule_another_prepare,
            } => {
                if schedule_another_prepare {
                    self.more_tiles_need_prepare_check_notifier_
                        .as_ref()
                        .expect("source live notifier")
                        .Schedule();
                }
                return;
            }
            NoRasterWorkAction::MarkRequiredTilesOutOfMemory => {}
        }
        let required_for_activation = context.BuildRasterQueue(
            self.client_,
            self.global_state_.tree_priority,
            super::raster_tile_priority_queue::Type::REQUIRED_FOR_ACTIVATION,
        );
        self.MarkTilesOutOfMemory(required_for_activation, context);
        let required_for_draw = context.BuildRasterQueue(
            self.client_,
            self.global_state_.tree_priority,
            super::raster_tile_priority_queue::Type::REQUIRED_FOR_DRAW,
        );
        self.MarkTilesOutOfMemory(required_for_draw, context);
        assert!(self.tile_task_manager_.is_some());
        debug_assert!(self.IsReadyToActivate(context));
        debug_assert!(self.IsReadyToDraw(context));
    }
    // Source: cc/tiles/tile_manager.h:191-193
    pub(crate) fn CheckIfMoreTilesNeedToBePreparedForTesting(
        &mut self,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        self.CheckIfMoreTilesNeedToBePrepared(context);
    }
}

// Source: cc/tiles/tile_manager.cc:59-61
fn GetTracingTrack(
    tile_manager: TileManagerId,
) -> crate::adapter::tile_manager::TileManagerTraceTrack {
    crate::adapter::tile_manager::TileManagerTraceTrack {
        name: "cc::TileManager",
        owner: tile_manager,
    }
}
impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:218-219
    // Source: cc/tiles/tile_manager.cc:2201-2206
    pub(crate) fn ActivationStateAsValue(
        &self,
        context: &mut impl TileManagerRuntime<B>,
    ) -> TracedValueJSON {
        let mut state = TracedValueJSON::default();
        self.ActivationStateAsValueInto(&mut state, context);
        state
    }
    // Source: cc/tiles/tile_manager.h:221
    // Source: cc/tiles/tile_manager.cc:2208-2264
    pub(crate) fn ActivationStateAsValueInto(
        &self,
        state: &mut impl PropertyTrace,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        use super::tile_priority::TreePriorityToString;
        state.SetString(
            "tree_priority",
            TreePriorityToString(self.global_state_.tree_priority),
        );
        state.SetInteger(
            "soft_memory_limit",
            self.global_state_.soft_memory_limit_in_bytes as i32,
        );
        state.SetInteger(
            "hard_memory_limit",
            self.global_state_.hard_memory_limit_in_bytes as i32,
        );
        state.SetInteger(
            "pending_required_for_activation_callback_id",
            self.pending_required_for_activation_callback_id_ as i32,
        );
        state.SetInteger(
            "current_memory_usage",
            context
                .ResourcePool(self.resource_pool_.expect("source resource_pool_"))
                .memory_usage_bytes() as i32,
        );
        state.SetInteger(
            "current_resource_usage",
            context
                .ResourcePool(self.resource_pool_.unwrap())
                .resource_count() as i32,
        );
        let tile_as_value = |prioritized_tile: &super::prioritized_tile::PrioritizedTile,
                             value: &mut _,
                             context: &_| {
            let tile = prioritized_tile
                .tile::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
                .expect("source non-null tile");
            let priority = *prioritized_tile.priority();
            TileActivationValue(value, tile, priority);
        };
        // The source lambda's body is in the same canonical helper below solely
        // to name the generic trace/context types; no state is materialized.
        let mut raster_priority_queue = context.BuildRasterQueue(
            self.client_,
            self.global_state_.tree_priority,
            super::raster_tile_priority_queue::Type::ALL,
        );
        state.BeginArray("raster_tiles");
        while !raster_priority_queue.IsEmpty() {
            state.BeginDictionaryUnnamed();
            tile_as_value(raster_priority_queue.Top(), state, context);
            state.EndDictionary();
            raster_priority_queue.Pop(context);
        }
        state.EndArray();
        let mut required_priority_queue = context.BuildRasterQueue(
            self.client_,
            self.global_state_.tree_priority,
            super::raster_tile_priority_queue::Type::REQUIRED_FOR_ACTIVATION,
        );
        state.BeginArray("activation_tiles");
        while !required_priority_queue.IsEmpty() {
            state.BeginDictionaryUnnamed();
            tile_as_value(required_priority_queue.Top(), state, context);
            state.EndDictionary();
            required_priority_queue.Pop(context);
        }
        state.EndArray();
    }
    // Source: cc/tiles/tile_manager.h:243-244
    // Source: cc/tiles/tile_manager.cc:2273-2321
    pub(crate) fn OnMemoryDump<N>(
        &self,
        level: crate::adapter::tile_manager::MemoryDumpLevel,
        dump: &mut dyn crate::adapter::tile_manager::TileManagerMemoryDump<N>,
        context: &mut impl TileManagerRuntime<B>,
    ) -> bool {
        use super::tile_priority::{TileMemoryLimitPolicyToString, TilePriorityBinToString};
        if level != crate::adapter::tile_manager::MemoryDumpLevel::Detailed
            || self.resource_pool_.is_none()
        {
            return true;
        }
        let manager_path = format!(
            "cc/tile_manager_{}",
            context
                .ResourcePool(self.resource_pool_.unwrap())
                .tracing_id()
        );
        let node = dump.CreateAllocatorDump(&manager_path);
        dump.AddString(
            &node,
            "memory_policy",
            "",
            TileMemoryLimitPolicyToString(self.global_state_.memory_limit_policy),
        );
        dump.AddScalar(
            &node,
            "soft_memory_limit",
            "bytes",
            self.global_state_.soft_memory_limit_in_bytes as u64,
        );
        dump.AddScalar(
            &node,
            "hard_memory_limit",
            "bytes",
            self.global_state_.hard_memory_limit_in_bytes as u64,
        );
        dump.AddScalar(
            &node,
            "num_resources_limit",
            "count",
            self.global_state_.num_resources_limit as u64,
        );
        let mut eviction_priority_queue = context.BuildEvictionQueue(self.client_);
        let _tiles_to_evict = std::collections::BTreeSet::<TileId>::new();
        while !eviction_priority_queue.IsEmpty() {
            let prioritized_tile = eviction_priority_queue.Top(context);
            let tile = prioritized_tile
                .tile::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
                .expect("source non-null tile");
            let name = format!("{}/tile_{}", manager_path, tile.id() as u32);
            let tile_dump = dump.CreateAllocatorDump(&name);
            dump.AddString(
                &tile_dump,
                "priority",
                "",
                TilePriorityBinToString(prioritized_tile.priority().priority_bin),
            );
            // AddScalar's native uint64_t parameter truncates this float.
            dump.AddScalar(
                &tile_dump,
                "distance_to_visible",
                "px",
                prioritized_tile.priority().distance_to_visible as u64,
            );
            dump.AddScalar(&tile_dump, "is_prepaint", "bool", tile.is_prepaint() as u64);
            dump.AddScalar(
                &tile_dump,
                "gpu_memory",
                "bytes",
                tile.GPUMemoryUsageInBytes() as u64,
            );
            let size = tile.desired_texture_size();
            dump.AddScalar(&tile_dump, "width", "px", size.width() as u64);
            dump.AddScalar(&tile_dump, "height", "px", size.height() as u64);
            dump.AddScalar(&tile_dump, "young", "bool", tile.used() as u64);
            eviction_priority_queue.Pop(context);
        }
        true
    }
    // Source: cc/tiles/tile_manager.h:160
    // Source: cc/tiles/tile_manager.cc:802-824
    pub(crate) fn InitializeTilesWithResourcesForTesting(
        &mut self,
        tiles: &[TileId],
        context: &mut impl TileManagerRuntime<B>,
    ) {
        for tile in tiles {
            let format = context.GetTileFormat(self.client_);
            let params = context.GetTargetColorParams(self.client_, B::SrgbContentColorUsage());
            let size = context.TileById(*tile).desired_texture_size();
            let resource = context
                .ResourcePoolMutable(self.resource_pool_.expect("source resource_pool_"))
                .AcquireResource(&size, format, params.color_space(), "");
            context.AcquireRasterBufferForResource(
                self.raster_buffer_provider_
                    .expect("source raster_buffer_provider_"),
                self.resource_pool_.expect("source resource_pool_"),
                &resource,
                0,
                0,
            );
            resource.WithBacking(|backing| {
                if let Some(backing) = backing {
                    backing.CreateSharedImageForTesting(context.TestSharedImageInterface());
                    backing.set_mailbox_sync_token(B::TestRasterSyncToken());
                }
            });
            let exported = context
                .ResourcePoolMutable(self.resource_pool_.unwrap())
                .PrepareForExport(&resource, B::TestResourceSource());
            debug_assert!(exported);
            let draw_info = context.TileByIdMutable(*tile).draw_info_mut();
            draw_info.SetResource(resource, false);
            draw_info.set_resource_ready_for_draw();
        }
    }
    // Source: cc/tiles/tile_manager.h:161
    // Source: cc/tiles/tile_manager.cc:826-832
    pub(crate) fn ReleaseTileResourcesForTesting(
        &mut self,
        tiles: &[TileId],
        context: &mut impl TileManagerRuntime<B>,
    ) {
        for tile in tiles {
            let (tile, pool) =
                context.TileAndPool(*tile, self.resource_pool_.expect("source resource_pool_"));
            FreeResourcesForTile(
                tile,
                pool,
                &mut self.num_of_tiles_with_checker_images_,
                &mut self.pending_gpu_work_tiles_,
            );
        }
    }
}
// Source: cc/tiles/tile_manager.cc:2224-2240
fn TileActivationValue<B: TileManagerTypes>(
    value: &mut impl PropertyTrace,
    tile: &Tile<B>,
    priority: TilePriority,
) {
    use super::tile_priority::{TilePriorityBinToString, TileResolutionToString};
    value.SetInteger("id", tile.id() as i32);
    value.SetString("content_rect", &tile.content_rect().ToString());
    value.SetDouble("contents_scale", f64::from(tile.contents_scale_key()));
    value.SetBoolean("is_ready_to_draw", tile.draw_info().IsReadyToDraw());
    value.SetString("resolution", TileResolutionToString(priority.resolution));
    value.SetString(
        "priority_bin",
        TilePriorityBinToString(priority.priority_bin),
    );
    value.SetDouble(
        "distance_to_visible",
        f64::from(priority.distance_to_visible),
    );
    value.SetBoolean("required_for_activation", tile.required_for_activation());
    value.SetBoolean("required_for_draw", tile.required_for_draw());
}

// Safe call-scoped projection of source Tile's manager pointer. It cannot
// escape the Create/Destroy operation and never retains a pool or owner route.
struct TileReleaseContext<
    'm,
    'c,
    B: TileManagerTypes,
    C: crate::adapter::tile_manager::TileReleaseRuntime<B>,
> {
    manager: &'m mut TileManager<B>,
    context: &'c mut C,
}
impl<B: TileManagerTypes, C: crate::adapter::tile_manager::TileReleaseRuntime<B>> TileTraceContext
    for TileReleaseContext<'_, '_, B, C>
{
    fn MakeDictIntoImplicitSnapshotWithCategory<V: PropertyTrace>(
        &self,
        category: &str,
        value: &mut V,
        name: &str,
        id: TileTraceId,
    ) {
        self.context
            .MakeDictIntoImplicitSnapshotWithCategory(category, value, name, id);
    }
    fn TraceTileDeleted(&self, category: &str, event: &str, id: TileTraceId, flow_scope: &str) {
        self.context
            .TraceTileDeleted(category, event, id, flow_scope);
    }
}
impl<B: TileManagerTypes, C: crate::adapter::tile_manager::TileReleaseRuntime<B>> TileIdAllocator
    for TileReleaseContext<'_, '_, B, C>
{
    fn identity(&self) -> TileManagerId {
        self.manager.identity_
    }
    fn GetUniqueTileId(&mut self) -> Id {
        self.manager.GetUniqueTileId()
    }
}
impl<B: TileManagerTypes, C: crate::adapter::tile_manager::TileReleaseRuntime<B>>
    crate::adapter::tile::TileManagerContext<B> for TileReleaseContext<'_, '_, B, C>
{
    fn Release(&mut self, tile: &mut Tile<B>) {
        self.manager.Release(tile, self.context);
    }
}
impl<B: TileManagerTypes> TileManager<B> {
    /// C++ stores a stable `Tile*` in this table. Rust resolves that pointer
    /// through its owning tiling, so a twin-tiling transfer must update the
    /// locator while preserving the native tile id and allocation.
    pub(crate) fn RebindTile(&mut self, old: TileId, new: TileId) {
        assert_eq!(old.manager, self.identity_);
        assert_eq!(new.manager, self.identity_);
        assert_eq!(old.id, new.id, "twin transfer preserves native Tile id");
        let resident = self
            .tiles_
            .get_mut(&old.id)
            .expect("resident TileManager entry");
        assert_eq!(*resident, old, "tile locator changes exactly once");
        *resident = new;
    }
    // Source call chain: cc/tiles/tile.cc:40-48 -> TileManager::Release.
    // The runtime lends these exact owner fields and the actual live Tile.
    pub(crate) fn DestroyTile(
        &mut self,
        tile: &mut Tile<B>,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        self.DestroyTileWithRelease(tile, &mut TileFullRuntimeRelease(context));
    }
    /// Same canonical source destructor with only the capabilities it consumes.
    pub(crate) fn DestroyTileWithRelease(
        &mut self,
        tile: &mut Tile<B>,
        context: &mut impl crate::adapter::tile_manager::TileReleaseRuntime<B>,
    ) {
        tile.Destroy(&mut TileReleaseContext {
            manager: self,
            context,
        });
    }
    /// Deliver an already-posted typed command on this compositor's sequence.
    /// The token is only the source WeakPtrFactory's non-owning generation.
    /// No command endpoint can invoke a manager or retain its state.
    pub(crate) fn OnCommand(
        &mut self,
        owner: TileManagerId,
        lifetime: std::sync::Weak<()>,
        command: TileManagerCommand,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        if owner != self.identity_ || lifetime.strong_count() == 0 || self.destroyed_ {
            return;
        }
        let expected = match command {
            TileManagerCommand::CheckIfMoreTilesNeedToBePrepared
            | TileManagerCommand::CheckForCompletedTasksAndIssueSignals => &self.owner_lifetime_,
            TileManagerCommand::ReduceTileMemoryWhenIdle
            | TileManagerCommand::TrimPrepaintTiles
            | TileManagerCommand::CheckPendingGpuWorkAndIssueSignals => {
                &self.ready_to_draw_callback_weak_ptr_factory_
            }
            TileManagerCommand::DidFinishRunningTileTasksRequiredForActivation
            | TileManagerCommand::DidFinishRunningTileTasksRequiredForDraw => {
                &self.task_set_finished_weak_ptr_factory_
            }
            TileManagerCommand::CheckRasterFinishedQueries => {
                let Some(lifetime) = self.check_pending_tile_queries_callback_.as_ref() else {
                    return;
                };
                lifetime
            }
        };
        if !std::sync::Weak::ptr_eq(&lifetime, &Arc::downgrade(expected)) {
            return;
        }
        match command {
            TileManagerCommand::CheckIfMoreTilesNeedToBePrepared => {
                self.CheckIfMoreTilesNeedToBePrepared(context)
            }
            TileManagerCommand::CheckForCompletedTasksAndIssueSignals => {
                self.CheckForCompletedTasksAndIssueSignals(context)
            }
            TileManagerCommand::ReduceTileMemoryWhenIdle => self.ReduceTileMemoryWhenIdle(context),
            TileManagerCommand::TrimPrepaintTiles => self.TrimPrepaintTiles(context),
            TileManagerCommand::CheckRasterFinishedQueries => {
                self.CheckRasterFinishedQueries(context)
            }
            TileManagerCommand::CheckPendingGpuWorkAndIssueSignals => {
                self.CheckPendingGpuWorkAndIssueSignals(context)
            }
            TileManagerCommand::DidFinishRunningTileTasksRequiredForActivation => {
                self.DidFinishRunningTileTasksRequiredForActivation(context)
            }
            TileManagerCommand::DidFinishRunningTileTasksRequiredForDraw => {
                self.DidFinishRunningTileTasksRequiredForDraw(context)
            }
        }
    }
    pub(crate) fn OnAllTileTasksFinished(
        &mut self,
        owner: TileManagerId,
        lifetime: std::sync::Weak<()>,
        start_time: TimeTicks,
        has_pending_queries: bool,
        context: &mut impl TileManagerRuntime<B>,
    ) {
        if owner != self.identity_
            || lifetime.strong_count() == 0
            || self.destroyed_
            || !std::sync::Weak::ptr_eq(
                &lifetime,
                &Arc::downgrade(&self.task_set_finished_weak_ptr_factory_),
            )
        {
            return;
        }
        self.DidFinishRunningAllTileTasks(start_time, has_pending_queries, context);
    }
    pub(crate) fn OnExternalRasterDependencyCompleted(
        &mut self,
        owner: TileManagerId,
        dependent: TileTaskRef<RasterCompletion<B>>,
    ) {
        assert_eq!(owner, self.identity_);
        self.ExternalDependencyCompletedForRasterTask(dependent);
    }
}
// Source: cc/tiles/tile_manager.h:246-248,253,386,390,477-479
// Source: cc/tiles/tile_manager.cc:56-57,352,2405
// Visibility, module/namespace closure, friendship and deleted copy operations
// map to Rust ownership and module visibility rather than executable stubs.

// Source: cc/tiles/tile_manager.cc:603-613,1190-1202. Borrow-only compatibility
// for existing complete runtime calls; no release branch/accounting is copied.
struct TileFullRuntimeRelease<'a, C>(&'a mut C);
impl<B: TileManagerTypes, C: TileManagerRuntime<B>>
    crate::adapter::tile_manager::TileReleaseRuntime<B> for TileFullRuntimeRelease<'_, C>
{
    fn ResourcePoolMutable(&mut self, id: ResourcePoolId) -> &mut ResourcePool<B> {
        self.0.ResourcePoolMutable(id)
    }
    fn NotifyTileStateChanged(
        &mut self,
        client: TileManagerClientId,
        tile: &Tile<B>,
        damage: bool,
        redraw: bool,
    ) {
        self.0.NotifyTileStateChanged(client, tile, damage, redraw)
    }
}
impl<C: TileTraceContext> TileTraceContext for TileFullRuntimeRelease<'_, C> {
    fn MakeDictIntoImplicitSnapshotWithCategory<V: PropertyTrace>(
        &self,
        category: &str,
        value: &mut V,
        name: &str,
        id: TileTraceId,
    ) {
        self.0
            .MakeDictIntoImplicitSnapshotWithCategory(category, value, name, id)
    }
    fn TraceTileDeleted(&self, category: &str, event: &str, id: TileTraceId, flow: &str) {
        self.0.TraceTileDeleted(category, event, id, flow)
    }
}

impl<B: TileManagerTypes> TileManager<B> {
    // Source: cc/tiles/tile_manager.h:226-228. This native external tracker
    // identity is immutable after construction and never denotes the manager.
    pub(crate) fn checker_image_tracker_id(&self) -> CheckerImageTrackerId {
        self.checker_image_tracker_
    }
    pub(crate) fn identity(&self) -> TileManagerId {
        self.identity_
    }
}
