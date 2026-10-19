use crate::adapter::layer_tree_impl::PropertyTreesFor;
// Translation status: in_progress
// Source: cc/trees/layer_tree_host_impl.h:1-1417
// Source: cc/trees/layer_tree_host_impl.cc:1-6496
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use crate::{
    adapter::{
        geometry::{Rect, Size},
        layer_tree_host_impl::*,
        property_tree::PropertyTreeTransform,
        time::TimeDelta,
    },
    cc::{
        base::math_util::MathUtil,
        trees::{
            mobile_optimized_viewport_util, property_ids::kInvalidPropertyNodeId,
            property_tree::TransformTree, raster_capabilities::RasterCapabilities,
        },
    },
};
use std::collections::{BTreeMap, HashMap};

// Source: cc/trees/layer_tree_host_impl.h:130-135
#[derive(Clone, Copy, Debug, Default)]
pub struct UIResourceChange {
    pub resource_created: bool,
    pub resource_deleted: bool,
}
pub type UIResourceChangeMap<Id> = HashMap<Id, UIResourceChange>;
// Source: cc/trees/layer_tree_host_impl.h:152-177
// Source: cc/trees/layer_tree_host_impl.cc:578-583
pub struct UIResourceData<H: HostImplResourceTypes> {
    pub opaque: bool,
    pub shared_mapping: H::SharedMemoryMapping,
    pub shared_image: Option<H::SharedImage>,
    pub resource_id_for_export: H::ResourceId,
    pub size: Size,
}
impl<H: HostImplResourceTypes> Default for UIResourceData<H> {
    fn default() -> Self {
        Self {
            opaque: false,
            shared_mapping: H::SharedMemoryMapping::default(),
            shared_image: None,
            resource_id_for_export: H::ResourceId::default(),
            size: Size::default(),
        }
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:166-187
const kAssumeOverlapThreshold: usize = 100;
const kHasInputResetDelay: TimeDelta = TimeDelta::Milliseconds(250);
const kScrollDeltaThreshold: f32 = 25.;
const kIntersectionThresholdAreaPercentage: f32 = 5. / 100.;
const kContainsSrgbCacheSize: usize = 3;

// Source: cc/trees/layer_tree_host_impl.cc:188-202
fn AccumulateInvalidatedArea<L: HostImplLayer>(
    layer: &L,
    trees: &PropertyTreesFor<L::Types>,
    total: &mut CheckedU32,
) {
    let region = layer.GetInvalidationRegionForDebugging();
    if region.IsEmpty() || !layer.draws_content() {
        return;
    }
    for rect in &region {
        total.AddCheckedRectArea(MathUtil::MapEnclosingClippedRect(
            &layer.ScreenSpaceTransform(trees),
            rect,
        ));
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:204-211
fn IsMobileOptimized<T: HostImplTreeView>(tree: &T) -> bool {
    mobile_optimized_viewport_util::IsMobileOptimized(
        tree.min_page_scale_factor(),
        tree.max_page_scale_factor(),
        tree.current_page_scale_factor(),
        tree.ScrollableViewportSize(),
        tree.ScrollableSize(),
        tree.viewport_mobile_optimized(),
    )
}
// Source: cc/trees/layer_tree_host_impl.cc:213-225
fn DidVisibilityChange(services: &(impl HostImplDebugServices + ?Sized), id: u64, visible: bool) {
    if visible {
        services.BeginVisibilityTrace(id);
        return;
    }
    services.EndVisibilityTrace(id);
}
// Source: cc/trees/layer_tree_host_impl.cc:227-238
fn PopulateMetadataContentColorUsage<T: HostImplTreeView>(
    tree: &T,
    frame: &impl HostImplFrameLayerList,
    metadata: &mut impl HostImplFrameMetadata,
) {
    metadata.set_content_color_usage(ContentColorUsage::kSRGB);
    for id in frame.will_draw_layers() {
        if let Some(layer) = tree.LayerById(*id) {
            metadata.set_content_color_usage(std::cmp::max(
                metadata.content_color_usage(),
                layer.GetContentColorUsage(),
            ));
        }
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:243-252
fn VerboseLogEnabled(services: &(impl HostImplDebugServices + ?Sized)) -> bool {
    if !services.VlogIsOn(3) {
        return false;
    }
    if services.VlogIsOn(4) {
        return true;
    }
    services
        .GetClientNameForMetrics()
        .is_some_and(|name| name == "Renderer")
}
// Source: cc/trees/layer_tree_host_impl.cc:254-257
fn ClientNameForVerboseLog(services: &(impl HostImplDebugServices + ?Sized)) -> &str {
    services
        .GetClientNameForMetrics()
        .unwrap_or("<unknown client>")
}
// Source: cc/trees/layer_tree_host_impl.cc:259-281
fn ShouldDumpCompositorFrame(
    services: &(impl HostImplDebugServices + ?Sized),
    begin: &mut u32,
    end: &mut u32,
) -> bool {
    let Some(param) = services.DumpCompositorFrameSwitch() else {
        *begin = 0;
        *end = 0;
        return false;
    };
    *begin = 1;
    *end = 10;
    if !param.is_empty() {
        let tokens: Vec<&str> = param
            .split(',')
            .map(|s| {
                s.trim_matches(|c: char| matches!(c, ' ' | '\t' | '\r' | '\n' | '\u{b}' | '\u{c}'))
            })
            .collect();
        if tokens.len() == 2 {
            if let (Ok(a), Ok(b)) = (tokens[0].parse::<u32>(), tokens[1].parse::<u32>()) {
                if a < b {
                    *begin = a;
                    *end = b;
                }
            }
        }
    }
    true
}
// Source: cc/trees/layer_tree_host_impl.cc:283-299
fn DoDumpCompositorFrame(
    services: &(impl HostImplDebugServices + ?Sized),
    data: &str,
    client: bool,
    viz: bool,
    token: u32,
) {
    let prefix;
    if client {
        prefix = "compositor_frame_client_";
    } else if viz {
        prefix = "compositor_frame_viz_";
    } else {
        return;
    }
    let filename = format!("{prefix}{token}.txt");
    let result = services.WriteFileInCurrentDirectory(&filename, data);
    assert!(result);
}
// Source: cc/trees/layer_tree_host_impl.cc:301-304
struct HeuristicFixedOrStickyOccluder {
    rect: Rect,
    transform_tree_index: i32,
}
// Source: cc/trees/layer_tree_host_impl.cc:309-335
fn GetFirstStickyAncestorOrViewportScrollNode<T: PropertyTreeTransform>(
    id: i32,
    tree: &TransformTree<T>,
    inner: i32,
    outer: i32,
    cache: &mut BTreeMap<i32, i32>,
) -> i32 {
    if id == kInvalidPropertyNodeId {
        return kInvalidPropertyNodeId;
    }
    if let Some(result) = cache.get(&id) {
        return *result;
    }
    let node = tree.Node(id);
    if node.sticky_position_constraint_id != kInvalidPropertyNodeId
        || id == inner
        || id == outer
        || node.parent_id == kInvalidPropertyNodeId
    {
        cache.insert(id, id);
        return id;
    }
    let root =
        GetFirstStickyAncestorOrViewportScrollNode(node.parent_id, tree, inner, outer, cache);
    cache.insert(id, root);
    root
}
// Source: cc/trees/layer_tree_host_impl.cc:351-362
fn IsFixedOrStickyOccluderTransform<T: PropertyTreeTransform>(
    id: i32,
    tree: &TransformTree<T>,
    inner: i32,
    outer: i32,
    cache: &mut BTreeMap<i32, i32>,
) -> bool {
    let nearest = GetFirstStickyAncestorOrViewportScrollNode(id, tree, inner, outer, cache);
    nearest != kInvalidPropertyNodeId && nearest != outer
}
// Source: cc/trees/layer_tree_host_impl.cc:366-411
fn CalculateVisibleBounds<T: PropertyTreeTransform, L: HostImplLayer<Transform = T>>(
    layer: &L,
    trees: &PropertyTreesFor<L::Types>,
    rect: &impl HostImplTrackedElementRect,
    occluders: &[HeuristicFixedOrStickyOccluder],
    tree: &TransformTree<T>,
    inner: i32,
    outer: i32,
    cache: &mut BTreeMap<i32, i32>,
) -> Rect {
    let mut visible = layer.VisibleLayerRect(trees);
    visible.Intersect(rect.visible_bounds());
    let mut screen =
        MathUtil::ProjectEnclosingClippedRect(&layer.ScreenSpaceTransform(trees), visible);
    if rect.should_exclude_fixed_and_sticky_occlusions() {
        let id = layer.transform_tree_index();
        let nearest = GetFirstStickyAncestorOrViewportScrollNode(id, tree, inner, outer, cache);
        for occluder in occluders {
            let occluder_nearest = GetFirstStickyAncestorOrViewportScrollNode(
                occluder.transform_tree_index,
                tree,
                inner,
                outer,
                cache,
            );
            if occluder_nearest != kInvalidPropertyNodeId && occluder_nearest == nearest {
                continue;
            }
            screen.Subtract(occluder.rect);
            if screen.IsEmpty() {
                break;
            }
        }
    }
    screen
}

// Source: cc/trees/layer_tree_host_impl.cc:426-470
// The native non-owning pointer can only alias this source owning field.
// Rust represents that alias choice without an unsafe self-referential pointer.
enum ImageDecodeCacheAlias {
    Owned,
}
struct ImageDecodeCacheHolder<F: HostImplImageDecodeFactory> {
    image_decode_cache_: Option<Box<F::ImageDecodeCache>>,
    image_decode_cache_ptr_: Option<ImageDecodeCacheAlias>,
}
impl<F: HostImplImageDecodeFactory> ImageDecodeCacheHolder<F> {
    fn new(
        factory: &F,
        caps: &RasterCapabilities,
        provider: F::RasterContextProvider,
        budget: usize,
        filter: Option<&F::DarkModeFilter>,
    ) -> Self {
        let cache;
        if caps.use_gpu_rasterization {
            let color = factory.ToClosestSkColorType(caps.tile_format);
            cache = factory.CreateGpuImageDecodeCache(
                provider,
                color,
                budget,
                caps.max_texture_size,
                filter,
            );
        } else {
            cache = factory.CreateSoftwareImageDecodeCache(
                factory.ToClosestSkColorType(caps.tile_format),
                budget,
            );
        }
        let image_decode_cache_ = Some(cache);
        let mut image_decode_cache_ptr_ = None;
        if image_decode_cache_.is_some() {
            image_decode_cache_ptr_ = Some(ImageDecodeCacheAlias::Owned);
        }
        Self {
            image_decode_cache_,
            image_decode_cache_ptr_,
        }
    }
    fn SetShouldAggressivelyFreeResources(&mut self, aggressive: bool) {
        if let Some(cache) = self.image_decode_cache_.as_mut() {
            cache.SetShouldAggressivelyFreeResources(aggressive);
        }
    }
    fn image_decode_cache(&self) -> &F::ImageDecodeCache {
        debug_assert!(self.image_decode_cache_ptr_.is_some());
        match self.image_decode_cache_ptr_.as_ref().unwrap() {
            ImageDecodeCacheAlias::Owned => self.image_decode_cache_.as_deref().unwrap(),
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:457-460.
    // Const C++ pointer getter permits mutation of the same owned allocation;
    // Rust exposes a separate mutable borrow for callers owning &mut self.
    fn image_decode_cache_mut(&mut self) -> &mut F::ImageDecodeCache {
        debug_assert!(self.image_decode_cache_ptr_.is_some());
        match self.image_decode_cache_ptr_.as_ref().unwrap() {
            ImageDecodeCacheAlias::Owned => self.image_decode_cache_.as_deref_mut().unwrap(),
        }
    }
}

use crate::adapter::{
    begin_frame::BeginFrameArgs,
    geometry::Vector2dF,
    input_types::ElementId,
    layer_tree_impl::{LayerTreeHostId, LayerTreeTypes, UnguessableToken},
    render_metadata::VerticalScrollDirection,
};
use crate::cc::{
    benchmarks::micro_benchmark_controller_impl::MicroBenchmarkControllerImpl,
    debug::layer_tree_debug_state::LayerTreeDebugState,
    input::{
        browser_controls_offset_manager::BrowserControlsOffsetManager,
        page_scale_animation::PageScaleAnimation,
        scrollbar_animation_controller::ScrollbarAnimationController,
    },
    layers::viewport::Viewport,
    resources::memory_history::MemoryHistory,
    scheduler::begin_frame_tracker::BeginFrameTracker,
    trees::{
        animated_paint_worklet_tracker::AnimatedPaintWorkletTracker,
        debug_rect_history::DebugRectHistory, image_animation_controller::ImageAnimationController,
        layer_tree_impl::LayerTreeImpl, layer_tree_settings::LayerTreeSettings,
        managed_memory_policy::ManagedMemoryPolicy,
        presentation_time_callback_buffer::PresentationTimeCallbackBuffer,
        render_frame_metadata::RenderFrameMetadata, throttle_decider::ThrottleDecider,
        unbounded_frame_sink_handler::UnboundedFrameSinkHandler,
    },
};
use std::{
    collections::{BTreeSet, HashSet},
    sync::Arc,
};

// Source: cc/trees/layer_tree_host_impl.h:1272-1276.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImplThreadPhase {
    IDLE,
    INSIDE_IMPL_FRAME,
}

// Source: cc/trees/layer_tree_host_impl.h:948-952,1092-1412.
// Rust drops fields in declaration order; C++ drops in reverse declaration
// order. Fields below deliberately follow the C++ destruction order. The
// explicit Destroy body runs first, then weak_factory_ invalidates all posted
// callbacks before any member can be destroyed. Identity is the typed `this`.
pub struct LayerTreeHostImpl<B: HostImplTypes> {
    // Source: cc/trees/layer_tree_host_impl.h:1412.
    weak_factory_: Arc<()>,
    // Source: cc/trees/layer_tree_host_impl.h:1408.
    unbounded_local_surface_id_: <B as LayerTreeTypes>::LocalSurfaceId,
    // Source: cc/trees/layer_tree_host_impl.h:1407.
    unbounded_frame_sink_id_:
        <B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
    // Source: cc/trees/layer_tree_host_impl.h:1405.
    unbounded_frame_sink_handler_: Option<Box<UnboundedFrameSinkHandler<B>>>,
    // Source: cc/trees/layer_tree_host_impl.h:1403.
    dump_compositor_frame_end_: u32,
    // Source: cc/trees/layer_tree_host_impl.h:1402.
    dump_compositor_frame_begin_: u32,
    // Source: cc/trees/layer_tree_host_impl.h:1401.
    dump_compositor_frame_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1397.
    tracked_element_rects_from_client_:
        <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects,
    // Source: cc/trees/layer_tree_host_impl.h:1393.
    is_handling_interaction_from_client_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1387.
    send_frame_token_to_embedder_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1379.
    previously_visible_scrollable_elements_: BTreeSet<ElementId>,
    // Source: cc/trees/layer_tree_host_impl.h:1375.
    pub(super) new_local_surface_id_expected_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1371.
    last_draw_active_tree_begin_frame_args_: BeginFrameArgs,
    // Source: cc/trees/layer_tree_host_impl.h:1367.
    view_transition_content_rects_: HashMap<u32, HostViewTransitionElementResourceRects>,
    // Source: cc/trees/layer_tree_host_impl.h:1364.
    pub(super) pending_invalidation_raster_inducing_scrolls_: BTreeSet<ElementId>,
    // Source: cc/trees/layer_tree_host_impl.h:1362.
    begin_frame_time_delta_: TimeDelta,
    // Source: cc/trees/layer_tree_host_impl.h:1359.
    frame_max_scroll_delta_: f32,
    // Source: cc/trees/layer_tree_host_impl.h:1355.
    top_controls_visible_height_: f32,
    // Source: cc/trees/layer_tree_host_impl.h:1353.
    screenshot_destination_: UnguessableToken,
    // Source: cc/trees/layer_tree_host_impl.h:1350.
    downsample_metrics_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1348.
    throttle_decider_: ThrottleDecider<B::ThrottleTypes>,
    // Source: cc/trees/layer_tree_host_impl.h:1344.
    contains_srgb_cache_: B::ColorSpaceCache,
    // Source: cc/trees/layer_tree_host_impl.h:1338.
    pub(super) has_observed_first_scroll_delay_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1337.
    has_non_fling_input_since_last_frame_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1336.
    has_input_resetter_:
        Arc<crate::cc::base::delayed_unique_notifier::DelayedUniqueNotifier>,
    // Source: cc/trees/layer_tree_host_impl.h:1335.
    has_input_for_frame_interval_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1333.
    pub(super) events_metrics_manager_: B::EventsMetricsManager,
    // Source: cc/trees/layer_tree_host_impl.h:1331.
    lag_tracking_manager_: B::AverageLagTrackingManager,
    // Source: cc/trees/layer_tree_host_impl.h:1329.
    pub(super) paint_worklet_tracker_: AnimatedPaintWorkletTracker<B>,
    #[cfg(debug_assertions)]
    // Source: cc/trees/layer_tree_host_impl.h:1322.
    doing_sync_draw_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1318.
    pub(super) pending_tree_fully_painted_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1313.
    pub(super) paint_worklet_painter_: Option<Box<B::PaintWorkletLayerPainter>>,
    // Source: cc/trees/layer_tree_host_impl.h:1309.
    pub(super) frame_trackers_: B::FrameSequenceTrackerCollection,
    // Source: cc/trees/layer_tree_host_impl.h:1308.
    compositor_frame_reporting_controller_: Option<Box<B::CompositorFrameReportingController>>,
    // Source: cc/trees/layer_tree_host_impl.h:1306.
    pub(super) frame_sorter_: B::FrameSorter,
    // Source: cc/trees/layer_tree_host_impl.h:1302.
    presentation_time_callbacks_: PresentationTimeCallbackBuffer<
        B::PresentationFeedback,
        B::HostFrameTimingDetails,
        B::SequenceChecker,
    >,
    // Source: cc/trees/layer_tree_host_impl.h:1299.
    last_vertical_scroll_direction_: VerticalScrollDirection,
    // Source: cc/trees/layer_tree_host_impl.h:1294.
    current_local_surface_id_from_client_: <B as LayerTreeTypes>::LocalSurfaceId,
    // Source: cc/trees/layer_tree_host_impl.h:1293.
    child_local_surface_id_allocator_: B::ChildLocalSurfaceIdAllocator,
    // Source: cc/trees/layer_tree_host_impl.h:1292.
    evicted_local_surface_id_: <B as LayerTreeTypes>::LocalSurfaceId,
    // Source: cc/trees/layer_tree_host_impl.h:1291.
    target_local_surface_id_: <B as LayerTreeTypes>::LocalSurfaceId,
    // Source: cc/trees/layer_tree_host_impl.h:1289.
    last_draw_render_frame_metadata_: Option<RenderFrameMetadata<B>>,
    // Source: cc/trees/layer_tree_host_impl.h:1288.
    last_draw_referenced_surfaces_: BTreeSet<B::SurfaceRange>,
    // Source: cc/trees/layer_tree_host_impl.h:1287.
    last_draw_local_surface_id_: <B as LayerTreeTypes>::LocalSurfaceId,
    // Source: cc/trees/layer_tree_host_impl.h:1285.
    next_frame_token_from_client_: u32,
    // Source: cc/trees/layer_tree_host_impl.h:1284.
    next_frame_token_: B::FrameTokenGenerator,
    // Source: cc/trees/layer_tree_host_impl.h:1282.
    render_frame_metadata_observer_: Option<Box<B::RenderFrameMetadataObserver>>,
    // Source: cc/trees/layer_tree_host_impl.h:1278.
    pub(super) image_animation_controller_: Option<Box<ImageAnimationController>>,
    // Source: cc/trees/layer_tree_host_impl.h:1276.
    pub(super) impl_thread_phase_: ImplThreadPhase,
    // Source: cc/trees/layer_tree_host_impl.h:1270.
    completed_image_decode_requests_: Vec<(i32, bool)>,
    // Source: cc/trees/layer_tree_host_impl.h:1265.
    may_throttle_if_undrawn_frames_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1263.
    prefers_reduced_motion_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1261.
    is_viewport_mobile_optimized_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1255.
    visual_device_viewport_size_: Size,
    // Source: cc/trees/layer_tree_host_impl.h:1253.
    viewport_: Option<Box<Viewport>>,
    // Source: cc/trees/layer_tree_host_impl.h:1251.
    consecutive_frame_with_damage_count_: i32,
    // Source: cc/trees/layer_tree_host_impl.h:1245.
    prioritize_new_content_due_to_checkerboarding_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1243.
    has_valid_layer_tree_frame_sink_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1239.
    is_likely_to_require_a_draw_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1238.
    requires_high_res_to_draw_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1236.
    latency_info_swap_promise_monitor_: BTreeSet<HostImplLatencyMonitorId>,
    // Source: cc/trees/layer_tree_host_impl.h:1233.
    id_: i32,
    // Source: cc/trees/layer_tree_host_impl.h:1232.
    task_graph_runner_: HostImplTaskGraphRunnerId,
    // Source: cc/trees/layer_tree_host_impl.h:1230.
    tree_activation_callback_: Option<<B as HostImplTypes>::RepeatingClosure>,
    // Source: cc/trees/layer_tree_host_impl.h:1227.
    single_thread_synchronous_task_graph_runner_: Option<Box<B::SynchronousTaskGraphRunner>>,
    // Source: cc/trees/layer_tree_host_impl.h:1225.
    pub(super) micro_benchmark_controller_: MicroBenchmarkControllerImpl<B::MicroBenchmarkImpl>,
    // Source: cc/trees/layer_tree_host_impl.h:1224.
    rendering_stats_instrumentation_: HostImplRenderingStatsId,
    // Source: cc/trees/layer_tree_host_impl.h:1222.
    flashed_scrollbars_: HashSet<ElementId>,
    // Source: cc/trees/layer_tree_host_impl.h:1219.
    scrollbar_animation_controllers_: HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    // Source: cc/trees/layer_tree_host_impl.h:1212.
    dark_mode_filter_: Option<HostImplDarkModeFilterId>,
    // Source: cc/trees/layer_tree_host_impl.h:1211.
    video_frame_controllers_: BTreeSet<HostImplVideoFrameControllerId>,
    // Source: cc/trees/layer_tree_host_impl.h:1209.
    mutator_events_: Option<Box<B::HostMutatorEvents>>,
    // Source: cc/trees/layer_tree_host_impl.h:1208.
    pub(super) mutator_host_: Box<<B as LayerTreeTypes>::MutatorHost>,
    // Source: cc/trees/layer_tree_host_impl.h:1206.
    total_invalidated_area_: Option<CheckedU32>,
    // Source: cc/trees/layer_tree_host_impl.h:1205.
    root_layer_damage_rect_: Rect,
    // Source: cc/trees/layer_tree_host_impl.h:1204.
    viewport_damage_rect_: Rect,
    // Source: cc/trees/layer_tree_host_impl.h:1202.
    resourceless_software_draw_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1201.
    viewport_rect_for_tile_priority_: Rect,
    // Source: cc/trees/layer_tree_host_impl.h:1200.
    external_viewport_: Rect,
    // Source: cc/trees/layer_tree_host_impl.h:1199.
    external_transform_: <B as LayerTreeTypes>::Transform,
    // Source: cc/trees/layer_tree_host_impl.h:1191.
    max_memory_needed_bytes_: usize,
    // Source: cc/trees/layer_tree_host_impl.h:1187.
    debug_rect_history_: Option<Box<DebugRectHistory>>,
    // Source: cc/trees/layer_tree_host_impl.h:1186.
    memory_history_: Option<Box<MemoryHistory>>,
    // Source: cc/trees/layer_tree_host_impl.h:1184.
    page_scale_animation_: Option<Box<PageScaleAnimation>>,
    // Source: cc/trees/layer_tree_host_impl.h:1182.
    pub(super) browser_controls_offset_manager_: Option<Box<BrowserControlsOffsetManager>>,
    // Source: cc/trees/layer_tree_host_impl.h:1179.
    pub(super) tile_manager_: B::TileManager,
    // Source: cc/trees/layer_tree_host_impl.h:1177.
    cached_managed_memory_policy_: ManagedMemoryPolicy,
    // Source: cc/trees/layer_tree_host_impl.h:1176.
    visible_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1175.
    debug_state_: LayerTreeDebugState,
    // Source: cc/trees/layer_tree_host_impl.h:1173.
    pub(super) tile_priorities_dirty_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1171.
    swap_promises_for_main_thread_scroll_update_: Vec<Box<B::SwapPromise>>,
    // Source: cc/trees/layer_tree_host_impl.h:1168.
    accumulated_scroll_deltas_by_element_id_: HashMap<ElementId, Vector2dF>,
    // Source: cc/trees/layer_tree_host_impl.h:1163.
    scroll_accumulated_this_frame_: Vector2dF,
    // Source: cc/trees/layer_tree_host_impl.h:1158.
    pub(super) recycle_tree_: Option<Box<LayerTreeImpl<B>>>,
    // Source: cc/trees/layer_tree_host_impl.h:1154.
    pub(super) pending_tree_: Option<Box<LayerTreeImpl<B>>>,
    // Source: cc/trees/layer_tree_host_impl.h:1150.
    pub(super) active_tree_: Option<Box<LayerTreeImpl<B>>>,
    // Source: cc/trees/layer_tree_host_impl.h:1147.
    pub(super) global_tile_state_: B::GlobalTileState,
    // Source: cc/trees/layer_tree_host_impl.h:1145.
    image_decode_cache_holder_: Option<Box<ImageDecodeCacheHolder<B::ImageDecodeFactory>>>,
    // Source: cc/trees/layer_tree_host_impl.h:1144.
    pending_raster_queries_: Option<Box<B::RasterQueryQueue>>,
    // Source: cc/trees/layer_tree_host_impl.h:1143.
    resource_pool_: Option<Box<B::ResourcePool>>,
    // Source: cc/trees/layer_tree_host_impl.h:1142.
    raster_buffer_provider_: Option<Box<B::RasterBufferProvider>>,
    // Source: cc/trees/layer_tree_host_impl.h:1140.
    pub(super) raster_caps_: RasterCapabilities,
    // Source: cc/trees/layer_tree_host_impl.h:1138.
    worker_context_visibility_: Option<Box<B::ContextVisibility>>,
    // Source: cc/trees/layer_tree_host_impl.h:1136.
    compositor_context_visibility_: Option<Box<B::ContextVisibility>>,
    // Source: cc/trees/layer_tree_host_impl.h:1129.
    layer_context_: Option<Box<<B as HostImplTypes>::LayerContext>>,
    // Source: cc/trees/layer_tree_host_impl.h:1124.
    layer_tree_frame_sink_: Option<HostImplFrameSinkId>,
    // Source: cc/trees/layer_tree_host_impl.h:1118.
    ui_resource_changes_: UIResourceChangeMap<crate::adapter::layer_tree_impl::UIResourceId>,
    // Source: cc/trees/layer_tree_host_impl.h:1114.
    evicted_ui_resources_: BTreeSet<crate::adapter::layer_tree_impl::UIResourceId>,
    // Source: cc/trees/layer_tree_host_impl.h:1109.
    deleted_ui_resources_:
        HashMap<crate::adapter::layer_tree_impl::UIResourceId, UIResourceData<B>>,
    // Source: cc/trees/layer_tree_host_impl.h:1106.
    ui_resource_map_: HashMap<crate::adapter::layer_tree_impl::UIResourceId, UIResourceData<B>>,
    // Source: cc/trees/layer_tree_host_impl.h:1104.
    pub(super) resource_provider_: Option<Box<B::ClientResourceProvider>>,
    // Source: cc/trees/layer_tree_host_impl.h:1102.
    is_synchronous_single_threaded_: bool,
    // Source: cc/trees/layer_tree_host_impl.h:1094.
    pub(super) settings_: LayerTreeSettings,
    // Source: cc/trees/layer_tree_host_impl.h:1092.
    pub(super) input_delegate_: Option<Box<B::InputHandler>>,
    // Source: cc/trees/layer_tree_host_impl.h:952.
    current_begin_frame_tracker_: BeginFrameTracker,
    // Source: cc/trees/layer_tree_host_impl.h:950.
    pub(super) task_runner_provider_: HostImplTaskRunnerProviderId,
    // Source: cc/trees/layer_tree_host_impl.h:949.
    scheduling_delegate_: HostImplSchedulingDelegateId,
    // Source: cc/trees/layer_tree_host_impl.h:948.
    pub(super) delegate_: HostImplDelegateId,
    pub(super) owner_id_: LayerTreeHostId,
}

use crate::adapter::trace_sink::SourceLocation;
use crate::cc::input::compositor_input_interfaces::InputDelegateForCompositor;
use std::{cell::RefCell, rc::Rc};

impl crate::adapter::micro_benchmark::HostHandle for LayerTreeHostId {
    fn IsNull(&self) -> bool {
        self.0 == 0
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:585-628.
    // Stable owner/service IDs represent C++ non-owning pointers; the call
    // context supplies their live referents. Factories below only construct
    // objects owned by the external resource, task and metrics subsystems.
    pub fn new(
        owner: LayerTreeHostId,
        settings: &LayerTreeSettings,
        delegate: HostImplDelegateId,
        task_runner_provider: HostImplTaskRunnerProviderId,
        rendering_stats_instrumentation: HostImplRenderingStatsId,
        task_graph_runner: HostImplTaskGraphRunnerId,
        mutator_host: Box<<B as LayerTreeTypes>::MutatorHost>,
        dark_mode_filter: Option<HostImplDarkModeFilterId>,
        id: i32,
        image_worker_task_runner: B::ImageWorkerTaskRunner,
        scheduling_delegate: HostImplSchedulingDelegateId,
        context: &mut HostImplContext<'_, B>,
    ) -> Self {
        let is_synchronous_single_threaded = !context.services.HasImplThread(task_runner_provider)
            && !settings.single_thread_proxy_scheduler;
        let task_runner = if context.services.HasImplThread(task_runner_provider) {
            context.services.ImplThreadTaskRunner(task_runner_provider)
        } else {
            context.services.MainThreadTaskRunner(task_runner_provider)
        };
        let alive = Arc::new(());
        let weak = HostImplWeak {
            owner,
            alive: Arc::downgrade(&alive),
        };
        let mut result = Self {
            delegate_: delegate,
            scheduling_delegate_: scheduling_delegate,
            task_runner_provider_: task_runner_provider,
            current_begin_frame_tracker_: BeginFrameTracker::new(SourceLocation {
                function_name: "LayerTreeHostImpl".into(),
                file_name: "cc/trees/layer_tree_host_impl.cc".into(),
                line_number: 599,
            }),
            input_delegate_: None,
            settings_: settings.clone(),
            is_synchronous_single_threaded_: is_synchronous_single_threaded,
            resource_provider_: None,
            ui_resource_map_: Default::default(),
            deleted_ui_resources_: Default::default(),
            evicted_ui_resources_: Default::default(),
            ui_resource_changes_: Default::default(),
            layer_tree_frame_sink_: None,
            layer_context_: None,
            compositor_context_visibility_: None,
            worker_context_visibility_: None,
            raster_caps_: Default::default(),
            raster_buffer_provider_: None,
            resource_pool_: None,
            pending_raster_queries_: None,
            image_decode_cache_holder_: None,
            global_tile_state_: context.services.DefaultGlobalTileState(),
            active_tree_: None,
            pending_tree_: None,
            recycle_tree_: None,
            scroll_accumulated_this_frame_: Default::default(),
            accumulated_scroll_deltas_by_element_id_: Default::default(),
            swap_promises_for_main_thread_scroll_update_: Default::default(),
            tile_priorities_dirty_: false,
            debug_state_: Default::default(),
            visible_: false,
            cached_managed_memory_policy_: settings.memory_policy,
            tile_manager_: context.services.CreateTileManager(
                owner,
                task_runner,
                image_worker_task_runner,
                if is_synchronous_single_threaded {
                    usize::MAX
                } else {
                    settings.scheduled_raster_task_limit
                },
                !settings.single_thread_proxy_scheduler,
                settings.ToTileManagerSettings(),
            ),
            browser_controls_offset_manager_: None,
            page_scale_animation_: None,
            memory_history_: Some(MemoryHistory::Create()),
            debug_rect_history_: Some(DebugRectHistory::Create()),
            max_memory_needed_bytes_: 0,
            external_transform_: Default::default(),
            external_viewport_: Default::default(),
            viewport_rect_for_tile_priority_: Default::default(),
            resourceless_software_draw_: false,
            viewport_damage_rect_: Default::default(),
            root_layer_damage_rect_: Default::default(),
            total_invalidated_area_: Some(CheckedU32::FromValue(0)),
            mutator_host_: mutator_host,
            mutator_events_: None,
            video_frame_controllers_: Default::default(),
            dark_mode_filter_: dark_mode_filter,
            scrollbar_animation_controllers_: Default::default(),
            flashed_scrollbars_: Default::default(),
            rendering_stats_instrumentation_: rendering_stats_instrumentation,
            micro_benchmark_controller_: MicroBenchmarkControllerImpl::new(&owner),
            single_thread_synchronous_task_graph_runner_: None,
            tree_activation_callback_: None,
            task_graph_runner_: task_graph_runner,
            id_: id,
            latency_info_swap_promise_monitor_: Default::default(),
            requires_high_res_to_draw_: false,
            is_likely_to_require_a_draw_: false,
            has_valid_layer_tree_frame_sink_: false,
            prioritize_new_content_due_to_checkerboarding_: false,
            consecutive_frame_with_damage_count_: settings.damaged_frame_limit,
            viewport_: None,
            visual_device_viewport_size_: Default::default(),
            is_viewport_mobile_optimized_: false,
            prefers_reduced_motion_: false,
            may_throttle_if_undrawn_frames_: true,
            completed_image_decode_requests_: Default::default(),
            impl_thread_phase_: ImplThreadPhase::IDLE,
            image_animation_controller_: None,
            render_frame_metadata_observer_: None,
            next_frame_token_: Default::default(),
            next_frame_token_from_client_: 0,
            last_draw_local_surface_id_: Default::default(),
            last_draw_referenced_surfaces_: Default::default(),
            last_draw_render_frame_metadata_: None,
            target_local_surface_id_: Default::default(),
            evicted_local_surface_id_: Default::default(),
            child_local_surface_id_allocator_: context
                .services
                .CreateChildLocalSurfaceIdAllocator(),
            current_local_surface_id_from_client_: Default::default(),
            last_vertical_scroll_direction_: Default::default(),
            presentation_time_callbacks_: PresentationTimeCallbackBuffer::new(
                context.services.NewSequenceChecker(),
            ),
            frame_sorter_: context.services.CreateFrameSorter(),
            compositor_frame_reporting_controller_: None,
            frame_trackers_: context
                .services
                .CreateFrameTrackers(settings.single_thread_proxy_scheduler),
            paint_worklet_painter_: None,
            pending_tree_fully_painted_: false,
            #[cfg(debug_assertions)]
            doing_sync_draw_: false,
            paint_worklet_tracker_: Default::default(),
            lag_tracking_manager_: context.services.CreateAverageLagTrackingManager(),
            events_metrics_manager_: context.services.CreateEventsMetricsManager(),
            has_input_for_frame_interval_: false,
            has_input_resetter_:
                crate::cc::base::delayed_unique_notifier::DelayedUniqueNotifier::new(
                    context.services.ResolveSequencedTaskRunner(task_runner),
                    context.services.RouteHostRepeatingClosure(HostImplTask {
                        target: weak.clone(),
                        kind: HostImplTaskKind::ResetHasInputForFrameInterval,
                    }),
                    kHasInputResetDelay,
                ),
            has_non_fling_input_since_last_frame_: false,
            has_observed_first_scroll_delay_: false,
            contains_srgb_cache_: context
                .services
                .CreateColorSpaceCache(kContainsSrgbCacheSize),
            throttle_decider_: Default::default(),
            downsample_metrics_: true,
            screenshot_destination_: Default::default(),
            top_controls_visible_height_: 0.0,
            frame_max_scroll_delta_: 0.0,
            begin_frame_time_delta_: Default::default(),
            pending_invalidation_raster_inducing_scrolls_: Default::default(),
            view_transition_content_rects_: Default::default(),
            last_draw_active_tree_begin_frame_args_: Default::default(),
            new_local_surface_id_expected_: false,
            previously_visible_scrollable_elements_: Default::default(),
            send_frame_token_to_embedder_: false,
            is_handling_interaction_from_client_: false,
            tracked_element_rects_from_client_: Default::default(),
            dump_compositor_frame_: false,
            dump_compositor_frame_begin_: 0,
            dump_compositor_frame_end_: 0,
            unbounded_frame_sink_handler_: None,
            unbounded_frame_sink_id_: context.services.InvalidFrameSinkId(),
            unbounded_local_surface_id_: Default::default(),
            weak_factory_: alive,
            owner_id_: owner,
        };
        // Source: cc/trees/layer_tree_host_impl.cc:629-635.
        if !result.settings_.trees_in_viz_in_viz_process {
            let runner = result.GetTaskRunner(context);
            let image_animation_runner = context.services.CreateImageAnimationTaskRunner(runner);
            result.image_animation_controller_ = Some(Box::new(ImageAnimationController::new(
                crate::adapter::image_animation::ImageAnimationControllerId(owner.0),
                image_animation_runner,
                result.settings_.enable_image_animation_resync,
            )));
        }
        // Source: cc/trees/layer_tree_host_impl.cc:637-643.
        assert!(
            !(settings.scrollbar_flash_once_after_scroll_update
                && settings.scrollbar_flash_after_any_scroll_update),
            "Only one of scrollbar_flash_once_after_scroll_update or scrollbar_flash_after_any_scroll_update can be enabled"
        );
        // Source: cc/trees/layer_tree_host_impl.cc:645-666.
        if settings.trees_in_viz_in_viz_process {
            result.compositor_frame_reporting_controller_ = Some(
                context
                    .services
                    .CreateStubCompositorFrameReportingController(),
            );
            result.next_frame_token_from_client_ = 1;
        } else {
            let is_trees_in_viz_client = result
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures());
            result.compositor_frame_reporting_controller_ =
                Some(context.services.CreateCompositorFrameReportingController(
                    !settings.single_thread_proxy_scheduler,
                    id,
                    is_trees_in_viz_client,
                ));
            #[cfg(target_os = "android")]
            if context
                .services
                .ShouldScrollJankV4MetricReportAndroidAppJankStats()
            {
                // Source cc:665-666: create the native weak callback identity
                // before exclusively borrowing its reporting-controller sibling.
                let owner = result.AsWeakPtr();
                context.services.ReportingSetScrollJankOsReporter(
                    result
                        .compositor_frame_reporting_controller_
                        .as_mut()
                        .unwrap(),
                    owner,
                );
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:668-673.
        if context.services.SettingsFeatures().TreesInViz()
            || context.services.SettingsFeatures().TreeAnimationsInViz()
        {
            result.dump_compositor_frame_ = ShouldDumpCompositorFrame(
                context.services,
                &mut result.dump_compositor_frame_begin_,
                &mut result.dump_compositor_frame_end_,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:675-682.
        let main_runner = context
            .services
            .MainThreadTaskRunner(result.task_runner_provider_);
        let impl_runner = result.GetTaskRunner(context);
        result.resource_provider_ = Some(context.services.CreateClientResourceProvider(
            main_runner,
            impl_runner,
            HostImplTask {
                target: result.AsWeakPtr(),
                kind: HostImplTaskKind::MaybeFlushPendingWork,
            },
        ));
        context
            .services
            .MutatorSetDelegate(&mut result.mutator_host_, Some(owner));
        result.mutator_events_ = Some(
            context
                .services
                .MutatorCreateEvents(&mut result.mutator_host_),
        );
        // Source: cc/trees/layer_tree_host_impl.cc:684-691.
        debug_assert!(context.services.IsImplThread(result.task_runner_provider_));
        DidVisibilityChange(context.services, owner.0, result.visible_);
        let (tree_id, property_owner) = context.services.AllocateLayerTreeIdentity();
        result.active_tree_ = Some(Box::new(LayerTreeImpl::new(
            tree_id,
            property_owner,
            owner,
            BeginFrameArgs::default(),
            Rc::new(RefCell::new(Default::default())),
            Rc::new(RefCell::new(Default::default())),
            Rc::new(RefCell::new(Default::default())),
        )));
        result
            .active_tree_
            .as_mut()
            .unwrap()
            .property_trees_mut()
            .set_is_active(true);
        // Source: cc/trees/layer_tree_host_impl.cc:693-704.
        result.viewport_ = Some(Viewport::Create(owner));
        context.services.TraceHostCreated(id);
        result.browser_controls_offset_manager_ =
            Some(BrowserControlsOffsetManager::CreateForHost(
                result
                    .active_tree_
                    .as_ref()
                    .unwrap()
                    .browser_controls_params(),
                context.services.BrowserControlsFeatureEnabled(
                    crate::adapter::browser_controls::BrowserControlsFeature::ScrollSnapAnimation,
                ),
                settings.top_controls_show_threshold,
                settings.top_controls_hide_threshold,
                settings.trees_in_viz_in_viz_process,
            ));
        result.SetDebugState(&settings.initial_debug_state, context);
        // Source: cc/trees/layer_tree_host_impl.cc:705-719.
        let reporting = result
            .compositor_frame_reporting_controller_
            .as_mut()
            .unwrap();
        context
            .services
            .ReportingSetFrameSorter(reporting, &mut result.frame_sorter_);
        context
            .services
            .ReportingSetFrameSequenceTrackerCollection(reporting, &mut result.frame_trackers_);
        if settings.is_layer_tree_for_ui {
            context
                .services
                .ReportingSetEventLatencyTracker(reporting, Some(owner));
            #[cfg(feature = "chromeos")]
            context
                .services
                .TrackersUpdateSmoothMainThreadHistory(&mut result.frame_trackers_, 1);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:721-726.
        context
            .services
            .TrackersSetCustomResultsAddedCallback(&mut result.frame_trackers_, weak);
        if result.settings_.is_layer_tree_for_ui {
            result.total_invalidated_area_ = Some(CheckedU32::FromValue(0));
        }
        result
    }

    // Source: cc/trees/layer_tree_host_impl.cc:728-774.
    // Explicit call context is the Rust representation of destructor access to
    // external non-owning pointers. Consuming self runs the complete destructor
    // body before automatic fields drop in the native reverse-member order.
    pub fn Destroy(mut self, context: &mut HostImplContext<'_, B>) {
        debug_assert!(context.services.IsImplThread(self.task_runner_provider_));
        context.services.TraceHostDeleted(self.id_);
        debug_assert!(self.layer_tree_frame_sink_.is_none());
        debug_assert!(self.resource_pool_.is_none());
        debug_assert!(self.image_decode_cache_holder_.is_none());
        debug_assert!(self.single_thread_synchronous_task_graph_runner_.is_none());
        self.DetachInputDelegateAndRenderFrameObserver(context);
        // Source: cc/trees/layer_tree_host_impl.cc:740-749. Detached owned
        // layers execute their actual derived destructors before each tree's
        // swap promises are broken, in recycle/pending/active source order.
        for id in [
            self.recycle_tree_.as_ref().map(|tree| tree.id),
            self.pending_tree_.as_ref().map(|tree| tree.id),
            self.active_tree_.as_ref().map(|tree| tree.id),
        ]
        .into_iter()
        .flatten()
        {
            B::ShutdownResidentTree(&mut self, id, context);
        }
        self.recycle_tree_ = None;
        self.pending_tree_ = None;
        self.active_tree_ = None;
        let mut release_client = UIResourceReleaseClient {
            owner: self.owner_id_,
            deleted: &mut self.deleted_ui_resources_,
        };
        context
            .services
            .ResourceProviderShutdownAndReleaseAllResources(
                self.resource_provider_.as_mut().unwrap(),
                &mut release_client,
            );
        context
            .services
            .MutatorClearMutators(&mut self.mutator_host_);
        context
            .services
            .MutatorSetDelegate(&mut self.mutator_host_, None);
        let reporting = self
            .compositor_frame_reporting_controller_
            .as_mut()
            .unwrap();
        context
            .services
            .ReportingSetEventLatencyTracker(reporting, None);
        context
            .services
            .ReportingClearFrameSequenceTrackerCollection(reporting);
        // Source: cc/trees/layer_tree_host_impl.h:1179; implicit member
        // destructor after cc:728-774. Rust requires its live services before
        // the canonical TileManager allocation itself can be dropped.
        B::DestroyResidentTileManager(&mut self, context);
    }

    // Source: cc/trees/layer_tree_host_impl.h:797-802.
    pub fn GetTaskRunner(&self, context: &HostImplContext<'_, B>) -> HostImplTaskRunnerId {
        debug_assert_ne!(self.task_runner_provider_.0, 0);
        if context.services.HasImplThread(self.task_runner_provider_) {
            context
                .services
                .ImplThreadTaskRunner(self.task_runner_provider_)
        } else {
            context
                .services
                .MainThreadTaskRunner(self.task_runner_provider_)
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:776-786.
    pub fn GetInputHandler(&self) -> &B::InputHandler {
        self.input_delegate_.as_deref().expect(
            "Requested InputHandler when one wasn't bound. Call BindToInputHandler to bind to one",
        )
    }
    pub fn GetInputHandlerMut(&mut self) -> &mut B::InputHandler {
        self.input_delegate_.as_deref_mut().expect(
            "Requested InputHandler when one wasn't bound. Call BindToInputHandler to bind to one",
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6155-6157.
    pub fn CommitsToActiveTree(&self) -> bool {
        self.settings_.commit_to_active_tree
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6257-6259.
    pub fn AsWeakPtr(&self) -> HostImplWeak {
        HostImplWeak {
            owner: self.owner_id_,
            alive: Arc::downgrade(&self.weak_factory_),
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6296-6301.
    pub fn RunningOnRendererProcess(&self) -> bool {
        !self.settings_.single_thread_proxy_scheduler
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6303-6305.
    pub fn ResetHasInputForFrameInterval(&mut self) {
        self.has_input_for_frame_interval_ = false;
    }
}

impl<B: HostImplTypes> crate::adapter::scroll_elasticity::LayerTreeHostImpl
    for LayerTreeHostImpl<B>
{
    fn CommitsToActiveTree(&self) -> bool {
        LayerTreeHostImpl::CommitsToActiveTree(self)
    }
}
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:1592-1594.
    pub fn SetViewportDamage(&mut self, damage: Rect, _context: &mut HostImplContext<'_, B>) {
        self.viewport_damage_rect_.Union(damage);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:1596-1598.
    pub fn SetRootLayerDamageRect(&mut self, damage: Rect, _context: &mut HostImplContext<'_, B>) {
        self.root_layer_damage_rect_.Union(damage);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:1809-1863.
    pub fn UpdateTileManagerMemoryPolicy(
        &mut self,
        policy: ManagedMemoryPolicy,
        context: &mut HostImplContext<'_, B>,
    ) {
        if self.resource_pool_.is_none() {
            return;
        }
        self.global_tile_state_.set_hard_memory_limit_in_bytes(0);
        self.global_tile_state_.set_soft_memory_limit_in_bytes(0);
        if self.visible_ && policy.bytes_limit_when_visible > 0 {
            self.global_tile_state_
                .set_hard_memory_limit_in_bytes(policy.bytes_limit_when_visible);
            let soft = (self.global_tile_state_.hard_memory_limit_in_bytes() as i64
                * self.settings_.max_memory_for_prepaint_percentage as i64)
                / 100;
            self.global_tile_state_
                .set_soft_memory_limit_in_bytes(soft as usize);
        }
        self.global_tile_state_.set_memory_limit_policy(
            ManagedMemoryPolicy::PriorityCutoffToTileMemoryLimitPolicy(if self.visible_ {
                policy.priority_cutoff_when_visible
            } else {
                crate::adapter::resource_types::PriorityCutoff::CUTOFF_ALLOW_NOTHING
            }),
        );
        self.global_tile_state_
            .set_num_resources_limit(policy.num_resources_limit);
        if self.global_tile_state_.hard_memory_limit_in_bytes() > 0 {
            self.SetContextVisibility(true, context);
            if let Some(holder) = self.image_decode_cache_holder_.as_mut() {
                holder.SetShouldAggressivelyFreeResources(false);
            }
        } else {
            let can_clear_decode_policy_tracking = false;
            B::ClearResidentCheckerImages(
                &mut self.tile_manager_,
                can_clear_decode_policy_tracking,
                context,
            );
        }
        debug_assert!(self.resource_pool_.is_some());
        context.services.ResourcePoolSetResourceUsageLimits(
            self.resource_pool_.as_mut().unwrap(),
            self.global_tile_state_.soft_memory_limit_in_bytes(),
            self.global_tile_state_.num_resources_limit(),
        );
        self.DidModifyTilePriorities(false, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:1865-1877.
    pub fn DidModifyTilePriorities(
        &mut self,
        pending_update_tiles: bool,
        context: &mut HostImplContext<'_, B>,
    ) {
        DidModifyTilePrioritiesForHost(
            &self.settings_,
            &mut self.tile_priorities_dirty_,
            &mut self.tile_manager_,
            self.delegate_,
            pending_update_tiles,
            context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4292-4305.
    pub fn ActualManagedMemoryPolicy(
        &self,
        _context: &HostImplContext<'_, B>,
    ) -> ManagedMemoryPolicy {
        use crate::adapter::resource_types::PriorityCutoff;
        let mut actual = self.cached_managed_memory_policy_;
        if self.debug_state_.rasterize_only_visible_content {
            actual.priority_cutoff_when_visible = PriorityCutoff::CUTOFF_ALLOW_REQUIRED_ONLY;
        } else if self.raster_caps_.use_gpu_rasterization
            && actual.priority_cutoff_when_visible == PriorityCutoff::CUTOFF_ALLOW_EVERYTHING
        {
            actual.priority_cutoff_when_visible = PriorityCutoff::CUTOFF_ALLOW_NICE_TO_HAVE;
        }
        actual
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4836-4842.
    pub fn SetRenderFrameObserver(
        &mut self,
        observer: Option<Box<B::RenderFrameMetadataObserver>>,
        context: &mut HostImplContext<'_, B>,
    ) {
        self.render_frame_metadata_observer_ = observer;
        if let Some(observer) = self.render_frame_metadata_observer_.as_mut() {
            context
                .services
                .BindRenderFrameMetadataObserverToCurrentSequence(observer);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4988-4994.
    pub fn DetachInputDelegateAndRenderFrameObserver(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) {
        if let Some(input) = self.input_delegate_.as_mut() {
            input.WillShutdown();
        }
        self.input_delegate_ = None;
        self.SetRenderFrameObserver(None, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6159-6192.
    pub fn SetContextVisibility(&mut self, is_visible: bool, context: &mut HostImplContext<'_, B>) {
        SetContextVisibilityForLiveParts::<B>(
            self.layer_tree_frame_sink_,
            &mut self.compositor_context_visibility_,
            &mut self.worker_context_visibility_,
            is_visible,
            context.services,
        );
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:5228-5237.
    pub fn SetFullViewportDamage(&mut self, _context: &mut HostImplContext<'_, B>) {
        SetFullViewportDamageForTree(
            self.active_tree_.as_ref().unwrap(),
            self.external_viewport_,
            &mut self.viewport_damage_rect_,
        );
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:5592-5609.
    pub fn SetDebugState(
        &mut self,
        new_debug_state: &LayerTreeDebugState,
        context: &mut HostImplContext<'_, B>,
    ) {
        if self.debug_state_ == *new_debug_state {
            return;
        }
        self.debug_state_ = new_debug_state.clone();
        if self.settings_.trees_in_viz_in_viz_process {
            context.services.SetRecordRenderingStats(
                self.rendering_stats_instrumentation_,
                self.debug_state_.RecordRenderingStats(),
            );
        }
        let policy = self.ActualManagedMemoryPolicy(context);
        self.UpdateTileManagerMemoryPolicy(policy, context);
        self.SetFullViewportDamage(context);
    }
}

// HostImpl chunk I translation. Source: cc/trees/layer_tree_host_impl.cc:5610-5977.
// This file is temporary assembly input; final output remains the same-stem
// cc/trees/layer_tree_host_impl.rs module.

use crate::adapter::resource_types::SinglePlaneFormat;

// Source: cc/trees/layer_tree_host_impl.cc:578-583 (native defaulted moves).
// C++ scalar members retain their values after a move; only owning class fields
// become empty. mem::take(UIResourceData) would incorrectly zero the live source.
fn MoveUIResourceData<B: HostImplResourceTypes>(data: &mut UIResourceData<B>) -> UIResourceData<B>
where
    <B as HostImplResourceTypes>::ResourceId: Copy,
{
    UIResourceData {
        opaque: data.opaque,
        shared_mapping: std::mem::take(&mut data.shared_mapping),
        shared_image: data.shared_image.take(),
        resource_id_for_export: data.resource_id_for_export,
        size: data.size,
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:581-583.
// Defaulted move assignment acts member-by-member in declaration order and
// destroys replaced mapping/image owners at those assignments, not at map erase.
fn AssignMoveUIResourceData<B: HostImplResourceTypes>(
    to: &mut UIResourceData<B>,
    from: &mut UIResourceData<B>,
) where
    <B as HostImplResourceTypes>::ResourceId: Copy,
{
    to.opaque = from.opaque;
    to.shared_mapping = std::mem::take(&mut from.shared_mapping);
    to.shared_image = from.shared_image.take();
    to.resource_id_for_export = from.resource_id_for_export;
    to.size = from.size;
}
// Source: cc/trees/layer_tree_host_impl.cc:5838-5842.
fn DeleteUIResourceBackingData<B: HostUIResourceTypes>(
    mut data: UIResourceData<B>,
    sync_token: &B::UISyncToken,
) where
    B::SharedImage: HostUISharedImage<B::UISyncToken>,
{
    data.shared_image
        .as_mut()
        .expect("source nonnull shared_image")
        .UpdateDestructionSyncToken(sync_token);
}
// Source: cc/trees/layer_tree_host_impl.cc:5844-5857.
// Both the public entry and synchronous resource-provider callback use this same
// algorithm over the resident deleted map. No copied host or deferred callback.
fn OnUIResourceReleasedInMap<B: HostUIResourceTypes>(
    deleted: &mut HashMap<crate::adapter::layer_tree_impl::UIResourceId, UIResourceData<B>>,
    uid: crate::adapter::layer_tree_impl::UIResourceId,
    sync_token: &B::UISyncToken,
    _lost: bool,
) where
    <B as HostImplResourceTypes>::ResourceId: Copy,
    B::SharedImage: HostUISharedImage<B::UISyncToken>,
{
    let Some(data) = deleted.get_mut(&uid) else {
        return;
    };
    DeleteUIResourceBackingData::<B>(MoveUIResourceData(data), sync_token);
    deleted.remove(&uid);
}
// Source dependency: base::BindOnce callback delivery at cc:5783-5785.
struct UIResourceReleaseClient<'a, B: HostUIResourceTypes> {
    owner: LayerTreeHostId,
    deleted: &'a mut HashMap<crate::adapter::layer_tree_impl::UIResourceId, UIResourceData<B>>,
}
impl<B: HostUIResourceTypes> HostUIResourceReleaseClient<B> for UIResourceReleaseClient<'_, B>
where
    <B as HostImplResourceTypes>::ResourceId: Copy,
    B::SharedImage: HostUISharedImage<B::UISyncToken>,
{
    fn OwnerId(&self) -> LayerTreeHostId {
        self.owner
    }
    fn OnUIResourceReleased(
        &mut self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
        token: &B::UISyncToken,
        lost: bool,
    ) {
        OnUIResourceReleasedInMap::<B>(self.deleted, uid, token, lost);
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B>
where
    <B as HostImplResourceTypes>::ResourceId: Copy + Eq,
    B::SharedImage: Clone + HostUISharedImage<B::UISyncToken>,
    B::ClientResourceProvider: HostUIResourceProvider<B>,
{
    // Source: cc/trees/layer_tree_host_impl.cc:5611-5805.
    pub fn CreateUIResource(
        &mut self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
        bitmap: &B::UIBitmap,
        context: &mut HostImplContext<'_, B>,
    ) {
        let HostImplRuntimeParts {
            mut core,
            active_tree_,
            ..
        } = self.BorrowRuntime();
        HostImplUIResourceState::new(&mut core, active_tree_.as_deref())
            .CreateUIResource(uid, bitmap, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5807-5836.
    pub fn DeleteUIResource(
        &mut self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
        context: &mut HostImplContext<'_, B>,
    ) {
        let HostImplRuntimeParts {
            mut core,
            active_tree_,
            ..
        } = self.BorrowRuntime();
        HostImplUIResourceState::new(&mut core, active_tree_.as_deref())
            .DeleteUIResource(uid, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5838-5842.
    pub fn DeleteUIResourceBacking(&mut self, data: UIResourceData<B>, token: &B::UISyncToken) {
        DeleteUIResourceBackingData::<B>(data, token);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5844-5857.
    pub fn OnUIResourceReleased(
        &mut self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
        token: &B::UISyncToken,
        lost: bool,
    ) {
        OnUIResourceReleasedInMap::<B>(&mut self.deleted_ui_resources_, uid, token, lost);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5859-5882.
    pub fn ClearUIResources(&mut self, _context: &mut HostImplContext<'_, B>) {
        for (&uid, data) in &mut self.ui_resource_map_ {
            let mut client = UIResourceReleaseClient::<B> {
                owner: self.owner_id_,
                deleted: &mut self.deleted_ui_resources_,
            };
            self.resource_provider_
                .as_mut()
                .unwrap()
                .RemoveImportedResource(data.resource_id_for_export, &mut client);
            DeleteUIResourceBackingData::<B>(MoveUIResourceData(data), &B::UISyncToken::default());
            self.evicted_ui_resources_.insert(uid);
        }
        self.ui_resource_map_.clear();
        for data in self.deleted_ui_resources_.values_mut() {
            DeleteUIResourceBackingData::<B>(MoveUIResourceData(data), &B::UISyncToken::default());
        }
        self.deleted_ui_resources_.clear();
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5884-5896.
    pub fn EvictAllUIResources(&mut self, context: &mut HostImplContext<'_, B>) {
        if self.ui_resource_map_.is_empty() {
            return;
        }
        while !self.ui_resource_map_.is_empty() {
            let uid = *self.ui_resource_map_.keys().next().unwrap();
            self.DeleteUIResource(uid, context);
            self.evicted_ui_resources_.insert(uid);
        }
        self.SetNeedsCommit(context);
        let can_draw = self.CanDraw(context);
        context
            .services
            .OnCanDrawStateChanged(self.delegate_, can_draw);
        self.RenewTreePriority(context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5898-5905.
    pub fn ResourceIdForUIResource(
        &self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
    ) -> <B as HostImplResourceTypes>::ResourceId {
        if let Some(data) = self.ui_resource_map_.get(&uid) {
            return data.resource_id_for_export;
        }
        <B as HostImplResourceTypes>::ResourceId::default()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5907-5913.
    pub fn GetUIResourceSize(&self, uid: crate::adapter::layer_tree_impl::UIResourceId) -> Size {
        if let Some(data) = self.ui_resource_map_.get(&uid) {
            return data.size;
        }
        Size::default()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5915-5919.
    pub fn IsUIResourceOpaque(&self, uid: crate::adapter::layer_tree_impl::UIResourceId) -> bool {
        self.ui_resource_map_
            .get(&uid)
            .expect("CHECK UIResource exists")
            .opaque
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5921-5923.
    pub fn EvictedUIResourcesExist(&self) -> bool {
        !self.evicted_ui_resources_.is_empty()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5925-5934.
    pub fn MarkUIResourceNotEvicted(
        &mut self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
        context: &mut HostImplContext<'_, B>,
    ) {
        let HostImplRuntimeParts {
            mut core,
            active_tree_,
            ..
        } = self.BorrowRuntime();
        HostImplUIResourceState::new(&mut core, active_tree_.as_deref())
            .MarkUIResourceNotEvicted(uid, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5936-5939.
    pub fn ScheduleMicroBenchmark(&mut self, benchmark: Box<B::MicroBenchmarkImpl>) {
        self.micro_benchmark_controller_.ScheduleRun(benchmark);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5941-5944.
    pub fn InsertLatencyInfoSwapPromiseMonitor(&mut self, monitor: HostImplLatencyMonitorId) {
        self.latency_info_swap_promise_monitor_.insert(monitor);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5946-5949.
    pub fn RemoveLatencyInfoSwapPromiseMonitor(&mut self, monitor: HostImplLatencyMonitorId) {
        self.latency_info_swap_promise_monitor_.remove(&monitor);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5951-5956.
    pub fn NotifyLatencyInfoSwapPromiseMonitors(&mut self, context: &mut HostImplContext<'_, B>) {
        let promises =
            NotifyLatencyMonitorsForHost(&self.latency_info_swap_promise_monitor_, context);
        self.QueuePinnedLatencyPromises(promises, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5958-5970.
    pub fn IsOwnerThread(&self, context: &mut HostImplContext<'_, B>) -> bool {
        let result;
        if context
            .services
            .OptionalImplThreadTaskRunner(self.task_runner_provider_)
            .is_some()
        {
            let runner = context
                .services
                .OptionalImplThreadTaskRunner(self.task_runner_provider_)
                .unwrap();
            result = context.services.RunsTasksInCurrentSequence(runner);
        } else {
            let runner = context
                .services
                .MainThreadTaskRunner(self.task_runner_provider_);
            result = context.services.RunsTasksInCurrentSequence(runner);
            debug_assert!(result);
        }
        result
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5973-5975.
    pub fn InProtectedSequence(&self) -> bool {
        false
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5977.
    pub fn WaitForProtectedSequenceCompletion(&self) {}
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:801-841.
    pub fn CanDraw(&self, context: &HostImplContext<'_, B>) -> bool {
        CanDrawForHost(
            &self.layer_tree_frame_sink_,
            &self.resourceless_software_draw_,
            self.active_tree_.as_deref(),
            &self.evicted_local_surface_id_,
            &self.external_viewport_,
            &self.evicted_ui_resources_,
            context.services,
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3954-3958.
    // Rust represents the source default argument as a named forwarding entry.
    pub fn SetNeedsCommit(&mut self, context: &mut HostImplContext<'_, B>) {
        self.SetNeedsCommitWithReason(B::OtherBeginMainFrameReason(), context);
    }
    pub fn SetNeedsCommitWithReason(
        &mut self,
        reason: <B as crate::adapter::input_services::InputHostTypes>::BeginMainFrameReason,
        context: &mut HostImplContext<'_, B>,
    ) {
        RequestCommitForHost(&self.settings_, self.delegate_, reason, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4832-4834.
    pub fn RenewTreePriority(&mut self, context: &mut HostImplContext<'_, B>) {
        context.services.DelegateRenewTreePriority(self.delegate_);
    }
}
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.h:222-224.
    pub fn set_resourceless_software_draw_for_testing(&mut self, value: bool) {
        self.resourceless_software_draw_ = value;
    }
    // Source: cc/trees/layer_tree_host_impl.h:226-228.
    pub fn viewport_damage_rect_for_testing(&self) -> &Rect {
        &self.viewport_damage_rect_
    }
    // Source: cc/trees/layer_tree_host_impl.h:230-232.
    pub fn ResetViewportDamageRectForTesting(&mut self) {
        self.viewport_damage_rect_ = Rect::default();
    }
    // Source: cc/trees/layer_tree_host_impl.h:234-236.
    pub fn HasPendingRasterInvalidationScrollForTesting(&self, id: ElementId) -> bool {
        self.pending_invalidation_raster_inducing_scrolls_
            .contains(&id)
    }
    // Source: cc/trees/layer_tree_host_impl.h:238.
    pub fn WillSendBeginMainFrame(&mut self) {}
    // Source: cc/trees/layer_tree_host_impl.h:320.
    pub fn frame_trackers(&mut self) -> &mut B::FrameSequenceTrackerCollection {
        &mut self.frame_trackers_
    }
    // Source: cc/trees/layer_tree_host_impl.h:331-333.
    pub fn set_viewport_mobile_optimized(&mut self, optimized: bool) {
        self.is_viewport_mobile_optimized_ = optimized;
    }
    // Source: cc/trees/layer_tree_host_impl.h:335-337.
    pub fn viewport_mobile_optimized(&self) -> bool {
        self.is_viewport_mobile_optimized_
    }
    // Source: cc/trees/layer_tree_host_impl.h:342-344.
    pub fn may_throttle_if_undrawn_frames(&self) -> bool {
        self.may_throttle_if_undrawn_frames_
    }
    // Source: cc/trees/layer_tree_host_impl.h:346-348.
    pub fn SetScreenshotDestinationToken(&mut self, token: UnguessableToken) {
        self.screenshot_destination_ = token;
    }
    // Source: cc/trees/layer_tree_host_impl.h:349-353.
    pub fn TakeScreenshotDestinationToken(&mut self) -> UnguessableToken {
        let destination = self.screenshot_destination_;
        self.screenshot_destination_ = UnguessableToken::default();
        destination
    }
    // Source: cc/trees/layer_tree_host_impl.h:439.
    pub fn settings(&self) -> &LayerTreeSettings {
        &self.settings_
    }
    // Source: cc/trees/layer_tree_host_impl.h:479-481.
    pub fn PrioritizeNewContentDueToCheckerboarding(&self) -> bool {
        self.prioritize_new_content_due_to_checkerboarding_
    }
    // Source: cc/trees/layer_tree_host_impl.h:565.
    pub fn id(&self) -> i32 {
        self.id_
    }
    // Source: cc/trees/layer_tree_host_impl.h:567-569.
    pub fn layer_tree_frame_sink(&self) -> Option<HostImplFrameSinkId> {
        self.layer_tree_frame_sink_
    }
    // Source: cc/trees/layer_tree_host_impl.h:570.
    pub fn max_texture_size(&self) -> i32 {
        self.raster_caps().max_texture_size
    }
    // Source: cc/trees/layer_tree_host_impl.h:576.
    pub fn tile_manager(&mut self) -> &mut B::TileManager {
        &mut self.tile_manager_
    }
    // Source: cc/trees/layer_tree_host_impl.h:578.
    pub fn raster_caps(&self) -> &RasterCapabilities {
        &self.raster_caps_
    }
    // Source: cc/trees/layer_tree_host_impl.h:579-581.
    pub fn use_gpu_rasterization(&self) -> bool {
        self.raster_caps().use_gpu_rasterization
    }
    // Source: cc/trees/layer_tree_host_impl.h:583.
    pub fn resource_pool(&mut self) -> Option<&mut B::ResourcePool> {
        self.resource_pool_.as_deref_mut()
    }
    // Source: cc/trees/layer_tree_host_impl.h:584-586.
    pub fn image_animation_controller(&mut self) -> Option<&mut ImageAnimationController> {
        self.image_animation_controller_.as_deref_mut()
    }
    // Source: cc/trees/layer_tree_host_impl.h:593-596.
    pub fn set_tracked_element_rects_from_client(
        &mut self,
        rects: <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects,
    ) {
        self.tracked_element_rects_from_client_ = rects;
    }
    // Source: cc/trees/layer_tree_host_impl.h:598-600.
    pub fn tracked_element_rects_from_client(
        &self,
    ) -> &<B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects
    {
        &self.tracked_element_rects_from_client_
    }
    // Source: cc/trees/layer_tree_host_impl.h:636-638.
    pub fn last_draw_local_surface_id(&self) -> &<B as LayerTreeTypes>::LocalSurfaceId {
        &self.last_draw_local_surface_id_
    }
    // Source: cc/trees/layer_tree_host_impl.h:640-645.
    pub fn GetCurrentLocalSurfaceId<'a>(
        &'a self,
        context: &HostImplContext<'_, B>,
    ) -> &'a <B as LayerTreeTypes>::LocalSurfaceId {
        if self.settings().trees_in_viz_in_viz_process {
            return &self.current_local_surface_id_from_client_;
        }
        context
            .services
            .ChildLocalSurfaceIdAllocatorCurrent(&self.child_local_surface_id_allocator_)
    }
    // Source: cc/trees/layer_tree_host_impl.h:646-648.
    pub fn target_local_surface_id(&self) -> &<B as LayerTreeTypes>::LocalSurfaceId {
        &self.target_local_surface_id_
    }
    // Source: cc/trees/layer_tree_host_impl.h:650-655.
    // Mutable/const overloads borrow the same unique owner allocation.
    pub fn active_tree(&self) -> Option<&LayerTreeImpl<B>> {
        self.active_tree_.as_deref()
    }
    pub fn active_tree_mut(&mut self) -> Option<&mut LayerTreeImpl<B>> {
        self.active_tree_.as_deref_mut()
    }
    pub fn pending_tree(&self) -> Option<&LayerTreeImpl<B>> {
        self.pending_tree_.as_deref()
    }
    pub fn pending_tree_mut(&mut self) -> Option<&mut LayerTreeImpl<B>> {
        self.pending_tree_.as_deref_mut()
    }
    pub fn recycle_tree(&self) -> Option<&LayerTreeImpl<B>> {
        self.recycle_tree_.as_deref()
    }
    pub fn recycle_tree_mut(&mut self) -> Option<&mut LayerTreeImpl<B>> {
        self.recycle_tree_.as_deref_mut()
    }
    // Source: cc/trees/layer_tree_host_impl.h:657-659.
    pub fn sync_tree(&mut self) -> Option<&mut LayerTreeImpl<B>> {
        if self.CommitsToActiveTree() {
            self.active_tree_.as_deref_mut()
        } else {
            self.pending_tree_.as_deref_mut()
        }
    }
    // Source: cc/trees/layer_tree_host_impl.h:683-685.
    pub fn set_force_smooth_wheel_scrolling_for_testing(&mut self, enabled: bool) {
        self.GetInputHandlerMut()
            .set_force_smooth_wheel_scrolling_for_testing(enabled);
    }
    // Source: cc/trees/layer_tree_host_impl.h:688.
    pub fn visible(&self) -> bool {
        self.visible_
    }
    // Source: cc/trees/layer_tree_host_impl.h:699-700.
    pub fn frame_sorter(&mut self) -> &mut B::FrameSorter {
        &mut self.frame_sorter_
    }
    pub fn memory_history(&mut self) -> Option<&mut MemoryHistory> {
        self.memory_history_.as_deref_mut()
    }
    // Source: cc/trees/layer_tree_host_impl.h:701.
    pub fn debug_rect_history(&mut self) -> Option<&mut DebugRectHistory> {
        self.debug_rect_history_.as_deref_mut()
    }
    // Source: cc/trees/layer_tree_host_impl.h:702-704.
    pub fn resource_provider(&mut self) -> Option<&mut B::ClientResourceProvider> {
        self.resource_provider_.as_deref_mut()
    }
    // Source: cc/trees/layer_tree_host_impl.h:705-707.
    pub fn browser_controls_manager(&mut self) -> Option<&mut BrowserControlsOffsetManager> {
        self.browser_controls_offset_manager_.as_deref_mut()
    }
    // Source: cc/trees/layer_tree_host_impl.h:708-710.
    pub fn global_tile_state(&self) -> &B::GlobalTileState {
        &self.global_tile_state_
    }
    // Source: cc/trees/layer_tree_host_impl.h:712-714.
    pub fn task_runner_provider(&self) -> HostImplTaskRunnerProviderId {
        self.task_runner_provider_
    }
    // Source: cc/trees/layer_tree_host_impl.h:716.
    pub fn mutator_host(&mut self) -> &mut <B as LayerTreeTypes>::MutatorHost {
        &mut self.mutator_host_
    }
    // Source: cc/trees/layer_tree_host_impl.h:719.
    pub fn debug_state(&self) -> &LayerTreeDebugState {
        &self.debug_state_
    }
    // Source: cc/trees/layer_tree_host_impl.h:738.
    pub fn page_scale_animation_active(&self) -> bool {
        self.page_scale_animation_.is_some()
    }
    // Source: cc/trees/layer_tree_host_impl.h:764.
    pub fn external_viewport(&self) -> &Rect {
        &self.external_viewport_
    }
    // Source: cc/trees/layer_tree_host_impl.h:768-770.
    pub fn viewport_rect_for_tile_priority(&self) -> &Rect {
        &self.viewport_rect_for_tile_priority_
    }
    // Source: cc/trees/layer_tree_host_impl.h:783-785.
    pub fn SetRequiresHighResToDraw(&mut self) {
        self.requires_high_res_to_draw_ = true;
    }
    pub fn ResetRequiresHighResToDraw(&mut self) {
        self.requires_high_res_to_draw_ = false;
    }
    pub fn RequiresHighResToDraw(&self) -> bool {
        self.requires_high_res_to_draw_
    }
    // Source: cc/trees/layer_tree_host_impl.h:795.
    pub fn prepare_tiles_needed(&self) -> bool {
        self.tile_priorities_dirty_
    }
    // Source: cc/trees/layer_tree_host_impl.h:817-819.
    pub fn GetPaintWorkletLayerPainterForTesting(&self) -> Option<&B::PaintWorkletLayerPainter> {
        self.paint_worklet_painter_.as_deref()
    }
    // Source: cc/trees/layer_tree_host_impl.h:837-839.
    pub fn FrameSequenceTrackerActiveTypes(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) -> B::ActiveFrameSequenceTrackers {
        context
            .services
            .FrameSequenceTrackerActiveTypes(&mut self.frame_trackers_)
    }
    // Source: cc/trees/layer_tree_host_impl.h:841.
    pub fn RenewTreePriorityForTesting(&mut self, context: &mut HostImplContext<'_, B>) {
        self.RenewTreePriority(context);
    }
    // Source: cc/trees/layer_tree_host_impl.h:854-857.
    pub fn compositor_frame_reporting_controller(
        &self,
    ) -> Option<&B::CompositorFrameReportingController> {
        self.compositor_frame_reporting_controller_.as_deref()
    }
    // Source: cc/trees/layer_tree_host_impl.h:859-861.
    pub fn set_pending_tree_fully_painted_for_testing(&mut self, painted: bool) {
        self.pending_tree_fully_painted_ = painted;
    }
    // Source: cc/trees/layer_tree_host_impl.h:862-864.
    pub fn paint_worklet_tracker(&mut self) -> &mut AnimatedPaintWorkletTracker<B> {
        &mut self.paint_worklet_tracker_
    }
    // Source: cc/trees/layer_tree_host_impl.h:866.
    pub fn can_use_msaa(&self) -> bool {
        self.raster_caps().can_use_msaa
    }
    // Source: cc/trees/layer_tree_host_impl.h:868.
    pub fn viewport(&mut self) -> &mut Viewport {
        self.viewport_
            .as_deref_mut()
            .expect("source nonnull viewport")
    }
    // Source: cc/trees/layer_tree_host_impl.h:870.
    pub fn frame_sorter_for_testing(&mut self) -> &mut B::FrameSorter {
        &mut self.frame_sorter_
    }
    // Source: cc/trees/layer_tree_host_impl.h:872-874.
    pub fn TakeSavedEventsMetricsForTesting(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) -> B::EventMetricsList {
        context
            .services
            .TakeSavedEventsMetrics(&mut self.events_metrics_manager_)
    }
    // Source: cc/trees/layer_tree_host_impl.h:879-881.
    pub fn GetRasterQueryQueueForTesting(&self) -> Option<&B::RasterQueryQueue> {
        self.pending_raster_queries_.as_deref()
    }
    // Source: cc/trees/layer_tree_host_impl.h:883-885.
    pub fn GetFrameSinksToThrottleForTesting(
        &self,
    ) -> BTreeSet<<B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId> {
        self.throttle_decider_.ids().clone()
    }
    // Source: cc/trees/layer_tree_host_impl.h:893-895.
    pub fn SetDownsampleMetricsForTesting(&mut self, value: bool) {
        self.downsample_metrics_ = value;
    }
    // Source: cc/trees/layer_tree_host_impl.h:896-899.
    pub fn delegate_for_testing(&self) -> HostImplDelegateId {
        self.delegate_
    }
    // Source: cc/trees/layer_tree_host_impl.h:914-916.
    pub fn send_frame_token_to_embedder(&self) -> bool {
        self.send_frame_token_to_embedder_
    }
    // Source: cc/trees/layer_tree_host_impl.h:941-943.
    pub fn is_likely_to_require_a_draw(&self) -> bool {
        self.is_likely_to_require_a_draw_
    }
}
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:419-422.
    pub fn GetTracingTrack(tree: Option<&LayerTreeImpl<B>>) -> HostImplNamedTrack {
        HostImplNamedTrack {
            name: "cc::PendingTree",
            tree: tree.map(|t| t.id),
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:472-482.
    pub fn DidUpdateScrollAnimationCurve(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::DidUpdateScrollAnimationCurve(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:484-487.
    pub fn DidStartPinchZoom(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::DidStartPinchZoom(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:489-496.
    pub fn DidEndPinchZoom(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::DidEndPinchZoom(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:498-501.
    pub fn DidUpdatePinchZoom(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::DidUpdatePinchZoom(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:503-508.
    pub fn DidStartScroll(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::DidStartScroll(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:510-528.
    pub fn DidEndScroll(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::DidEndScroll(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:543-546.
    pub fn SetDeferBeginMainFrame(&self, defer: bool, context: &mut HostImplContext<'_, B>) {
        context
            .services
            .DelegateSetDeferBeginMainFrameFromImpl(self.delegate_, defer);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:558-560.
    pub fn HasScrollLinkedAnimation(
        &self,
        for_scroller: ElementId,
        context: &HostImplContext<'_, B>,
    ) -> bool {
        context
            .services
            .MutatorHasScrollLinkedAnimation(&self.mutator_host_, for_scroller)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:562-564.
    pub fn IsInHighLatencyMode(&self) -> bool {
        self.impl_thread_phase_ == ImplThreadPhase::IDLE
    }
    // Source: cc/trees/layer_tree_host_impl.cc:566-568.
    pub fn GetSettings(&self) -> &LayerTreeSettings {
        self.settings()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:570-576.
    pub fn GetImplDeprecated(&self) -> &Self {
        self
    }
    pub fn GetImplDeprecatedMut(&mut self) -> &mut Self {
        self
    }
    // Source: cc/trees/layer_tree_host_impl.cc:788-799.
    pub fn GatherAnimatedImageDriverState(&self) -> B::AnimatedImageDriverMap {
        GatherAnimatedImageDriverStateForTrees(
            self.active_tree_.as_deref().unwrap(),
            self.pending_tree_.as_deref(),
            self.recycle_tree_.as_deref(),
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:959-961.
    pub fn DidScrollForMetrics(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::DidScrollForMetrics(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:963-967.
    pub fn NotifyInputEvent(&mut self, is_fling: bool) {
        NotifyInputEventForHost(
            &mut self.has_input_for_frame_interval_,
            &self.has_input_resetter_,
            &mut self.has_non_fling_input_since_last_frame_,
            is_fling,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:969-973.
    pub fn QueueSwapPromiseForMainThreadScrollUpdate(&mut self, promise: Box<B::SwapPromise>) {
        self.swap_promises_for_main_thread_scroll_update_
            .push(promise);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:975-985.
    pub fn GetDrawMode(&self, context: &HostImplContext<'_, B>) -> HostDrawMode {
        if self.resourceless_software_draw_ {
            HostDrawMode::DRAW_MODE_RESOURCELESS_SOFTWARE
        } else if context
            .services
            .FrameSinkContextProvider(
                self.layer_tree_frame_sink_
                    .expect("source nonnull frame sink"),
            )
            .is_some()
            || (self.settings_.trees_in_viz_in_viz_process
                && self.settings_.display_tree_draw_mode_is_gpu)
        {
            HostDrawMode::DRAW_MODE_HARDWARE
        } else {
            HostDrawMode::DRAW_MODE_SOFTWARE
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4268-4272.
    pub fn SetNeedsOneBeginImplFrame(&mut self, context: &mut HostImplContext<'_, B>) {
        let promises = RequestBeginImplFrameForHost(
            self.delegate_,
            &self.latency_info_swap_promise_monitor_,
            &mut self.events_metrics_manager_,
            context,
        );
        self.QueuePinnedLatencyPromises(promises, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4274-4290.
    pub fn SetNeedsRedraw(
        &mut self,
        animation_only: bool,
        skip_if_inside_draw: bool,
        context: &mut HostImplContext<'_, B>,
    ) {
        let promises = RequestRedrawForHost(
            &self.settings_,
            self.delegate_,
            &self.latency_info_swap_promise_monitor_,
            &mut self.events_metrics_manager_,
            animation_only,
            skip_if_inside_draw,
            context,
        );
        self.QueuePinnedLatencyPromises(promises, context);
    }

    fn QueuePinnedLatencyPromises(
        &mut self,
        promises: Vec<Box<B::SwapPromise>>,
        context: &mut HostImplContext<'_, B>,
    ) {
        if promises.is_empty() {
            return;
        }
        self.WithActiveTree(context, |tree, tree_context| {
            for promise in promises {
                tree.QueuePinnedSwapPromise(promise, tree_context);
            }
        });
    }
}

use crate::adapter::{
    geometry::Point, input_services::InputHostTypes, layer_tree_impl::ScrollbarSet, time::TimeTicks,
};
use crate::cc::trees::{
    compositor_commit_data::CompositorCommitData, frame_data::FrameData,
};
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:5182-5226.
    pub fn ProcessCompositorDeltas(
        &mut self,
        main_thread_mutator_host: Option<&<B as InputHostTypes>::MutatorHost>,
        context: &mut HostImplContext<'_, B>,
    ) -> Box<CompositorCommitData> {
        let next_bmf = main_thread_mutator_host.is_some();
        self.ProcessCompositorDeltasInternal(main_thread_mutator_host, next_bmf, context)
    }

    /// ProxyImpl derives `next_bmf` from whether `data_for_commit_` already
    /// owns the previous main-thread mutator host. The native Engine retains
    /// that pipeline state separately, so its boundary passes the same source
    /// decision without fabricating a second mutator-host owner.
    pub(crate) fn ProcessCompositorDeltasForPipeline(
        &mut self,
        next_bmf: bool,
        context: &mut HostImplContext<'_, B>,
    ) -> Box<CompositorCommitData> {
        self.ProcessCompositorDeltasInternal(None, next_bmf, context)
    }

    fn ProcessCompositorDeltasInternal(
        &mut self,
        main_thread_mutator_host: Option<&<B as InputHostTypes>::MutatorHost>,
        next_bmf: bool,
        context: &mut HostImplContext<'_, B>,
    ) -> Box<CompositorCommitData> {
        let mut commit_data = Box::new(CompositorCommitData::default());
        self.WithInputHandler(context, |input, host| {
            if let Some(input) = input {
                input.ProcessCommitDeltas(host, &mut commit_data, main_thread_mutator_host);
            }
        });
        self.CollectScrollbarUpdatesForCommit(&mut commit_data);
        if let Some(controller) = self.image_animation_controller_.as_mut() {
            commit_data.advanced_image_animation_clients =
                controller.TakeAdvancedAnimationClients();
            commit_data.animated_image_frame_index_map = Some(controller.GatherFrameIndexes());
        }
        commit_data.page_scale_delta = self
            .active_tree_
            .as_mut()
            .unwrap()
            .page_scale_factor_mut()
            .PullDeltaForMainThread(next_bmf);
        // Source forwarding: LayerTreeImpl::PinchGestureActive immediately
        // queries this owner, so no current-tree alias is required.
        commit_data.is_pinch_gesture_active = self.IsPinchGestureActive();
        commit_data.is_scroll_active = self.WithInputHandler(context, |input, host| {
            input.is_some_and(|input| input.IsCurrentlyScrolling(host))
        });
        // Source pointer-to-bool conversion passes whether next BMF is supplied.
        commit_data.top_controls_delta = self
            .active_tree_
            .as_mut()
            .unwrap()
            .top_controls_shown_ratio_mut()
            .PullDeltaForMainThread(next_bmf);
        commit_data.bottom_controls_delta = self
            .active_tree_
            .as_mut()
            .unwrap()
            .bottom_controls_shown_ratio_mut()
            .PullDeltaForMainThread(next_bmf);
        // Source swap transfers the unique owners; erasure only changes vtable.
        commit_data.swap_promises =
            std::mem::take(&mut self.swap_promises_for_main_thread_scroll_update_)
                .into_iter()
                .map(|promise| promise as Box<dyn crate::adapter::commit_types::SwapPromise>)
                .collect();
        commit_data.ongoing_scroll_animation = context
            .services
            .MutatorHasImplOnlyScrollAnimatingElement(&self.mutator_host_);
        commit_data.is_auto_scrolling = context
            .services
            .MutatorHasImplOnlyAutoScrollAnimatingElement(&self.mutator_host_);
        if let Some(manager) = self.browser_controls_offset_manager_.as_ref() {
            commit_data.browser_controls_constraint = manager
                .PullConstraintForMainThread(&mut commit_data.browser_controls_constraint_changed);
        }
        commit_data
    }

    // Integrated by ctor owner: Source: cc/trees/layer_tree_host_impl.cc:5228-5237.
    // Source: cc/trees/layer_tree_host_impl.cc:5239-5266.
    pub fn AnimatePageScale(
        &mut self,
        monotonic_time: TimeTicks,
        context: &mut HostImplContext<'_, B>,
    ) -> bool {
        if self.page_scale_animation_.is_none() {
            return false;
        }
        let scroll_total = self.active_tree_.as_ref().unwrap().TotalScrollOffset();
        let animation = self.page_scale_animation_.as_mut().unwrap();
        if !animation.IsAnimationStarted() {
            animation.StartAnimation(monotonic_time);
        }
        let scale = animation.PageScaleFactorAtTime(monotonic_time);
        self.WithActiveTree(context, |tree, c| tree.SetPageScaleOnActiveTree(scale, c));
        let next_scroll = self
            .page_scale_animation_
            .as_ref()
            .unwrap()
            .ScrollOffsetAtTime(monotonic_time);
        self.WithViewport(context, |viewport, c| {
            viewport.ScrollByInnerFirst(next_scroll - scroll_total, c)
        });
        if self
            .page_scale_animation_
            .as_ref()
            .unwrap()
            .IsAnimationCompleteAtTime(monotonic_time)
        {
            self.page_scale_animation_ = None;
            self.SetNeedsCommit(context);
            self.RenewTreePriority(context);
            context
                .services
                .DidCompletePageScaleAnimationOnImplThread(self.delegate_);
        } else {
            self.SetNeedsOneBeginImplFrame(context);
        }
        true
    }

    // Source: cc/trees/layer_tree_host_impl.cc:5268-5305.
    pub fn AnimateBrowserControls(
        &mut self,
        time: TimeTicks,
        context: &mut HostImplContext<'_, B>,
    ) -> bool {
        if !self
            .browser_controls_offset_manager_
            .as_ref()
            .unwrap()
            .HasAnimation()
        {
            return false;
        }
        let scroll_delta =
            self.WithBrowserControls(context, |manager, client| manager.Animate(client, time));
        if self
            .browser_controls_offset_manager_
            .as_ref()
            .unwrap()
            .HasAnimation()
        {
            self.SetNeedsOneBeginImplFrame(context);
        }
        if self.active_tree_.as_ref().unwrap().TotalScrollOffset().y() == 0.0
            || self.OnlyExpandTopControlsAtPageTop()
        {
            return false;
        }
        if scroll_delta.IsZero() {
            return false;
        }
        self.WithViewport(context, |viewport, c| {
            viewport.ScrollBy(scroll_delta, Point::default(), false, false, true, false, c);
        });
        self.WithViewport(context, |viewport, c| viewport.SnapIfNeeded(c));
        self.SetNeedsCommit(context);
        self.RenewTreePriority(context);
        true
    }

    // Source: cc/trees/layer_tree_host_impl.cc:5307-5313.
    pub fn AnimateScrollbars(
        &mut self,
        monotonic_time: TimeTicks,
        context: &mut HostImplContext<'_, B>,
    ) -> bool {
        self.WithScrollbarAnimationControllers(context, |controllers, client| {
            let mut animated = false;
            for controller in controllers.values_mut() {
                animated |= controller.as_mut().unwrap().Animate(client, monotonic_time);
            }
            animated
        })
    }

    // Source: cc/trees/layer_tree_host_impl.cc:5315-5359.
    pub fn AnimateLayers(
        &mut self,
        monotonic_time: TimeTicks,
        is_active_tree: bool,
        context: &mut HostImplContext<'_, B>,
    ) -> bool {
        let tick_result = self.WithLiveAnimation(context, |animation, owner, events, target| {
            animation.MutatorTickAnimations(owner, monotonic_time, is_active_tree, events, target)
        });
        use HostAnimationSequence::*;
        if tick_result.needs_next_frame {
            self.SetNeedsOneBeginImplFrame(context);
            context
                .services
                .TrackersStartSequence(&mut self.frame_trackers_, kCompositorAnimation);
            if context
                .services
                .MutatorHasInvalidationAnimation(&self.mutator_host_)
            {
                context
                    .services
                    .TrackersStartSequence(&mut self.frame_trackers_, kCompositorRasterAnimation);
            } else {
                context
                    .services
                    .TrackersStopSequence(&mut self.frame_trackers_, kCompositorRasterAnimation);
            }
            if context
                .services
                .MutatorHasNativePropertyAnimation(&self.mutator_host_)
            {
                context
                    .services
                    .TrackersStartSequence(&mut self.frame_trackers_, kCompositorNativeAnimation);
            } else {
                context
                    .services
                    .TrackersStopSequence(&mut self.frame_trackers_, kCompositorNativeAnimation);
            }
        } else {
            context
                .services
                .TrackersStopSequence(&mut self.frame_trackers_, kCompositorAnimation);
            context
                .services
                .TrackersStopSequence(&mut self.frame_trackers_, kCompositorRasterAnimation);
            context
                .services
                .TrackersStopSequence(&mut self.frame_trackers_, kCompositorNativeAnimation);
        }
        if tick_result.needs_next_frame
            && context
                .services
                .MutatorHasViewTransition(&self.mutator_host_)
        {
            context
                .services
                .TrackersStartSequence(&mut self.frame_trackers_, kSETCompositorAnimation);
        } else {
            context
                .services
                .TrackersStopSequence(&mut self.frame_trackers_, kSETCompositorAnimation);
        }
        tick_result.animated
    }

    // Source: cc/trees/layer_tree_host_impl.cc:5361-5371.
    pub fn UpdateAnimationState(
        &mut self,
        start_ready_animations: bool,
        context: &mut HostImplContext<'_, B>,
    ) {
        let has_active_animations =
            self.WithLiveAnimation(context, |animation, owner, events, target| {
                animation.MutatorUpdateAnimationState(owner, start_ready_animations, events, target)
            });
        if has_active_animations {
            self.SetNeedsOneBeginImplFrame(context);
        }
        if !context
            .services
            .MutatorEventsIsEmpty(self.mutator_events_.as_deref().unwrap())
        {
            self.SetNeedsCommit(context);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5373-5387.
    pub fn ActivateAnimations(&mut self, context: &mut HostImplContext<'_, B>) {
        let activated = self.WithLiveAnimation(context, |animation, owner, events, target| {
            animation.MutatorActivateAnimations(owner, events, target)
        });
        if activated {
            self.active_tree_
                .as_mut()
                .unwrap()
                .set_needs_update_draw_properties();
            self.SetNeedsOneBeginImplFrame(context);
        }
        if !context
            .services
            .MutatorEventsIsEmpty(self.mutator_events_.as_deref().unwrap())
        {
            self.SetNeedsCommit(context);
        }
    }

    // Source: cc/trees/layer_tree_host_impl.cc:5389-5408.
    pub fn RegisterScrollbarAnimationController(
        &mut self,
        scroll_element_id: ElementId,
        scrollbar_opacity: f32,
        context: &mut HostImplContext<'_, B>,
    ) {
        RegisterScrollbarControllerForHost(
            &self.settings_,
            self.active_tree_.as_deref_mut().unwrap(),
            &mut self.scrollbar_animation_controllers_,
            &self.flashed_scrollbars_,
            &self.previously_visible_scrollable_elements_,
            self.delegate_,
            &mut self.events_metrics_manager_,
            &self.latency_info_swap_promise_monitor_,
            scroll_element_id,
            scrollbar_opacity,
            context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5410-5416.
    pub fn DidRegisterScrollbarLayer(
        &mut self,
        scroll_element_id: ElementId,
        orientation: crate::cc::input::scrollbar::ScrollbarOrientation,
        context: &mut HostImplContext<'_, B>,
    ) {
        let owner = self.AsWeakPtr();
        NotifyRegisteredScrollbarForHost(
            self.input_delegate_.as_deref_mut(),
            self.mutator_host_.as_mut(),
            owner,
            scroll_element_id,
            orientation,
            context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5418-5431.
    pub fn DidUnregisterScrollbarLayer(
        &mut self,
        scroll_element_id: ElementId,
        orientation: crate::cc::input::scrollbar::ScrollbarOrientation,
        context: &mut HostImplContext<'_, B>,
    ) {
        RemoveUnregisteredScrollbarStateForHost(
            self.active_tree_.as_deref().unwrap(),
            &mut self.scrollbar_animation_controllers_,
            &mut self.flashed_scrollbars_,
            &mut self.previously_visible_scrollable_elements_,
            &mut self.accumulated_scroll_deltas_by_element_id_,
            scroll_element_id,
        );
        let owner = self.AsWeakPtr();
        NotifyUnregisteredScrollbarForHost(
            self.input_delegate_.as_deref_mut(),
            self.mutator_host_.as_mut(),
            owner,
            scroll_element_id,
            orientation,
            context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5433-5464.
    // Rust splits the source non-owning result into a stable map identity and
    // a final borrow; both const lookup and mutable facade use this same flow.
    pub(crate) fn ScrollbarAnimationControllerElementId(
        &self,
        scroll_element_id: ElementId,
    ) -> Option<ElementId> {
        ScrollbarControllerElementIdForTree(
            self.active_tree_.as_ref().unwrap().as_ref(),
            &self.scrollbar_animation_controllers_,
            scroll_element_id,
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5433-5464 (pointer resolution).
    pub fn ScrollbarAnimationControllerForElementId(
        &self,
        id: ElementId,
    ) -> Option<&ScrollbarAnimationController> {
        self.ScrollbarAnimationControllerElementId(id)
            .and_then(|key| {
                self.scrollbar_animation_controllers_
                    .get(&key)
                    .unwrap()
                    .as_deref()
            })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5466-5472.
    pub fn OnPageScaleUpdated(&mut self, context: &mut HostImplContext<'_, B>) {
        let mut client = HostImplScrollbarClient {
            active_tree: self.active_tree_.as_deref_mut().unwrap(),
            settings: &self.settings_,
            delegate: self.delegate_,
            events: &mut self.events_metrics_manager_,
            latency_monitors: &self.latency_info_swap_promise_monitor_,
            context,
        };
        OnPageScaleUpdatedForHost(
            &self.settings_,
            &mut self.scrollbar_animation_controllers_,
            &mut self.flashed_scrollbars_,
            &mut client,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5474-5478.
    pub fn PostDelayedScrollbarAnimationTask(
        &mut self,
        task: crate::adapter::scrollbar_animation::ScrollbarAnimationTask,
        delay: TimeDelta,
        context: &mut HostImplContext<'_, B>,
    ) {
        context
            .services
            .PostDelayedAnimationTaskOnImplThread(self.delegate_, task, delay);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5480-5483.
    pub fn SetNeedsAnimateForScrollbarAnimation(&mut self, context: &mut HostImplContext<'_, B>) {
        context.services.TraceScrollbarNeedsAnimate();
        self.SetNeedsOneBeginImplFrame(context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5485-5487.
    pub fn SetNeedsRedrawForScrollbarAnimation(&mut self, context: &mut HostImplContext<'_, B>) {
        self.SetNeedsRedraw(true, false, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5489-5491.
    pub fn ScrollbarsFor(&self, id: ElementId) -> ScrollbarSet {
        self.active_tree_.as_ref().unwrap().ScrollbarsFor(id)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5493-5495.
    pub fn IsFluentOverlayScrollbar(&self) -> bool {
        self.settings_.enable_fluent_overlay_scrollbar
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5497-5508.
    pub fn AddVideoFrameController(
        &mut self,
        controller: HostImplVideoFrameControllerId,
        context: &mut HostImplContext<'_, B>,
    ) {
        let was_empty = self.video_frame_controllers_.is_empty();
        self.video_frame_controllers_.insert(controller);
        if self
            .current_begin_frame_tracker_
            .DangerousMethodHasStarted()
            && !self
                .current_begin_frame_tracker_
                .DangerousMethodHasFinished()
        {
            context.services.VideoFrameControllerOnBeginFrame(
                controller,
                self.current_begin_frame_tracker_.Current(),
            );
        }
        if was_empty {
            context
                .services
                .SetVideoNeedsBeginFrames(self.delegate_, true);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5510-5516.
    pub fn RemoveVideoFrameController(
        &mut self,
        controller: HostImplVideoFrameControllerId,
        context: &mut HostImplContext<'_, B>,
    ) {
        self.video_frame_controllers_.remove(&controller);
        if self.video_frame_controllers_.is_empty() {
            context
                .services
                .SetVideoNeedsBeginFrames(self.delegate_, false);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5518-5520.
    pub fn GetTreePriority(
        &self,
        context: &mut HostImplContext<'_, B>,
    ) -> crate::adapter::scheduler::TreePriority {
        context
            .services
            .GlobalTileStateTreePriority(&self.global_tile_state_)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5522-5526.
    pub fn CurrentBeginFrameArgs(&self) -> &crate::adapter::begin_frame::BeginFrameArgs {
        self.current_begin_frame_tracker_
            .DangerousMethodCurrentOrLast()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5528-5530.
    pub fn CurrentBeginFrameInterval(&self) -> TimeDelta {
        self.current_begin_frame_tracker_.Interval()
    }

    // Source: cc/trees/layer_tree_host_impl.cc:5532-5538.
    pub fn AsValueWithFrame(
        &mut self,
        frame: Option<&FrameData<B>>,
        context: &mut HostImplContext<'_, B>,
    ) -> Box<B::HostTraceValue> {
        let mut state = context.services.NewHostTraceValue();
        self.AsValueWithFrameInto(frame, &mut state, context);
        state
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5540-5582.
    pub fn AsValueWithFrameInto(
        &mut self,
        frame: Option<&FrameData<B>>,
        state: &mut B::HostTraceValue,
        context: &mut HostImplContext<'_, B>,
    ) {
        use crate::adapter::geometry::TracedValue;
        if self.pending_tree_.is_some() {
            state.BeginDictionary("activation_state");
            self.ActivationStateAsValueInto(state, context);
            state.EndDictionary();
        }
        let viewport = self
            .active_tree_
            .as_ref()
            .unwrap()
            .GetDeviceViewportWithExternal(self.external_viewport_);
        let size = Size::new(viewport.width, viewport.height);
        crate::cc::base::math_util::MathUtil::AddToTracedValue(
            "device_viewport_size",
            &size,
            state,
        );
        let mut prioritized_tiles = Vec::new();
        self.WithActiveTree(context, |tree, c| {
            tree.GetAllPrioritizedTilesForTracing(&mut prioritized_tiles, c)
        });
        if self.pending_tree_.is_some() {
            self.WithPendingTree(context, |tree, c| {
                tree.GetAllPrioritizedTilesForTracing(&mut prioritized_tiles, c)
            });
        }
        state.BeginArray("active_tiles");
        for tile in &prioritized_tiles {
            state.BeginDictionaryUnnamed();
            B::TracePrioritizedTile(
                tile,
                [
                    self.active_tree_.as_deref(),
                    self.pending_tree_.as_deref(),
                    self.recycle_tree_.as_deref(),
                ],
                state,
                context.services,
            );
            state.EndDictionary();
        }
        state.EndArray();
        state.BeginDictionary("tile_manager_basic_state");
        context
            .services
            .TileManagerBasicStateAsValueInto(&self.tile_manager_, state);
        state.EndDictionary();
        state.BeginDictionary("active_tree");
        self.WithActiveTree(context, |tree, c| tree.AsValueInto(state, c));
        state.EndDictionary();
        if self.pending_tree_.is_some() {
            state.BeginDictionary("pending_tree");
            self.WithPendingTree(context, |tree, c| tree.AsValueInto(state, c));
            state.EndDictionary();
        }
        if let Some(frame) = frame {
            state.BeginDictionary("frame");
            frame.AsValueInto(
                &mut HostFrameTraceValue(state),
                &HostFrameTraceServices::<B>(context.services),
            );
            state.EndDictionary();
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5584-5590.
    pub fn ActivationStateAsValueInto(
        &self,
        state: &mut B::HostTraceValue,
        context: &mut HostImplContext<'_, B>,
    ) {
        use crate::adapter::geometry::TracedValue;
        context
            .services
            .TraceSetHostIdRef(self.owner_id_, state, "lthi");
        state.BeginDictionary("tile_manager");
        context
            .services
            .TileManagerBasicStateAsValueInto(&self.tile_manager_, state);
        state.EndDictionary();
    }
    // Integrated by ctor owner: Source: cc/trees/layer_tree_host_impl.cc:5592-5609.
}

use crate::adapter::{
    geometry::{self, PointF},
    input_handler::InputLayerId,
    scrollbar::ScrollbarLayerAccess,
};
use crate::cc::input::{
    scrollbar::{ScrollbarOrientation, ScrollbarPart},
    scrollbar_animation_controller::ScrollbarAnimationControllerClient,
};

/// Borrow-only live client. The map and controllers are separately borrowed;
/// no child stores this object or any route back to its canonical owner.
struct HostImplScrollbarClient<'a, 'c, B: HostImplTypes> {
    active_tree: &'a mut LayerTreeImpl<B>,
    settings: &'a LayerTreeSettings,
    delegate: HostImplDelegateId,
    events: &'a mut B::EventsMetricsManager,
    latency_monitors: &'a BTreeSet<HostImplLatencyMonitorId>,
    context: &'a mut HostImplContext<'c, B>,
}
impl<B: HostImplTypes> HostImplScrollbarClient<'_, '_, B> {
    fn Validate(&self, layer: &InputLayerId) {
        assert_eq!(
            self.active_tree.id, layer.tree,
            "scrollbar belongs to another resident tree"
        );
        let resident = self
            .active_tree
            .LayerById(layer.id)
            .expect("resident scrollbar layer");
        assert!(
            crate::adapter::layer_tree_impl::LayerTreeLayer::AsScrollbarLayer(resident).is_some(),
            "source scrollbar layer cast"
        );
    }
}
impl<B: HostImplTypes> ScrollbarLayerAccess for HostImplScrollbarClient<'_, '_, B> {
    type Transform = <B as LayerTreeTypes>::Transform;
    fn Scrollbar_orientation(&mut self, layer: &InputLayerId) -> ScrollbarOrientation {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_orientation(self.active_tree, layer)
    }
    fn Scrollbar_scroll_element_id(&mut self, layer: &InputLayerId) -> ElementId {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_scroll_element_id(self.active_tree, layer)
    }
    fn Scrollbar_OverlayScrollbarOpacity(&mut self, layer: &InputLayerId) -> f32 {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_OverlayScrollbarOpacity(self.active_tree, layer)
    }
    fn Scrollbar_UsesAuraOverlayAnimator(&mut self, layer: &InputLayerId) -> bool {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_UsesAuraOverlayAnimator(self.active_tree, layer)
    }
    fn Scrollbar_has_find_in_page_tickmarks(&mut self, layer: &InputLayerId) -> bool {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_has_find_in_page_tickmarks(self.active_tree, layer)
    }
    fn Scrollbar_JumpOnTrackClick(&mut self, layer: &InputLayerId) -> bool {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_JumpOnTrackClick(self.active_tree, layer)
    }
    fn Scrollbar_current_pos(&mut self, layer: &InputLayerId) -> f32 {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_current_pos(self.active_tree, layer)
    }
    fn Scrollbar_scroll_layer_length(&mut self, layer: &InputLayerId) -> f32 {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_scroll_layer_length(self.active_tree, layer)
    }
    fn Scrollbar_ComputeThumbQuadRect(&mut self, layer: &InputLayerId) -> Rect {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_ComputeThumbQuadRect(self.active_tree, layer)
    }
    fn Scrollbar_SupportsDragSnapBack(&mut self, layer: &InputLayerId) -> bool {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_SupportsDragSnapBack(self.active_tree, layer)
    }
    fn Scrollbar_ForwardTrackRect(&mut self, layer: &InputLayerId) -> Rect {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_ForwardTrackRect(self.active_tree, layer)
    }
    fn Scrollbar_BackTrackRect(&mut self, layer: &InputLayerId) -> Rect {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_BackTrackRect(self.active_tree, layer)
    }
    fn Scrollbar_BackButtonRect(&mut self, layer: &InputLayerId) -> Rect {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_BackButtonRect(self.active_tree, layer)
    }
    fn Scrollbar_ForwardButtonRect(&mut self, layer: &InputLayerId) -> Rect {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_ForwardButtonRect(self.active_tree, layer)
    }
    fn Scrollbar_ThumbThickness(&mut self, layer: &InputLayerId) -> i32 {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_ThumbThickness(self.active_tree, layer)
    }
    fn Scrollbar_ThumbLength(&mut self, layer: &InputLayerId) -> i32 {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_ThumbLength(self.active_tree, layer)
    }
    fn Scrollbar_TrackLength(&mut self, layer: &InputLayerId) -> f32 {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_TrackLength(self.active_tree, layer)
    }
    fn Scrollbar_ScreenSpaceTransform(&mut self, layer: &InputLayerId) -> Self::Transform {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_ScreenSpaceTransform(self.active_tree, layer)
    }
    fn Scrollbar_IdentifyScrollbarPart(
        &mut self,
        layer: &InputLayerId,
        position: PointF,
    ) -> ScrollbarPart {
        self.Validate(layer);
        self.context.services.Layer_Scrollbar_IdentifyScrollbarPart(
            self.active_tree,
            layer,
            position,
        )
    }
    fn Scrollbar_is_overlay_scrollbar(&mut self, layer: &InputLayerId) -> bool {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_is_overlay_scrollbar(self.active_tree, layer)
    }
    fn Scrollbar_bounds(&mut self, layer: &InputLayerId) -> geometry::Size {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_bounds(self.active_tree, layer)
    }
    fn Scrollbar_device_scale_factor(&mut self, layer: &InputLayerId) -> f32 {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_device_scale_factor(self.active_tree, layer)
    }
    fn Scrollbar_ComputeHitTestableExpandedThumbQuadRect(&mut self, layer: &InputLayerId) -> Rect {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_ComputeHitTestableExpandedThumbQuadRect(self.active_tree, layer)
    }
    fn Scrollbar_thumb_thickness_scale_factor(&mut self, layer: &InputLayerId) -> f32 {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_thumb_thickness_scale_factor(self.active_tree, layer)
    }
    fn Scrollbar_SetThumbThicknessScaleFactor(&mut self, layer: &InputLayerId, scale: f32) {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_SetThumbThicknessScaleFactor(self.active_tree, layer, scale)
    }
    fn Scrollbar_CanScrollOrientation(&mut self, layer: &InputLayerId) -> bool {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_CanScrollOrientation(self.active_tree, layer)
    }
    fn Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
        &mut self,
        layer: &InputLayerId,
        opacity: f32,
        fade_out_animation: bool,
    ) {
        self.Validate(layer);
        self.context
            .services
            .Layer_Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
                self.active_tree,
                layer,
                opacity,
                fade_out_animation,
            )
    }
}
impl<B: HostImplTypes> ScrollbarAnimationControllerClient for HostImplScrollbarClient<'_, '_, B> {
    // Source forwarding: cc/trees/layer_tree_host_impl.cc:5474-5478.
    fn PostDelayedScrollbarAnimationTask(
        &mut self,
        task: crate::adapter::scrollbar_animation::ScrollbarAnimationTask,
        delay: TimeDelta,
    ) {
        self.context
            .services
            .PostDelayedAnimationTaskOnImplThread(self.delegate, task, delay);
    }
    // Source forwarding: cc/trees/layer_tree_host_impl.cc:5485-5487.
    fn SetNeedsRedrawForScrollbarAnimation(&mut self) {
        let promises = RequestRedrawForHost(
            self.settings,
            self.delegate,
            self.latency_monitors,
            self.events,
            true,
            false,
            self.context,
        );
        QueuePinnedLatencyPromisesOnActiveTree(self.active_tree, promises);
    }
    // Source forwarding: cc/trees/layer_tree_host_impl.cc:5480-5483.
    fn SetNeedsAnimateForScrollbarAnimation(&mut self) {
        self.context.services.TraceScrollbarNeedsAnimate();
        let promises = RequestBeginImplFrameForHost(
            self.delegate,
            self.latency_monitors,
            self.events,
            self.context,
        );
        QueuePinnedLatencyPromisesOnActiveTree(self.active_tree, promises);
    }
    // Source forwarding: cc/trees/layer_tree_host_impl.cc:4534-4539.
    fn DidChangeScrollbarVisibility(&mut self) {
        RequestCommitForHost(
            self.settings,
            self.delegate,
            B::OtherBeginMainFrameReason(),
            self.context,
        );
    }
    // Source forwarding: cc/trees/layer_tree_host_impl.cc:5489-5491.
    fn ScrollbarsFor(&mut self, id: ElementId) -> Vec<InputLayerId> {
        self.active_tree
            .ScrollbarsFor(id)
            .iter()
            .map(|layer| InputLayerId {
                tree: self.active_tree.id,
                id: layer.layer_id,
            })
            .collect()
    }
    // Source forwarding: cc/trees/layer_tree_host_impl.cc:5493-5495.
    fn IsFluentOverlayScrollbar(&mut self) -> bool {
        self.settings.enable_fluent_overlay_scrollbar
    }
    fn animation_controller_id(&self, element: ElementId) -> u64 {
        element.GetInternalValue()
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    fn WithScrollbarAnimationClient<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            &LayerTreeSettings,
            &mut dyn ScrollbarAnimationControllerClient<
                Transform = <B as LayerTreeTypes>::Transform,
            >,
        ) -> R,
    ) -> R {
        let mut client = HostImplScrollbarClient {
            active_tree: self.active_tree_.as_mut().unwrap(),
            settings: &self.settings_,
            delegate: self.delegate_,
            events: &mut self.events_metrics_manager_,
            latency_monitors: &self.latency_info_swap_promise_monitor_,
            context,
        };
        callback(&self.settings_, &mut client)
    }
    fn WithScrollbarAnimationControllers<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
            &mut dyn ScrollbarAnimationControllerClient<
                Transform = <B as LayerTreeTypes>::Transform,
            >,
        ) -> R,
    ) -> R {
        let mut client = HostImplScrollbarClient {
            active_tree: self.active_tree_.as_mut().unwrap(),
            settings: &self.settings_,
            delegate: self.delegate_,
            events: &mut self.events_metrics_manager_,
            latency_monitors: &self.latency_info_swap_promise_monitor_,
            context,
        };
        callback(&mut self.scrollbar_animation_controllers_, &mut client)
    }
    fn WithScrollbarAnimationController<R>(
        &mut self,
        element: ElementId,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            Option<&mut ScrollbarAnimationController>,
            &mut dyn ScrollbarAnimationControllerClient<
                Transform = <B as LayerTreeTypes>::Transform,
            >,
        ) -> R,
    ) -> R {
        let key = self.ScrollbarAnimationControllerElementId(element);
        let mut client = HostImplScrollbarClient {
            active_tree: self.active_tree_.as_mut().unwrap(),
            settings: &self.settings_,
            delegate: self.delegate_,
            events: &mut self.events_metrics_manager_,
            latency_monitors: &self.latency_info_swap_promise_monitor_,
            context,
        };
        let controller = key
            .and_then(|key| self.scrollbar_animation_controllers_.get_mut(&key))
            .and_then(Option::as_deref_mut);
        callback(controller, &mut client)
    }
    fn WithScrollbarFlashControllers<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
            &mut HashSet<ElementId>,
            &mut BTreeSet<ElementId>,
            &mut dyn ScrollbarAnimationControllerClient<
                Transform = <B as LayerTreeTypes>::Transform,
            >,
        ) -> R,
    ) -> R {
        let mut client = HostImplScrollbarClient {
            active_tree: self.active_tree_.as_mut().unwrap(),
            settings: &self.settings_,
            delegate: self.delegate_,
            events: &mut self.events_metrics_manager_,
            latency_monitors: &self.latency_info_swap_promise_monitor_,
            context,
        };
        callback(
            &mut self.scrollbar_animation_controllers_,
            &mut self.flashed_scrollbars_,
            &mut self.previously_visible_scrollable_elements_,
            &mut client,
        )
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:5951-5956.
// Rust borrow split: one native body, shared by the owner and child clients.
fn NotifyLatencyMonitorsForHost<B: HostImplTypes>(
    monitors: &BTreeSet<HostImplLatencyMonitorId>,
    context: &mut HostImplContext<'_, B>,
) -> Vec<Box<B::SwapPromise>> {
    NotifyLatencyMonitorsWithPictureServices(monitors, &mut HostFullPictureRedraw(context.services))
}

fn QueuePinnedLatencyPromisesOnActiveTree<B: HostImplTypes>(
    active: &mut LayerTreeImpl<B>,
    promises: Vec<Box<B::SwapPromise>>,
) {
    for promise in promises {
        active.QueuePinnedSwapPromiseFromActiveHost(promise);
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:3954-3958.
fn RequestCommitForHost<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    delegate: HostImplDelegateId,
    reason: <B as InputHostTypes>::BeginMainFrameReason,
    context: &mut HostImplContext<'_, B>,
) {
    if !settings.trees_in_viz_in_viz_process {
        context
            .services
            .DelegateSetNeedsCommitOnImplThread(delegate, reason);
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:4268-4272.
fn RequestBeginImplFrameForHost<B: HostImplTypes>(
    delegate: HostImplDelegateId,
    monitors: &BTreeSet<HostImplLatencyMonitorId>,
    events: &mut B::EventsMetricsManager,
    context: &mut HostImplContext<'_, B>,
) -> Vec<Box<B::SwapPromise>> {
    let promises = NotifyLatencyMonitorsForHost(monitors, context);
    context.services.SaveActiveEventMetrics(events);
    context
        .services
        .DelegateSetNeedsOneBeginImplFrameOnImplThread(delegate);
    promises
}

// Source: cc/trees/layer_tree_host_impl.cc:4274-4290.
fn RequestRedrawForHost<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    delegate: HostImplDelegateId,
    monitors: &BTreeSet<HostImplLatencyMonitorId>,
    events: &mut B::EventsMetricsManager,
    animation_only: bool,
    skip_if_inside_draw: bool,
    context: &mut HostImplContext<'_, B>,
) -> Vec<Box<B::SwapPromise>> {
    RequestRedrawWithPictureServices::<B>(
        settings,
        delegate,
        monitors,
        events,
        animation_only,
        skip_if_inside_draw,
        &mut HostFullPictureRedraw(context.services),
    )
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:3980-3985.
    pub fn IsPinchGestureActive(&self) -> bool {
        if self.input_delegate_.is_none() {
            return false;
        }
        self.GetInputHandler().pinch_gesture_active()
    }
}

use crate::adapter::{layer_tree_impl::ElementListType, presentation::SuccessfulCallback};
use crate::cc::trees::{
    layer_tree_impl::LayerTreeContext, property_animation_state::PropertyAnimationState,
    target_property::PropertyToElementIdMap,
};
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:5979-5990.
    pub fn IsElementInPropertyTrees(
        &self,
        element_id: ElementId,
        list_type: ElementListType,
    ) -> bool {
        if list_type == ElementListType::ACTIVE {
            return self
                .active_tree_
                .as_ref()
                .is_some_and(|t| t.IsElementInPropertyTree(element_id));
        }
        self.pending_tree_
            .as_ref()
            .is_some_and(|t| t.IsElementInPropertyTree(element_id))
            || self
                .recycle_tree_
                .as_ref()
                .is_some_and(|t| t.IsElementInPropertyTree(element_id))
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5992. Source hook has an empty body.
    pub fn SetMutatorsNeedCommit(&mut self) {}
    // Source: cc/trees/layer_tree_host_impl.cc:5994. Source hook has an empty body.
    pub fn SetMutatorsNeedRebuildPropertyTrees(&mut self) {}
    // Source: cc/trees/layer_tree_host_impl.cc:5996-6012.
    // Rust splits the resident tree from its live owner before this source helper.
    fn SetTreeLayerScrollOffsetMutated(
        element_id: ElementId,
        tree: Option<&mut LayerTreeImpl<B>>,
        scroll_offset: &PointF,
        context: &mut LayerTreeContext<'_, B>,
    ) {
        let Some(tree) = tree else {
            return;
        };
        let property_trees = tree.property_trees();
        let Some(scroll_node) = property_trees
            .scroll_tree()
            .FindNodeFromElementId(element_id)
        else {
            return;
        };
        let node_id = scroll_node.id;
        tree.OnScrollOffsetAnimated(element_id, node_id, scroll_offset, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6014-6028.
    pub fn SetElementFilterMutated(
        &mut self,
        element_id: ElementId,
        list_type: ElementListType,
        filters: &<B as LayerTreeTypes>::Filters,
        context: &mut HostImplContext<'_, B>,
    ) {
        if list_type == ElementListType::ACTIVE {
            self.WithActiveTree(context, |tree, c| {
                tree.SetFilterMutated(element_id, filters, c)
            });
        } else {
            if self.pending_tree_.is_some() {
                self.WithPendingTree(context, |tree, c| {
                    tree.SetFilterMutated(element_id, filters, c)
                });
            }
            if self.recycle_tree_.is_some() {
                self.WithRecycleTree(context, |tree, c| {
                    tree.SetFilterMutated(element_id, filters, c)
                });
            }
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6030-6035.
    pub fn OnCustomPropertyMutated(
        &mut self,
        property_key: B::PropertyKey,
        property_value: B::PropertyValue,
    ) {
        self.paint_worklet_tracker_
            .OnCustomPropertyMutated(property_key, property_value);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6037-6041.
    pub fn RunsOnCurrentThread(&self, context: &mut HostImplContext<'_, B>) -> bool {
        !context.services.HasImplThread(self.task_runner_provider_)
            || context.services.IsImplThread(self.task_runner_provider_)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6043-6057.
    pub fn SetElementBackdropFilterMutated(
        &mut self,
        element_id: ElementId,
        list_type: ElementListType,
        filters: &<B as LayerTreeTypes>::Filters,
        context: &mut HostImplContext<'_, B>,
    ) {
        if list_type == ElementListType::ACTIVE {
            self.WithActiveTree(context, |tree, c| {
                tree.SetBackdropFilterMutated(element_id, filters, c)
            });
        } else {
            if self.pending_tree_.is_some() {
                self.WithPendingTree(context, |tree, c| {
                    tree.SetBackdropFilterMutated(element_id, filters, c)
                });
            }
            if self.recycle_tree_.is_some() {
                self.WithRecycleTree(context, |tree, c| {
                    tree.SetBackdropFilterMutated(element_id, filters, c)
                });
            }
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6059-6072.
    pub fn SetElementOpacityMutated(
        &mut self,
        element_id: ElementId,
        list_type: ElementListType,
        opacity: f32,
        context: &mut HostImplContext<'_, B>,
    ) {
        if list_type == ElementListType::ACTIVE {
            self.WithActiveTree(context, |tree, c| {
                tree.SetOpacityMutated(element_id, opacity, c)
            });
        } else {
            if self.pending_tree_.is_some() {
                self.WithPendingTree(context, |tree, c| {
                    tree.SetOpacityMutated(element_id, opacity, c)
                });
            }
            if self.recycle_tree_.is_some() {
                self.WithRecycleTree(context, |tree, c| {
                    tree.SetOpacityMutated(element_id, opacity, c)
                });
            }
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6074-6088.
    pub fn SetElementTransformMutated(
        &mut self,
        element_id: ElementId,
        list_type: ElementListType,
        transform: &<B as LayerTreeTypes>::Transform,
        context: &mut HostImplContext<'_, B>,
    ) {
        if list_type == ElementListType::ACTIVE {
            self.WithActiveTree(context, |tree, c| {
                tree.SetTransformMutated(element_id, transform, c)
            });
        } else {
            if self.pending_tree_.is_some() {
                self.WithPendingTree(context, |tree, c| {
                    tree.SetTransformMutated(element_id, transform, c)
                });
            }
            if self.recycle_tree_.is_some() {
                self.WithRecycleTree(context, |tree, c| {
                    tree.SetTransformMutated(element_id, transform, c)
                });
            }
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6090-6101.
    pub fn SetElementScrollOffsetMutated(
        &mut self,
        element_id: ElementId,
        list_type: ElementListType,
        offset: &PointF,
        context: &mut HostImplContext<'_, B>,
    ) {
        if list_type == ElementListType::ACTIVE {
            if self.active_tree_.is_some() {
                self.WithActiveTree(context, |tree, c| {
                    Self::SetTreeLayerScrollOffsetMutated(element_id, Some(tree), offset, c)
                });
            }
            self.ShowScrollbarsForImplScroll(element_id, context);
        } else {
            if self.pending_tree_.is_some() {
                self.WithPendingTree(context, |tree, c| {
                    Self::SetTreeLayerScrollOffsetMutated(element_id, Some(tree), offset, c)
                });
            }
            if self.recycle_tree_.is_some() {
                self.WithRecycleTree(context, |tree, c| {
                    Self::SetTreeLayerScrollOffsetMutated(element_id, Some(tree), offset, c)
                });
            }
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6103-6114.
    pub fn ElementIsAnimatingChanged(
        &mut self,
        map: &PropertyToElementIdMap,
        list_type: ElementListType,
        mask: &PropertyAnimationState,
        state: &PropertyAnimationState,
    ) {
        let tree = if list_type == ElementListType::ACTIVE {
            self.active_tree_.as_mut()
        } else {
            self.pending_tree_.as_mut()
        };
        if let Some(tree) = tree {
            if tree
                .property_trees_mut()
                .ElementIsAnimatingChanged(map, mask, state, false)
            {
                tree.set_needs_update_draw_properties();
            }
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6116-6125.
    pub fn MaximumScaleChanged(
        &mut self,
        element_id: ElementId,
        list_type: ElementListType,
        scale: f32,
    ) {
        if let Some(tree) = if list_type == ElementListType::ACTIVE {
            self.active_tree_.as_mut()
        } else {
            self.pending_tree_.as_mut()
        } {
            tree.property_trees_mut()
                .MaximumAnimationScaleChanged(element_id, scale);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6127-6131.
    pub fn ScrollOffsetAnimationFinished(
        &mut self,
        element_id: ElementId,
        context: &mut HostImplContext<'_, B>,
    ) {
        self.ScrollOffsetAnimationFinishedWithReentry(element_id, None, context);
    }
    fn ScrollOffsetAnimationFinishedWithReentry(
        &mut self,
        element_id: ElementId,
        mut selected: Option<&mut dyn crate::adapter::animation_host::ScrollAnimationReentry>,
        context: &mut HostImplContext<'_, B>,
    ) {
        if self.input_delegate_.is_some() {
            self.WithInputHandlerAndAnimation(
                context,
                selected.as_mut().map(|receiver| {
                    &mut **receiver
                        as &mut dyn crate::adapter::animation_host::ScrollAnimationReentry
                }),
                |delegate, host| {
                    delegate
                        .unwrap()
                        .ScrollOffsetAnimationFinished(host, element_id)
                },
            );
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6133-6138.
    pub fn ElasticOverscrollAnimationFinished(
        &mut self,
        id: ElementId,
        context: &mut HostImplContext<'_, B>,
    ) {
        if self.input_delegate_.is_some() {
            self.WithInputHandler(context, |delegate, host| {
                delegate
                    .unwrap()
                    .ElasticOverscrollAnimationFinished(host, id)
            });
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6140-6153.
    pub fn NotifyAnimationWorkletStateChange(
        &mut self,
        state: AnimationWorkletMutationState,
        tree_type: ElementListType,
        context: &mut HostImplContext<'_, B>,
    ) {
        context
            .services
            .NotifyAnimationWorkletStateChange(self.delegate_, state, tree_type);
        if state != AnimationWorkletMutationState::CANCELED {
            self.SetNeedsOneBeginImplFrame(context);
            if state == AnimationWorkletMutationState::COMPLETED_WITH_UPDATE
                && tree_type == ElementListType::ACTIVE
            {
                self.SetNeedsRedraw(false, false, context);
            }
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6194-6206.
    pub fn ShowScrollbarsForImplScroll(
        &mut self,
        element_id: ElementId,
        context: &mut HostImplContext<'_, B>,
    ) {
        if element_id == ElementId::default()
            || !self.MaybeFlashAllScrollbars(element_id, true, context)
        {
            return;
        }
        self.WithScrollbarAnimationController(element_id, context, |controller, client| {
            if let Some(controller) = controller {
                controller.DidScrollUpdate(client);
            }
        });
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6208-6215.
    pub fn NotifyDidPresentCompositorFrameOnImplThread(
        &mut self,
        _frame_token: u32,
        callbacks: Vec<SuccessfulCallback>,
        details: &B::HostFrameTimingDetails,
        context: &mut HostImplContext<'_, B>,
    ) {
        for callback in callbacks {
            match callback {
                SuccessfulCallback::External(callback) => callback(details.PresentationTimestamp()),
                SuccessfulCallback::FirstScroll { target, payload } => {
                    if target.IsValid() {
                        let current = self.AsWeakPtr();
                        assert_eq!(target.owner, current.owner);
                        assert!(target.alive.ptr_eq(&current.alive));
                        self.OnFirstScrollPresented(
                            payload,
                            details.PresentationTimestamp(),
                            context,
                        );
                    }
                }
            }
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6217-6220.
    pub fn AllocateLocalSurfaceId(&mut self) {
        debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
        self.child_local_surface_id_allocator_.GenerateId();
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6222-6225.
    pub fn RequestBeginFrameForAnimatedImages(&mut self, context: &mut HostImplContext<'_, B>) {
        let promises = RequestBeginFrameForAnimatedImagesForHost(
            &self.settings_,
            self.delegate_,
            &self.latency_info_swap_promise_monitor_,
            &mut self.events_metrics_manager_,
            context,
        );
        self.QueuePinnedLatencyPromises(promises, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6227-6235.
    pub fn RequestInvalidationForAnimatedImages(&mut self, context: &mut HostImplContext<'_, B>) {
        RequestInvalidationForAnimatedImagesForHost(
            &self.settings_,
            &self.impl_thread_phase_,
            self.delegate_,
            context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6237-6239.
    pub fn IsInSynchronousComposite(&self, context: &mut HostImplContext<'_, B>) -> bool {
        context.services.IsInSynchronousComposite(self.delegate_)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6241-6243.
    pub fn IsReadyToActivate(&self, context: &mut HostImplContext<'_, B>) -> bool {
        context.services.IsReadyToActivate(self.delegate_)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6245-6248.
    pub fn RequestImplSideInvalidationForRerasterTiling(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) {
        let needs_first_draw_on_activation = true;
        context
            .services
            .SetNeedsImplSideInvalidation(self.delegate_, needs_first_draw_on_activation);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6250-6255.
    pub fn RequestImplSideInvalidationForRasterInducingScroll(
        &mut self,
        id: ElementId,
        context: &mut HostImplContext<'_, B>,
    ) {
        self.pending_invalidation_raster_inducing_scrolls_
            .insert(id);
        context
            .services
            .SetNeedsImplSideInvalidation(self.delegate_, true);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6261-6294.
    pub fn ApplyFirstScrollTracking(
        &mut self,
        latency: &impl HostScrollLatency,
        frame_token: u32,
        context: &mut HostImplContext<'_, B>,
    ) {
        let mut creation_timestamp = TimeTicks::default();
        if !latency.FindLatency(
            HostScrollLatencyComponent::FirstScrollUpdateOriginal,
            &mut creation_timestamp,
        ) && !latency.FindLatency(
            HostScrollLatencyComponent::ScrollUpdateOriginal,
            &mut creation_timestamp,
        ) {
            return;
        }
        let mut callbacks: Vec<SuccessfulCallback> = Vec::new();
        let source_frame_number = self.active_tree_.as_ref().unwrap().source_frame_number();
        let payload = FirstScrollTrackingPayload {
            source_frame_number,
            event_creation: creation_timestamp,
        };
        callbacks.push(
            context
                .services
                .RouteFirstScrollCallback(self.AsWeakPtr(), payload),
        );
        self.presentation_time_callbacks_
            .RegisterCompositorThreadSuccessfulCallbacks(frame_token, callbacks);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6279-6288.
    // Native continuation of the weak callback route, called synchronously on
    // the same compositor sequence after the root validates the target identity.
    pub(crate) fn OnFirstScrollPresented(
        &mut self,
        payload: FirstScrollTrackingPayload,
        presentation_timestamp: TimeTicks,
        context: &mut HostImplContext<'_, B>,
    ) {
        self.DidObserveScrollDelay(
            payload.source_frame_number,
            presentation_timestamp - payload.event_creation,
            payload.event_creation,
            context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6307-6322.
    pub fn MaybeFlashAllScrollbars(
        &mut self,
        tracking: ElementId,
        did_scroll: bool,
        context: &mut HostImplContext<'_, B>,
    ) -> bool {
        let mut client = HostImplScrollbarClient {
            active_tree: self.active_tree_.as_deref_mut().unwrap(),
            settings: &self.settings_,
            delegate: self.delegate_,
            events: &mut self.events_metrics_manager_,
            latency_monitors: &self.latency_info_swap_promise_monitor_,
            context,
        };
        MaybeFlashAllScrollbarsForHost(
            &self.settings_,
            &mut self.scrollbar_animation_controllers_,
            &mut self.flashed_scrollbars_,
            tracking,
            did_scroll,
            &mut client,
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6324-6333.
    pub fn FlashAllScrollbars(&mut self, did_scroll: bool, context: &mut HostImplContext<'_, B>) {
        let mut client = HostImplScrollbarClient {
            active_tree: self.active_tree_.as_deref_mut().unwrap(),
            settings: &self.settings_,
            delegate: self.delegate_,
            events: &mut self.events_metrics_manager_,
            latency_monitors: &self.latency_info_swap_promise_monitor_,
            context,
        };
        FlashAllScrollbarsForHost(
            &self.settings_,
            &mut self.scrollbar_animation_controllers_,
            did_scroll,
            &mut client,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6335-6338.
    pub fn EraseFlashedScrollbars(&mut self) {
        EraseFlashedScrollbarsForHost(&self.settings_, &mut self.flashed_scrollbars_);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6340-6363.
    pub fn MaybeFlashAllScrollbarsOnce(
        &mut self,
        tracking: ElementId,
        did_scroll: bool,
        context: &mut HostImplContext<'_, B>,
    ) -> bool {
        let mut client = HostImplScrollbarClient {
            active_tree: self.active_tree_.as_deref_mut().unwrap(),
            settings: &self.settings_,
            delegate: self.delegate_,
            events: &mut self.events_metrics_manager_,
            latency_monitors: &self.latency_info_swap_promise_monitor_,
            context,
        };
        MaybeFlashAllScrollbarsOnceForHost(
            &self.settings_,
            &mut self.scrollbar_animation_controllers_,
            &mut self.flashed_scrollbars_,
            tracking,
            did_scroll,
            &mut client,
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6365-6438.
    pub fn MaybeFlashEnteredViewportScrollbars(
        &mut self,
        element_id: ElementId,
        scroll_delta: &Vector2dF,
        context: &mut HostImplContext<'_, B>,
    ) {
        self.WithInputRuntime(context, |host| {
            host.MaybeFlashEnteredViewportScrollbars(element_id, scroll_delta)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6440-6450.
    pub fn SetUnboundedFrameSink(
        &mut self,
        sink: Option<Box<<B as crate::adapter::unbounded::UnboundedTypes>::FrameSink>>,
        local_id: &<B as LayerTreeTypes>::LocalSurfaceId,
        context: &mut HostImplContext<'_, B>,
    ) {
        debug_assert!(context.services.IsImplThread(self.task_runner_provider_));
        if self.unbounded_frame_sink_handler_.is_none() {
            // Source: cc/trees/unbounded_frame_sink_handler.cc:17-21. The
            // source ctor owner query is kept at this live owner call site.
            debug_assert!(self.settings_.enable_unbounded_element);
            let client = context.services.UnboundedClientIdentity(self.owner_id_);
            self.unbounded_frame_sink_handler_ =
                Some(Box::new(UnboundedFrameSinkHandler::new(client)));
        }
        self.unbounded_frame_sink_handler_
            .as_mut()
            .unwrap()
            .SetFrameSink(sink, local_id);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6452-6463.
    pub fn SetUnboundedFrameSinkId(
        &mut self,
        frame_sink_id: &<B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
        local_id: &<B as LayerTreeTypes>::LocalSurfaceId,
        context: &mut HostImplContext<'_, B>,
    ) {
        debug_assert!(context.services.IsImplThread(self.task_runner_provider_));
        assert!(context.services.SettingsFeatures().TreesInViz());
        assert!(self
            .settings_
            .TreesInVizInClientProcess(context.services.SettingsFeatures()));
        self.unbounded_frame_sink_id_ = *frame_sink_id;
        self.unbounded_local_surface_id_ = local_id.clone();
        if let Some(layer_context) = self.layer_context_.as_mut() {
            layer_context.SetUnboundedFrameSinkId(frame_sink_id, local_id);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6465-6479.
    pub fn DismissUnboundedFrameSink(&mut self, context: &mut HostImplContext<'_, B>) {
        debug_assert!(
            context.services.IsImplThread(self.task_runner_provider_)
                || !context.services.HasImplThread(self.task_runner_provider_)
        );
        if self
            .settings_
            .TreesInVizInClientProcess(context.services.SettingsFeatures())
        {
            self.unbounded_frame_sink_id_ = context.services.InvalidFrameSinkId();
            self.unbounded_local_surface_id_ = Default::default();
            if let Some(layer_context) = self.layer_context_.as_mut() {
                layer_context.DismissUnboundedFrameSink();
            }
            return;
        }
        if let Some(handler) = self.unbounded_frame_sink_handler_.as_mut() {
            handler.DismissFrameSink();
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6481-6494.
    pub fn SetUnboundedLocalSurfaceId(
        &mut self,
        local_id: &<B as LayerTreeTypes>::LocalSurfaceId,
        context: &mut HostImplContext<'_, B>,
    ) {
        debug_assert!(context.services.IsImplThread(self.task_runner_provider_));
        if self
            .settings_
            .TreesInVizInClientProcess(context.services.SettingsFeatures())
        {
            self.unbounded_local_surface_id_ = local_id.clone();
            if let Some(layer_context) = self.layer_context_.as_mut() {
                layer_context.SetUnboundedLocalSurfaceId(local_id);
            }
            return;
        }
        if let Some(handler) = self.unbounded_frame_sink_handler_.as_mut() {
            handler.SetLocalSurfaceId(local_id);
        }
    }
}

use crate::adapter::property_types::PropertyTrace;

use crate::adapter::{
    draw_property::DrawTransform,
    geometry::{RRectF, RectF, Transform},
    layer_tree_impl::LayerTreeLayer,
};
use crate::cc::{
    base::region::Region, scheduler::scheduler::FrameSkippedReason,
    trees::damage_tracker::DamageTracker,
};
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:3493-3514.
    pub fn RequestedMSAASampleCount(&self, context: &mut HostImplContext<'_, B>) -> i32 {
        RequestedMSAASampleCountForHost(
            &self.settings_,
            || context.services.IsLowEndDevice(),
            || {
                if let Some(tree) = self.pending_tree_.as_ref() {
                    tree.device_scale_factor()
                } else {
                    self.active_tree_.as_ref().unwrap().device_scale_factor()
                }
            },
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3516-3580.
    pub fn UpdateRasterCapabilities(&mut self, context: &mut HostImplContext<'_, B>) {
        let sink = self
            .layer_tree_frame_sink_
            .expect("CHECK(layer_tree_frame_sink_)");
        self.raster_caps_ = RasterCapabilities::default();
        let context_provider = context.services.FrameSinkContextProvider(sink);
        let worker_context_provider = context.services.FrameSinkWorkerContextProvider(sink);
        assert_eq!(
            worker_context_provider.is_some(),
            context_provider.is_some()
        );
        let Some(worker) = worker_context_provider else {
            self.raster_caps_.max_texture_size = self.settings_.max_render_buffer_bounds_for_sw;
            self.raster_caps_.tile_format = SinglePlaneFormat::BGRA_8888;
            self.raster_caps_.ui_rgba_format = SinglePlaneFormat::BGRA_8888;
            return;
        };
        let scoped_lock = context.services.LockRasterContext(worker);
        self.raster_caps_.max_texture_size = scoped_lock.max_texture_size();
        self.raster_caps_.ui_rgba_format = scoped_lock.BestSupportedTextureFormat();
        self.raster_caps_.tile_overlay_candidate = self.settings_.use_gpu_memory_buffer_resources
            && scoped_lock.supports_scanout_shared_images();
        let is_graphite_enabled = scoped_lock
            .FeatureStatus(HostGpuFeatureType::GPU_FEATURE_TYPE_SKIA_GRAPHITE)
            == HostGpuFeatureStatus::kGpuFeatureStatusEnabled;
        let use_rgba_4444 = self.settings_.prefer_rgba_4444 && !is_graphite_enabled;
        if self.settings_.gpu_rasterization_disabled
            || scoped_lock
                .FeatureStatus(HostGpuFeatureType::GPU_FEATURE_TYPE_GPU_TILE_RASTERIZATION)
                != HostGpuFeatureStatus::kGpuFeatureStatusEnabled
        {
            self.raster_caps_.tile_format = if use_rgba_4444 {
                SinglePlaneFormat::RGBA_4444
            } else {
                scoped_lock.BestSupportedTextureFormat()
            };
            return;
        }
        self.raster_caps_.use_gpu_rasterization = true;
        self.raster_caps_.can_use_msaa =
            !scoped_lock.msaa_is_slow() && !scoped_lock.avoid_stencil_buffers();
        self.raster_caps_.tile_format = if use_rgba_4444 {
            SinglePlaneFormat::RGBA_4444
        } else {
            scoped_lock.BestSupportedRenderbufferFormat()
        };
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3582-3586.
    pub fn GetImageDecodeCache(
        &self,
    ) -> Option<&<B::ImageDecodeFactory as HostImplImageDecodeFactory>::ImageDecodeCache> {
        self.image_decode_cache_holder_
            .as_ref()
            .map(|holder| holder.image_decode_cache())
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3588-3594.
    pub fn next_frame_token(&self, context: &mut HostImplContext<'_, B>) -> u32 {
        if self.settings_.trees_in_viz_in_viz_process {
            debug_assert_ne!(self.next_frame_token_from_client_, 0);
            return self.next_frame_token_from_client_;
        }
        context.services.FrameTokenValue(&self.next_frame_token_)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3596-3603.
    pub fn RegisterMainThreadPresentationTimeCallbackForTesting(
        &mut self,
        frame_token: u32,
        callback: crate::adapter::presentation::Callback<B::PresentationFeedback>,
    ) {
        let mut as_vector = Vec::new();
        as_vector.push(callback);
        self.presentation_time_callbacks_
            .RegisterMainThreadCallbacks(frame_token, as_vector);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3605-3615.
    pub fn RegisterMainThreadSuccessfulPresentationTimeCallbackForTesting(
        &mut self,
        frame_token: u32,
        callback: crate::adapter::presentation::SuccessfulCallbackWithDetails<
            B::HostFrameTimingDetails,
        >,
    ) {
        let mut as_vector = Vec::new();
        as_vector.push(callback);
        self.presentation_time_callbacks_
            .RegisterMainThreadSuccessfulCallbacks(frame_token, as_vector);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3617-3625.
    pub fn RegisterCompositorThreadSuccessfulPresentationTimeCallbackForTesting(
        &mut self,
        frame_token: u32,
        callback: crate::adapter::presentation::SuccessfulCallback,
    ) {
        let mut as_vector = Vec::new();
        as_vector.push(callback);
        self.presentation_time_callbacks_
            .RegisterCompositorThreadSuccessfulCallbacks(frame_token, as_vector);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3627-3719.
    pub fn WillBeginImplFrame(
        &mut self,
        args: &BeginFrameArgs,
        context: &mut HostImplContext<'_, B>,
    ) -> bool {
        if !self.settings_.single_thread_proxy_scheduler {
            let waiting = self.WithInputHandler(context, |input, host| {
                input.is_some_and(|input| {
                    input.IsCurrentlyScrolling(host) && !input.HasQueuedInput()
                })
            });
            context
                .services
                .DelegateSetWaitingForScrollEvent(self.delegate_, waiting);
        }
        self.frame_max_scroll_delta_ = 0.0;
        self.begin_frame_time_delta_ = if self.current_begin_frame_tracker_.HasLast() {
            args.frame_time - self.current_begin_frame_tracker_.Last().frame_time
        } else {
            TimeDelta::default()
        };
        self.impl_thread_phase_ = ImplThreadPhase::INSIDE_IMPL_FRAME;
        let now = context.services.MonotonicTimeNow();
        self.current_begin_frame_tracker_
            .Start(args, now, context.services.BeginFrameTrace());
        context
            .services
            .TrackersNotifyBeginImplFrame(&mut self.frame_trackers_, args);
        context.services.ReportingSetNeedsRasterPropertiesAnimated(
            self.compositor_frame_reporting_controller_
                .as_deref_mut()
                .unwrap(),
            self.paint_worklet_tracker_
                .HasInputPropertiesAnimatedOnImpl(),
        );
        if !self.settings_.is_layer_tree_for_ui {
            context.services.DevtoolsDidBeginFrame(
                self.id_,
                args.frame_time,
                args.frame_id.sequence_number,
            );
        }
        if !self.settings_.using_synchronous_renderer_compositor {
            let upcoming_lsid = if let Some(tree) = self.pending_tree_.as_ref() {
                tree.local_surface_id_from_parent()
            } else {
                self.active_tree_
                    .as_ref()
                    .unwrap()
                    .local_surface_id_from_parent()
            };
            if context
                .services
                .LocalSurfaceIdIsNewerThan(&self.target_local_surface_id_, upcoming_lsid)
            {
                return false;
            }
        }
        if self.is_likely_to_require_a_draw_ {
            self.SetNeedsRedraw(true, false, context);
        }
        self.WithInputHandler(context, |input, host| {
            if let Some(input) = input {
                input.WillBeginImplFrame(host, args);
            }
        });
        if !self.settings_.trees_in_viz_in_viz_process
            || self
                .settings_
                .TreeAnimationsInVizInVizProcess(context.services.SettingsFeatures())
        {
            self.Animate(context);
        }
        if self.image_animation_controller_.is_some() {
            assert!(!self.settings_.trees_in_viz_in_viz_process);
            self.WithImageAnimationController(context, |controller, client| {
                controller.unwrap().WillBeginImplFrame(args, client)
            });
        }
        for controller in &self.video_frame_controllers_ {
            context
                .services
                .VideoFrameControllerOnBeginFrame(*controller, args);
        }
        let recent_frame_had_no_damage =
            self.consecutive_frame_with_damage_count_ < self.settings_.damaged_frame_limit;
        if self.settings_.enable_early_damage_check
            && recent_frame_had_no_damage
            && self.CanDraw(context)
        {
            let ok = self.WithActiveTree(context, |tree, c| {
                tree.UpdateDrawProperties(true, true, None, c)
            });
            debug_assert!(ok);
            self.WithActiveTreeDamage(context, |damage| {
                DamageTracker::UpdateDamageTracking(damage)
            });
            let has_damage = self.HasDamage(context);
            if !has_damage {
                self.UpdateAnimationState(true, context);
            }
            return has_damage;
        }
        true
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3721-3728.
    pub fn DidFinishImplFrame(
        &mut self,
        args: &BeginFrameArgs,
        context: &mut HostImplContext<'_, B>,
    ) {
        context.services.TrackersNotifyFrameEnd(
            &mut self.frame_trackers_,
            self.current_begin_frame_tracker_.Current(),
            args,
        );
        self.impl_thread_phase_ = ImplThreadPhase::IDLE;
        let now = context.services.MonotonicTimeNow();
        self.current_begin_frame_tracker_
            .Finish(now, context.services.BeginFrameTrace());
        if let Some(input) = self.input_delegate_.as_mut() {
            input.DidFinishImplFrame();
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3730-3760.
    pub fn DidNotProduceFrame(
        &mut self,
        ack: &<B as crate::adapter::frame_data::FrameDataTypes>::BeginFrameAck,
        reason: FrameSkippedReason,
        context: &mut HostImplContext<'_, B>,
    ) {
        let reason_name = match reason {
            FrameSkippedReason::kRecoverLatency => "kRecoverLatency",
            FrameSkippedReason::kNoDamage => "kNoDamage",
            FrameSkippedReason::kWaitingOnMain => "kWaitingOnMain",
            FrameSkippedReason::kDrawThrottled => "kDrawThrottled",
        };
        context.services.TraceDidNotProduceFrame(ack, reason_name);
        if let Some(sink) = self.layer_tree_frame_sink_ {
            context
                .services
                .FrameSinkDidNotProduceFrame(sink, ack, reason);
        }
        if reason == FrameSkippedReason::kNoDamage {
            context
                .services
                .DropSavedEventMetricsForNoFrameUpdate(&mut self.events_metrics_manager_);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3762-3764.
    pub fn DidChangeBeginFrameSourcePaused(
        &mut self,
        paused: bool,
        context: &mut HostImplContext<'_, B>,
    ) {
        context
            .services
            .DidChangeBeginFrameSourcePaused(self.delegate_, paused);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3766-3771.
    pub fn OnBeginImplFrameDeadline(&mut self, context: &mut HostImplContext<'_, B>) {
        if self.input_delegate_.is_none() {
            return;
        }
        self.WithInputHandler(context, |input, host| {
            input.unwrap().OnBeginImplFrameDeadline(host)
        });
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3773-3778.
    pub fn SynchronouslyInitializeAllTiles(&mut self, context: &mut HostImplContext<'_, B>) {
        B::FlushResidentTileImageTasks(&mut self.tile_manager_, context);
        context.services.SynchronousTaskGraphRunnerRunUntilIdle(
            self.single_thread_synchronous_task_graph_runner_
                .as_deref_mut()
                .unwrap(),
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3820-3930.
    pub fn BuildHitTestData(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) -> Option<HostHitTestRegionList<B>> {
        context.services.TraceBuildHitTestData();
        use HostHitTestFlags::*;
        let mut hit_test_region_list = HostHitTestRegionList::<B>::default();
        hit_test_region_list.flags = kHitTestMine | kHitTestMouse | kHitTestTouch;
        hit_test_region_list.bounds =
            self.WithActiveTree(context, |tree, c| tree.GetDeviceViewport(c));
        hit_test_region_list.transform = self.DrawTransform().clone();
        let tree = self.active_tree_.as_ref().unwrap();
        let device_scale_factor = tree.device_scale_factor();
        let mut overlapping_region = Region::<B::RegionBackend>::default();
        let mut num_iterated_layers = 0usize;
        let mut assume_overlap = false;
        for layer in tree.rbegin() {
            if let Some(surface_layer) = layer.AsHitTestSurfaceLayer() {
                if !layer.HitTestable(tree.property_trees())
                    && !HostImplHitTestLayer::<B>::draws_content(layer)
                    && !surface_layer.has_pointer_events_none()
                {
                    continue;
                }
                if !surface_layer.surface_hit_testable() || !surface_layer.surface_range_is_valid()
                {
                    if !surface_layer.has_pointer_events_none() && !assume_overlap {
                        overlapping_region.Union(&MathUtil::MapEnclosingClippedRect(
                            &LayerTreeLayer::<B>::ScreenSpaceTransform(
                                layer,
                                tree.property_trees(),
                            ),
                            Rect::new(
                                0,
                                0,
                                layer.bounds(tree.property_trees()).width(),
                                layer.bounds(tree.property_trees()).height(),
                            ),
                        ));
                    }
                    continue;
                }
                let mut content_rect = context.services.ScaleToEnclosingRect(
                    Rect::new(
                        0,
                        0,
                        layer.bounds(tree.property_trees()).width(),
                        layer.bounds(tree.property_trees()).height(),
                    ),
                    device_scale_factor,
                );
                let layer_screen_space_rect = MathUtil::MapEnclosingClippedRect(
                    &LayerTreeLayer::<B>::ScreenSpaceTransform(layer, tree.property_trees()),
                    Rect::new(
                        0,
                        0,
                        layer.bounds(tree.property_trees()).width(),
                        layer.bounds(tree.property_trees()).height(),
                    ),
                );
                let mut flags = GetFlagsForSurfaceLayer::<B>(surface_layer);
                let mut async_hit_test_reasons = kNotAsyncHitTest;
                if surface_layer.has_pointer_events_none() {
                    flags |= kHitTestIgnore;
                }
                if assume_overlap || overlapping_region.Intersects(&layer_screen_space_rect) {
                    flags |= kHitTestAsk;
                    async_hit_test_reasons |= kOverlappedRegion;
                }
                let effect_tree = tree.property_trees().effect_tree();
                let layer_hit_test_region_is_masked =
                    effect_tree.HitTestMayBeAffectedByMask(layer.effect_tree_index());
                if layer.is_clipped() || layer_hit_test_region_is_masked {
                    let layer_hit_test_region_is_rectangle = !layer_hit_test_region_is_masked
                        && LayerTreeLayer::<B>::ScreenSpaceTransform(layer, tree.property_trees())
                            .Preserves2dAxisAlignment()
                        && tree
                            .property_trees()
                            .ClippedHitTestRegionIsRectangle(layer.effect_tree_index());
                    content_rect = context
                        .services
                        .ScaleToEnclosingRect(layer.visible_layer_rect(), device_scale_factor);
                    if !layer_hit_test_region_is_rectangle {
                        flags |= kHitTestAsk;
                        async_hit_test_reasons |= kIrregularClip;
                    }
                }
                let surface_id = surface_layer.surface_range_end();
                hit_test_region_list
                    .regions
                    .push(HostHitTestRegion::<B>::default());
                PopulateHitTestRegion::<B>(
                    hit_test_region_list.regions.last_mut().unwrap(),
                    layer,
                    flags,
                    async_hit_test_reasons,
                    RRectF {
                        rect: RectF::new(
                            content_rect.x as f32,
                            content_rect.y as f32,
                            content_rect.width as f32,
                            content_rect.height as f32,
                        ),
                        ..Default::default()
                    },
                    surface_id,
                    device_scale_factor,
                    tree.property_trees(),
                );
                continue;
            }
            if !layer.HitTestable(tree.property_trees()) {
                continue;
            }
            num_iterated_layers += 1;
            if num_iterated_layers > kAssumeOverlapThreshold {
                assume_overlap = true;
            }
            if !assume_overlap {
                overlapping_region.Union(&MathUtil::MapEnclosingClippedRect(
                    &LayerTreeLayer::<B>::ScreenSpaceTransform(layer, tree.property_trees()),
                    Rect::new(
                        0,
                        0,
                        layer.bounds(tree.property_trees()).width(),
                        layer.bounds(tree.property_trees()).height(),
                    ),
                ));
            }
        }
        Some(hit_test_region_list)
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:3780-3790.
fn GetFlagsForSurfaceLayer<B: HostImplTypes>(layer: &dyn HostHitTestSurfaceLayer<B>) -> u32 {
    use HostHitTestFlags::*;
    let mut flags = kHitTestMouse | kHitTestTouch;
    if layer.surface_range_is_valid() {
        flags |= kHitTestChildSurface;
    } else {
        flags |= kHitTestMine;
    }
    flags
}
// Source: cc/trees/layer_tree_host_impl.cc:3792-3818.
fn PopulateHitTestRegion<B: HostImplTypes>(
    hit_test_region: &mut HostHitTestRegion<B>,
    layer: &B::Layer,
    flags: u32,
    async_hit_test_reasons: u32,
    rect: RRectF,
    surface_id: &B::HitTestSurfaceId,
    device_scale_factor: f32,
    trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
) {
    hit_test_region.frame_sink_id = surface_id.frame_sink_id();
    hit_test_region.flags = flags;
    hit_test_region.async_hit_test_reasons = async_hit_test_reasons;
    debug_assert_eq!(
        async_hit_test_reasons != 0,
        flags & HostHitTestFlags::kHitTestAsk != 0
    );
    hit_test_region.rect = rect;
    let mut surface_to_root_transform = LayerTreeLayer::<B>::ScreenSpaceTransform(layer, trees);
    surface_to_root_transform.Scale(1.0 / device_scale_factor, 1.0 / device_scale_factor);
    surface_to_root_transform.Flatten();
    hit_test_region.transform = surface_to_root_transform.InverseOrIdentity();
}
/// Borrowed source ImageAnimationController::Delegate. The controller owns its
/// animation state; this client only lends the actual owner fields/services.
struct HostImplImageAnimationClient<'a, 'c, B: HostImplTypes> {
    active_tree: &'a mut LayerTreeImpl<B>,
    settings: &'a LayerTreeSettings,
    phase: &'a ImplThreadPhase,
    delegate: HostImplDelegateId,
    monitors: &'a BTreeSet<HostImplLatencyMonitorId>,
    events: &'a mut B::EventsMetricsManager,
    context: &'a mut HostImplContext<'c, B>,
}
impl<B: HostImplTypes> crate::adapter::image_animation::ImageAnimationClient
    for HostImplImageAnimationClient<'_, '_, B>
{
    fn MonotonicTimeNow(&self) -> TimeTicks {
        self.context.services.MonotonicTimeNow()
    }
    fn RequestBeginFrameForAnimatedImages(&mut self) {
        let promises = RequestBeginFrameForAnimatedImagesForHost(
            self.settings,
            self.delegate,
            self.monitors,
            self.events,
            self.context,
        );
        QueuePinnedLatencyPromisesOnActiveTree(self.active_tree, promises);
    }
    fn RequestInvalidationForAnimatedImages(&mut self) {
        RequestInvalidationForAnimatedImagesForHost(
            self.settings,
            self.phase,
            self.delegate,
            self.context,
        );
    }
    fn TraceEnabled(&self, event: &'static str) -> bool {
        self.context.services.ImageAnimationTraceEnabled(event)
    }
    fn Trace(
        &mut self,
        event: &'static str,
        value: crate::adapter::image_animation::ImageAnimationTraceValue<'_>,
    ) {
        self.context.services.ImageAnimationTrace(event, value);
    }
}
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    pub(super) fn WithImageAnimationController<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            Option<&mut ImageAnimationController>,
            &mut dyn crate::adapter::image_animation::ImageAnimationClient,
        ) -> R,
    ) -> R {
        let mut client = HostImplImageAnimationClient {
            active_tree: self
                .active_tree_
                .as_deref_mut()
                .expect("source active tree"),
            settings: &self.settings_,
            phase: &self.impl_thread_phase_,
            delegate: self.delegate_,
            monitors: &self.latency_info_swap_promise_monitor_,
            events: &mut self.events_metrics_manager_,
            context,
        };
        callback(self.image_animation_controller_.as_deref_mut(), &mut client)
    }
    fn WithScrollbarOpacityTrees<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            &HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
            &mut dyn HostImplScrollbarOpacityClient<B>,
            &mut LayerTreeImpl<B>,
        ) -> R,
    ) -> R {
        let mut client = HostImplScrollbarClient {
            active_tree: self.active_tree_.as_mut().unwrap(),
            settings: &self.settings_,
            delegate: self.delegate_,
            events: &mut self.events_metrics_manager_,
            latency_monitors: &self.latency_info_swap_promise_monitor_,
            context,
        };
        callback(
            &self.scrollbar_animation_controllers_,
            &mut client,
            self.pending_tree_.as_deref_mut().unwrap(),
        )
    }
}
/// Narrow live view required by source PushScrollbarOpacitiesFromActiveToPending.
pub trait HostImplScrollbarOpacityClient<B: HostImplTypes>:
    ScrollbarAnimationControllerClient<Transform = <B as LayerTreeTypes>::Transform>
{
    fn ActivePropertyTrees(&self) -> &crate::adapter::layer_tree_impl::PropertyTreesFor<B>;
    fn Scrollbar_element_id(
        &mut self,
        layer: &crate::adapter::input_handler::InputLayerId,
    ) -> ElementId;
}
impl<B: HostImplTypes> HostImplScrollbarOpacityClient<B> for HostImplScrollbarClient<'_, '_, B> {
    fn ActivePropertyTrees(&self) -> &crate::adapter::layer_tree_impl::PropertyTreesFor<B> {
        self.active_tree.property_trees()
    }
    fn Scrollbar_element_id(
        &mut self,
        layer: &crate::adapter::input_handler::InputLayerId,
    ) -> ElementId {
        use crate::adapter::layer_tree_impl::LayerTreeLayer;
        assert_eq!(layer.tree, self.active_tree.id);
        self.active_tree
            .LayerById(layer.id)
            .expect("resident scrollbar layer")
            .element_id()
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:6222-6225.
// Single canonical body, shared across the explicit owner/child-client borrows.
fn RequestBeginFrameForAnimatedImagesForHost<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    delegate: HostImplDelegateId,
    monitors: &BTreeSet<HostImplLatencyMonitorId>,
    events: &mut B::EventsMetricsManager,
    context: &mut HostImplContext<'_, B>,
) -> Vec<Box<B::SwapPromise>> {
    assert!(!settings.trees_in_viz_in_viz_process);
    RequestBeginImplFrameForHost(delegate, monitors, events, context)
}

// Source: cc/trees/layer_tree_host_impl.cc:6227-6235.
fn RequestInvalidationForAnimatedImagesForHost<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    phase: &ImplThreadPhase,
    delegate: HostImplDelegateId,
    context: &mut HostImplContext<'_, B>,
) {
    assert!(!settings.trees_in_viz_in_viz_process);
    assert_eq!(*phase, ImplThreadPhase::INSIDE_IMPL_FRAME);
    let needs_first_draw_on_activation = true;
    context
        .services
        .SetNeedsImplSideInvalidation(delegate, needs_first_draw_on_activation);
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:3582-3586.
    pub fn GetImageDecodeCacheMutable(
        &mut self,
    ) -> Option<&mut <B::ImageDecodeFactory as HostImplImageDecodeFactory>::ImageDecodeCache> {
        self.image_decode_cache_holder_
            .as_mut()
            .map(|holder| holder.image_decode_cache_mut())
    }
}

// Integrated HostImpl source chunks B/C/G.
mod source_chunk_b {
    use super::*;
    // Source: cc/trees/layer_tree_host_impl.cc:987-1807.
    use crate::adapter::{
        damage_context::DamageContext,
        draw_property::{DrawSurface, ViewTransitionToken},
        frame_data::FrameDataTypes,
        geometry::{Rect, Transform},
        layer_tree_host_impl::*,
        layer_tree_impl::{
            Color4f, LayerTreeLayer, LayerTreeLayerList, LayerTreeLayerStorage, LayerTreeTypes,
        },
        occlusion::{OcclusionTransform, SkBlendMode},
        property_types::{MaskFilterInfo, ViewTransitionElementResourceId},
        throttle::{ThrottleContext, ThrottlePass},
    };
    use crate::cc::{
        base::region::Region,
        input::compositor_input_interfaces::InputDelegateForCompositor,
        layers::{
            append_quads_context::AppendQuadsContext,
            append_quads_data::AppendQuadsData,
            draw_mode::DrawMode,
            effect_tree_layer_list_iterator::{
                EffectTreeLayerListIterator, IteratorContext, State,
            },
        },
        scheduler::draw_result::DrawResult,
        trees::{
            damage_tracker::DamageTracker,
            frame_data::{FrameData, TrackedElementStatus},
            layer_tree_host_impl::LayerTreeHostImpl,
            layer_tree_impl::LayerTreeImpl,
            managed_memory_policy::ManagedMemoryPolicy,
            property_ids::kContentsRootPropertyNodeId,
        },
    };
    use std::collections::{BTreeMap, BTreeSet};

    // Source: cc/trees/layer_tree_host_impl.cc:987-1022.
    fn AppendQuadsToFillScreen<B: HostImplTypes>(
        target_render_pass: &mut B::RenderPass,
        root_render_surface: Option<&B::RenderSurface>,
        screen_background_color: Color4f,
        fill_region: &Region<B::RegionBackend>,
    ) {
        let Some(root) = root_render_surface else {
            return;
        };
        if screen_background_color.alpha == 0.0 {
            return;
        }
        if fill_region.IsEmpty() {
            return;
        }
        let root_target_rect = DrawSurface::<<B as LayerTreeTypes>::Transform>::content_rect(root);
        let shared_quad_state = target_render_pass.CreateAndAppendSharedQuadState();
        target_render_pass.SetSharedQuadStateAll(
            shared_quad_state,
            Default::default(),
            root_target_rect,
            root_target_rect,
            MaskFilterInfo::default(),
            None,
            screen_background_color.alpha == 1.0,
            1.0,
            SkBlendMode::kSrcOver,
            0,
            0,
            false,
        );
        for screen_space_rect in fill_region {
            let visible_screen_space_rect = screen_space_rect;
            let quad = target_render_pass.CreateAndAppendSolidColorDrawQuad();
            target_render_pass.SolidColorDrawQuadSetNew(
                quad,
                shared_quad_state,
                screen_space_rect,
                visible_screen_space_rect,
                screen_background_color,
                false,
            );
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:1024-1035.
    // Native non-owning pass pointer becomes its stable slot during this frame.
    fn FindRenderPassById<B: HostImplTypes>(
        frame: &FrameData<B>,
        id: B::RenderPassId,
    ) -> Option<HostRenderPassLocation> {
        if let Some(index) = frame.render_passes.iter().position(|pass| pass.id() == id) {
            return Some(HostRenderPassLocation::Bounded(index));
        }
        frame
            .unbounded_render_passes
            .iter()
            .position(|pass| pass.id() == id)
            .map(HostRenderPassLocation::Unbounded)
    }
    impl<B: HostImplTypes> LayerTreeHostImpl<B> {
        // Source: cc/trees/layer_tree_host_impl.cc:1037-1106.
        pub fn HasDamage(&self, context: &mut HostImplContext<'_, B>) -> bool {
            let tree = self.active_tree_.as_ref().unwrap();
            debug_assert!(!tree.needs_update_draw_properties());
            debug_assert!(self.CanDraw(context));
            if tree.HandleVisibilityChanged() {
                return true;
            }
            if !self.viewport_damage_rect_.IsEmpty() {
                return true;
            }
            if &self.last_draw_referenced_surfaces_ != tree.SurfaceRanges() {
                return true;
            }
            if &self.last_draw_local_surface_id_ != self.GetCurrentLocalSurfaceId(context) {
                return true;
            }
            if self
                .unbounded_frame_sink_handler_
                .as_ref()
                .is_some_and(|handler| handler.HasUnsubmittedLocalSurfaceId())
            {
                return true;
            }
            if self
                .last_draw_render_frame_metadata_
                .as_ref()
                .is_some_and(|metadata| {
                    metadata.primary_main_frame_item_sequence_number
                        != tree.primary_main_frame_item_sequence_number()
                })
            {
                return true;
            }
            if self.settings_.enable_unbounded_element {
                for effect_id in tree.GetRenderSurfaceList() {
                    let surface = tree.GetRenderSurface(*effect_id).unwrap();
                    if DrawSurface::<<B as LayerTreeTypes>::Transform>::IsUnbounded(
                        surface,
                        tree.property_trees(),
                        self.settings_.enable_unbounded_element,
                    ) && HostDrawSurface::<B>::GetDamageRect(surface).Intersects(DrawSurface::<
                        <B as LayerTreeTypes>::Transform,
                    >::content_rect(
                        surface
                    )) {
                        return true;
                    }
                }
            }
            let root_surface = tree.RootRenderSurface().unwrap();
            let root_surface_has_visible_damage = HostDrawSurface::<B>::GetDamageRect(root_surface)
                .Intersects(
                    DrawSurface::<<B as LayerTreeTypes>::Transform>::content_rect(root_surface),
                );
            let hud_wants_to_draw = tree.hud_layer().is_some_and(|hud| {
                hud.AsHeadsUpDisplayLayer()
                    .expect("source hud_layer subtype")
                    .IsAnimatingHUDContents()
            });
            root_surface_has_visible_damage
                || tree.property_trees().effect_tree().HasCopyRequests()
                || hud_wants_to_draw
                || tree.HasViewTransitionRequests()
        }

        // Source: cc/trees/layer_tree_host_impl.cc:1108-1585.
        pub fn CalculateRenderPasses(
            &mut self,
            frame: &mut FrameData<B>,
            expects_to_draw: bool,
            host_context: &mut HostImplContext<'_, B>,
        ) -> DrawResult {
            debug_assert!(frame.render_passes.is_empty());
            debug_assert!(self.CanDraw(host_context));
            debug_assert!(!expects_to_draw || self.settings_.trees_in_viz_in_viz_process);
            if self.active_tree_.as_ref().unwrap().LayerListIsEmpty() {
                let mut pass = <B::RenderPass as HostDrawPass<B>>::Create();
                let viewport_rect =
                    self.WithActiveTree(host_context, |tree, c| tree.GetDeviceViewport(c));
                pass.SetNew(
                    1u64.into(),
                    viewport_rect,
                    viewport_rect,
                    Default::default(),
                );
                frame.render_passes.push(pass);
                return DrawResult::kSuccess;
            }
            // Source: cc/trees/layer_tree_host_impl.cc:1125-1134.
            self.WithActiveTreeDamage(host_context, |damage| {
                DamageTracker::UpdateDamageTracking(damage);
                frame.damage_reasons = damage
                    .damage_tracker(
                        damage
                            .GetRenderSurface(kContentsRootPropertyNodeId)
                            .unwrap(),
                    )
                    .GetDamageReasons();
            });
            let mut has_damage = self.HasDamage(host_context);
            // Source: cc/trees/layer_tree_host_impl.cc:1136-1209.
            if expects_to_draw {
                let _client_damage = host_context.services.CrashKey(
                    "client_damage",
                    HostCrashKeyValue::String64(self.root_layer_damage_rect_.ToString()),
                );
                if self
                    .active_tree_
                    .as_ref()
                    .unwrap()
                    .RootRenderSurface()
                    .is_some()
                {
                    let root = self
                        .active_tree_
                        .as_ref()
                        .unwrap()
                        .RootRenderSurface()
                        .unwrap();
                    let mut viz_damage_rect = HostDrawSurface::<B>::GetDamageRect(root);
                    host_context.services.RectOutset(&mut viz_damage_rect, 2);
                    if !self.root_layer_damage_rect_.IsEmpty() {
                        let _viz_damage = host_context.services.CrashKey(
                            "viz_damage",
                            HostCrashKeyValue::String64(viz_damage_rect.ToString()),
                        );
                        let _viz_content = host_context.services.CrashKey(
                            "viz_content",
                            HostCrashKeyValue::String64(
                                DrawSurface::<<B as LayerTreeTypes>::Transform>::content_rect(root)
                                    .ToString(),
                            ),
                        );
                        let _insets = host_context.services.CrashKey(
                            "insets",
                            HostCrashKeyValue::String64(
                                host_context.services.RectInsetsString(
                                    viz_damage_rect,
                                    self.root_layer_damage_rect_,
                                ),
                            ),
                        );
                        let tree = self.active_tree_.as_ref().unwrap();
                        let _dsf = host_context.services.CrashKey(
                            "dsf",
                            HostCrashKeyValue::String32(tree.device_scale_factor().to_string()),
                        );
                        let _transform = host_context.services.CrashKey(
                            "transform",
                            HostCrashKeyValue::String256(
                                HostDrawSurface::<B>::draw_transform(root).ToString(),
                            ),
                        );
                        let _damage_reasons = host_context.services.CrashKey(
                            "damage_reasons",
                            HostCrashKeyValue::String64(
                                frame.damage_reasons.ToEnumBitmask().to_string(),
                            ),
                        );
                        let _viewport_damage = host_context.services.CrashKey(
                            "viewport_damage",
                            HostCrashKeyValue::String64(self.viewport_damage_rect_.ToString()),
                        );
                        let _has_view_transitions = host_context.services.CrashKey(
                            "has_view_transitions",
                            HostCrashKeyValue::Boolean(tree.HasViewTransitionRequests()),
                        );
                        let _root_layer_bounds = host_context.services.CrashKey(
                            "root_layer_bounds",
                            HostCrashKeyValue::String64(tree.root_layer().map_or_else(
                                || "null".to_string(),
                                |root| {
                                    format!(
                                        "{}x{}",
                                        root.bounds(tree.property_trees()).width(),
                                        root.bounds(tree.property_trees()).height()
                                    )
                                },
                            )),
                        );
                        let _surface_count = host_context.services.CrashKey(
                            "surface_count",
                            HostCrashKeyValue::Number(tree.GetRenderSurfaceList().len() as u64),
                        );
                        let (top, bottom, scale, viewport) =
                            self.WithActiveTree(host_context, |tree, c| {
                                (
                                    tree.CurrentTopControlsShownRatio(c),
                                    tree.CurrentBottomControlsShownRatio(c),
                                    tree.page_scale_factor_for_scroll(c),
                                    tree.GetDeviceViewport(c),
                                )
                            });
                        let _top_controls = host_context
                            .services
                            .CrashKey("top_controls", HostCrashKeyValue::String32(top.to_string()));
                        let _bottom_controls = host_context.services.CrashKey(
                            "bottom_controls",
                            HostCrashKeyValue::String32(bottom.to_string()),
                        );
                        let _page_scale = host_context
                            .services
                            .CrashKey("page_scale", HostCrashKeyValue::String32(scale.to_string()));
                        let _device_viewport = host_context.services.CrashKey(
                            "device_viewport",
                            HostCrashKeyValue::String64(viewport.ToString()),
                        );
                        if host_context.services.ShouldRecordSubsampledMetric(0.1) {
                            host_context.services.DumpWillBeCheck(
                                viz_damage_rect.Contains(self.root_layer_damage_rect_),
                                "crbug.com/454680865: Viz damage does not contain client damage!",
                            );
                        }
                    }
                }
                host_context.services.DumpWillBeCheck(
                    has_damage,
                    "crbug.com/454680865: Has no damage while expects_to_draw is set.",
                );
                has_damage = true;
            }
            // Source: cc/trees/layer_tree_host_impl.cc:1211-1239.
            if has_damage {
                self.consecutive_frame_with_damage_count_ += 1;
            } else {
                host_context.services.TraceEmptyDamageRect();
                frame.has_no_damage = true;
                debug_assert!(!self.resourceless_software_draw_);
                self.consecutive_frame_with_damage_count_ = 0;
                return DrawResult::kSuccess;
            }
            host_context.services.TraceCalculateRenderPassesBegin(
                frame
                    .RenderSurfaceList(self.active_tree_.as_ref().unwrap().as_ref())
                    .unwrap()
                    .len() as u64,
                self.RequiresHighResToDraw(),
            );
            self.active_tree_
                .as_mut()
                .unwrap()
                .ResetHandleVisibilityChanged();
            frame.has_view_transition_save_directive = self
                .active_tree_
                .as_ref()
                .unwrap()
                .HasViewTransitionSaveRequest();
            let mut known_resource_ids = BTreeSet::new();
            let capture_view_transition_tokens = self.WithActiveTree(host_context, |tree, c| {
                tree.GetCaptureViewTransitionTokens(c)
            });
            // Source: cc/trees/layer_tree_host_impl.cc:1241-1274.
            let render_surface_list_size = frame
                .RenderSurfaceList(self.active_tree_.as_ref().unwrap().as_ref())
                .unwrap()
                .len();
            for i in 0..render_surface_list_size {
                let surface_index = render_surface_list_size - 1 - i;
                let tree = self.active_tree_.as_ref().unwrap();
                let effect_id = frame.RenderSurfaceList(tree.as_ref()).unwrap()[surface_index];
                let surface = tree.GetRenderSurface(effect_id).unwrap();
                let is_root_surface =
                    DrawSurface::<<B as LayerTreeTypes>::Transform>::EffectTreeIndex(surface)
                        == kContentsRootPropertyNodeId;
                let should_draw_into_render_pass = is_root_surface
                    || HostDrawSurface::<B>::contributes_to_drawn_surface(surface)
                    || surface.CopyOfOutputRequired(tree.property_trees());
                let resource_id = surface.ViewTransitionElementResourceId(tree.property_trees());
                if should_draw_into_render_pass {
                    let target_list =
                        if DrawSurface::<<B as LayerTreeTypes>::Transform>::IsUnbounded(
                            surface,
                            tree.property_trees(),
                            self.settings_.enable_unbounded_element,
                        ) {
                            &mut frame.unbounded_render_passes
                        } else {
                            &mut frame.render_passes
                        };
                    let mut surface_context = crate::runtime::surface::RenderPassContext::<B> {
                        tree_id: tree.id,
                        trees: tree.property_trees(),
                        layers: tree.LayersForRead(),
                        settings: &self.settings_,
                        debug: &self.debug_state_,
                        scale: tree.device_scale_factor(),
                        services: host_context.services,
                    };
                    if surface.has_view_transition_capture_contributions() {
                        target_list.push(surface.CreateViewTransitionCaptureRenderPass(
                            &capture_view_transition_tokens,
                            &mut surface_context,
                        ));
                    }
                    target_list.push(
                        surface.CreateRenderPass(
                            &capture_view_transition_tokens,
                            &mut surface_context,
                        ),
                    );
                }
                if resource_id.IsValid() {
                    known_resource_ids.insert(resource_id);
                }
            }
            // Source: cc/trees/layer_tree_host_impl.cc:1276-1327.
            if self.active_tree_.as_ref().unwrap().hud_layer().is_some() {
                let root_pass = frame.render_passes.last_mut().unwrap();
                root_pass.set_damage_rect(root_pass.output_rect());
            }
            // These native queries read immutable settings/owner scalars. The
            // iterator retains only its source position; each operation below
            // reborrows the resident tree for exactly the source call lifetime.
            let draw_mode = self.GetDrawMode(host_context);
            let commits_to_active_tree = self.CommitsToActiveTree();
            #[cfg(debug_assertions)]
            let debug_viewport_origin =
                self.WithActiveTree(host_context, |tree, c| tree.GetDeviceViewport(c).origin());
            let background_color = self.active_tree_.as_ref().unwrap().background_color();
            let tree_id = self.active_tree_.as_ref().unwrap().id;
            let surface_device_scale = self.active_tree_.as_ref().unwrap().device_scale_factor();
            let mut draw_result = DrawResult::kSuccess;
            let mut num_missing_tiles = 0;
            assert!(!frame.checkerboarded_needs_record);
            frame.has_copy_requests = self
                .active_tree_
                .as_ref()
                .unwrap()
                .property_trees()
                .effect_tree()
                .HasCopyRequests();
            let mut have_missing_animated_tiles = false;
            if self.settings_.enable_compositing_based_throttling {
                self.throttle_decider_.Prepare();
            }
            let context = AppendQuadsContext::new(draw_mode, capture_view_transition_tokens, false);
            let view_transition_capture_context = AppendQuadsContext::new(
                context.draw_mode,
                context.capture_view_transition_tokens.clone(),
                true,
            );
            let output_frame_data = !self
                .settings_
                .TreesInVizInClientProcess(host_context.services.SettingsFeatures())
                || self.dump_compositor_frame_;
            let mut tracked_element_status = TrackedElementStatus::kNone;
            // Source: cc/trees/layer_tree_host_impl.cc:1329-1472.
            let mut it =
                EffectTreeLayerListIterator::new(self.active_tree_.as_ref().unwrap().as_ref());
            while it.state() != State::kEnd {
                let position = it.position(self.active_tree_.as_ref().unwrap().as_ref());
                let target_surface_id = position.target_render_surface.unwrap();
                let target_surface = self
                    .active_tree_
                    .as_ref()
                    .unwrap()
                    .property_trees()
                    .effect_tree()
                    .GetRenderSurface(target_surface_id)
                    .unwrap();
                let target_render_pass_id = target_surface.render_pass_id();
                let view_transition_capture_render_pass_id = (output_frame_data
                    && target_surface.has_view_transition_capture_contributions())
                .then(|| target_surface.view_transition_capture_render_pass_id());
                let mut append_quads_data = AppendQuadsData::<B::SurfaceId>::default();
                if it.state() == State::kTargetSurface {
                    let has_copy_request = output_frame_data
                        && target_surface
                            .HasCopyRequest(self.active_tree_.as_ref().unwrap().property_trees());
                    if has_copy_request {
                        let target = FindRenderPassById::<B>(frame, target_render_pass_id)
                            .unwrap()
                            .get_mut::<B>(frame);
                        self.active_tree_
                            .as_mut()
                            .unwrap()
                            .property_trees_mut()
                            .TakeCopyRequestsAndTransformToSurface(
                                target_surface_id,
                                target.copy_requests_mut(),
                            );
                    }
                    if self.settings_.enable_compositing_based_throttling {
                        if let Some(target) = FindRenderPassById::<B>(frame, target_render_pass_id)
                        {
                            let throttle = BorrowedFrameThrottle::<B>::new(frame);
                            self.throttle_decider_.ProcessRenderPass(&throttle, target);
                        }
                    }
                } else if it.state() == State::kContributingSurface {
                    if output_frame_data {
                        let surface_id = position.current_render_surface.unwrap();
                        let tree = self.active_tree_.as_ref().unwrap();
                        let surface = tree
                            .property_trees()
                            .effect_tree()
                            .GetRenderSurface(surface_id)
                            .unwrap();
                        if HostDrawSurface::<B>::contributes_to_drawn_surface(surface)
                            && !DrawSurface::<<B as LayerTreeTypes>::Transform>::IsUnbounded(
                                surface,
                                tree.property_trees(),
                                self.settings_.enable_unbounded_element,
                            )
                        {
                            let surface_context = crate::runtime::surface::RenderPassContext::<B> {
                                tree_id,
                                trees: tree.property_trees(),
                                layers: tree.LayersForRead(),
                                settings: &self.settings_,
                                debug: &self.debug_state_,
                                scale: surface_device_scale,
                                services: host_context.services,
                            };
                            surface.AppendQuads(
                                &context,
                                FindRenderPassById::<B>(frame, target_render_pass_id)
                                    .unwrap()
                                    .get_mut::<B>(frame),
                                &mut append_quads_data,
                                &surface_context,
                            );
                            if let Some(capture_id) = view_transition_capture_render_pass_id {
                                let mut data = AppendQuadsData::default();
                                surface.AppendQuads(
                                    &view_transition_capture_context,
                                    FindRenderPassById::<B>(frame, capture_id)
                                        .unwrap()
                                        .get_mut::<B>(frame),
                                    &mut data,
                                    &surface_context,
                                );
                            }
                        }
                    }
                } else if it.state() == State::kLayer {
                    let layer_id = position.current_layer.unwrap();
                    let (will_draw, layer_animating, checkerboarded_needs_record) =
                        self.WithActiveTree(host_context, |tree, tree_context| {
                            B::WithPictureLayersContext(
                                tree,
                                tree_context.host,
                                |storage, picture_context| {
                                    let layer = storage.LayerByIdMut(layer_id).unwrap();
                                    if layer.tracked_element_rects_empty() == Some(false) {
                                        if tracked_element_status == TrackedElementStatus::kNone {
                                            tracked_element_status =
                                                TrackedElementStatus::kHasTrackedElements;
                                        }
                                        if tracked_element_status
                                            != TrackedElementStatus::kHasTrackedElementsNeedingOcclusion
                                        {
                                            for list in layer.tracked_element_rects() {
                                                for should_exclude in list {
                                                    if should_exclude {
                                                        tracked_element_status = TrackedElementStatus::kHasTrackedElementsNeedingOcclusion;
                                                        break;
                                                    }
                                                }
                                                if tracked_element_status == TrackedElementStatus::kHasTrackedElementsNeedingOcclusion {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    let will_draw = layer.WillDraw(context.draw_mode, picture_context);
                                    if will_draw {
                                        debug_assert_eq!(tree_id, layer.layer_tree_impl_id());
                                        frame.will_draw_layers.push(layer.id());
                                        layer.NotifyKnownResourceIdsBeforeAppendQuads(&known_resource_ids);
                                        if output_frame_data {
                                            layer.AppendQuads(
                                                &context,
                                                FindRenderPassById::<B>(frame, target_render_pass_id).unwrap().get_mut::<B>(frame),
                                                &mut append_quads_data,
                                                picture_context,
                                            );
                                            if let Some(capture_id) = view_transition_capture_render_pass_id {
                                                let mut data = AppendQuadsData::default();
                                                layer.AppendQuads(
                                                    &view_transition_capture_context,
                                                    FindRenderPassById::<B>(frame, capture_id).unwrap().get_mut::<B>(frame),
                                                    &mut data,
                                                    picture_context,
                                                );
                                            }
                                        }
                                    }
                                    let layer_animating = layer.screen_space_transform_is_animating();
                                    let missing = !output_frame_data
                                        && will_draw
                                        && layer.HasMissingTiles(picture_context);
                                    let checkerboarded = context.draw_mode
                                        != DrawMode::DRAW_MODE_RESOURCELESS_SOFTWARE
                                        && layer.ComputeCheckerboardedNeedsRecord(picture_context);
                                    (will_draw, layer_animating && missing, checkerboarded)
                                },
                            )
                        });
                    have_missing_animated_tiles |= layer_animating;
                    if !will_draw && self.settings_.enable_compositing_based_throttling {
                        let layer = self
                            .active_tree_
                            .as_ref()
                            .unwrap()
                            .LayerById(layer_id)
                            .unwrap();
                        self.throttle_decider_.ProcessLayerNotToDraw(layer);
                    }
                    if output_frame_data {
                        host_context.services.AddVisibleContentArea(
                            self.rendering_stats_instrumentation_,
                            append_quads_data.visible_layer_area,
                        );
                        host_context.services.AddApproximatedVisibleContentArea(
                            self.rendering_stats_instrumentation_,
                            append_quads_data.approximated_visible_content_area,
                        );
                        num_missing_tiles += append_quads_data.num_missing_tiles;
                        if append_quads_data.num_missing_tiles > 0 {
                            have_missing_animated_tiles |= self
                                .active_tree_
                                .as_ref()
                                .unwrap()
                                .LayerById(layer_id)
                                .unwrap()
                                .screen_space_transform_is_animating();
                        }
                    }
                    if checkerboarded_needs_record {
                        frame.checkerboarded_needs_record = true;
                    }
                    if self.settings_.is_layer_tree_for_ui {
                        let tree = self.active_tree_.as_ref().unwrap();
                        AccumulateInvalidatedArea(
                            tree.LayerById(layer_id).unwrap(),
                            tree.property_trees(),
                            self.total_invalidated_area_.as_mut().unwrap(),
                        );
                    }
                }
                if output_frame_data {
                    frame
                        .activation_dependencies
                        .extend(append_quads_data.activation_dependencies.iter().cloned());
                    if let Some(deadline) = append_quads_data.deadline_in_frames {
                        frame.deadline_in_frames =
                            Some(if let Some(current) = frame.deadline_in_frames {
                                current.max(deadline)
                            } else {
                                deadline
                            });
                    }
                    frame.use_default_lower_bound_deadline |=
                        append_quads_data.use_default_lower_bound_deadline;
                    frame.has_shared_element_resources |=
                        append_quads_data.has_shared_element_resources;
                }
                it.Increment(self.active_tree_.as_ref().unwrap().as_ref());
            }
            // Source: cc/trees/layer_tree_host_impl.cc:1474-1539.
            frame.tracked_element_status = tracked_element_status;
            if have_missing_animated_tiles && !commits_to_active_tree {
                if expects_to_draw {
                    debug_assert!(
                        false,
                        "crbug.com/454680865: Has checkerboarded animations while expects_to_draw is set"
                    );
                } else {
                    draw_result = DrawResult::kAbortedCheckerboardAnimations;
                }
            }
            if frame.checkerboarded_needs_raster && self.requires_high_res_to_draw_ {
                if expects_to_draw {
                    debug_assert!(
                        false,
                        "crbug.com/454680865: Is missing high res content while expects_to_draw is set"
                    );
                } else {
                    draw_result = DrawResult::kAbortedMissingHighResContent;
                }
            }
            if self.resourceless_software_draw_ {
                draw_result = DrawResult::kSuccess;
            }
            if frame.has_view_transition_save_directive {
                draw_result = DrawResult::kSuccess;
            }
            #[cfg(debug_assertions)]
            {
                for pass in &frame.render_passes {
                    for quad in 0..pass.quad_count() {
                        debug_assert!(pass.quad_has_shared_quad_state(quad));
                    }
                }
                debug_assert!(
                    self.settings_.enable_unbounded_element
                        || frame.unbounded_render_passes.is_empty()
                );
                for pass in &frame.unbounded_render_passes {
                    for quad in 0..pass.quad_count() {
                        debug_assert!(pass.quad_has_shared_quad_state(quad));
                    }
                }
                debug_assert_eq!(
                    frame.render_passes.last().unwrap().output_rect().origin(),
                    debug_viewport_origin
                );
            }
            // Source: cc/trees/layer_tree_host_impl.cc:1540-1585.
            let has_transparent_background = background_color.alpha != 1.0;
            let tree = self.active_tree_.as_ref().unwrap();
            let root_surface = tree
                .property_trees()
                .effect_tree()
                .GetRenderSurface(kContentsRootPropertyNodeId);
            if let Some(root) = root_surface.filter(|_| !has_transparent_background) {
                frame
                    .render_passes
                    .last_mut()
                    .unwrap()
                    .set_has_transparent_background(false);
                if output_frame_data {
                    // This Region copy is explicitly present in the source here.
                    let mut fill_region = tree.UnoccludedScreenSpaceRegion().clone();
                    if num_missing_tiles > 0 {
                        fill_region = Region::new(
                            DrawSurface::<<B as LayerTreeTypes>::Transform>::content_rect(root),
                        );
                    }
                    AppendQuadsToFillScreen::<B>(
                        frame.render_passes.last_mut().unwrap(),
                        Some(root),
                        background_color,
                        &fill_region,
                    );
                }
            }
            if output_frame_data {
                self.RemoveRenderPasses(frame);
            }
            debug_assert!(!frame.render_passes.is_empty());
            if self
                .settings_
                .TreesInVizInClientProcess(host_context.services.SettingsFeatures())
            {
                host_context
                    .services
                    .TraceCalculateRenderPassesEnd(draw_result, None);
            } else {
                host_context
                    .services
                    .TraceCalculateRenderPassesEnd(draw_result, Some(num_missing_tiles));
            }
            debug_assert!(
                (!frame.has_copy_requests && !frame.has_view_transition_save_directive)
                    || draw_result == DrawResult::kSuccess
            );
            draw_result
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1587-1590.
        pub fn DidAnimateScrollOffset(&mut self, context: &mut HostImplContext<'_, B>) {
            DidAnimateScrollOffsetForHost(&self.settings_, self.delegate_, context);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1592-1594,1596-1598.
        // SetViewportDamage / SetRootLayerDamageRect already integrated by owner.
        // Source: cc/trees/layer_tree_host_impl.cc:1602-1661.
        pub fn PrepareToDraw(
            &mut self,
            frame: &mut FrameData<B>,
            expects_to_draw: bool,
            context: &mut HostImplContext<'_, B>,
        ) -> DrawResult {
            debug_assert!(!expects_to_draw || self.settings_.trees_in_viz_in_viz_process);
            context
                .services
                .TracePrepareToDraw(self.active_tree_.as_ref().unwrap().source_frame_number());
            if let Some(input) = self.input_delegate_.as_mut() {
                input.WillDraw();
            }
            context
                .services
                .TickWorkletAnimations(&mut self.mutator_host_);
            let ok = self.WithActiveTree(context, |tree, c| {
                tree.UpdateDrawProperties(true, true, None, c)
            });
            debug_assert!(ok, "UpdateDrawProperties failed during draw");
            if !self.settings_.trees_in_viz_in_viz_process {
                frame.checkerboarded_needs_raster = !B::PrepareResidentTilesToDraw(self, context);
            }
            frame.render_surface_list = Some(self.active_tree_.as_ref().unwrap().id);
            frame.render_passes.clear();
            frame.unbounded_render_passes.clear();
            frame.will_draw_layers.clear();
            frame.has_no_damage = false;
            if self
                .active_tree_
                .as_ref()
                .unwrap()
                .RootRenderSurface()
                .is_some()
            {
                let viewport_damage = self.viewport_damage_rect_;
                self.WithActiveTreeDamage(context, |damage| {
                    damage
                        .damage_tracker(
                            damage
                                .GetRenderSurface(kContentsRootPropertyNodeId)
                                .unwrap(),
                        )
                        .AddDamageNextUpdate(viewport_damage)
                });
            }
            let draw_result = self.CalculateRenderPasses(frame, expects_to_draw, context);
            context.services.VerboseLogFrame(frame);
            if draw_result != DrawResult::kSuccess {
                debug_assert!(!self.resourceless_software_draw_);
                if expects_to_draw {
                    context.services.DumpWillBeCheck(
                    false,
                    "crbug.com/454680865: Draw result is not success while expects_to_draw is set",
                );
                    return DrawResult::kSuccess;
                }
                return draw_result;
            }
            DrawResult::kSuccess
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1663-1766.
        pub fn RemoveRenderPasses(&mut self, frame: &mut FrameData<B>) {
            debug_assert!(frame.render_passes.len() >= 1);
            let mut pass_exists = BTreeSet::new();
            let mut pass_references: BTreeMap<B::RenderPassId, i32> = BTreeMap::new();
            let mut view_transition_quad_references = BTreeSet::new();
            let mut i = 0;
            while i < frame.render_passes.len() {
                let pass = &mut frame.render_passes[i];
                let mut quad = 0;
                while quad < pass.quad_count() {
                    if pass.quad_material(quad) == HostQuadMaterial::kSharedElement {
                        view_transition_quad_references
                            .insert(pass.quad_shared_element_resource_id(quad));
                        quad += 1;
                        continue;
                    }
                    if pass.quad_material(quad) != HostQuadMaterial::kCompositorRenderPass {
                        quad += 1;
                        continue;
                    }
                    let id = pass.quad_render_pass_id(quad).unwrap();
                    if pass_exists.contains(&id) {
                        *pass_references.entry(id).or_default() += 1;
                        quad += 1;
                    } else {
                        pass.EraseQuadAndInvalidateAllPointers(quad);
                    }
                }
                if i == frame.render_passes.len() - 1 {
                    break;
                }
                let pass = &frame.render_passes[i];
                if pass.quad_count() == 0
                    && pass.copy_requests().is_empty()
                    && !pass.subtree_capture_id_is_valid()
                    && pass.filters_is_empty()
                    && pass.backdrop_filters_is_empty()
                    && !pass.view_transition_element_resource_id().IsValid()
                {
                    frame.render_passes.remove(i);
                    continue;
                }
                pass_exists.insert(pass.id());
                i += 1;
            }
            i = 0;
            while i < frame.render_passes.len() - 1 {
                let index = frame.render_passes.len() - 2 - i;
                let pass = &frame.render_passes[index];
                if !pass.copy_requests().is_empty() {
                    i += 1;
                    continue;
                }
                if *pass_references.entry(pass.id()).or_default() != 0 {
                    i += 1;
                    continue;
                }
                if pass.view_transition_element_resource_id().IsValid()
                    && (frame.has_view_transition_save_directive
                        || view_transition_quad_references
                            .contains(&pass.view_transition_element_resource_id()))
                {
                    i += 1;
                    continue;
                }
                for quad in 0..pass.quad_count() {
                    if let Some(id) = pass.quad_render_pass_id(quad) {
                        *pass_references.entry(id).or_default() -= 1;
                    }
                }
                frame.render_passes.remove(index);
                // Source --i then for increment leaves i unchanged after erasure.
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1768-1770.
        pub fn EvictTexturesForTesting(&mut self, context: &mut HostImplContext<'_, B>) {
            self.UpdateTileManagerMemoryPolicy(ManagedMemoryPolicy::new(0), context);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1772-1776.
        pub fn BlockNotifyReadyToActivateForTesting(
            &mut self,
            _block: bool,
            _notify_if_blocked: bool,
        ) {
            panic!("NOTREACHED");
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1778-1781.
        pub fn BlockImplSideInvalidationRequestsForTesting(&mut self, _block: bool) {
            panic!("NOTREACHED");
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1783-1801.
        pub fn ResetTreesForTesting(&mut self, context: &mut HostImplContext<'_, B>) {
            if let Some(tree) = self.active_tree_.as_mut() {
                drop(tree.DetachLayers());
            }
            let tree = self.active_tree_.as_ref().unwrap();
            let page_scale = tree.SharedPageScaleFactor();
            let top_controls = tree.SharedTopControlsShownRatio();
            let bottom_controls = tree.SharedBottomControlsShownRatio();
            let (id, property_owner) = context.services.AllocateLayerTreeIdentity();
            let replacement = Box::new(LayerTreeImpl::<B>::new(
                id,
                property_owner,
                self.owner_id_,
                self.CurrentBeginFrameArgs().clone(),
                page_scale,
                top_controls,
                bottom_controls,
            ));
            let old = self.active_tree_.replace(replacement);
            if let Some(mut tree) = old {
                tree.ShutdownWithHost(
                    self.owner_id_,
                    |other| other == id,
                    || context.services.MonotonicTimeNow(),
                    drop,
                );
            }
            self.active_tree_
                .as_mut()
                .unwrap()
                .property_trees_mut()
                .set_is_active(true);
            self.active_tree_
                .as_mut()
                .unwrap()
                .property_trees_mut()
                .clear();
            if let Some(tree) = self.pending_tree_.as_mut() {
                drop(tree.DetachLayers());
            }
            if let Some(mut tree) = self.pending_tree_.take() {
                tree.ShutdownWithHost(
                    self.owner_id_,
                    |other| other == id,
                    || context.services.MonotonicTimeNow(),
                    drop,
                );
            }
            if let Some(tree) = self.recycle_tree_.as_mut() {
                drop(tree.DetachLayers());
            }
            if let Some(mut tree) = self.recycle_tree_.take() {
                tree.ShutdownWithHost(
                    self.owner_id_,
                    |other| other == id,
                    || context.services.MonotonicTimeNow(),
                    drop,
                );
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1803.
        pub fn DidSendEarlyFinalBeginMainFrame(&mut self) {}
        // Source: cc/trees/layer_tree_host_impl.cc:1805-1807.
        pub fn SourceAnimationFrameNumberForTesting(
            &self,
            context: &mut HostImplContext<'_, B>,
        ) -> usize {
            self.next_frame_token(context) as usize
        }
    }
}

mod source_chunk_c {
    use super::*;
    // Source: cc/trees/layer_tree_host_impl.cc:1808-2395.
    // Existing canonical definitions excluded: cc:1809-1863,1865-1877.
    use crate::adapter::{
        geometry::RectF,
        image_animation::{kDefaultFrameIndex, WhichTree},
        layer_tree_impl::{LayerTreeLayer, LayerTreeLayerList},
        property_types::ViewTransitionElementResourceId,
    };
    impl<B: HostImplTypes> LayerTreeHostImpl<B> {
        // Source: cc/trees/layer_tree_host_impl.cc:1879-1885.
        pub fn SetTargetLocalSurfaceId(
            &mut self,
            id: &<B as LayerTreeTypes>::LocalSurfaceId,
            context: &mut HostImplContext<'_, B>,
        ) {
            use crate::adapter::unbounded::LocalSurfaceId;
            self.target_local_surface_id_ = id.clone();
            if let Some(layer_context) = self.layer_context_.as_mut() {
                if id.is_valid() {
                    context
                        .services
                        .LayerContextSetTargetLocalSurfaceId(layer_context, id);
                }
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1887-1899.
        pub fn BuildRasterQueue(
            &self,
            priority: crate::adapter::scheduler::TreePriority,
            queue_type: HostRasterQueueType,
            context: &mut HostImplContext<'_, B>,
        ) -> Box<dyn crate::cc::tiles::raster_tile_priority_queue::RasterTilePriorityQueue<B>>
        where
            B: crate::adapter::picture_layer_impl::PictureLayerImplTypes<PictureTargetColorParams=HostTargetColorParams<B::ColorSpace>>
                +LayerTreeTypes<PictureLayer=crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,GlobalTileState=crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority>,
            <<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList:HostDisplayItemList,
        {
            context.services.TraceHostImplCEvent(
                "disabled-by-default-cc.debug",
                "LayerTreeHostImpl::BuildRasterQueue",
            );
            self.WithRasterQueueContext(context,|queries| {
                let active=HostPictureQueueLayers::new(queries.active.as_ref());
                let pending=HostPictureQueueLayers::new(queries.pending.as_ref().filter(|_|self.pending_tree_fully_painted_));
                <dyn crate::cc::tiles::raster_tile_priority_queue::RasterTilePriorityQueue<B>>::Create(active,pending,priority,queue_type,queries)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1901-1912.
        pub fn BuildEvictionQueue(
            &self,
            context: &mut HostImplContext<'_, B>,
        ) -> Box<crate::cc::tiles::eviction_tile_priority_queue::EvictionTilePriorityQueue<B>>
        where
            B: crate::adapter::picture_layer_impl::PictureLayerImplTypes<PictureTargetColorParams=HostTargetColorParams<B::ColorSpace>>
                +LayerTreeTypes<PictureLayer=crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,GlobalTileState=crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority>,
            <<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList:HostDisplayItemList,
        {
            context.services.TraceHostImplCEvent(
                "disabled-by-default-cc.debug",
                "LayerTreeHostImpl::BuildEvictionQueue",
            );
            let mut queue=Box::new(crate::cc::tiles::eviction_tile_priority_queue::EvictionTilePriorityQueue::new());
            self.WithRasterQueueContext(context, |queries| {
                queue.Build(
                    HostPictureQueueLayers::new(queries.active.as_ref()),
                    HostPictureQueueLayers::new(queries.pending.as_ref()),
                    queries,
                );
            });
            queue
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1914-1920.
        pub fn CreateTilesWithResourceIterator(
            &mut self,
        ) -> crate::cc::tiles::tiles_with_resource_iterator::LayerResourceTileTraversal<'_,B>
        where
            B: crate::adapter::picture_layer_impl::PictureLayerImplTypes
                + LayerTreeTypes<PictureLayer=crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>>,
        {
            // The source range contains pointers to these actual resident
            // pointees. The lending visitor keeps each mutable borrow scoped
            // to its source FindNext* traversal instead of retaining a pointer.
            crate::cc::tiles::tiles_with_resource_iterator::LayerResourceTileTraversal::FromResidentLayers(
                self.active_tree_.as_mut().unwrap().LayersForMutable(),
                self.pending_tree_.as_mut().map(|tree|tree.LayersForMutable()),
            )
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1922-1932.
        pub fn GetDisplayColorSpaces(&self) -> B::DisplayColorSpaces {
            if let Some(pending) = self.pending_tree_.as_ref() {
                return pending.display_color_spaces().clone();
            }
            if let Some(active) = self.active_tree_.as_ref() {
                return active.display_color_spaces().clone();
            }
            Default::default()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1934-1941.
        pub fn SetIsLikelyToRequireADraw(&mut self, value: bool) {
            self.is_likely_to_require_a_draw_ = value;
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1943-1945.
        pub fn GetTileFormat(&self) -> crate::adapter::resource_types::SharedImageFormat {
            self.raster_caps_.tile_format
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1947-1968.
        pub fn GetTargetColorParams(
            &self,
            usage: ContentColorUsage,
            context: &mut HostImplContext<'_, B>,
        ) -> HostTargetColorParams<B::ColorSpace> {
            TargetColorParamsForHost::<B>(
                self.layer_tree_frame_sink_,
                usage,
                context.services,
                || self.GetDisplayColorSpaces(),
            )
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1970-1988.
        // The source const method owns a mutable LRU. Rust takes the live mutable
        // owner explicitly; no interior-mutable wrapper or copied cache is needed.
        pub fn CheckColorSpaceContainsSrgb(&mut self, color_space: &B::ColorSpace) -> bool {
            // Source always passes CreateSRGB(); the external value capability
            // binds that fixed argument rather than a generalized color query.
            if color_space.GetPrimaryID() != HostColorSpacePrimaryID::CUSTOM {
                return color_space.ContainsSRGB();
            }
            if let Some(result) = self.contains_srgb_cache_.Get(color_space) {
                return *result;
            }
            let result = color_space.ContainsSRGB();
            self.contains_srgb_cache_.Put(color_space.clone(), result);
            result
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1990-1995.
        pub fn RequestImplSideInvalidationForCheckerImagedTiles(
            &mut self,
            context: &mut HostImplContext<'_, B>,
        ) {
            let needs_first_draw_on_activation = false;
            context
                .services
                .SetNeedsImplSideInvalidation(self.delegate_, needs_first_draw_on_activation);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1997-2007.
        pub fn GetFrameIndexForImage(&self, image: &B::PaintImage, tree: WhichTree) -> usize {
            GetFrameIndexForImageForLiveParts(
                image,
                tree,
                &self.settings_,
                &self.image_animation_controller_,
            )
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2009-2024.
        pub fn GetMSAASampleCountForRaster(
            &self,
            display_list: &B::DisplayItemList,
            context: &mut HostImplContext<'_, B>,
        ) -> i32 {
            RasterMSAASampleCountForHost(
                display_list,
                &self.raster_caps_,
                &self.settings_,
                || context.services.IsLowEndDevice(),
                || {
                    if let Some(tree) = self.pending_tree_.as_ref() {
                        tree.device_scale_factor()
                    } else {
                        self.active_tree_.as_ref().unwrap().device_scale_factor()
                    }
                },
            )
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2026-2028.
        pub fn HasPendingTree(&self) -> bool {
            self.pending_tree_.is_some()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2030-2040.
        pub fn NotifyReadyToActivate(&mut self, context: &mut HostImplContext<'_, B>) {
            NotifyReadyToActivateForLiveParts::<B>(
                self.pending_tree_fully_painted_,
                self.delegate_,
                context.services,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2042-2049.
        pub fn NotifyReadyToDraw(&mut self, context: &mut HostImplContext<'_, B>) {
            NotifyReadyToDrawForLiveParts::<B>(
                &mut self.is_likely_to_require_a_draw_,
                self.delegate_,
                context.services,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2051-2066.
        pub fn NotifyAllTileTasksCompleted(&mut self, context: &mut HostImplContext<'_, B>) {
            NotifyAllTileTasksCompletedForLiveParts::<B>(
                &self.settings_,
                &self.global_tile_state_,
                &mut self.image_decode_cache_holder_,
                self.layer_tree_frame_sink_,
                &mut self.compositor_context_visibility_,
                &mut self.worker_context_visibility_,
                context.services,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2068-2108.
        pub fn NotifyTileStateChanged(
            &mut self,
            tile: &B::Tile,
            update_damage: bool,
            set_needs_redraw: bool,
            context: &mut HostImplContext<'_, B>,
        ) {
            debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
            context
                .services
                .TraceHostImplCEvent("cc", "LayerTreeHostImpl::NotifyTileStateChanged");
            // Rust live-borrow mapping of source tile->tiling()->tree(): the
            // non-owning tiling pointer is an id, resolved only through these
            // actual resident PictureLayerImpl owners before a mutable borrow.
            let Some(tiling_id) = tile.tiling_id() else {
                return;
            };
            let find_tree = |tree: Option<&LayerTreeImpl<B>>| {
                tree.and_then(|tree| tree.LayerById(tile.layer_id()))
                    .and_then(|layer| layer.AsPictureLayer())
                    .and_then(|picture| picture.ResidentTilingTree(tiling_id))
            };
            let tile_tree = find_tree(self.pending_tree_.as_deref())
                .or_else(|| find_tree(self.active_tree_.as_deref()));
            let Some(tile_tree) = tile_tree else {
                return;
            };
            let is_pending_tree = tile_tree == WhichTree::PENDING_TREE;
            // The source virtual Picture callback needs the same actual tree,
            // tiling and services. Complete that call before borrowing the layer
            // again for the subsequent Viz transport query.
            let mut notify = |tree: &mut LayerTreeImpl<B>, c: &mut LayerTreeContext<'_, B>| {
                B::WithPictureLayersContext(tree, c.host, |storage, picture_context| {
                    use crate::adapter::layer_tree_impl::LayerTreeLayerStorage;
                    let Some(layer) = storage.LayerByIdMut(tile.layer_id()) else {
                        return false;
                    };
                    layer.NotifyTileStateChanged(tile, update_damage, picture_context);
                    true
                })
            };
            let notified = if is_pending_tree {
                self.WithPendingTree(context, &mut notify)
            } else {
                self.WithActiveTree(context, &mut notify)
            };
            if !notified {
                return;
            }
            let tree = if is_pending_tree {
                self.pending_tree_.as_mut()
            } else {
                self.active_tree_.as_mut()
            };
            let layer_impl = tree.unwrap().LayerByIdMutable(tile.layer_id()).unwrap();
            if self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures())
                && !layer_impl
                    .AsPictureLayer()
                    .unwrap()
                    .should_batch_updated_tiles()
            {
                context.services.UpdateDisplayTile(
                    self.layer_context_.as_mut().unwrap(),
                    layer_impl.AsPictureLayer().unwrap(),
                    layer_impl
                        .AsPictureLayer()
                        .unwrap()
                        .ResidentDisplayTiling(tiling_id)
                        .expect("native resident tile tiling"),
                    tile,
                    self.resource_provider_.as_mut().unwrap(),
                    self.layer_tree_frame_sink_.unwrap(),
                    update_damage,
                );
            }
            if set_needs_redraw
                && !context.services.DelegateIsInsideDraw(self.delegate_)
                && tile.required_for_draw()
            {
                self.SetNeedsRedraw(false, false, context);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2110-2131.
        pub fn SetMemoryPolicy(
            &mut self,
            policy: &ManagedMemoryPolicy,
            context: &mut HostImplContext<'_, B>,
        ) {
            debug_assert!(context.services.IsImplThread(self.task_runner_provider_));
            self.SetMemoryPolicyImpl(policy, context);
            if policy.bytes_limit_when_visible == 0
                && self.resource_pool_.is_some()
                && self.settings_.using_synchronous_renderer_compositor
            {
                self.ReleaseTileResources(context);
                self.CleanUpTileManagerResources(context);
                self.NotifyAllTileTasksCompleted(context);
                self.CreateTileManagerResources(context);
                self.RecreateTileResources(context);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2133-2137.
        pub fn SetTreeActivationCallback(
            &mut self,
            callback: Option<<B as HostImplTypes>::RepeatingClosure>,
            context: &mut HostImplContext<'_, B>,
        ) {
            debug_assert!(context.services.IsImplThread(self.task_runner_provider_));
            self.tree_activation_callback_ = callback;
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2139-2170.
        pub fn SetMemoryPolicyImpl(
            &mut self,
            policy: &ManagedMemoryPolicy,
            context: &mut HostImplContext<'_, B>,
        ) {
            if self.cached_managed_memory_policy_ == *policy {
                return;
            }
            let old_policy = self.ActualManagedMemoryPolicy(context);
            self.cached_managed_memory_policy_ = *policy;
            let actual_policy = self.ActualManagedMemoryPolicy(context);
            if old_policy == actual_policy {
                return;
            }
            self.UpdateTileManagerMemoryPolicy(actual_policy, context);
            let mut needs_commit = true;
            if self.visible_
                && actual_policy.bytes_limit_when_visible >= self.max_memory_needed_bytes_
                && old_policy.bytes_limit_when_visible >= self.max_memory_needed_bytes_
                && actual_policy.priority_cutoff_when_visible
                    == old_policy.priority_cutoff_when_visible
            {
                needs_commit = false;
            }
            if needs_commit {
                self.SetNeedsCommit(context);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2172-2190.
        pub fn SetExternalTilePriorityConstraints(
            &mut self,
            viewport_rect: &Rect,
            _transform: &<B as LayerTreeTypes>::Transform,
            context: &mut HostImplContext<'_, B>,
        ) {
            let tile_priority_params_changed =
                self.viewport_rect_for_tile_priority_ != *viewport_rect;
            self.viewport_rect_for_tile_priority_ = *viewport_rect;
            if tile_priority_params_changed {
                self.active_tree_
                    .as_mut()
                    .unwrap()
                    .set_needs_update_draw_properties();
                if let Some(pending) = self.pending_tree_.as_mut() {
                    pending.set_needs_update_draw_properties();
                }
                self.SetFullViewportDamage(context);
                self.SetNeedsRedraw(false, false, context);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2192-2194.
        pub fn DidReceiveCompositorFrameAck(&mut self, context: &mut HostImplContext<'_, B>) {
            context
                .services
                .DelegateDidReceiveCompositorFrameAckOnImplThread(self.delegate_);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2196-2216.
        pub fn DidPresentCompositorFrame(
            &mut self,
            frame_token: u32,
            details: &B::HostFrameTimingDetails,
            context: &mut HostImplContext<'_, B>,
        ) {
            let mut activated_callbacks = self.presentation_time_callbacks_.PopPendingCallbacks(
                frame_token,
                context.services.PresentationFeedbackFailed(details),
            );
            // Source: cc/trees/proxy_impl.cc:724-726. Rust lexical continuation:
            // run on the same Host before releasing this borrow, then post only
            // the original main callbacks and Scheduler feedback values.
            self.NotifyDidPresentCompositorFrameOnImplThread(
                frame_token,
                std::mem::take(&mut activated_callbacks.compositor_successful_callbacks),
                details,
                context,
            );
            context
                .services
                .DelegateDidPresentCompositorFrameOnImplThread(
                    self.delegate_,
                    frame_token,
                    activated_callbacks,
                    details,
                );
            let runner = self.GetTaskRunner(context);
            let task = HostImplPostedCompositorTask {
                target: self.AsWeakPtr(),
                kind: HostImplPostedCompositorTaskKind::LogAverageLagEvents {
                    frame_token,
                    details: details.clone(),
                },
            };
            context.services.PostCompositorTask(
                runner,
                crate::adapter::task_runner::TaskLocation {
                    function: "LayerTreeHostImpl::DidPresentCompositorFrame",
                    file: "cc/trees/layer_tree_host_impl.cc",
                    line: 2212,
                },
                task,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2218-2222.
        pub fn LogAverageLagEvents(
            &mut self,
            frame_token: u32,
            details: &B::HostFrameTimingDetails,
            context: &mut HostImplContext<'_, B>,
        ) {
            context.services.LagTrackingDidPresentCompositorFrame(
                &mut self.lag_tracking_manager_,
                frame_token,
                details,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2224-2227.
        pub fn NotifyCompositorMetricsTrackerResults(
            &mut self,
            results: &B::CustomTrackerResults,
            context: &mut HostImplContext<'_, B>,
        ) {
            context
                .services
                .DelegateNotifyCompositorMetricsTrackerResults(self.delegate_, results);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2229-2231.
        pub fn DidNotNeedBeginFrame(&mut self, context: &mut HostImplContext<'_, B>) {
            context
                .services
                .FrameTrackersNotifyPauseFrameProduction(&mut self.frame_trackers_);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2233-2263.
        pub fn ReclaimResources(
            &mut self,
            resources: Vec<<B as crate::adapter::unbounded::UnboundedTypes>::ReturnedResource>,
            context: &mut HostImplContext<'_, B>,
        ) {
            ReclaimResourcesWithLiveParts::<B>(
                &mut self.ResourceReturnParts(),
                resources,
                context.services,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2265-2277.
        pub fn MaybeFlushPendingWork(&mut self, context: &mut HostImplContext<'_, B>) {
            MaybeFlushPendingWorkWithLiveParts::<B>(&self.ResourceReturnParts(), context.services);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2279-2333.
        pub fn OnDraw(
            &mut self,
            transform: &<B as LayerTreeTypes>::Transform,
            viewport: &Rect,
            resourceless_software_draw: bool,
            skip_draw: bool,
            context: &mut HostImplContext<'_, B>,
        ) {
            debug_assert!(!self.resourceless_software_draw_);
            let origin = self
                .active_tree_
                .as_ref()
                .unwrap()
                .internal_device_viewport()
                .origin();
            debug_assert!(origin.x() == 0 && origin.y() == 0);
            // Source: cc/trees/layer_tree_host_impl.cc:2289-2291.
            #[cfg(debug_assertions)]
            let mut sync_draw_reset =
                ScopedHostImplFlagReset::new(self, HostImplScopedFlag::DoingSyncDraw, true);
            #[cfg(debug_assertions)]
            let this = &mut *sync_draw_reset;
            #[cfg(not(debug_assertions))]
            let this = self;
            if skip_draw {
                context.services.DelegateOnDrawForLayerTreeFrameSink(
                    this.delegate_,
                    this.resourceless_software_draw_,
                    true,
                );
                return;
            }
            let transform_changed = this.external_transform_ != *transform;
            let viewport_changed = this.external_viewport_ != *viewport;
            this.external_transform_ = transform.clone();
            this.external_viewport_ = *viewport;
            {
                // Source: cc/trees/layer_tree_host_impl.cc:2305-2322.
                let mut resourceless_reset = ScopedHostImplFlagReset::new(
                    this,
                    HostImplScopedFlag::ResourcelessSoftwareDraw,
                    resourceless_software_draw,
                );
                let this = &mut *resourceless_reset;
                if transform_changed || viewport_changed || this.resourceless_software_draw_ {
                    this.SetFullViewportDamage(context);
                    this.SetNeedsRedraw(false, false, context);
                    this.active_tree_
                        .as_mut()
                        .unwrap()
                        .set_needs_update_draw_properties();
                }
                if resourceless_software_draw {
                    let can_draw = this.CanDraw(context);
                    context
                        .services
                        .OnCanDrawStateChanged(this.delegate_, can_draw);
                }
                context.services.DelegateOnDrawForLayerTreeFrameSink(
                    this.delegate_,
                    this.resourceless_software_draw_,
                    skip_draw,
                );
            }
            if resourceless_software_draw {
                this.active_tree_
                    .as_mut()
                    .unwrap()
                    .set_needs_update_draw_properties();
                let can_draw = this.CanDraw(context);
                context
                    .services
                    .OnCanDrawStateChanged(this.delegate_, can_draw);
                this.SetFullViewportDamage(context);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2335-2352.
        pub fn OnCompositorFrameTransitionDirectiveProcessed(
            &mut self,
            sequence_id: u32,
            context: &mut HostImplContext<'_, B>,
        ) {
            let mut rects_for_this_sequence = HostViewTransitionElementResourceRects::new();
            if let Some(rects) = self.view_transition_content_rects_.get_mut(&sequence_id) {
                std::mem::swap(rects, &mut rects_for_this_sequence);
                self.view_transition_content_rects_.remove(&sequence_id);
            }
            context.services.DelegateNotifyTransitionRequestFinished(
                self.delegate_,
                sequence_id,
                rects_for_this_sequence,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2354-2359.
        pub fn SetViewTransitionContentRect(
            &mut self,
            sequence_id: u32,
            id: &ViewTransitionElementResourceId,
            rect: &RectF,
        ) {
            SetViewTransitionContentRectForHost(
                &mut self.view_transition_content_rects_,
                sequence_id,
                id,
                rect,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2361-2372.
        pub fn OnSurfaceEvicted(
            &mut self,
            id: &<B as LayerTreeTypes>::LocalSurfaceId,
            context: &mut HostImplContext<'_, B>,
        ) {
            if context
                .services
                .LocalSurfaceIdIsNewerThanOrEmbeddingChanged(&self.target_local_surface_id_, id)
            {
                return;
            }
            self.evicted_local_surface_id_ = id.clone();
            context
                .services
                .ResourceProviderSetEvicted(self.resource_provider_.as_mut().unwrap(), true);
            let can_draw = self.CanDraw(context);
            context
                .services
                .OnCanDrawStateChanged(self.delegate_, can_draw);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2374-2380.
        pub fn ReportEventLatency(
            &mut self,
            args: &crate::adapter::begin_frame::BeginFrameArgs,
            latencies: Vec<B::EventLatencyData>,
            context: &mut HostImplContext<'_, B>,
        ) {
            if context.services.HasCustomMetricRecorder() {
                context
                    .services
                    .CustomMetricRecorderReportEventLatency(args, latencies);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2382-2391.
        pub fn ReportScrollJankStats(
            &mut self,
            total_frames: u32,
            janky_frames: u32,
            _context: &mut HostImplContext<'_, B>,
        ) {
            assert!(janky_frames <= total_frames);
            #[cfg(target_os = "android")]
            if let Some(observer) = self.render_frame_metadata_observer_.as_mut() {
                _context
                    .services
                    .RenderFrameMetadataObserverReportScrollJankStats(
                        observer,
                        total_frames,
                        janky_frames,
                    );
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2393-2395.
        pub fn OnCanDrawStateChangedForTree(&mut self, context: &mut HostImplContext<'_, B>) {
            let can_draw = self.CanDraw(context);
            context
                .services
                .OnCanDrawStateChanged(self.delegate_, can_draw);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2233-2277.
        // Borrow-only language bridge shared by source owner entries and the scoped
        // frame-sink client. No field value or native owner state is cloned here.
        pub(super) fn ResourceReturnParts(&mut self) -> HostImplResourceReturnParts<'_, B> {
            HostImplResourceReturnParts {
                owner: self.owner_id_,
                resource_provider: &mut self.resource_provider_,
                resource_pool: &mut self.resource_pool_,
                deleted_ui_resources: &mut self.deleted_ui_resources_,
                visible: &self.visible_,
                has_valid_sink: &self.has_valid_layer_tree_frame_sink_,
                sink: &self.layer_tree_frame_sink_,
                task_runner_provider: &self.task_runner_provider_,
                weak_factory: &self.weak_factory_,
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2212-2215,2259-2262.
        // Language bridge for source WeakPtr-bound once callbacks. The supplied
        // task is consumed once even if its target was destroyed before delivery.
        pub(crate) fn RunPostedCompositorTask(
            &mut self,
            task: HostImplPostedCompositorTask<B::HostFrameTimingDetails>,
            context: &mut HostImplContext<'_, B>,
        ) {
            if !task.target.IsValid() {
                return;
            }
            assert_eq!(
                task.target.owner, self.owner_id_,
                "posted callback routed to another HostImpl"
            );
            assert!(
                task.target
                    .alive
                    .ptr_eq(&std::sync::Arc::downgrade(&self.weak_factory_)),
                "posted callback belongs to another HostImpl generation"
            );
            match task.kind {
                HostImplPostedCompositorTaskKind::LogAverageLagEvents {
                    frame_token,
                    details,
                } => self.LogAverageLagEvents(frame_token, &details, context),
                HostImplPostedCompositorTaskKind::MaybeFlushPendingWork => {
                    self.MaybeFlushPendingWork(context)
                }
            }
        }
    }
    // Source dependency: base/auto_reset.h:27-56, instantiated at
    // cc/trees/layer_tree_host_impl.cc:2289-2291,2305-2306. This local Rust guard owns
    // only its lexical mutable borrow, restoring source flags on return/unwind.
    // It stores no persistent owner reference and uses no interior mutability.
    enum HostImplScopedFlag {
        ResourcelessSoftwareDraw,
        #[cfg(debug_assertions)]
        DoingSyncDraw,
    }
    struct ScopedHostImplFlagReset<'a, B: HostImplTypes> {
        host: &'a mut LayerTreeHostImpl<B>,
        flag: HostImplScopedFlag,
        old: bool,
    }
    impl<'a, B: HostImplTypes> ScopedHostImplFlagReset<'a, B> {
        fn new(host: &'a mut LayerTreeHostImpl<B>, flag: HostImplScopedFlag, value: bool) -> Self {
            let target = match flag {
                HostImplScopedFlag::ResourcelessSoftwareDraw => {
                    &mut host.resourceless_software_draw_
                }
                #[cfg(debug_assertions)]
                HostImplScopedFlag::DoingSyncDraw => &mut host.doing_sync_draw_,
            };
            let old = std::mem::replace(target, value);
            Self { host, flag, old }
        }
    }
    impl<B: HostImplTypes> std::ops::Deref for ScopedHostImplFlagReset<'_, B> {
        type Target = LayerTreeHostImpl<B>;
        fn deref(&self) -> &Self::Target {
            self.host
        }
    }
    impl<B: HostImplTypes> std::ops::DerefMut for ScopedHostImplFlagReset<'_, B> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.host
        }
    }
    impl<B: HostImplTypes> Drop for ScopedHostImplFlagReset<'_, B> {
        fn drop(&mut self) {
            match self.flag {
                HostImplScopedFlag::ResourcelessSoftwareDraw => {
                    self.host.resourceless_software_draw_ = self.old
                }
                #[cfg(debug_assertions)]
                HostImplScopedFlag::DoingSyncDraw => self.host.doing_sync_draw_ = self.old,
            }
        }
    }

    // Source: cc/trees/layer_tree_host_impl.cc:2233-2277.
    // Live references to the exact resident siblings read/written by resource return.
    // The frame-sink client gets this narrow lexical borrow, never a HostImpl owner.
    pub(super) struct HostImplResourceReturnParts<'a, B: HostImplTypes> {
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        resource_provider: &'a mut Option<Box<B::ClientResourceProvider>>,
        resource_pool: &'a mut Option<Box<B::ResourcePool>>,
        deleted_ui_resources:
            &'a mut HashMap<crate::adapter::layer_tree_impl::UIResourceId, UIResourceData<B>>,
        visible: &'a bool,
        has_valid_sink: &'a bool,
        sink: &'a Option<HostImplFrameSinkId>,
        task_runner_provider: &'a HostImplTaskRunnerProviderId,
        weak_factory: &'a std::sync::Arc<()>,
    }
    // Source: cc/trees/layer_tree_host_impl.h:797-802.
    // The existing owner GetTaskRunner entry delegates to this same source body.
    fn GetTaskRunnerForLiveProvider(
        provider: HostImplTaskRunnerProviderId,
        services: &dyn HostImplTaskRunnerServices,
    ) -> HostImplTaskRunnerId {
        debug_assert_ne!(provider.0, 0);
        if services.HasImplThread(provider) {
            services.ImplThreadTaskRunner(provider)
        } else {
            services.MainThreadTaskRunner(provider)
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:2233-2263.
    pub(super) fn ReclaimResourcesWithLiveParts<B: HostImplTypes>(
        parts: &mut HostImplResourceReturnParts<'_, B>,
        resources: Vec<<B as crate::adapter::unbounded::UnboundedTypes>::ReturnedResource>,
        services: &mut dyn HostImplResourceReturnServices<B>,
    ) {
        let mut release_client = UIResourceReleaseClient {
            owner: parts.owner,
            deleted: parts.deleted_ui_resources,
        };
        services.ReceiveReturnsFromParent(
            parts.resource_provider.as_mut().unwrap(),
            resources,
            parts.resource_pool.as_deref_mut(),
            &mut release_client,
        );
        if let Some(pool) = parts.resource_pool.as_mut() {
            services.ResourcePoolReduceResourceUsage(pool);
        }
        MaybeFlushPendingWorkWithLiveParts::<B>(parts, services);
        let runner = GetTaskRunnerForLiveProvider(*parts.task_runner_provider, services);
        services.PostDelayedCompositorTask(
            runner,
            crate::adapter::task_runner::TaskLocation {
                function: "LayerTreeHostImpl::ReclaimResources",
                file: "cc/trees/layer_tree_host_impl.cc",
                line: 2259,
            },
            HostImplPostedCompositorTask {
                target: HostImplWeak {
                    owner: parts.owner,
                    alive: std::sync::Arc::downgrade(parts.weak_factory),
                },
                kind: HostImplPostedCompositorTaskKind::MaybeFlushPendingWork,
            },
            TimeDelta::Seconds(1),
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:2265-2277.
    fn MaybeFlushPendingWorkWithLiveParts<B: HostImplTypes>(
        parts: &HostImplResourceReturnParts<'_, B>,
        services: &mut dyn HostImplResourceReturnServices<B>,
    ) {
        if *parts.visible || !*parts.has_valid_sink {
            return;
        }
        let Some(compositor_context) = services.FrameSinkContextProvider(parts.sink.unwrap())
        else {
            return;
        };
        if !services.ContextHasSupport(compositor_context) {
            return;
        }
        services.ContextSupportFlushPendingWork(compositor_context);
    }
}

mod source_chunk_g {
    use super::*;
    // Source: cc/trees/layer_tree_host_impl.cc:4431-5180.
    // Existing canonical owner definitions excluded:4832-4834,4836-4842,4988-4994.
    use crate::adapter::{
        geometry::PointF,
        image_animation::{kDefaultFrameIndex, PaintImageId},
        input_handler::InputScrollNodeId,
        time::TimeTicks,
    };
    use crate::cc::trees::{
        compositor_commit_data::{CompositorCommitData, ScrollbarsUpdateInfo},
        property_tree::ScrollTree,
    };
    impl<B: HostImplTypes> LayerTreeHostImpl<B> {
        // Source: cc/trees/layer_tree_host_impl.cc:4432-4435.
        pub fn SetLayerTreeMutator(
            &mut self,
            mutator: Option<Box<B::LayerTreeMutator>>,
            context: &mut HostImplContext<'_, B>,
        ) {
            context
                .services
                .MutatorSetLayerTreeMutator(&mut self.mutator_host_, mutator);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4437-4440.
        pub fn SetPaintWorkletLayerPainter(
            &mut self,
            painter: Option<Box<B::PaintWorkletLayerPainter>>,
        ) {
            self.paint_worklet_painter_ = painter;
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4442-4463.
        pub fn QueueImageDecode(
            &mut self,
            request_id: i32,
            image: &B::DrawImage,
            speculative: bool,
            context: &mut HostImplContext<'_, B>,
        ) {
            debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
            let paint_image = image.paint_image();
            context
                .services
                .TraceQueueImageDecode(paint_image, kDefaultFrameIndex);
            let params = self.GetTargetColorParams(paint_image.GetContentColorUsage(), context);
            let image_copy =
                context
                    .services
                    .CreateDrawImageForDecode(image, 1.0, kDefaultFrameIndex, params);
            let callback = HostImageDecodeCallback {
                target: self.AsWeakPtr(),
                request_id,
                speculative,
            };
            B::QueueResidentImageDecode(
                &mut self.tile_manager_,
                &image_copy,
                callback,
                speculative,
                context,
            );
            B::DisallowResidentImageCheckering(&mut self.tile_manager_, paint_image, context);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4465-4477.
        pub fn ImageDecodeFinished(
            &mut self,
            request_id: i32,
            speculative: bool,
            decode_succeeded: bool,
            context: &mut HostImplContext<'_, B>,
        ) {
            debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
            context.services.TraceHostImplGEvent(
                "disabled-by-default-cc.debug",
                "LayerTreeHostImpl::ImageDecodeFinished",
            );
            if !context.services.SendExplicitDecodeRequestsImmediately() {
                self.completed_image_decode_requests_
                    .push((request_id, decode_succeeded));
            }
            context.services.DelegateNotifyImageDecodeRequestFinished(
                self.delegate_,
                request_id,
                speculative,
                decode_succeeded,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4479-4484.
        pub fn TakeCompletedImageDecodeRequests(&mut self) -> Vec<(i32, bool)> {
            let result = std::mem::take(&mut self.completed_image_decode_requests_);
            self.completed_image_decode_requests_.clear();
            result
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4486-4491.
        pub fn TakeMutatorEvents(
            &mut self,
            context: &mut HostImplContext<'_, B>,
        ) -> Option<Box<B::HostMutatorEvents>> {
            let mut events = Some(
                context
                    .services
                    .MutatorCreateEvents(&mut self.mutator_host_),
            );
            std::mem::swap(&mut events, &mut self.mutator_events_);
            context
                .services
                .MutatorTakeTimeUpdatedEvents(&mut self.mutator_host_, events.as_deref_mut());
            events
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4493-4510.
        pub fn TakeUIResourceChanges(
            &mut self,
            require_full_sync: bool,
            context: &mut HostImplContext<'_, B>,
        ) -> UIResourceChangeMap<crate::adapter::layer_tree_impl::UIResourceId> {
            if require_full_sync {
                self.ui_resource_changes_.clear();
                if self
                    .settings_
                    .TreesInVizInClientProcess(context.services.SettingsFeatures())
                {
                    for (&uid, _) in self.ui_resource_map_.iter() {
                        self.ui_resource_changes_
                            .entry(uid)
                            .or_default()
                            .resource_created = true;
                    }
                }
            }
            std::mem::take(&mut self.ui_resource_changes_)
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4512-4514.
        pub fn ClearHistory(&mut self, context: &mut HostImplContext<'_, B>) {
            context.services.DelegateClearHistory(self.delegate_);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4516-4518.
        pub fn CommitDurationSampleCountForTesting(
            &self,
            context: &HostImplContext<'_, B>,
        ) -> usize {
            context
                .services
                .DelegateCommitDurationSampleCountForTesting(self.delegate_)
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4520-4532.
        pub fn ClearCaches(&mut self, context: &mut HostImplContext<'_, B>) {
            ClearCachesForHost(
                &self.settings_,
                &mut self.tile_manager_,
                &mut self.image_decode_cache_holder_,
                &mut self.image_animation_controller_,
                context,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4534-4539.
        pub fn DidChangeScrollbarVisibility(&mut self, context: &mut HostImplContext<'_, B>) {
            self.SetNeedsCommit(context);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4541-4569.
        pub fn CleanUpTileManagerResources(&mut self, context: &mut HostImplContext<'_, B>) {
            debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
            B::FinishResidentTileTasks(self, context);
            self.single_thread_synchronous_task_graph_runner_ = None;
            self.image_decode_cache_holder_ = None;
            self.raster_buffer_provider_ = None;
            self.pending_raster_queries_ = None;
            context
                .services
                .ResourcePoolInvalidateResources(self.resource_pool_.as_mut().unwrap());
            if let Some(sink) = self.layer_tree_frame_sink_ {
                if let Some(worker) = context.services.FrameSinkWorkerContextProvider(sink) {
                    let mut hold = context.services.LockRasterContext(worker);
                    context.services.RasterContextOrderingBarrier(&mut hold);
                }
                if let Some(compositor) = context.services.FrameSinkContextProvider(sink) {
                    context.services.ContextSupportFlushPendingWork(compositor);
                }
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4571-4637.
        pub fn ReleaseLayerTreeFrameSink(&mut self, context: &mut HostImplContext<'_, B>) {
            context
                .services
                .TraceHostImplGEvent("cc", "LayerTreeHostImpl::ReleaseLayerTreeFrameSink");
            let Some(sink) = self.layer_tree_frame_sink_ else {
                debug_assert!(!self.has_valid_layer_tree_frame_sink_);
                return;
            };
            self.has_valid_layer_tree_frame_sink_ = false;
            self.ReleaseTreeResources(context);
            if !self.settings_.trees_in_viz_in_viz_process {
                self.CleanUpTileManagerResources(context);
                self.resource_pool_ = None;
                self.ClearUIResources(context);
            }
            self.SetContextVisibility(false, context);
            context
                .services
                .FrameTrackersClearAll(&mut self.frame_trackers_);
            {
                let access = context.services.FrameSinkAccess(sink);
                let delegate = self.delegate_;
                let mut client = HostImplLiveFrameSinkClient {
                    delegate,
                    returns: self.ResourceReturnParts(),
                    services: access.callbacks,
                };
                access.sink.DetachFromClient(&mut client);
            }
            self.layer_tree_frame_sink_ = None;
            self.layer_context_ = None;
            {
                let mut client = UIResourceReleaseClient {
                    owner: self.owner_id_,
                    deleted: &mut self.deleted_ui_resources_,
                };
                context
                    .services
                    .ResourceProviderReleaseAllExportedResources(
                        self.resource_provider_.as_mut().unwrap(),
                        self.resource_pool_.as_deref_mut(),
                        true,
                        &mut client,
                    );
            }
            self.raster_caps_.use_gpu_rasterization = false;
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4639-4698.
        pub fn InitializeFrameSink(
            &mut self,
            sink: HostImplFrameSinkId,
            context: &mut HostImplContext<'_, B>,
        ) -> bool {
            context
                .services
                .TraceHostImplGEvent("cc", "LayerTreeHostImpl::InitializeFrameSink");
            self.ReleaseLayerTreeFrameSink(context);
            let did_bind = {
                let weak = self.AsWeakPtr();
                let delegate = self.delegate_;
                let access = context.services.FrameSinkAccess(sink);
                let mut client = HostImplLiveFrameSinkClient {
                    delegate,
                    returns: self.ResourceReturnParts(),
                    services: access.callbacks,
                };
                access.sink.BindToClient(weak, &mut client)
            };
            if !did_bind {
                return false;
            }
            self.layer_tree_frame_sink_ = Some(sink);
            self.has_valid_layer_tree_frame_sink_ = true;
            if self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures())
            {
                let draw_mode = self.GetDrawMode(context);
                self.layer_context_ = Some(context.services.FrameSinkCreateLayerContext(
                    sink,
                    self.AsWeakPtr(),
                    draw_mode,
                    &self.settings_,
                ));
                if context
                    .services
                    .FrameSinkIdIsValid(&self.unbounded_frame_sink_id_)
                {
                    self.layer_context_
                        .as_mut()
                        .unwrap()
                        .SetUnboundedFrameSinkId(
                            &self.unbounded_frame_sink_id_,
                            &self.unbounded_local_surface_id_,
                        );
                }
            }
            self.UpdateRasterCapabilities(context);
            self.active_tree_
                .as_mut()
                .unwrap()
                .set_needs_update_draw_properties();
            if let Some(pending) = self.pending_tree_.as_mut() {
                pending.set_needs_update_draw_properties();
            }
            if !self.settings_.trees_in_viz_in_viz_process {
                let provider = context.services.FrameSinkContextProvider(sink);
                let runner = self.GetTaskRunner(context);
                self.resource_pool_ = Some(context.services.CreateResourcePool(
                    self.resource_provider_.as_mut().unwrap(),
                    provider,
                    runner,
                    TimeDelta::Seconds(5),
                    self.settings_.disallow_non_exact_resource_reuse,
                ));
                self.CreateTileManagerResources(context);
                self.RecreateTileResources(context);
            }
            let can_draw = self.CanDraw(context);
            context
                .services
                .OnCanDrawStateChanged(self.delegate_, can_draw);
            self.SetFullViewportDamage(context);
            self.SetRequiresHighResToDraw();
            use crate::adapter::unbounded::LocalSurfaceId;
            if self.GetCurrentLocalSurfaceId(context).is_valid()
                && !self.settings_.trees_in_viz_in_viz_process
            {
                self.AllocateLocalSurfaceId();
            }
            true
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4700-4702.
        pub fn SetBeginFrameSource(
            &mut self,
            source: Option<&<B as crate::adapter::unbounded::UnboundedTypes>::BeginFrameSource>,
            context: &mut HostImplContext<'_, B>,
        ) {
            SetBeginFrameSourceForLiveDelegate::<B>(self.delegate_, source, context.services);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4704-4706.
        pub fn DrawTransform(&self) -> &<B as LayerTreeTypes>::Transform {
            &self.external_transform_
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4708-4716.
        pub fn DidChangeBrowserControlsPosition(&mut self, context: &mut HostImplContext<'_, B>) {
            self.WithBrowserControls(context, |manager, client| {
                client.DidChangeBrowserControlsPosition(manager)
            });
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4718-4728.
        pub fn DidObserveScrollDelay(
            &mut self,
            source_frame_number: i32,
            scroll_delay: TimeDelta,
            scroll_timestamp: TimeTicks,
            context: &mut HostImplContext<'_, B>,
        ) {
            if !self.has_observed_first_scroll_delay_ {
                context.services.DelegateDidObserveFirstScrollDelay(
                    self.delegate_,
                    source_frame_number,
                    scroll_delay,
                    scroll_timestamp,
                );
                self.has_observed_first_scroll_delay_ = true;
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4730-4732.
        pub fn TopControlsHeight(&self) -> f32 {
            self.active_tree_.as_ref().unwrap().top_controls_height()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4734-4736.
        pub fn TopControlsMinHeight(&self) -> f32 {
            self.active_tree_
                .as_ref()
                .unwrap()
                .top_controls_min_height()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4738-4740.
        pub fn BottomControlsHeight(&self) -> f32 {
            self.active_tree_.as_ref().unwrap().bottom_controls_height()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4742-4744.
        pub fn BottomControlsMinHeight(&self) -> f32 {
            self.active_tree_
                .as_ref()
                .unwrap()
                .bottom_controls_min_height()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4746-4753.
        pub fn SetCurrentBrowserControlsShownRatio(
            &mut self,
            top: f32,
            bottom: f32,
            context: &mut HostImplContext<'_, B>,
        ) {
            if self.WithActiveTree(context, |tree, c| {
                tree.SetCurrentBrowserControlsShownRatio(top, bottom, c)
            }) {
                self.DidChangeBrowserControlsPosition(context);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4755-4757.
        pub fn CurrentTopControlsShownRatio(
            &mut self,
            context: &mut HostImplContext<'_, B>,
        ) -> f32 {
            self.WithActiveTree(context, |tree, c| tree.CurrentTopControlsShownRatio(c))
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4759-4761.
        pub fn CurrentBottomControlsShownRatio(
            &mut self,
            context: &mut HostImplContext<'_, B>,
        ) -> f32 {
            self.WithActiveTree(context, |tree, c| tree.CurrentBottomControlsShownRatio(c))
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4763-4765.
        pub fn ViewportScrollOffset(&mut self, context: &mut HostImplContext<'_, B>) -> PointF {
            self.WithViewport(context, |viewport, c| viewport.TotalScrollOffset(c))
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4767-4769.
        pub fn MaxViewportScrollOffsetY(&mut self, context: &mut HostImplContext<'_, B>) -> f32 {
            self.WithViewport(context, |viewport, c| {
                viewport.MaxUserReachableTotalScrollOffsetY(c)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4771-4789.
        pub fn AutoScrollAnimationCreate(
            &mut self,
            scroll_node: InputScrollNodeId,
            target: &PointF,
            velocity: f32,
            context: &mut HostImplContext<'_, B>,
        ) {
            let animation_start_offset = self.CurrentBeginFrameArgs().interval;
            let tree = self.active_tree_.as_ref().unwrap();
            assert_eq!(tree.id, scroll_node.tree);
            let property_trees = tree.property_trees();
            let scroll_tree = property_trees.scroll_tree();
            let element_id = scroll_tree.Node(scroll_node.index).element_id;
            let current_offset =
                scroll_tree.current_scroll_offset(element_id, property_trees.state());
            context.services.MutatorImplOnlyAutoScrollAnimationCreate(
                &mut self.mutator_host_,
                element_id,
                target,
                current_offset,
                velocity,
                animation_start_offset,
            );
            self.SetNeedsOneBeginImplFrame(context);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4791-4823.
        pub fn ScrollAnimationCreate(
            &mut self,
            scroll_node: InputScrollNodeId,
            delta: &Vector2dF,
            delayed_by: TimeDelta,
            context: &mut HostImplContext<'_, B>,
        ) -> bool {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::ScrollAnimationCreate(
                    host,
                    &scroll_node,
                    delta,
                    delayed_by,
                )
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4825-4830.
        pub fn UpdateImageDecodingHints(
            &mut self,
            map: std::collections::BTreeMap<PaintImageId, HostImageDecodingMode>,
            context: &mut HostImplContext<'_, B>,
        ) {
            context
                .services
                .UpdateCheckerImageDecodingHints(&mut self.tile_manager_, map);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4844-4862.
        pub fn WillScrollContent(
            &mut self,
            element_id: ElementId,
            context: &mut HostImplContext<'_, B>,
        ) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::WillScrollContent(host, element_id)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4864-4970.
        pub fn DidScrollContent(
            &mut self,
            element_id: ElementId,
            animated: bool,
            scroll_delta: &Vector2dF,
            context: &mut HostImplContext<'_, B>,
        ) {
            self.WithInputRuntimeView(context, |host, view| {
                crate::adapter::input_handler::InputHost::<B>::DidScrollContent(
                    host,
                    view,
                    element_id,
                    animated,
                    scroll_delta,
                )
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4972-4976.
        pub fn DeviceScaleFactor(&self) -> f32 {
            if self.settings_.use_painted_device_scale_factor {
                self.active_tree_
                    .as_ref()
                    .unwrap()
                    .painted_device_scale_factor()
            } else {
                self.active_tree_.as_ref().unwrap().device_scale_factor()
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4978-4980.
        pub fn PageScaleFactor(&mut self, context: &mut HostImplContext<'_, B>) -> f32 {
            self.WithActiveTree(context, |tree, c| tree.page_scale_factor_for_scroll(c))
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4982-4986.
        pub fn BindToInputHandler(&mut self, delegate: Box<B::InputHandler>) {
            self.input_delegate_ = Some(delegate);
            self.input_delegate_
                .as_mut()
                .unwrap()
                .SetPrefersReducedMotion(self.prefers_reduced_motion_);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4996-4999.
        pub fn SetVisualDeviceViewportSize(&mut self, size: &Size) {
            self.visual_device_viewport_size_ = *size;
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5001-5003.
        pub fn VisualDeviceViewportSize(&self) -> Size {
            self.visual_device_viewport_size_
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5005-5014.
        pub fn SetPrefersReducedMotion(&mut self, value: bool) {
            if self.prefers_reduced_motion_ == value {
                return;
            }
            self.prefers_reduced_motion_ = value;
            if let Some(delegate) = self.input_delegate_.as_mut() {
                delegate.SetPrefersReducedMotion(self.prefers_reduced_motion_);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5016-5019.
        pub fn SetMayThrottleIfUndrawnFrames(&mut self, value: bool) {
            self.may_throttle_if_undrawn_frames_ = value;
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5021-5023.
        pub fn GetScrollTree(&mut self) -> &mut ScrollTree {
            self.active_tree_
                .as_mut()
                .unwrap()
                .property_trees_mut()
                .scroll_tree_mutable()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5025-5027.
        pub fn ScrollAnimationAbort(
            &mut self,
            element_id: ElementId,
            context: &mut HostImplContext<'_, B>,
        ) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::ScrollAnimationAbort(
                    host, element_id,
                )
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5029-5031.
        pub fn GetBrowserControlsTopOffset(&mut self, context: &mut HostImplContext<'_, B>) -> f32 {
            self.WithBrowserControls(context, |manager, client| manager.ControlsTopOffset(client))
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5033-5035.
        pub fn ScrollBegin(&mut self, context: &mut HostImplContext<'_, B>) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::ScrollBegin(host)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5037-5040.
        pub fn ScrollEnd(&mut self, delta: &Vector2dF, context: &mut HostImplContext<'_, B>) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::ScrollEnd(host, delta)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5042-5046.
        pub fn StartScrollSequence(
            &mut self,
            sequence: B::FrameSequenceTrackerType,
            thread: B::SmoothEffectDrivingThread,
            context: &mut HostImplContext<'_, B>,
        ) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::StartScrollSequence(
                    host, sequence, thread,
                )
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5048-5050.
        pub fn StopSequence(
            &mut self,
            sequence: B::FrameSequenceTrackerType,
            context: &mut HostImplContext<'_, B>,
        ) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::StopSequence(host, sequence)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5052-5054.
        pub fn PinchBegin(&mut self, context: &mut HostImplContext<'_, B>) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::PinchBegin(host)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5056-5058.
        pub fn PinchEnd(&mut self, context: &mut HostImplContext<'_, B>) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::PinchEnd(host)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5060-5067.
        pub fn ScrollbarAnimationMouseLeave(
            &mut self,
            id: ElementId,
            context: &mut HostImplContext<'_, B>,
        ) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::ScrollbarAnimationMouseLeave(
                    host, id,
                )
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5069-5077.
        pub fn ScrollbarAnimationMouseMove(
            &mut self,
            id: ElementId,
            point: PointF,
            context: &mut HostImplContext<'_, B>,
        ) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::ScrollbarAnimationMouseMove(
                    host, id, point,
                )
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5079-5088.
        pub fn ScrollbarAnimationMouseDown(
            &mut self,
            id: ElementId,
            context: &mut HostImplContext<'_, B>,
        ) -> bool {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::ScrollbarAnimationMouseDown(host, id)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5090-5098.
        pub fn ScrollbarAnimationMouseUp(
            &mut self,
            id: ElementId,
            context: &mut HostImplContext<'_, B>,
        ) -> bool {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::ScrollbarAnimationMouseUp(host, id)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5100-5103.
        pub fn TickScrollAnimations(&mut self, context: &mut HostImplContext<'_, B>) {
            self.WithInputRuntime(context, |host| {
                crate::adapter::input_handler::InputHost::<B>::TickScrollAnimations(host)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5105-5110.
        pub fn PredictViewportBoundsDelta(
            &mut self,
            current: f64,
            distance: Vector2dF,
            context: &mut HostImplContext<'_, B>,
        ) -> f64 {
            self.WithBrowserControls(context, |manager, client| {
                manager.PredictViewportBoundsDelta(client, current, distance)
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5112-5115.
        pub fn ElementHasImplOnlyScrollAnimation(
            &self,
            id: ElementId,
            context: &HostImplContext<'_, B>,
        ) -> bool {
            context
                .services
                .MutatorElementHasImplOnlyScrollAnimation(&self.mutator_host_, id)
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5117-5126.
        pub fn UpdateImplAnimationScrollTargetWithDelta(
            &mut self,
            delta: Vector2dF,
            node_id: i32,
            delayed_by: TimeDelta,
            id: ElementId,
            context: &mut HostImplContext<'_, B>,
        ) -> Option<PointF> {
            self.WithInputRuntime(context,|host|crate::adapter::input_handler::InputHost::<B>::UpdateImplAnimationScrollTargetWithDelta(host,delta,node_id,delayed_by,id))
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5128-5130.
        pub fn HasAnimatedScrollbars(&self) -> bool {
            !self.scrollbar_animation_controllers_.is_empty()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5132-5153.
        pub fn UpdateChildLocalSurfaceId(&mut self, context: &mut HostImplContext<'_, B>) {
            use crate::adapter::unbounded::LocalSurfaceId;
            if !self
                .active_tree_
                .as_ref()
                .unwrap()
                .local_surface_id_from_parent()
                .is_valid()
            {
                return;
            }
            context
                .services
                .ChildLocalSurfaceIdAllocatorUpdateFromParent(
                    &mut self.child_local_surface_id_allocator_,
                    self.active_tree_
                        .as_ref()
                        .unwrap()
                        .local_surface_id_from_parent(),
                );
            if self
                .active_tree_
                .as_mut()
                .unwrap()
                .TakeNewLocalSurfaceIdRequest()
            {
                self.AllocateLocalSurfaceId();
            }
            if self.evicted_local_surface_id_.is_valid()
                && context
                    .services
                    .LocalSurfaceIdIsNewerThanOrEmbeddingChanged(
                        self.GetCurrentLocalSurfaceId(context),
                        &self.evicted_local_surface_id_,
                    )
            {
                self.evicted_local_surface_id_ = Default::default();
                if let Some(provider) = self.resource_provider_.as_mut() {
                    context.services.ResourceProviderSetEvicted(provider, false);
                }
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5155-5158.
        pub fn ReturnResource(
            &mut self,
            resource: <B as crate::adapter::unbounded::UnboundedTypes>::ReturnedResource,
            context: &mut HostImplContext<'_, B>,
        ) {
            context
                .services
                .DelegateReturnResource(self.delegate_, resource);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5160-5168.
        pub fn NotifyNewLocalSurfaceIdExpectedWhilePaused(
            &mut self,
            context: &mut HostImplContext<'_, B>,
        ) {
            if self.new_local_surface_id_expected_ {
                return;
            }
            self.new_local_surface_id_expected_ = true;
            if let Some(sink) = self.layer_tree_frame_sink_ {
                context
                    .services
                    .FrameSinkNotifyNewLocalSurfaceIdExpectedWhilePaused(sink);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5170-5180.
        pub fn CollectScrollbarUpdatesForCommit(&mut self, commit_data: &mut CompositorCommitData) {
            commit_data
                .scrollbars
                .reserve(self.scrollbar_animation_controllers_.len());
            for (&id, controller) in self.scrollbar_animation_controllers_.iter_mut() {
                let controller = controller
                    .as_mut()
                    .expect("source scrollbar controller dereference");
                if controller.visibility_changed() {
                    commit_data.scrollbars.push(ScrollbarsUpdateInfo {
                        element_id: id,
                        hidden: controller.ScrollbarsHidden(),
                    });
                    controller.ClearVisibilityChanged();
                }
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4459-4461.
        // Native continuation of the typed weak once-callback. The tile owner must
        // consume the payload once and call this on the compositor sequence.
        pub(crate) fn RunImageDecodeCallback(
            &mut self,
            callback: HostImageDecodeCallback,
            decode_succeeded: bool,
            context: &mut HostImplContext<'_, B>,
        ) {
            if !callback.target.IsValid() {
                return;
            }
            assert_eq!(
                callback.target.owner, self.owner_id_,
                "image decode callback routed to another HostImpl"
            );
            assert!(
                callback
                    .target
                    .alive
                    .ptr_eq(&std::sync::Arc::downgrade(&self.weak_factory_)),
                "image decode callback belongs to another HostImpl generation"
            );
            self.ImageDecodeFinished(
                callback.request_id,
                callback.speculative,
                decode_succeeded,
                context,
            );
        }
    }

    // Source: cc/trees/layer_tree_host_impl.cc:4700-4702.
    // One source body shared by the owner entry and synchronous binding callback.
    fn SetBeginFrameSourceForLiveDelegate<B: HostImplTypes>(
        delegate: HostImplDelegateId,
        source: Option<&<B as crate::adapter::unbounded::UnboundedTypes>::BeginFrameSource>,
        services: &mut dyn HostFrameSinkCallbackServices<B>,
    ) {
        services.DelegateSetBeginFrameSource(delegate, source);
    }
    // Source: cc/trees/layer_tree_frame_sink_client.h:33-45;
    // cc/trees/layer_tree_host_impl.cc:4645,4598-4605.
    // This lexical facade holds only live sibling borrows. The external sink may
    // invoke these two callbacks synchronously during bind/detach, but cannot retain
    // the facade or access a HostImpl. Future callbacks use the separate weak ID.
    struct HostImplLiveFrameSinkClient<'a, B: HostImplTypes> {
        delegate: HostImplDelegateId,
        returns: super::source_chunk_c::HostImplResourceReturnParts<'a, B>,
        services: &'a mut dyn HostFrameSinkCallbackServices<B>,
    }
    impl<B: HostImplTypes> HostFrameSinkClientFacade<B> for HostImplLiveFrameSinkClient<'_, B> {
        fn SetBeginFrameSource(
            &mut self,
            source: Option<&<B as crate::adapter::unbounded::UnboundedTypes>::BeginFrameSource>,
        ) {
            SetBeginFrameSourceForLiveDelegate::<B>(self.delegate, source, self.services);
        }
        fn ReclaimResources(
            &mut self,
            resources: Vec<<B as crate::adapter::unbounded::UnboundedTypes>::ReturnedResource>,
        ) {
            super::source_chunk_c::ReclaimResourcesWithLiveParts::<B>(
                &mut self.returns,
                resources,
                self.services,
            );
        }
    }
}

// Source: cc/trees/layer_tree_host_impl.h:948-1412.
// Rust borrow split: these are references to the unique owner fields, never a
// state snapshot. The eight separately borrowed children are held by Parts.
pub(crate) struct HostImplRuntimeCore<'a, B: HostImplTypes> {
    pub(crate) weak_factory_: &'a mut Arc<()>,
    pub(crate) unbounded_local_surface_id_: &'a mut <B as LayerTreeTypes>::LocalSurfaceId,
    pub(crate) unbounded_frame_sink_id_:
        &'a mut <B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
    pub(crate) unbounded_frame_sink_handler_: &'a mut Option<Box<UnboundedFrameSinkHandler<B>>>,
    pub(crate) dump_compositor_frame_end_: &'a mut u32,
    pub(crate) dump_compositor_frame_begin_: &'a mut u32,
    pub(crate) dump_compositor_frame_: &'a mut bool,
    pub(crate) tracked_element_rects_from_client_: &'a mut <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects,
    pub(crate) is_handling_interaction_from_client_: &'a mut bool,
    pub(crate) send_frame_token_to_embedder_: &'a mut bool,
    pub(crate) previously_visible_scrollable_elements_: &'a mut BTreeSet<ElementId>,
    pub(crate) new_local_surface_id_expected_: &'a mut bool,
    pub(crate) last_draw_active_tree_begin_frame_args_: &'a mut BeginFrameArgs,
    pub(crate) view_transition_content_rects_:
        &'a mut HashMap<u32, HostViewTransitionElementResourceRects>,
    pub(crate) pending_invalidation_raster_inducing_scrolls_: &'a mut BTreeSet<ElementId>,
    pub(crate) begin_frame_time_delta_: &'a mut TimeDelta,
    pub(crate) frame_max_scroll_delta_: &'a mut f32,
    pub(crate) top_controls_visible_height_: &'a mut f32,
    pub(crate) screenshot_destination_: &'a mut UnguessableToken,
    pub(crate) downsample_metrics_: &'a mut bool,
    pub(crate) throttle_decider_: &'a mut ThrottleDecider<B::ThrottleTypes>,
    pub(crate) contains_srgb_cache_: &'a mut B::ColorSpaceCache,
    pub(crate) has_observed_first_scroll_delay_: &'a mut bool,
    pub(crate) has_non_fling_input_since_last_frame_: &'a mut bool,
    pub(crate) has_input_resetter_:
        &'a mut Arc<crate::cc::base::delayed_unique_notifier::DelayedUniqueNotifier>,
    pub(crate) has_input_for_frame_interval_: &'a mut bool,
    pub(crate) events_metrics_manager_: &'a mut B::EventsMetricsManager,
    pub(crate) lag_tracking_manager_: &'a mut B::AverageLagTrackingManager,
    pub(crate) paint_worklet_tracker_: &'a mut AnimatedPaintWorkletTracker<B>,
    #[cfg(debug_assertions)]
    pub(crate) doing_sync_draw_: &'a mut bool,
    pub(crate) pending_tree_fully_painted_: &'a mut bool,
    pub(crate) paint_worklet_painter_: &'a mut Option<Box<B::PaintWorkletLayerPainter>>,
    pub(crate) frame_trackers_: &'a mut B::FrameSequenceTrackerCollection,
    pub(crate) compositor_frame_reporting_controller_:
        &'a mut Option<Box<B::CompositorFrameReportingController>>,
    pub(crate) frame_sorter_: &'a mut B::FrameSorter,
    pub(crate) presentation_time_callbacks_: &'a mut PresentationTimeCallbackBuffer<
        B::PresentationFeedback,
        B::HostFrameTimingDetails,
        B::SequenceChecker,
    >,
    pub(crate) last_vertical_scroll_direction_: &'a mut VerticalScrollDirection,
    pub(crate) current_local_surface_id_from_client_: &'a mut <B as LayerTreeTypes>::LocalSurfaceId,
    pub(crate) child_local_surface_id_allocator_: &'a mut B::ChildLocalSurfaceIdAllocator,
    pub(crate) evicted_local_surface_id_: &'a mut <B as LayerTreeTypes>::LocalSurfaceId,
    pub(crate) target_local_surface_id_: &'a mut <B as LayerTreeTypes>::LocalSurfaceId,
    pub(crate) last_draw_render_frame_metadata_: &'a mut Option<RenderFrameMetadata<B>>,
    pub(crate) last_draw_referenced_surfaces_: &'a mut BTreeSet<B::SurfaceRange>,
    pub(crate) last_draw_local_surface_id_: &'a mut <B as LayerTreeTypes>::LocalSurfaceId,
    pub(crate) next_frame_token_from_client_: &'a mut u32,
    pub(crate) next_frame_token_: &'a mut B::FrameTokenGenerator,
    pub(crate) render_frame_metadata_observer_: &'a mut Option<Box<B::RenderFrameMetadataObserver>>,
    pub(crate) impl_thread_phase_: &'a mut ImplThreadPhase,
    pub(crate) completed_image_decode_requests_: &'a mut Vec<(i32, bool)>,
    pub(crate) may_throttle_if_undrawn_frames_: &'a mut bool,
    pub(crate) prefers_reduced_motion_: &'a mut bool,
    pub(crate) is_viewport_mobile_optimized_: &'a mut bool,
    pub(crate) visual_device_viewport_size_: &'a mut Size,
    pub(crate) consecutive_frame_with_damage_count_: &'a mut i32,
    pub(crate) prioritize_new_content_due_to_checkerboarding_: &'a mut bool,
    pub(crate) has_valid_layer_tree_frame_sink_: &'a mut bool,
    pub(crate) is_likely_to_require_a_draw_: &'a mut bool,
    pub(crate) requires_high_res_to_draw_: &'a mut bool,
    pub(crate) latency_info_swap_promise_monitor_: &'a mut BTreeSet<HostImplLatencyMonitorId>,
    pub(crate) id_: &'a mut i32,
    pub(crate) task_graph_runner_: &'a mut HostImplTaskGraphRunnerId,
    pub(crate) tree_activation_callback_: &'a mut Option<<B as HostImplTypes>::RepeatingClosure>,
    pub(crate) single_thread_synchronous_task_graph_runner_:
        &'a mut Option<Box<B::SynchronousTaskGraphRunner>>,
    pub(crate) micro_benchmark_controller_:
        &'a mut MicroBenchmarkControllerImpl<B::MicroBenchmarkImpl>,
    pub(crate) rendering_stats_instrumentation_: &'a mut HostImplRenderingStatsId,
    pub(crate) flashed_scrollbars_: &'a mut HashSet<ElementId>,
    pub(crate) dark_mode_filter_: &'a mut Option<HostImplDarkModeFilterId>,
    pub(crate) video_frame_controllers_: &'a mut BTreeSet<HostImplVideoFrameControllerId>,
    pub(crate) mutator_events_: &'a mut Option<Box<B::HostMutatorEvents>>,
    pub(crate) mutator_host_: &'a mut Box<<B as LayerTreeTypes>::MutatorHost>,
    pub(crate) total_invalidated_area_: &'a mut Option<CheckedU32>,
    pub(crate) root_layer_damage_rect_: &'a mut Rect,
    pub(crate) viewport_damage_rect_: &'a mut Rect,
    pub(crate) resourceless_software_draw_: &'a mut bool,
    pub(crate) viewport_rect_for_tile_priority_: &'a mut Rect,
    pub(crate) external_viewport_: &'a mut Rect,
    pub(crate) external_transform_: &'a mut <B as LayerTreeTypes>::Transform,
    pub(crate) max_memory_needed_bytes_: &'a mut usize,
    pub(crate) debug_rect_history_: &'a mut Option<Box<DebugRectHistory>>,
    pub(crate) memory_history_: &'a mut Option<Box<MemoryHistory>>,
    pub(crate) page_scale_animation_: &'a mut Option<Box<PageScaleAnimation>>,
    pub(crate) tile_manager_: &'a mut B::TileManager,
    pub(crate) cached_managed_memory_policy_: &'a mut ManagedMemoryPolicy,
    pub(crate) visible_: &'a mut bool,
    pub(crate) debug_state_: &'a mut LayerTreeDebugState,
    pub(crate) tile_priorities_dirty_: &'a mut bool,
    pub(crate) swap_promises_for_main_thread_scroll_update_: &'a mut Vec<Box<B::SwapPromise>>,
    pub(crate) accumulated_scroll_deltas_by_element_id_: &'a mut HashMap<ElementId, Vector2dF>,
    pub(crate) scroll_accumulated_this_frame_: &'a mut Vector2dF,
    pub(crate) global_tile_state_: &'a mut B::GlobalTileState,
    pub(crate) image_decode_cache_holder_:
        &'a mut Option<Box<ImageDecodeCacheHolder<B::ImageDecodeFactory>>>,
    pub(crate) pending_raster_queries_: &'a mut Option<Box<B::RasterQueryQueue>>,
    pub(crate) resource_pool_: &'a mut Option<Box<B::ResourcePool>>,
    pub(crate) raster_buffer_provider_: &'a mut Option<Box<B::RasterBufferProvider>>,
    pub(crate) raster_caps_: &'a mut RasterCapabilities,
    pub(crate) worker_context_visibility_: &'a mut Option<Box<B::ContextVisibility>>,
    pub(crate) compositor_context_visibility_: &'a mut Option<Box<B::ContextVisibility>>,
    pub(crate) layer_context_: &'a mut Option<Box<<B as HostImplTypes>::LayerContext>>,
    pub(crate) layer_tree_frame_sink_: &'a mut Option<HostImplFrameSinkId>,
    pub(crate) ui_resource_changes_:
        &'a mut UIResourceChangeMap<crate::adapter::layer_tree_impl::UIResourceId>,
    pub(crate) evicted_ui_resources_:
        &'a mut BTreeSet<crate::adapter::layer_tree_impl::UIResourceId>,
    pub(crate) deleted_ui_resources_:
        &'a mut HashMap<crate::adapter::layer_tree_impl::UIResourceId, UIResourceData<B>>,
    pub(crate) ui_resource_map_:
        &'a mut HashMap<crate::adapter::layer_tree_impl::UIResourceId, UIResourceData<B>>,
    pub(crate) resource_provider_: &'a mut Option<Box<B::ClientResourceProvider>>,
    pub(crate) is_synchronous_single_threaded_: &'a mut bool,
    pub(crate) settings_: &'a LayerTreeSettings,
    pub(crate) current_begin_frame_tracker_: &'a mut BeginFrameTracker,
    pub(crate) task_runner_provider_: &'a mut HostImplTaskRunnerProviderId,
    pub(crate) scheduling_delegate_: &'a mut HostImplSchedulingDelegateId,
    pub(crate) delegate_: &'a mut HostImplDelegateId,
    pub(crate) owner_id_: &'a mut LayerTreeHostId,
}

pub(crate) struct HostImplRuntimeParts<'a, B: HostImplTypes> {
    pub(crate) core: HostImplRuntimeCore<'a, B>,
    pub(crate) image_animation_controller_: &'a mut Option<Box<ImageAnimationController>>,
    pub(crate) viewport_: &'a mut Option<Box<Viewport>>,
    pub(crate) scrollbar_animation_controllers_:
        &'a mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    pub(crate) browser_controls_offset_manager_: &'a mut Option<Box<BrowserControlsOffsetManager>>,
    pub(crate) recycle_tree_: &'a mut Option<Box<LayerTreeImpl<B>>>,
    pub(crate) pending_tree_: &'a mut Option<Box<LayerTreeImpl<B>>>,
    pub(crate) active_tree_: &'a mut Option<Box<LayerTreeImpl<B>>>,
    pub(crate) input_delegate_: &'a mut Option<Box<B::InputHandler>>,
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    fn BorrowRuntime(&mut self) -> HostImplRuntimeParts<'_, B> {
        let Self {
            weak_factory_,
            unbounded_local_surface_id_,
            unbounded_frame_sink_id_,
            unbounded_frame_sink_handler_,
            dump_compositor_frame_end_,
            dump_compositor_frame_begin_,
            dump_compositor_frame_,
            tracked_element_rects_from_client_,
            is_handling_interaction_from_client_,
            send_frame_token_to_embedder_,
            previously_visible_scrollable_elements_,
            new_local_surface_id_expected_,
            last_draw_active_tree_begin_frame_args_,
            view_transition_content_rects_,
            pending_invalidation_raster_inducing_scrolls_,
            begin_frame_time_delta_,
            frame_max_scroll_delta_,
            top_controls_visible_height_,
            screenshot_destination_,
            downsample_metrics_,
            throttle_decider_,
            contains_srgb_cache_,
            has_observed_first_scroll_delay_,
            has_non_fling_input_since_last_frame_,
            has_input_resetter_,
            has_input_for_frame_interval_,
            events_metrics_manager_,
            lag_tracking_manager_,
            paint_worklet_tracker_,
            #[cfg(debug_assertions)]
            doing_sync_draw_,
            pending_tree_fully_painted_,
            paint_worklet_painter_,
            frame_trackers_,
            compositor_frame_reporting_controller_,
            frame_sorter_,
            presentation_time_callbacks_,
            last_vertical_scroll_direction_,
            current_local_surface_id_from_client_,
            child_local_surface_id_allocator_,
            evicted_local_surface_id_,
            target_local_surface_id_,
            last_draw_render_frame_metadata_,
            last_draw_referenced_surfaces_,
            last_draw_local_surface_id_,
            next_frame_token_from_client_,
            next_frame_token_,
            render_frame_metadata_observer_,
            image_animation_controller_,
            impl_thread_phase_,
            completed_image_decode_requests_,
            may_throttle_if_undrawn_frames_,
            prefers_reduced_motion_,
            is_viewport_mobile_optimized_,
            visual_device_viewport_size_,
            viewport_,
            consecutive_frame_with_damage_count_,
            prioritize_new_content_due_to_checkerboarding_,
            has_valid_layer_tree_frame_sink_,
            is_likely_to_require_a_draw_,
            requires_high_res_to_draw_,
            latency_info_swap_promise_monitor_,
            id_,
            task_graph_runner_,
            tree_activation_callback_,
            single_thread_synchronous_task_graph_runner_,
            micro_benchmark_controller_,
            rendering_stats_instrumentation_,
            flashed_scrollbars_,
            scrollbar_animation_controllers_,
            dark_mode_filter_,
            video_frame_controllers_,
            mutator_events_,
            mutator_host_,
            total_invalidated_area_,
            root_layer_damage_rect_,
            viewport_damage_rect_,
            resourceless_software_draw_,
            viewport_rect_for_tile_priority_,
            external_viewport_,
            external_transform_,
            max_memory_needed_bytes_,
            debug_rect_history_,
            memory_history_,
            page_scale_animation_,
            browser_controls_offset_manager_,
            tile_manager_,
            cached_managed_memory_policy_,
            visible_,
            debug_state_,
            tile_priorities_dirty_,
            swap_promises_for_main_thread_scroll_update_,
            accumulated_scroll_deltas_by_element_id_,
            scroll_accumulated_this_frame_,
            recycle_tree_,
            pending_tree_,
            active_tree_,
            global_tile_state_,
            image_decode_cache_holder_,
            pending_raster_queries_,
            resource_pool_,
            raster_buffer_provider_,
            raster_caps_,
            worker_context_visibility_,
            compositor_context_visibility_,
            layer_context_,
            layer_tree_frame_sink_,
            ui_resource_changes_,
            evicted_ui_resources_,
            deleted_ui_resources_,
            ui_resource_map_,
            resource_provider_,
            is_synchronous_single_threaded_,
            settings_,
            input_delegate_,
            current_begin_frame_tracker_,
            task_runner_provider_,
            scheduling_delegate_,
            delegate_,
            owner_id_,
        } = self;
        HostImplRuntimeParts {
            core: HostImplRuntimeCore {
                weak_factory_,
                unbounded_local_surface_id_,
                unbounded_frame_sink_id_,
                unbounded_frame_sink_handler_,
                dump_compositor_frame_end_,
                dump_compositor_frame_begin_,
                dump_compositor_frame_,
                tracked_element_rects_from_client_,
                is_handling_interaction_from_client_,
                send_frame_token_to_embedder_,
                previously_visible_scrollable_elements_,
                new_local_surface_id_expected_,
                last_draw_active_tree_begin_frame_args_,
                view_transition_content_rects_,
                pending_invalidation_raster_inducing_scrolls_,
                begin_frame_time_delta_,
                frame_max_scroll_delta_,
                top_controls_visible_height_,
                screenshot_destination_,
                downsample_metrics_,
                throttle_decider_,
                contains_srgb_cache_,
                has_observed_first_scroll_delay_,
                has_non_fling_input_since_last_frame_,
                has_input_resetter_,
                has_input_for_frame_interval_,
                events_metrics_manager_,
                lag_tracking_manager_,
                paint_worklet_tracker_,
                #[cfg(debug_assertions)]
                doing_sync_draw_,
                pending_tree_fully_painted_,
                paint_worklet_painter_,
                frame_trackers_,
                compositor_frame_reporting_controller_,
                frame_sorter_,
                presentation_time_callbacks_,
                last_vertical_scroll_direction_,
                current_local_surface_id_from_client_,
                child_local_surface_id_allocator_,
                evicted_local_surface_id_,
                target_local_surface_id_,
                last_draw_render_frame_metadata_,
                last_draw_referenced_surfaces_,
                last_draw_local_surface_id_,
                next_frame_token_from_client_,
                next_frame_token_,
                render_frame_metadata_observer_,
                impl_thread_phase_,
                completed_image_decode_requests_,
                may_throttle_if_undrawn_frames_,
                prefers_reduced_motion_,
                is_viewport_mobile_optimized_,
                visual_device_viewport_size_,
                consecutive_frame_with_damage_count_,
                prioritize_new_content_due_to_checkerboarding_,
                has_valid_layer_tree_frame_sink_,
                is_likely_to_require_a_draw_,
                requires_high_res_to_draw_,
                latency_info_swap_promise_monitor_,
                id_,
                task_graph_runner_,
                tree_activation_callback_,
                single_thread_synchronous_task_graph_runner_,
                micro_benchmark_controller_,
                rendering_stats_instrumentation_,
                flashed_scrollbars_,
                dark_mode_filter_,
                video_frame_controllers_,
                mutator_events_,
                mutator_host_,
                total_invalidated_area_,
                root_layer_damage_rect_,
                viewport_damage_rect_,
                resourceless_software_draw_,
                viewport_rect_for_tile_priority_,
                external_viewport_,
                external_transform_,
                max_memory_needed_bytes_,
                debug_rect_history_,
                memory_history_,
                page_scale_animation_,
                tile_manager_,
                cached_managed_memory_policy_,
                visible_,
                debug_state_,
                tile_priorities_dirty_,
                swap_promises_for_main_thread_scroll_update_,
                accumulated_scroll_deltas_by_element_id_,
                scroll_accumulated_this_frame_,
                global_tile_state_,
                image_decode_cache_holder_,
                pending_raster_queries_,
                resource_pool_,
                raster_buffer_provider_,
                raster_caps_,
                worker_context_visibility_,
                compositor_context_visibility_,
                layer_context_,
                layer_tree_frame_sink_,
                ui_resource_changes_,
                evicted_ui_resources_,
                deleted_ui_resources_,
                ui_resource_map_,
                resource_provider_,
                is_synchronous_single_threaded_,
                settings_,
                current_begin_frame_tracker_,
                task_runner_provider_,
                scheduling_delegate_,
                delegate_,
                owner_id_,
            },
            image_animation_controller_,
            viewport_,
            scrollbar_animation_controllers_,
            browser_controls_offset_manager_,
            recycle_tree_,
            pending_tree_,
            active_tree_,
            input_delegate_,
        }
    }
}

/// The selected tree is lent to the canonical callback. Reserved records only
/// its resident identity; it is distinct from a native nullptr slot.
pub(crate) enum HostImplTreeSlot<'a, B: HostImplTypes> {
    Direct(&'a mut LayerTreeImpl<B>),
    Resident(&'a mut Option<Box<LayerTreeImpl<B>>>),
    Reserved(crate::adapter::layer_tree_impl::LayerTreeId),
}
impl<B: HostImplTypes> HostImplTreeSlot<'_, B> {
    pub(crate) fn identity(&self) -> Option<crate::adapter::layer_tree_impl::LayerTreeId> {
        match self {
            Self::Resident(slot) => slot.as_deref().map(|tree| tree.id),
            Self::Reserved(id) => Some(*id),
            Self::Direct(tree) => Some(tree.id),
        }
    }
    pub(crate) fn reborrow(&mut self) -> HostImplTreeSlot<'_, B> {
        match self {
            Self::Resident(slot) => HostImplTreeSlot::Resident(&mut **slot),
            Self::Reserved(id) => HostImplTreeSlot::Reserved(*id),
            Self::Direct(tree) => HostImplTreeSlot::Direct(&mut **tree),
        }
    }
}

/// Per-call live host facade. Native owner state is shared through field borrows;
/// selected receiver trees are deliberately excluded from this facade.
pub(crate) struct HostImplTreeHostFacade<'a, 'c, B: HostImplTypes> {
    pub(crate) core: HostImplRuntimeCore<'a, B>,
    pub(crate) active_tree_: HostImplTreeSlot<'a, B>,
    pub(crate) pending_tree_: HostImplTreeSlot<'a, B>,
    pub(crate) recycle_tree_: HostImplTreeSlot<'a, B>,
    pub(crate) input_delegate_: &'a mut Option<Box<B::InputHandler>>,
    pub(crate) browser_controls_offset_manager_: HostImplControlsSlot<'a>,
    pub(crate) scrollbar_animation_controllers_:
        &'a mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    pub(crate) image_animation_controller_: &'a mut Option<Box<ImageAnimationController>>,
    pub(crate) context: &'a mut HostImplContext<'c, B>,
}

/// Actual tree references for input calls. Unlike the TreeHost roles above,
/// every variant allows access to the current resident tree allocation.
pub(crate) enum HostImplInputTree<'a, B: HostImplTypes> {
    Slot(&'a mut Option<Box<LayerTreeImpl<B>>>),
    Direct(&'a mut LayerTreeImpl<B>),
}
impl<B: HostImplTypes> HostImplInputTree<'_, B> {
    pub(crate) fn as_ref(&self) -> Option<&LayerTreeImpl<B>> {
        match self {
            Self::Slot(slot) => slot.as_deref(),
            Self::Direct(tree) => Some(tree),
        }
    }
    pub(crate) fn as_mut(&mut self) -> Option<&mut LayerTreeImpl<B>> {
        match self {
            Self::Slot(slot) => slot.as_deref_mut(),
            Self::Direct(tree) => Some(tree),
        }
    }
    pub(crate) fn is_some(&self) -> bool {
        self.as_ref().is_some()
    }
}

pub(crate) struct HostImplInputFacade<'a, B: HostImplTypes> {
    pub(crate) animation_reentry:
        Option<&'a mut dyn crate::adapter::animation_host::ScrollAnimationReentry>,
    pub(crate) core: HostImplRuntimeCore<'a, B>,
    pub(crate) active_tree_: HostImplInputTree<'a, B>,
    pub(crate) pending_tree_: HostImplInputTree<'a, B>,
    pub(crate) recycle_tree_: HostImplInputTree<'a, B>,
    pub(crate) browser_controls_offset_manager_: HostImplControlsSlot<'a>,
    pub(crate) viewport_: HostImplViewportSlot<'a>,
    pub(crate) scrollbar_animation_controllers_:
        &'a mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    pub(crate) image_animation_controller_: &'a mut Option<Box<ImageAnimationController>>,
    pub(crate) context: HostImplContext<'a, B>,
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    /// Rust borrow split for the source raw_ptr child-to-owner calls.
    /// The selected resident tree is borrowed independently of its host facade.
    pub(super) fn WithActiveTree<'a, R>(
        &'a mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            &'a mut LayerTreeImpl<B>,
            &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        ) -> R,
    ) -> R {
        let HostImplRuntimeParts {
            core,
            active_tree_,
            pending_tree_,
            recycle_tree_,
            input_delegate_,
            browser_controls_offset_manager_,
            viewport_: _,
            scrollbar_animation_controllers_,
            image_animation_controller_,
        } = self.BorrowRuntime();
        let tree = active_tree_
            .as_deref_mut()
            .expect("source resident active tree");
        let identity = tree.id;
        let mut host = HostImplTreeHostFacade {
            core,
            active_tree_: HostImplTreeSlot::Reserved(identity),
            pending_tree_: HostImplTreeSlot::Resident(pending_tree_),
            recycle_tree_: HostImplTreeSlot::Resident(recycle_tree_),
            input_delegate_,
            browser_controls_offset_manager_: HostImplControlsSlot::Resident(
                browser_controls_offset_manager_,
            ),
            scrollbar_animation_controllers_,
            image_animation_controller_,
            context,
        };
        let mut child_context =
            crate::cc::trees::layer_tree_impl::LayerTreeContext { host: &mut host };
        callback(tree, &mut child_context)
    }
    /// Rust borrow split for the source raw_ptr child-to-owner calls.
    /// The selected resident tree is borrowed independently of its host facade.
    pub(crate) fn WithPendingTree<'a, R>(
        &'a mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            &'a mut LayerTreeImpl<B>,
            &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        ) -> R,
    ) -> R {
        let HostImplRuntimeParts {
            core,
            active_tree_,
            pending_tree_,
            recycle_tree_,
            input_delegate_,
            browser_controls_offset_manager_,
            viewport_: _,
            scrollbar_animation_controllers_,
            image_animation_controller_,
        } = self.BorrowRuntime();
        let tree = pending_tree_
            .as_deref_mut()
            .expect("source resident pending tree");
        let identity = tree.id;
        let mut host = HostImplTreeHostFacade {
            core,
            active_tree_: HostImplTreeSlot::Resident(active_tree_),
            pending_tree_: HostImplTreeSlot::Reserved(identity),
            recycle_tree_: HostImplTreeSlot::Resident(recycle_tree_),
            input_delegate_,
            browser_controls_offset_manager_: HostImplControlsSlot::Resident(
                browser_controls_offset_manager_,
            ),
            scrollbar_animation_controllers_,
            image_animation_controller_,
            context,
        };
        let mut child_context =
            crate::cc::trees::layer_tree_impl::LayerTreeContext { host: &mut host };
        callback(tree, &mut child_context)
    }
    /// Rust borrow split for the source raw_ptr child-to-owner calls.
    /// The selected resident tree is borrowed independently of its host facade.
    fn WithRecycleTree<'a, R>(
        &'a mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            &'a mut LayerTreeImpl<B>,
            &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        ) -> R,
    ) -> R {
        let HostImplRuntimeParts {
            core,
            active_tree_,
            pending_tree_,
            recycle_tree_,
            input_delegate_,
            browser_controls_offset_manager_,
            viewport_: _,
            scrollbar_animation_controllers_,
            image_animation_controller_,
        } = self.BorrowRuntime();
        let tree = recycle_tree_
            .as_deref_mut()
            .expect("source resident recycle tree");
        let identity = tree.id;
        let mut host = HostImplTreeHostFacade {
            core,
            active_tree_: HostImplTreeSlot::Resident(active_tree_),
            pending_tree_: HostImplTreeSlot::Resident(pending_tree_),
            recycle_tree_: HostImplTreeSlot::Reserved(identity),
            input_delegate_,
            browser_controls_offset_manager_: HostImplControlsSlot::Resident(
                browser_controls_offset_manager_,
            ),
            scrollbar_animation_controllers_,
            image_animation_controller_,
            context,
        };
        let mut child_context =
            crate::cc::trees::layer_tree_impl::LayerTreeContext { host: &mut host };
        callback(tree, &mut child_context)
    }
    fn WithActiveAndPendingTrees<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            &mut LayerTreeImpl<B>,
            &mut LayerTreeImpl<B>,
            &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        ) -> R,
    ) -> R {
        let HostImplRuntimeParts {
            core,
            active_tree_,
            pending_tree_,
            recycle_tree_,
            input_delegate_,
            browser_controls_offset_manager_,
            viewport_: _,
            scrollbar_animation_controllers_,
            image_animation_controller_,
        } = self.BorrowRuntime();
        let active = active_tree_.as_deref_mut().expect("source active tree");
        let pending = pending_tree_.as_deref_mut().expect("source pending tree");
        let mut host = HostImplTreeHostFacade {
            core,
            active_tree_: HostImplTreeSlot::Reserved(active.id),
            pending_tree_: HostImplTreeSlot::Reserved(pending.id),
            recycle_tree_: HostImplTreeSlot::Resident(recycle_tree_),
            input_delegate_,
            browser_controls_offset_manager_: HostImplControlsSlot::Resident(
                browser_controls_offset_manager_,
            ),
            scrollbar_animation_controllers_,
            image_animation_controller_,
            context,
        };
        let mut child_context =
            crate::cc::trees::layer_tree_impl::LayerTreeContext { host: &mut host };
        callback(active, pending, &mut child_context)
    }
    pub(crate) fn WithInputHandler<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(Option<&mut B::InputHandler>, &mut HostImplInputFacade<'_, B>) -> R,
    ) -> R {
        self.WithInputHandlerAndAnimation(context, None, callback)
    }
    fn WithInputHandlerAndAnimation<'a, R>(
        &'a mut self,
        context: &'a mut HostImplContext<'_, B>,
        selected: Option<&'a mut dyn crate::adapter::animation_host::ScrollAnimationReentry>,
        callback: impl FnOnce(Option<&mut B::InputHandler>, &mut HostImplInputFacade<'_, B>) -> R,
    ) -> R {
        let HostImplRuntimeParts {
            core,
            active_tree_,
            pending_tree_,
            recycle_tree_,
            input_delegate_,
            browser_controls_offset_manager_,
            viewport_,
            scrollbar_animation_controllers_,
            image_animation_controller_,
        } = self.BorrowRuntime();
        let mut host = HostImplInputFacade {
            animation_reentry: selected,
            core,
            active_tree_: HostImplInputTree::Slot(active_tree_),
            pending_tree_: HostImplInputTree::Slot(pending_tree_),
            recycle_tree_: HostImplInputTree::Slot(recycle_tree_),
            browser_controls_offset_manager_: HostImplControlsSlot::Resident(
                browser_controls_offset_manager_,
            ),
            viewport_: HostImplViewportSlot::Resident(viewport_),
            scrollbar_animation_controllers_,
            image_animation_controller_,
            context: HostImplContext {
                services: &mut *context.services,
            },
        };
        callback(input_delegate_.as_deref_mut(), &mut host)
    }
    fn WithActiveTreeDamage<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(&B::DamageContext<'_>) -> R,
    ) -> R {
        self.WithActiveTree(context, |tree, c| {
            let damage = B::MakeDamageContext(tree, &*c.host);
            callback(&damage)
        })
    }
}

use crate::adapter::unbounded::LocalSurfaceId;
use crate::cc::trees::scroll_node::ScrollNode;
use crate::cc::{
    input::actively_scrolling_type::ActivelyScrollingType,
    trees::{layer_tree_impl::LifecycleState, tree_synchronizer::TreeSynchronizer},
};
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:3932-3944.
    pub fn DidLoseLayerTreeFrameSink(&mut self, context: &mut HostImplContext<'_, B>) {
        if !self.has_valid_layer_tree_frame_sink_ {
            return;
        }
        self.has_valid_layer_tree_frame_sink_ = false;
        context
            .services
            .DelegateDidLoseLayerTreeFrameSink(self.delegate_);
        context
            .services
            .LagTrackingClear(&mut self.lag_tracking_manager_);
        context
            .services
            .FrameSorterReset(&mut self.frame_sorter_, false);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3946-3948.
    pub fn OnlyExpandTopControlsAtPageTop(&self) -> bool {
        self.active_tree_
            .as_ref()
            .unwrap()
            .only_expand_top_controls_at_page_top()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3950-3952.
    pub fn HaveRootScrollNode(&self) -> bool {
        self.InnerViewportScrollNode().is_some()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3954-3958.
    // Existing owner SetNeedsCommit/SetNeedsCommitWithReason share the source body.
    // Source: cc/trees/layer_tree_host_impl.cc:3960-3962.
    pub fn CurrentFrameInterval(&self) -> TimeDelta {
        self.CurrentBeginFrameInterval()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3964-3966.
    pub fn InnerViewportScrollNode(&self) -> Option<&ScrollNode> {
        self.active_tree_
            .as_ref()
            .unwrap()
            .InnerViewportScrollNode()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3968-3970.
    pub fn OuterViewportScrollNode(&self) -> Option<&ScrollNode> {
        self.active_tree_
            .as_ref()
            .unwrap()
            .OuterViewportScrollNode()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3972-3974.
    pub fn CurrentlyScrollingNodeMutable(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) -> Option<&mut ScrollNode> {
        self.WithActiveTree(context, |tree, c| tree.CurrentlyScrollingNodeMutable(c))
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3976-3978.
    pub fn CurrentlyScrollingNode(&self) -> Option<&ScrollNode> {
        self.active_tree_.as_ref().unwrap().CurrentlyScrollingNode()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3980-3985.
    // IsPinchGestureActive is already implemented by the canonical owner.
    // Source: cc/trees/layer_tree_host_impl.cc:3987-3992.
    pub fn GetActivelyScrollingType(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) -> ActivelyScrollingType {
        if self.input_delegate_.is_none() {
            return ActivelyScrollingType::kNone;
        }
        self.WithInputHandler(context, |input, host| {
            input.unwrap().GetActivelyScrollingType(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3994-4005.
    pub fn IsHandlingInteraction(&mut self, context: &mut HostImplContext<'_, B>) -> bool {
        if self.settings_.trees_in_viz_in_viz_process {
            return self.is_handling_interaction_from_client_;
        }
        if self.input_delegate_.is_some() {
            return self.GetActivelyScrollingType(context) != ActivelyScrollingType::kNone
                || self
                    .input_delegate_
                    .as_ref()
                    .unwrap()
                    .IsHandlingTouchSequence();
        }
        false
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4007-4009.
    pub fn IsCurrentScrollMainRepainted(&mut self, context: &mut HostImplContext<'_, B>) -> bool {
        if self.input_delegate_.is_none() {
            return false;
        }
        self.WithInputHandler(context, |input, host| {
            input.unwrap().IsCurrentScrollMainRepainted(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4011-4016.
    pub fn SetExternalPinchGestureActive(&mut self, active: bool) {
        debug_assert!(self.input_delegate_.is_some() || !active);
        if let Some(input) = self.input_delegate_.as_mut() {
            input.set_external_pinch_gesture_active(active);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4018-4049.
    pub fn PushScrollbarOpacitiesFromActiveToPending(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) {
        if self.active_tree_.is_none() {
            return;
        }
        self.WithScrollbarOpacityTrees(context, |controllers, client, pending| {
            for controller in controllers.values() {
                for scrollbar in controller.as_ref().unwrap().Scrollbars(client) {
                    let element_id = client.Scrollbar_element_id(&scrollbar);
                    if let Some(source) = client
                        .ActivePropertyTrees()
                        .effect_tree()
                        .FindNodeFromElementId(element_id)
                    {
                        if let Some(target) = pending
                            .property_trees_mut()
                            .effect_tree_mutable()
                            .MutableFindNodeFromElementId(element_id)
                        {
                            let source_opacity = source.opacity;
                            let target_opacity = target.opacity;
                            if source_opacity == target_opacity {
                                continue;
                            }
                            target.opacity = source_opacity;
                            pending
                                .property_trees_mut()
                                .effect_tree_mutable()
                                .set_needs_update(true);
                        }
                    }
                }
            }
        });
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4051-4197.
    pub fn ActivateSyncTree(&mut self, context: &mut HostImplContext<'_, B>) {
        context
            .services
            .TraceActivateSyncTree(&mut || self.sync_tree().unwrap().trace_id());
        if self.pending_tree_.is_some() {
            let pending = self.pending_tree_.as_ref().unwrap();
            context
                .services
                .TracePendingTreeActivated(pending.id, pending.local_surface_id_from_parent());
            self.active_tree_
                .as_mut()
                .unwrap()
                .lifecycle()
                .AdvanceTo(LifecycleState::kBeginningSync);
            self.WithPendingTree(context, |tree, c| tree.ProcessUIResourceRequestQueue(c));
            if self.pending_tree_.as_ref().unwrap().needs_full_tree_sync() {
                self.WithActiveAndPendingTrees(context, |active, pending, c| {
                    TreeSynchronizer::SynchronizeTreesFromPending(pending, active, c);
                    active.set_needs_full_tree_sync(true);
                });
            }
            self.PushScrollbarOpacitiesFromActiveToPending(context);
            self.WithActiveAndPendingTrees(context, |active, pending, c| {
                pending.PushPropertyTreesTo(active, c)
            });
            self.active_tree_
                .as_mut()
                .unwrap()
                .lifecycle()
                .AdvanceTo(LifecycleState::kSyncedPropertyTrees);
            self.WithActiveAndPendingTrees(context, |active, pending, c| {
                TreeSynchronizer::PushLayerPropertiesFromPending(pending, active, c);
            });
            self.active_tree_
                .as_mut()
                .unwrap()
                .lifecycle()
                .AdvanceTo(LifecycleState::kSyncedLayerProperties);
            self.WithActiveAndPendingTrees(context, |active, pending, c| {
                pending.PushPropertiesTo(active, c)
            });
            if !self.pending_tree_.as_ref().unwrap().LayerListIsEmpty() {
                self.pending_tree_
                    .as_mut()
                    .unwrap()
                    .property_trees_mut()
                    .ResetAllChangeTracking();
            }
            self.active_tree_
                .as_mut()
                .unwrap()
                .lifecycle()
                .AdvanceTo(LifecycleState::kNotSyncing);
            debug_assert!(self.recycle_tree_.is_none());
            // Source explicit owner swap; never takes a whole tree to evade a borrow.
            std::mem::swap(&mut self.pending_tree_, &mut self.recycle_tree_);
            context
                .services
                .PromoteScrollTimelinesPendingToActive(&mut self.mutator_host_);
            self.ActivateAnimations(context);
            self.ActivateStateForImages(context);
        } else {
            self.WithActiveTree(context, |tree, c| tree.ProcessUIResourceRequestQueue(c));
        }
        self.WithActiveTree(context, |tree, c| tree.UpdateViewportContainerSizes(c));
        self.WithActiveTree(context, |tree, c| {
            if let Some(inner) = tree.InnerViewportScrollNode().map(|node| node.id) {
                tree.ClampScrollNodeToMaxScrollOffset(inner, c);
                debug_assert!(tree.OuterViewportScrollNode().is_some());
                let outer = tree.OuterViewportScrollNode().unwrap().id;
                tree.ClampScrollNodeToMaxScrollOffset(outer, c);
            }
        });
        self.WithActiveTree(context, |tree, c| tree.DidBecomeActive(c));
        self.RenewTreePriority(context);
        if self
            .active_tree_
            .as_ref()
            .unwrap()
            .picture_layers()
            .next()
            .is_some()
        {
            self.DidModifyTilePriorities(false, context);
        }
        let screenshot_token = self
            .active_tree_
            .as_mut()
            .unwrap()
            .TakeScreenshotDestinationToken();
        if self.GetCurrentLocalSurfaceId(context).is_valid() {
            self.screenshot_destination_ = screenshot_token;
        } else if !screenshot_token.is_empty() {
            context.services.InvalidPreviousSurfaceScreenshot();
        }
        self.UpdateChildLocalSurfaceId(context);
        let can_draw = self.CanDraw(context);
        context
            .services
            .OnCanDrawStateChanged(self.delegate_, can_draw);
        context.services.DelegateDidActivateSyncTree(self.delegate_);
        if let Some(callback) = self.tree_activation_callback_.as_ref() {
            context.services.RunTreeActivationCallback(callback);
        }
        let pending_page_scale_animation = self
            .active_tree_
            .as_mut()
            .unwrap()
            .TakePendingPageScaleAnimation();
        if let Some(animation) = pending_page_scale_animation {
            self.StartPageScaleAnimation(
                animation.target_offset,
                animation.use_anchor,
                animation.scale,
                animation.duration,
                context,
            );
        }
        self.WithInputHandler(context, |input, host| {
            if let Some(input) = input {
                input.DidActivatePendingTree(host);
            }
        });
        if self.layer_context_.is_some()
            && !self.visible_
            && self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures())
        {
            self.FlushDisplayTree(context);
        }
        context
            .services
            .VerboseLogActivatedTree(self.active_tree_.as_ref().unwrap());
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4199-4204.
    pub fn ActivateStateForImages(&mut self, context: &mut HostImplContext<'_, B>) {
        assert!(!self.settings_.trees_in_viz_in_viz_process);
        assert!(self.image_animation_controller_.is_some());
        self.WithImageAnimationController(context, |controller, client| {
            controller.unwrap().DidActivate(client)
        });
        B::ResidentTileDidActivate(&mut self.tile_manager_, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4206-4266.
    pub fn SetVisible(&mut self, visible: bool, context: &mut HostImplContext<'_, B>) {
        debug_assert!(context.services.IsImplThread(self.task_runner_provider_));
        if self.visible_ == visible {
            return;
        }
        self.visible_ = visible;
        if !self.visible_ {
            context
                .services
                .FrameSorterReset(&mut self.frame_sorter_, false);
            drop(self.active_tree_.as_mut().unwrap().TakeEventsMetrics());
            drop(
                self.active_tree_
                    .as_mut()
                    .unwrap()
                    .TakeRasterEventsMetrics(),
            );
            context
                .services
                .TakeSavedEventsMetrics(&mut self.events_metrics_manager_);
            if let Some(pending) = self.pending_tree_.as_mut() {
                drop(pending.TakeEventsMetrics());
                drop(pending.TakeRasterEventsMetrics());
            }
        }
        context.services.ReportingSetVisible(
            self.compositor_frame_reporting_controller_
                .as_deref_mut()
                .unwrap(),
            self.visible_,
        );
        DidVisibilityChange(context.services, self.owner_id_.0, self.visible_);
        if !self.settings_.trees_in_viz_in_viz_process {
            let policy = self.ActualManagedMemoryPolicy(context);
            self.UpdateTileManagerMemoryPolicy(policy, context);
        }
        if self.visible_ {
            self.SetRequiresHighResToDraw();
            if !self.active_tree_.as_ref().unwrap().LayerListIsEmpty() {
                self.SetFullViewportDamage(context);
                self.SetNeedsRedraw(false, false, context);
            }
        } else if !self.settings_.trees_in_viz_in_viz_process {
            self.EvictAllUIResources(context);
            self.PrepareTiles(context);
            B::UnlockResidentTileImages(&mut self.tile_manager_, context);
        }
        if self.layer_context_.is_some() {
            if self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures())
                && !visible
            {
                self.FlushDisplayTree(context);
            }
            context
                .services
                .LayerContextSetVisible(self.layer_context_.as_deref_mut().unwrap(), visible);
        }
        self.active_tree_.as_mut().unwrap().SetVisible(visible);
        context
            .services
            .ResourceProviderSetVisible(self.resource_provider_.as_deref_mut().unwrap(), visible);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4268-4272,4274-4290,4292-4305.
    // SetNeedsOneBeginImplFrame, SetNeedsRedraw, ActualManagedMemoryPolicy are
    // already integrated by scheduler, including their exact source decisions.
    // Source: cc/trees/layer_tree_host_impl.cc:4307-4317.
    pub fn ReleaseTreeResources(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithActiveTree(context, |tree, c| tree.ReleaseResources(c));
        if self.pending_tree_.is_some() {
            self.WithPendingTree(context, |tree, c| tree.ReleaseResources(c));
        }
        if self.recycle_tree_.is_some() {
            self.WithRecycleTree(context, |tree, c| tree.ReleaseResources(c));
        }
        self.EvictAllUIResources(context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4319-4335.
    pub fn ReleaseTileResources(&mut self, context: &mut HostImplContext<'_, B>) {
        if self.settings_.trees_in_viz_in_viz_process {
            return;
        }
        self.WithActiveTree(context, |tree, c| tree.ReleaseTileResources(c));
        if self.pending_tree_.is_some() {
            self.WithPendingTree(context, |tree, c| tree.ReleaseTileResources(c));
        }
        if self.recycle_tree_.is_some() {
            self.WithRecycleTree(context, |tree, c| tree.ReleaseTileResources(c));
        }
        self.active_tree_
            .as_mut()
            .unwrap()
            .set_needs_update_draw_properties();
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4337-4349.
    pub fn RecreateTileResources(&mut self, context: &mut HostImplContext<'_, B>) {
        if self.settings_.trees_in_viz_in_viz_process {
            return;
        }
        self.WithActiveTree(context, |tree, c| tree.RecreateTileResources(c));
        if self.pending_tree_.is_some() {
            self.WithPendingTree(context, |tree, c| tree.RecreateTileResources(c));
        }
        if self.recycle_tree_.is_some() {
            self.WithRecycleTree(context, |tree, c| tree.RecreateTileResources(c));
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4351-4382.
    pub fn CreateTileManagerResources(&mut self, context: &mut HostImplContext<'_, B>) {
        debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
        let sink = self.layer_tree_frame_sink_.unwrap();
        let worker = context.services.FrameSinkWorkerContextProvider(sink);
        let provider = context.services.ImageDecodeContextProvider(worker);
        self.image_decode_cache_holder_ = Some(Box::new(ImageDecodeCacheHolder::new(
            context.services.ImageDecodeFactory(),
            &self.raster_caps_,
            provider,
            self.settings_.decoded_image_working_set_budget_bytes,
            context
                .services
                .ImageDecodeDarkModeFilter(self.dark_mode_filter_),
        )));
        if self.raster_caps_.use_gpu_rasterization {
            let worker = context
                .services
                .FrameSinkWorkerContextProvider(sink)
                .unwrap();
            self.pending_raster_queries_ = Some(context.services.CreateRasterQueryQueue(worker));
        }
        self.raster_buffer_provider_ = Some(self.CreateRasterBufferProvider(context));
        if self.is_synchronous_single_threaded_ {
            debug_assert!(self.single_thread_synchronous_task_graph_runner_.is_none());
            self.single_thread_synchronous_task_graph_runner_ =
                Some(context.services.CreateSynchronousTaskGraphRunner());
        }
        B::ConfigureResidentTileResources(self, context);
        B::SetResidentCheckerDisabled(
            &mut self.tile_manager_,
            self.settings_.only_checker_images_with_gpu_raster
                && !self.raster_caps_.use_gpu_rasterization,
            context,
        );
        let policy = self.ActualManagedMemoryPolicy(context);
        self.UpdateTileManagerMemoryPolicy(policy, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4384-4430.
    pub fn CreateRasterBufferProvider(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) -> Box<B::RasterBufferProvider> {
        debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
        #[cfg(debug_assertions)]
        debug_assert_ne!(self.GetTaskRunner(context).0, 0);
        let sink = self.layer_tree_frame_sink_.unwrap();
        let compositor = context.services.FrameSinkContextProvider(sink);
        let Some(compositor) = compositor else {
            let interface = context.services.FrameSinkSharedImageInterface(sink);
            return context
                .services
                .CreateZeroCopyRasterBufferProvider(interface, true);
        };
        let worker = context.services.FrameSinkWorkerContextProvider(sink);
        if self.raster_caps_.use_gpu_rasterization {
            debug_assert!(worker.is_some());
            let worker = worker.unwrap();
            let interface = context.services.ContextProviderSharedImageInterface(worker);
            return context.services.CreateGpuRasterBufferProvider(
                interface,
                compositor,
                worker,
                self.raster_caps_.tile_overlay_candidate,
                self.settings_.max_gpu_raster_tile_size,
                self.pending_raster_queries_.as_deref(),
            );
        }
        let mut use_zero_copy = self.settings_.use_zero_copy;
        if !use_zero_copy && worker.is_none() {
            context.services.MissingWorkerContextForcingZeroCopy();
            use_zero_copy = true;
        }
        if use_zero_copy {
            let interface = context
                .services
                .ContextProviderSharedImageInterface(compositor);
            return context
                .services
                .CreateZeroCopyRasterBufferProvider(interface, false);
        }
        let worker = worker.unwrap();
        let interface = context.services.ContextProviderSharedImageInterface(worker);
        let task_runner = self.GetTaskRunner(context);
        context.services.CreateOneCopyRasterBufferProvider(
            interface,
            task_runner,
            compositor,
            worker,
            self.settings_.use_partial_raster,
            self.settings_.max_staging_buffer_usage_in_bytes,
            self.raster_caps_.tile_overlay_candidate,
        )
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:530-535.
    pub fn DidMouseEnterNonViewportScroller(
        &mut self,
        element_id: ElementId,
        context: &mut HostImplContext<'_, B>,
    ) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::DidMouseEnterNonViewportScroller(
                host, element_id,
            )
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:537-541.
    pub fn DidMouseLeave(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::DidMouseLeave(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:548-556.
    pub fn UpdateBrowserControlsState(
        &mut self,
        constraints: crate::cc::input::browser_controls_state::BrowserControlsState,
        current: crate::cc::input::browser_controls_state::BrowserControlsState,
        animate: bool,
        offset_tag_modifications:Option<&crate::cc::input::browser_controls_offset_tag_modifications::BrowserControlsOffsetTagModifications>,
        context: &mut HostImplContext<'_, B>,
    ) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::UpdateBrowserControlsState(
                host,
                constraints,
                current,
                animate,
                offset_tag_modifications,
            )
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:843-845.
    pub fn Animate(&mut self, context: &mut HostImplContext<'_, B>) {
        self.AnimateInternal(context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:847-875.
    fn AnimateInternal(&mut self, context: &mut HostImplContext<'_, B>) {
        debug_assert!(context.services.IsImplThread(self.task_runner_provider_));
        let monotonic_time = self.CurrentBeginFrameArgs().frame_time;
        let mut did_animate = false;
        // TODO(bokan): This should return did_animate, see the source TODO in
        // ElasticOverscrollController::Animate, crbug.com/551138.
        if let Some(input) = self.input_delegate_.as_mut() {
            input.TickAnimations(monotonic_time);
        }
        did_animate |= self.AnimatePageScale(monotonic_time, context);
        did_animate |= self.AnimateLayers(monotonic_time, true, context);
        did_animate |= self.AnimateScrollbars(monotonic_time, context);
        did_animate |= self.AnimateBrowserControls(monotonic_time, context);
        if did_animate {
            if self.input_delegate_.is_some() {
                self.WithInputHandler(context, |input, host| {
                    input.unwrap().RootLayerStateMayHaveChanged(host)
                });
            }
            self.SetNeedsRedraw(true, false, context);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:877-903.
    pub fn PrepareTiles(&mut self, context: &mut HostImplContext<'_, B>) -> bool {
        context
            .services
            .TraceHostImplAEvent("LayerTreeHostImpl::PrepareTiles");
        debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
        if self.active_tree_.is_some() {
            self.tile_priorities_dirty_ |=
                self.WithActiveTree(context, |tree, c| tree.UpdateTiles(c));
        }
        if self.pending_tree_.is_some() {
            self.tile_priorities_dirty_ |=
                self.WithPendingTree(context, |tree, c| tree.UpdateTiles(c));
        }
        if !self.tile_priorities_dirty_ {
            return false;
        }
        if let Some(active) = self.active_tree_.as_ref() {
            let viewport = active.GetDeviceViewportWithExternal(self.external_viewport_);
            self.global_tile_state_
                .set_viewport_size(Size::new(viewport.width, viewport.height));
        }
        let did_prepare_tiles = B::PrepareResidentTiles(self, context);
        if did_prepare_tiles {
            self.tile_priorities_dirty_ = false;
        }
        if let Some(sync) = self.sync_tree() {
            let source_frame = sync.source_frame_number();
            B::SetResidentTileSyncFrame(&mut self.tile_manager_, source_frame, context);
        }
        context.services.DelegateDidPrepareTiles(self.delegate_);
        did_prepare_tiles
    }
    // Source: cc/trees/layer_tree_host_impl.cc:905-941.
    pub fn StartPageScaleAnimation(
        &mut self,
        target_offset: Point,
        anchor_point: bool,
        page_scale: f32,
        duration: TimeDelta,
        context: &mut HostImplContext<'_, B>,
    ) {
        StartPageScaleAnimationForHost(
            self.active_tree_.as_mut().unwrap(),
            &mut self.page_scale_animation_,
            &self.settings_,
            self.delegate_,
            &self.latency_info_swap_promise_monitor_,
            &mut self.events_metrics_manager_,
            target_offset,
            anchor_point,
            page_scale,
            duration,
            context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:943-945.
    pub fn SetNeedsAnimateInput(&mut self, context: &mut HostImplContext<'_, B>) {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::SetNeedsAnimateInput(host)
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:947-951.
    pub fn CreateLatencyInfoSwapPromiseMonitor(
        &mut self,
        latency: Option<&mut B::LatencyInfo>,
        context: &mut HostImplContext<'_, B>,
    ) -> Option<Box<B::LatencyInfoSwapPromiseMonitor>> {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::CreateLatencyInfoSwapPromiseMonitor(
                host, latency,
            )
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:953-957.
    pub fn GetScopedEventMetricsMonitor(
        &mut self,
        done_callback: B::EventMetricsDoneCallback,
        context: &mut HostImplContext<'_, B>,
    ) -> Option<Box<B::EventMetricsScopedMonitor>> {
        self.WithInputRuntime(context, |host| {
            crate::adapter::input_handler::InputHost::<B>::GetScopedEventMetricsMonitor(
                host,
                done_callback,
            )
        })
    }
}

impl<B: HostImplTypes> HostImplRuntimeCore<'_, B> {
    pub(crate) fn reborrow(&mut self) -> HostImplRuntimeCore<'_, B> {
        HostImplRuntimeCore {
            weak_factory_: &mut *self.weak_factory_,
            unbounded_local_surface_id_: &mut *self.unbounded_local_surface_id_,
            unbounded_frame_sink_id_: &mut *self.unbounded_frame_sink_id_,
            unbounded_frame_sink_handler_: &mut *self.unbounded_frame_sink_handler_,
            dump_compositor_frame_end_: &mut *self.dump_compositor_frame_end_,
            dump_compositor_frame_begin_: &mut *self.dump_compositor_frame_begin_,
            dump_compositor_frame_: &mut *self.dump_compositor_frame_,
            tracked_element_rects_from_client_: &mut *self.tracked_element_rects_from_client_,
            is_handling_interaction_from_client_: &mut *self.is_handling_interaction_from_client_,
            send_frame_token_to_embedder_: &mut *self.send_frame_token_to_embedder_,
            previously_visible_scrollable_elements_: &mut *self
                .previously_visible_scrollable_elements_,
            new_local_surface_id_expected_: &mut *self.new_local_surface_id_expected_,
            last_draw_active_tree_begin_frame_args_: &mut *self
                .last_draw_active_tree_begin_frame_args_,
            view_transition_content_rects_: &mut *self.view_transition_content_rects_,
            pending_invalidation_raster_inducing_scrolls_: &mut *self
                .pending_invalidation_raster_inducing_scrolls_,
            begin_frame_time_delta_: &mut *self.begin_frame_time_delta_,
            frame_max_scroll_delta_: &mut *self.frame_max_scroll_delta_,
            top_controls_visible_height_: &mut *self.top_controls_visible_height_,
            screenshot_destination_: &mut *self.screenshot_destination_,
            downsample_metrics_: &mut *self.downsample_metrics_,
            throttle_decider_: &mut *self.throttle_decider_,
            contains_srgb_cache_: &mut *self.contains_srgb_cache_,
            has_observed_first_scroll_delay_: &mut *self.has_observed_first_scroll_delay_,
            has_non_fling_input_since_last_frame_: &mut *self.has_non_fling_input_since_last_frame_,
            has_input_resetter_: &mut *self.has_input_resetter_,
            has_input_for_frame_interval_: &mut *self.has_input_for_frame_interval_,
            events_metrics_manager_: &mut *self.events_metrics_manager_,
            lag_tracking_manager_: &mut *self.lag_tracking_manager_,
            paint_worklet_tracker_: &mut *self.paint_worklet_tracker_,
            #[cfg(debug_assertions)]
            doing_sync_draw_: &mut *self.doing_sync_draw_,
            pending_tree_fully_painted_: &mut *self.pending_tree_fully_painted_,
            paint_worklet_painter_: &mut *self.paint_worklet_painter_,
            frame_trackers_: &mut *self.frame_trackers_,
            compositor_frame_reporting_controller_: &mut *self
                .compositor_frame_reporting_controller_,
            frame_sorter_: &mut *self.frame_sorter_,
            presentation_time_callbacks_: &mut *self.presentation_time_callbacks_,
            last_vertical_scroll_direction_: &mut *self.last_vertical_scroll_direction_,
            current_local_surface_id_from_client_: &mut *self.current_local_surface_id_from_client_,
            child_local_surface_id_allocator_: &mut *self.child_local_surface_id_allocator_,
            evicted_local_surface_id_: &mut *self.evicted_local_surface_id_,
            target_local_surface_id_: &mut *self.target_local_surface_id_,
            last_draw_render_frame_metadata_: &mut *self.last_draw_render_frame_metadata_,
            last_draw_referenced_surfaces_: &mut *self.last_draw_referenced_surfaces_,
            last_draw_local_surface_id_: &mut *self.last_draw_local_surface_id_,
            next_frame_token_from_client_: &mut *self.next_frame_token_from_client_,
            next_frame_token_: &mut *self.next_frame_token_,
            render_frame_metadata_observer_: &mut *self.render_frame_metadata_observer_,
            impl_thread_phase_: &mut *self.impl_thread_phase_,
            completed_image_decode_requests_: &mut *self.completed_image_decode_requests_,
            may_throttle_if_undrawn_frames_: &mut *self.may_throttle_if_undrawn_frames_,
            prefers_reduced_motion_: &mut *self.prefers_reduced_motion_,
            is_viewport_mobile_optimized_: &mut *self.is_viewport_mobile_optimized_,
            visual_device_viewport_size_: &mut *self.visual_device_viewport_size_,
            consecutive_frame_with_damage_count_: &mut *self.consecutive_frame_with_damage_count_,
            prioritize_new_content_due_to_checkerboarding_: &mut *self
                .prioritize_new_content_due_to_checkerboarding_,
            has_valid_layer_tree_frame_sink_: &mut *self.has_valid_layer_tree_frame_sink_,
            is_likely_to_require_a_draw_: &mut *self.is_likely_to_require_a_draw_,
            requires_high_res_to_draw_: &mut *self.requires_high_res_to_draw_,
            latency_info_swap_promise_monitor_: &mut *self.latency_info_swap_promise_monitor_,
            id_: &mut *self.id_,
            task_graph_runner_: &mut *self.task_graph_runner_,
            tree_activation_callback_: &mut *self.tree_activation_callback_,
            single_thread_synchronous_task_graph_runner_: &mut *self
                .single_thread_synchronous_task_graph_runner_,
            micro_benchmark_controller_: &mut *self.micro_benchmark_controller_,
            rendering_stats_instrumentation_: &mut *self.rendering_stats_instrumentation_,
            flashed_scrollbars_: &mut *self.flashed_scrollbars_,
            dark_mode_filter_: &mut *self.dark_mode_filter_,
            video_frame_controllers_: &mut *self.video_frame_controllers_,
            mutator_events_: &mut *self.mutator_events_,
            mutator_host_: &mut *self.mutator_host_,
            total_invalidated_area_: &mut *self.total_invalidated_area_,
            root_layer_damage_rect_: &mut *self.root_layer_damage_rect_,
            viewport_damage_rect_: &mut *self.viewport_damage_rect_,
            resourceless_software_draw_: &mut *self.resourceless_software_draw_,
            viewport_rect_for_tile_priority_: &mut *self.viewport_rect_for_tile_priority_,
            external_viewport_: &mut *self.external_viewport_,
            external_transform_: &mut *self.external_transform_,
            max_memory_needed_bytes_: &mut *self.max_memory_needed_bytes_,
            debug_rect_history_: &mut *self.debug_rect_history_,
            memory_history_: &mut *self.memory_history_,
            page_scale_animation_: &mut *self.page_scale_animation_,
            tile_manager_: &mut *self.tile_manager_,
            cached_managed_memory_policy_: &mut *self.cached_managed_memory_policy_,
            visible_: &mut *self.visible_,
            debug_state_: &mut *self.debug_state_,
            tile_priorities_dirty_: &mut *self.tile_priorities_dirty_,
            swap_promises_for_main_thread_scroll_update_: &mut *self
                .swap_promises_for_main_thread_scroll_update_,
            accumulated_scroll_deltas_by_element_id_: &mut *self
                .accumulated_scroll_deltas_by_element_id_,
            scroll_accumulated_this_frame_: &mut *self.scroll_accumulated_this_frame_,
            global_tile_state_: &mut *self.global_tile_state_,
            image_decode_cache_holder_: &mut *self.image_decode_cache_holder_,
            pending_raster_queries_: &mut *self.pending_raster_queries_,
            resource_pool_: &mut *self.resource_pool_,
            raster_buffer_provider_: &mut *self.raster_buffer_provider_,
            raster_caps_: &mut *self.raster_caps_,
            worker_context_visibility_: &mut *self.worker_context_visibility_,
            compositor_context_visibility_: &mut *self.compositor_context_visibility_,
            layer_context_: &mut *self.layer_context_,
            layer_tree_frame_sink_: &mut *self.layer_tree_frame_sink_,
            ui_resource_changes_: &mut *self.ui_resource_changes_,
            evicted_ui_resources_: &mut *self.evicted_ui_resources_,
            deleted_ui_resources_: &mut *self.deleted_ui_resources_,
            ui_resource_map_: &mut *self.ui_resource_map_,
            resource_provider_: &mut *self.resource_provider_,
            is_synchronous_single_threaded_: &mut *self.is_synchronous_single_threaded_,
            settings_: self.settings_,
            current_begin_frame_tracker_: &mut *self.current_begin_frame_tracker_,
            task_runner_provider_: &mut *self.task_runner_provider_,
            scheduling_delegate_: &mut *self.scheduling_delegate_,
            delegate_: &mut *self.delegate_,
            owner_id_: &mut *self.owner_id_,
        }
    }
}

/// A native manager can be owned by the resident slot or explicitly lent back
/// by its current callback. Both variants expose the same live object.
pub(crate) enum HostImplControlsSlot<'a> {
    Resident(&'a mut Option<Box<BrowserControlsOffsetManager>>),
    Direct(&'a mut BrowserControlsOffsetManager),
}
impl HostImplControlsSlot<'_> {
    pub(crate) fn as_ref(&self) -> Option<&BrowserControlsOffsetManager> {
        match self {
            Self::Resident(slot) => slot.as_deref(),
            Self::Direct(manager) => Some(manager),
        }
    }
    pub(crate) fn as_mut(&mut self) -> Option<&mut BrowserControlsOffsetManager> {
        match self {
            Self::Resident(slot) => slot.as_deref_mut(),
            Self::Direct(manager) => Some(manager),
        }
    }
    pub(crate) fn reborrow(&mut self) -> HostImplControlsSlot<'_> {
        match self {
            Self::Resident(slot) => HostImplControlsSlot::Resident(&mut **slot),
            Self::Direct(manager) => HostImplControlsSlot::Direct(&mut **manager),
        }
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:3946-3952,3960-3962,4708-4727,4730-4769.
// Rust borrow split: manager callbacks lend the current manager explicitly;
// this client owns only references to its resident siblings and external services.
pub(crate) struct HostImplBrowserControlsClient<'a, 'c, B: HostImplTypes> {
    pub(crate) core: HostImplRuntimeCore<'a, B>,
    pub(crate) active_tree_: HostImplInputTree<'a, B>,
    pub(crate) pending_tree_: HostImplInputTree<'a, B>,
    pub(crate) recycle_tree_: HostImplInputTree<'a, B>,
    pub(crate) scrollbar_animation_controllers_:
        &'a mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    pub(crate) image_animation_controller_: &'a mut Option<Box<ImageAnimationController>>,
    pub(crate) context: &'a mut HostImplContext<'c, B>,
}
impl<B: HostImplTypes> HostImplBrowserControlsClient<'_, '_, B> {
    fn WithTreeAndManager<R>(
        &mut self,
        pending: bool,
        manager: &mut BrowserControlsOffsetManager,
        callback: impl FnOnce(&mut LayerTreeImpl<B>, &mut HostImplMutationFacade<'_, '_, B>) -> R,
    ) -> R {
        let (tree, active, pending) = if pending {
            let tree = self.pending_tree_.as_mut().expect("source pending tree");
            let id = tree.id;
            (
                tree,
                self.active_tree_.as_host_slot(),
                HostImplTreeSlot::Reserved(id),
            )
        } else {
            let tree = self.active_tree_.as_mut().expect("source active tree");
            let id = tree.id;
            (
                tree,
                HostImplTreeSlot::Reserved(id),
                self.pending_tree_.as_host_slot(),
            )
        };
        let mut host = HostImplMutationFacade {
            core: self.core.reborrow(),
            active_tree_: active,
            pending_tree_: pending,
            recycle_tree_: self.recycle_tree_.as_host_slot(),
            browser_controls_offset_manager_: HostImplControlsSlot::Direct(manager),
            scrollbar_animation_controllers_: &mut *self.scrollbar_animation_controllers_,
            image_animation_controller_: &mut *self.image_animation_controller_,
            context: &mut *self.context,
        };
        callback(tree, &mut host)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4708-4717.
    fn DidChangePosition(&mut self, manager: &mut BrowserControlsOffsetManager) {
        self.WithTreeAndManager(false, manager, |tree, c| {
            tree.UpdateViewportContainerSizesWithHost(c)
        });
        if self.pending_tree_.is_some() {
            self.WithTreeAndManager(true, manager, |tree, c| {
                tree.UpdateViewportContainerSizesWithHost(c)
            });
        }
        let redraw_promises = RequestRedrawForHost(
            self.core.settings_,
            *self.core.delegate_,
            self.core.latency_info_swap_promise_monitor_,
            self.core.events_metrics_manager_,
            false,
            false,
            self.context,
        );
        let begin_promises = RequestBeginImplFrameForHost(
            *self.core.delegate_,
            self.core.latency_info_swap_promise_monitor_,
            self.core.events_metrics_manager_,
            self.context,
        );
        let active = self.active_tree_.as_mut().expect("source active tree");
        QueuePinnedLatencyPromisesOnActiveTree(active, redraw_promises);
        QueuePinnedLatencyPromisesOnActiveTree(active, begin_promises);
        SetFullViewportDamageForTree(
            self.active_tree_.as_ref().unwrap(),
            *self.core.external_viewport_,
            self.core.viewport_damage_rect_,
        );
    }
}
impl<B: HostImplTypes> crate::adapter::browser_controls::BrowserControlsReadEnvironment
    for HostImplBrowserControlsClient<'_, '_, B>
{
    type BeginMainFrameReason =
        <B as crate::adapter::input_services::InputHostTypes>::BeginMainFrameReason;
    fn OtherBeginMainFrameReason(&self) -> Self::BeginMainFrameReason {
        B::OtherBeginMainFrameReason()
    }
    fn FeatureEnabled(
        &self,
        feature: crate::adapter::browser_controls::BrowserControlsFeature,
    ) -> bool {
        self.context.services.BrowserControlsFeatureEnabled(feature)
    }
    fn MonotonicTimeNow(&self) -> TimeTicks {
        self.context.services.MonotonicTimeNow()
    }
}
impl<B: HostImplTypes> crate::adapter::browser_controls::BrowserControlsEnvironment
    for HostImplBrowserControlsClient<'_, '_, B>
{
    fn EmitDiagnostic(
        &mut self,
        event: crate::adapter::browser_controls::BrowserControlsDiagnostic,
    ) {
        self.context.services.EmitBrowserControlsDiagnostic(event);
    }
}

pub(crate) struct HostImplBrowserControlsReadClient<'a, 'c, B: HostImplTypes> {
    pub(crate) active_tree: &'a LayerTreeImpl<B>,
    pub(crate) frame_tracker: &'a BeginFrameTracker,
    pub(crate) services: &'a (dyn HostImplServices<B> + 'c),
}
impl<B: HostImplTypes> crate::adapter::browser_controls::BrowserControlsReadEnvironment
    for HostImplBrowserControlsReadClient<'_, '_, B>
{
    type BeginMainFrameReason =
        <B as crate::adapter::input_services::InputHostTypes>::BeginMainFrameReason;
    fn OtherBeginMainFrameReason(&self) -> Self::BeginMainFrameReason {
        B::OtherBeginMainFrameReason()
    }
    fn FeatureEnabled(
        &self,
        feature: crate::adapter::browser_controls::BrowserControlsFeature,
    ) -> bool {
        self.services.BrowserControlsFeatureEnabled(feature)
    }
    fn MonotonicTimeNow(&self) -> TimeTicks {
        self.services.MonotonicTimeNow()
    }
}
impl<B: HostImplTypes> BrowserControlsOffsetManagerQueries
    for HostImplBrowserControlsReadClient<'_, '_, B>
{
    fn TopControlsHeight(&self) -> f32 {
        self.active_tree.top_controls_height()
    }
    fn TopControlsMinHeight(&self) -> f32 {
        self.active_tree.top_controls_min_height()
    }
    fn BottomControlsHeight(&self) -> f32 {
        self.active_tree.bottom_controls_height()
    }
    fn BottomControlsMinHeight(&self) -> f32 {
        self.active_tree.bottom_controls_min_height()
    }
    fn CurrentTopControlsShownRatio(&self) -> f32 {
        self.active_tree.top_controls_shown_ratio().Current(true)
    }
    fn CurrentBottomControlsShownRatio(&self) -> f32 {
        self.active_tree.bottom_controls_shown_ratio().Current(true)
    }
    fn ViewportScrollOffset(&self) -> PointF {
        Viewport::TotalScrollOffsetForTree(self.active_tree)
    }
    fn MaxViewportScrollOffsetY(&self) -> f32 {
        Viewport::MaxUserReachableTotalScrollOffsetYForTree(self.active_tree)
    }
    fn OnlyExpandTopControlsAtPageTop(&self) -> bool {
        self.active_tree.only_expand_top_controls_at_page_top()
    }
    fn HaveRootScrollNode(&self) -> bool {
        self.active_tree.InnerViewportScrollNode().is_some()
    }
    fn CurrentFrameInterval(&self) -> TimeDelta {
        self.frame_tracker.Interval()
    }
    fn DeviceScaleFactor(&self) -> f32 {
        self.active_tree.device_scale_factor()
    }
}
impl<B: HostImplTypes> HostImplBrowserControlsClient<'_, '_, B> {
    fn ReadClient(&self) -> HostImplBrowserControlsReadClient<'_, '_, B> {
        HostImplBrowserControlsReadClient {
            active_tree: self.active_tree_.as_ref().unwrap(),
            frame_tracker: self.core.current_begin_frame_tracker_,
            services: &*self.context.services,
        }
    }
}
impl<B: HostImplTypes> BrowserControlsOffsetManagerQueries
    for HostImplBrowserControlsClient<'_, '_, B>
{
    fn TopControlsHeight(&self) -> f32 {
        self.ReadClient().TopControlsHeight()
    }
    fn TopControlsMinHeight(&self) -> f32 {
        self.ReadClient().TopControlsMinHeight()
    }
    fn BottomControlsHeight(&self) -> f32 {
        self.ReadClient().BottomControlsHeight()
    }
    fn BottomControlsMinHeight(&self) -> f32 {
        self.ReadClient().BottomControlsMinHeight()
    }
    fn CurrentTopControlsShownRatio(&self) -> f32 {
        self.ReadClient().CurrentTopControlsShownRatio()
    }
    fn CurrentBottomControlsShownRatio(&self) -> f32 {
        self.ReadClient().CurrentBottomControlsShownRatio()
    }
    fn ViewportScrollOffset(&self) -> PointF {
        self.ReadClient().ViewportScrollOffset()
    }
    fn MaxViewportScrollOffsetY(&self) -> f32 {
        self.ReadClient().MaxViewportScrollOffsetY()
    }
    fn OnlyExpandTopControlsAtPageTop(&self) -> bool {
        self.ReadClient().OnlyExpandTopControlsAtPageTop()
    }
    fn HaveRootScrollNode(&self) -> bool {
        self.ReadClient().HaveRootScrollNode()
    }
    fn CurrentFrameInterval(&self) -> TimeDelta {
        self.ReadClient().CurrentFrameInterval()
    }
    fn DeviceScaleFactor(&self) -> f32 {
        self.ReadClient().DeviceScaleFactor()
    }
}
impl<B: HostImplTypes> BrowserControlsOffsetManagerClient
    for HostImplBrowserControlsClient<'_, '_, B>
{
    fn SetCurrentBrowserControlsShownRatio(
        &mut self,
        top_ratio: f32,
        bottom_ratio: f32,
        manager: &mut BrowserControlsOffsetManager,
    ) {
        let changed = self
            .active_tree_
            .as_mut()
            .unwrap()
            .SetCurrentBrowserControlsShownRatioWithRanges(
                top_ratio,
                bottom_ratio,
                || manager.TopControlsShownRatioRange(),
                || manager.BottomControlsShownRatioRange(),
            );
        if changed {
            self.DidChangePosition(manager);
        }
    }
    fn DidChangeBrowserControlsPosition(&mut self, manager: &mut BrowserControlsOffsetManager) {
        self.DidChangePosition(manager);
    }
    fn SetNeedsCommit(&mut self, reason: Self::BeginMainFrameReason) {
        RequestCommitForHost(
            self.core.settings_,
            *self.core.delegate_,
            reason,
            self.context,
        );
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    pub(super) fn WithBrowserControls<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback:impl FnOnce(&mut BrowserControlsOffsetManager,&mut dyn crate::cc::input::browser_controls_offset_manager_client::BrowserControlsOffsetManagerClient<BeginMainFrameReason=<B as crate::adapter::input_services::InputHostTypes>::BeginMainFrameReason>)->R,
    ) -> R {
        let HostImplRuntimeParts {
            core,
            active_tree_,
            pending_tree_,
            recycle_tree_,
            input_delegate_: _,
            browser_controls_offset_manager_,
            viewport_: _,
            scrollbar_animation_controllers_,
            image_animation_controller_,
        } = self.BorrowRuntime();
        let manager = browser_controls_offset_manager_
            .as_deref_mut()
            .expect("source browser controls manager");
        let mut client = HostImplBrowserControlsClient {
            core,
            active_tree_: HostImplInputTree::Slot(active_tree_),
            pending_tree_: HostImplInputTree::Slot(pending_tree_),
            recycle_tree_: HostImplInputTree::Slot(recycle_tree_),
            scrollbar_animation_controllers_,
            image_animation_controller_,
            context,
        };
        callback(manager, &mut client)
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:5228-5237.
// Canonical source body shared by direct owner and browser-controls callback.
fn SetFullViewportDamageForTree<B: HostImplTypes>(
    tree: &LayerTreeImpl<B>,
    external: Rect,
    damage: &mut Rect,
) {
    SetFullViewportDamageForViewport::<B>(tree.internal_device_viewport(), external, damage);
}

pub(crate) enum HostImplViewportSlot<'a> {
    Resident(&'a mut Option<Box<Viewport>>),
    Direct(&'a mut Viewport),
}
impl HostImplViewportSlot<'_> {
    pub(crate) fn as_ref(&self) -> Option<&Viewport> {
        match self {
            Self::Resident(slot) => slot.as_deref(),
            Self::Direct(viewport) => Some(viewport),
        }
    }
    pub(crate) fn as_mut(&mut self) -> Option<&mut Viewport> {
        match self {
            Self::Resident(slot) => slot.as_deref_mut(),
            Self::Direct(viewport) => Some(viewport),
        }
    }
}
impl<B: HostImplTypes> HostImplInputTree<'_, B> {
    pub(crate) fn reborrow(&mut self) -> HostImplInputTree<'_, B> {
        match self {
            Self::Slot(slot) => HostImplInputTree::Slot(&mut **slot),
            Self::Direct(tree) => HostImplInputTree::Direct(&mut **tree),
        }
    }
}

// Source: cc/layers/viewport.h:109-138; cc/trees/layer_tree_host_impl.h:525-527.
// Rust live-borrow routing. Viewport and InputHandler are excluded; a source
// callback explicitly lends the current Viewport for its nested short call.
pub(crate) struct HostImplViewportRuntime<'a, 'c, B: HostImplTypes> {
    pub(crate) animation_reentry:
        Option<&'a mut dyn crate::adapter::animation_host::ScrollAnimationReentry>,
    pub(crate) core: HostImplRuntimeCore<'a, B>,
    pub(crate) active_tree_: HostImplInputTree<'a, B>,
    pub(crate) pending_tree_: HostImplInputTree<'a, B>,
    pub(crate) recycle_tree_: HostImplInputTree<'a, B>,
    pub(crate) browser_controls_offset_manager_: HostImplControlsSlot<'a>,
    pub(crate) scrollbar_animation_controllers_:
        &'a mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    pub(crate) image_animation_controller_: &'a mut Option<Box<ImageAnimationController>>,
    pub(crate) input_view:
        Option<crate::cc::input::input_handler::InputHandlerView<'a>>,
    pub(crate) context: &'a mut HostImplContext<'c, B>,
}
impl<B: HostImplTypes> HostImplViewportRuntime<'_, '_, B> {
    fn WithInputHost<R>(
        &mut self,
        viewport: &mut Viewport,
        callback: impl FnOnce(
            &mut HostImplInputFacade<'_, B>,
            Option<&crate::cc::input::input_handler::InputHandlerView<'_>>,
        ) -> R,
    ) -> R {
        let mut host = HostImplInputFacade {
            animation_reentry: self.animation_reentry.as_mut().map(|receiver| {
                &mut **receiver as &mut dyn crate::adapter::animation_host::ScrollAnimationReentry
            }),
            core: self.core.reborrow(),
            active_tree_: self.active_tree_.reborrow(),
            pending_tree_: self.pending_tree_.reborrow(),
            recycle_tree_: self.recycle_tree_.reborrow(),
            browser_controls_offset_manager_: self.browser_controls_offset_manager_.reborrow(),
            viewport_: HostImplViewportSlot::Direct(viewport),
            scrollbar_animation_controllers_: &mut *self.scrollbar_animation_controllers_,
            image_animation_controller_: &mut *self.image_animation_controller_,
            context: HostImplContext {
                services: &mut *self.context.services,
            },
        };
        callback(&mut host, self.input_view.as_ref())
    }
}
impl<B: HostImplTypes> crate::adapter::viewport::ViewportRuntime<B>
    for HostImplViewportRuntime<'_, '_, B>
{
    fn ActiveTree(&self) -> &LayerTreeImpl<B> {
        self.active_tree_.as_ref().expect("source active tree")
    }
    fn ScrollNode(&self, id: &crate::adapter::input_handler::InputScrollNodeId) -> &ScrollNode {
        let tree = self.active_tree_.as_ref().unwrap();
        assert_eq!(tree.id, id.tree);
        tree.property_trees().scroll_tree().Node(id.index)
    }
    fn CurrentPageScaleFactor(&self) -> f32 {
        self.active_tree_
            .as_ref()
            .unwrap()
            .current_page_scale_factor_for_tree(true)
    }
    fn PageScaleFactorForScroll(&self) -> f32 {
        self.active_tree_
            .as_ref()
            .unwrap()
            .page_scale_factor_for_scroll_for_tree(true, self.core.settings_.is_for_scalable_page)
    }
    fn ScrollNodeBy(&mut self, id: i32, delta: &Vector2dF, viewport: &mut Viewport) -> Vector2dF {
        self.WithInputHost(viewport, |host, _| host.ScrollNodeByForViewport(id, delta))
    }
    fn ClampScrollNodeToMaxScrollOffset(&mut self, id: i32, viewport: &mut Viewport) -> Vector2dF {
        self.WithInputHost(viewport, |host, _| host.ClampScrollNodeForViewport(id))
    }
    fn SetPageScaleOnActiveTree(&mut self, scale: f32, viewport: &mut Viewport) {
        self.WithInputHost(viewport, |host, _| host.SetPageScaleForViewport(scale));
    }
    fn ScrollSingleNode(
        &mut self,
        node: crate::adapter::input_handler::InputScrollNodeId,
        delta: Vector2dF,
        point: Point,
        direct: bool,
        viewport: &mut Viewport,
    ) -> Vector2dF {
        self.WithInputHost(viewport, |host, view| {
            view.expect("source GetInputHandler")
                .ScrollSingleNode::<B, _>(host, &node, delta, point, direct)
        })
    }
    fn CanConsumeDelta(
        &mut self,
        state: &crate::cc::input::scroll_state::ScrollState,
        node: crate::adapter::input_handler::InputScrollNodeId,
        viewport: &mut Viewport,
    ) -> bool {
        self.WithInputHost(viewport, |host, view| {
            view.expect("source GetInputHandler")
                .CanConsumeDelta::<B, _>(host, state, &node)
        })
    }
    fn ComputeScrollDelta(
        &mut self,
        node: crate::adapter::input_handler::InputScrollNodeId,
        delta: Vector2dF,
        viewport: &mut Viewport,
    ) -> Vector2dF {
        self.WithInputHost(viewport, |host, view| {
            view.expect("source GetInputHandler")
                .ComputeScrollDelta::<B, _>(host, &node, delta, None)
        })
    }
    fn ScrollAnimationCreate(
        &mut self,
        node: crate::adapter::input_handler::InputScrollNodeId,
        delta: Vector2dF,
        delay: TimeDelta,
        viewport: &mut Viewport,
    ) {
        self.WithInputHost(viewport, |host, _| {
            crate::adapter::input_handler::InputHost::<B>::ScrollAnimationCreate(
                host, &node, &delta, delay,
            );
        });
    }
    fn ScrollBrowserControls(&mut self, delta: &Vector2dF, inertial: bool) -> Vector2dF {
        let manager = self
            .browser_controls_offset_manager_
            .as_mut()
            .expect("source browser controls manager");
        let mut client = HostImplBrowserControlsClient {
            core: self.core.reborrow(),
            active_tree_: self.active_tree_.reborrow(),
            pending_tree_: self.pending_tree_.reborrow(),
            recycle_tree_: self.recycle_tree_.reborrow(),
            scrollbar_animation_controllers_: &mut *self.scrollbar_animation_controllers_,
            image_animation_controller_: &mut *self.image_animation_controller_,
            context: &mut *self.context,
        };
        manager.ScrollBy(&mut client, delta, inertial)
    }
    fn ScrollbarsFor(&mut self, id: ElementId) -> Vec<InputLayerId> {
        let tree = self.active_tree_.as_ref().unwrap();
        tree.ScrollbarsFor(id)
            .iter()
            .map(|handle| InputLayerId {
                tree: tree.id,
                id: handle.layer_id,
            })
            .collect()
    }
    fn ScrollbarOrientation(&mut self, layer: &InputLayerId) -> ScrollbarOrientation {
        let tree = self.active_tree_.as_mut().unwrap();
        assert_eq!(layer.tree, tree.id);
        self.context
            .services
            .Layer_Scrollbar_orientation(tree, layer)
    }
    fn ScrollbarBounds(&mut self, layer: &InputLayerId) -> Size {
        let tree = self.active_tree_.as_mut().unwrap();
        assert_eq!(layer.tree, tree.id);
        self.context.services.Layer_Scrollbar_bounds(tree, layer)
    }
    fn StartPageScaleAnimation(
        &mut self,
        target: Point,
        use_anchor: bool,
        scale: f32,
        duration: TimeDelta,
    ) {
        StartPageScaleAnimationForHost(
            self.active_tree_.as_mut().unwrap(),
            self.core.page_scale_animation_,
            self.core.settings_,
            *self.core.delegate_,
            self.core.latency_info_swap_promise_monitor_,
            self.core.events_metrics_manager_,
            target,
            use_anchor,
            scale,
            duration,
            self.context,
        );
    }
}
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    fn WithViewport<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(&mut Viewport, &mut crate::adapter::viewport::ViewportContext<'_, B>) -> R,
    ) -> R {
        let HostImplRuntimeParts {
            core,
            active_tree_,
            pending_tree_,
            recycle_tree_,
            input_delegate_,
            browser_controls_offset_manager_,
            viewport_,
            scrollbar_animation_controllers_,
            image_animation_controller_,
        } = self.BorrowRuntime();
        let view = input_delegate_.as_ref().map(|input| input.View());
        let viewport = viewport_.as_deref_mut().expect("source viewport");
        let mut runtime = HostImplViewportRuntime {
            animation_reentry: None,
            core,
            active_tree_: HostImplInputTree::Slot(active_tree_),
            pending_tree_: HostImplInputTree::Slot(pending_tree_),
            recycle_tree_: HostImplInputTree::Slot(recycle_tree_),
            browser_controls_offset_manager_: HostImplControlsSlot::Resident(
                browser_controls_offset_manager_,
            ),
            scrollbar_animation_controllers_,
            image_animation_controller_,
            input_view: view,
            context,
        };
        let mut c = crate::adapter::viewport::ViewportContext {
            runtime: &mut runtime,
        };
        callback(viewport, &mut c)
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:905-941.
// Single canonical body shared by the owner and viewport's temporary client.
fn StartPageScaleAnimationForHost<B: HostImplTypes>(
    active: &mut LayerTreeImpl<B>,
    animation: &mut Option<Box<PageScaleAnimation>>,
    settings: &LayerTreeSettings,
    delegate: HostImplDelegateId,
    monitors: &BTreeSet<HostImplLatencyMonitorId>,
    events: &mut B::EventsMetricsManager,
    target_offset: Point,
    anchor_point: bool,
    page_scale: f32,
    duration: TimeDelta,
    context: &mut HostImplContext<'_, B>,
) {
    if active.InnerViewportScrollNode().is_none() {
        return;
    }
    let scroll_total = active.TotalScrollOffset();
    let scrollable_size = active.ScrollableSize();
    let bounds = active.InnerViewportScrollNode().unwrap().container_bounds;
    let viewport_size =
        crate::adapter::geometry::SizeF::new(bounds.width() as f32, bounds.height() as f32);
    if viewport_size.IsEmpty() {
        return;
    }
    // TODO(miletus): Pass in ScrollOffset.
    *animation = Some(PageScaleAnimation::Create(
        &scroll_total,
        active.current_page_scale_factor_for_tree(true),
        &viewport_size,
        &scrollable_size,
    ));
    if anchor_point {
        let anchor = PointF::new(target_offset.x() as f32, target_offset.y() as f32);
        animation
            .as_mut()
            .unwrap()
            .ZoomWithAnchor(&anchor, page_scale, duration.InSecondsF());
    } else {
        let scaled_target_offset = PointF::new(target_offset.x() as f32, target_offset.y() as f32);
        animation.as_mut().unwrap().ZoomTo(
            &scaled_target_offset,
            page_scale,
            duration.InSecondsF(),
        );
    }
    let promises = RequestBeginImplFrameForHost(delegate, monitors, events, context);
    QueuePinnedLatencyPromisesOnActiveTree(active, promises);
    RequestCommitForHost(settings, delegate, B::OtherBeginMainFrameReason(), context);
    context.services.DelegateRenewTreePriority(delegate);
}

use crate::cc::input::browser_controls_offset_manager_client::{
    BrowserControlsOffsetManagerClient, BrowserControlsOffsetManagerQueries,
};

// Source: cc/trees/layer_tree_host_impl.cc:5611-5836,5898-5919,5925-5934.
// A short borrow of only the source UI-resource owner fields. The active tree
// is a readonly native receiver used by CanDraw; no host/tree state is copied.
struct HostImplUIResourceState<'a, B: HostImplTypes> {
    settings_: &'a LayerTreeSettings,
    owner_id_: &'a LayerTreeHostId,
    weak_factory_: &'a std::sync::Arc<()>,
    delegate_: &'a HostImplDelegateId,
    has_valid_layer_tree_frame_sink_: &'a bool,
    layer_tree_frame_sink_: &'a Option<HostImplFrameSinkId>,
    raster_caps_: &'a RasterCapabilities,
    resource_provider_: &'a mut Option<Box<B::ClientResourceProvider>>,
    ui_resource_changes_:
        &'a mut UIResourceChangeMap<crate::adapter::layer_tree_impl::UIResourceId>,
    evicted_ui_resources_: &'a mut BTreeSet<crate::adapter::layer_tree_impl::UIResourceId>,
    deleted_ui_resources_:
        &'a mut HashMap<crate::adapter::layer_tree_impl::UIResourceId, UIResourceData<B>>,
    ui_resource_map_:
        &'a mut HashMap<crate::adapter::layer_tree_impl::UIResourceId, UIResourceData<B>>,
    active_tree_: Option<&'a LayerTreeImpl<B>>,
    external_viewport_: &'a Rect,
    evicted_local_surface_id_: &'a <B as LayerTreeTypes>::LocalSurfaceId,
    resourceless_software_draw_: &'a bool,
}
impl<'a, B: HostImplTypes> HostImplUIResourceState<'a, B> {
    fn new(
        core: &'a mut HostImplRuntimeCore<'_, B>,
        active_tree: Option<&'a LayerTreeImpl<B>>,
    ) -> Self {
        Self {
            settings_: core.settings_,
            owner_id_: core.owner_id_,
            weak_factory_: core.weak_factory_,
            delegate_: core.delegate_,
            has_valid_layer_tree_frame_sink_: core.has_valid_layer_tree_frame_sink_,
            layer_tree_frame_sink_: core.layer_tree_frame_sink_,
            raster_caps_: core.raster_caps_,
            resource_provider_: core.resource_provider_,
            ui_resource_changes_: core.ui_resource_changes_,
            evicted_ui_resources_: core.evicted_ui_resources_,
            deleted_ui_resources_: core.deleted_ui_resources_,
            ui_resource_map_: core.ui_resource_map_,
            active_tree_: active_tree,
            external_viewport_: core.external_viewport_,
            evicted_local_surface_id_: core.evicted_local_surface_id_,
            resourceless_software_draw_: core.resourceless_software_draw_,
        }
    }
    fn ResourceIdForUIResource(
        &self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
    ) -> crate::adapter::layer_tree_impl::ResourceId {
        self.ui_resource_map_.get(&uid).map_or(
            crate::adapter::layer_tree_impl::ResourceId::default(),
            |data| data.resource_id_for_export,
        )
    }
    fn CanDraw(&self, context: &HostImplContext<'_, B>) -> bool {
        CanDrawForHost(
            self.layer_tree_frame_sink_,
            self.resourceless_software_draw_,
            self.active_tree_,
            self.evicted_local_surface_id_,
            self.external_viewport_,
            self.evicted_ui_resources_,
            context.services,
        )
    }

    // Source: cc/trees/layer_tree_host_impl.cc:5611-5805.
    fn CreateUIResource(
        &mut self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
        bitmap: &B::UIBitmap,
        context: &mut HostImplContext<'_, B>,
    ) {
        context
            .services
            .TraceUIResourceEvent("LayerTreeHostImpl::CreateUIResource");
        debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
        debug_assert!(uid.0 > 0);
        let mut id = self.ResourceIdForUIResource(uid);
        if id != <B as HostImplResourceTypes>::ResourceId::default() {
            self.DeleteUIResource(uid, context);
        }
        if !*self.has_valid_layer_tree_frame_sink_ {
            self.evicted_ui_resources_.insert(uid);
            return;
        }
        let format = match bitmap.GetFormat() {
            HostUIBitmapFormat::RGBA8 => self.raster_caps_.ui_rgba_format,
            HostUIBitmapFormat::ETC1 => SinglePlaneFormat::ETC1,
        };
        let source_size = bitmap.GetSize();
        let mut upload_size = bitmap.GetSize();
        let mut scaled = false;
        let color_space = context.services.SRGBColorSpace();
        if source_size.width() > self.raster_caps_.max_texture_size
            || source_size.height() > self.raster_caps_.max_texture_size
        {
            scaled = true;
            let edge = std::cmp::max(source_size.width(), source_size.height());
            let scale = (self.raster_caps_.max_texture_size - 1) as f32 / edge as f32;
            debug_assert!(scale < 1.0);
            upload_size = context.services.ScaleToCeiledSize(source_size, scale);
        }
        let mut client_shared_image: Option<B::SharedImage> = None;
        let mut shared_image_usage = HostUISharedImageUsage::DISPLAY_READ;
        let mut shared_mapping: Option<B::UIScopedMapping> = None;
        let sink = self
            .layer_tree_frame_sink_
            .expect("source nonnull frame sink");
        // Source: cc/trees/layer_tree_host_impl.cc:5665-5689.
        if context.services.UIResourceContextProviderExists(sink) {
            let supports_scanout = context.services.UIResourceSupportsScanoutSharedImages(sink);
            let overlay_candidate = self.settings_.use_gpu_memory_buffer_resources
                && supports_scanout
                && context.services.CanCreateNativeBufferForFormat(format);
            if overlay_candidate {
                shared_image_usage |= HostUISharedImageUsage::SCANOUT;
            }
        } else {
            debug_assert_eq!(bitmap.GetFormat(), HostUIBitmapFormat::RGBA8);
            shared_image_usage = HostUISharedImageUsage::CPU_WRITE_ONLY;
            assert!(context.services.UIResourceSharedImageInterfaceExists(sink));
            client_shared_image = context
                .services
                .CreateUIResourceSharedImageForSoftwareCompositor(
                    sink,
                    format,
                    upload_size,
                    &color_space,
                    shared_image_usage,
                    "LayerTreeHostUIResourceSoftware",
                );
            assert!(client_shared_image.is_some());
            shared_mapping = Some(
                context
                    .services
                    .MapUIResourceSharedImage(client_shared_image.as_ref().unwrap()),
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5691-5712.
        if !scaled {
            if context.services.UIResourceContextProviderExists(sink) {
                client_shared_image = context.services.CreateUIResourceSharedImage(
                    sink,
                    format,
                    upload_size,
                    &color_space,
                    shared_image_usage,
                    "LayerTreeHostUIResourceBitmap",
                    bitmap.GetPixels(),
                );
                assert!(client_shared_image.is_some());
            } else {
                debug_assert_eq!(bitmap.GetFormat(), HostUIBitmapFormat::RGBA8);
                let src_info = context.services.MakeN32Premul(source_size);
                let dst_info = context.services.MakeN32Premul(upload_size);
                let row_bytes = dst_info.minRowBytes();
                let mut surface = context
                    .services
                    .WrapUIResourcePixels(
                        dst_info,
                        shared_mapping.as_mut().unwrap().GetMemoryForPlane(0),
                        row_bytes,
                    )
                    .expect("source WrapPixels surface");
                surface.getCanvas().writePixels(
                    &src_info,
                    bitmap.GetPixels(),
                    bitmap.row_bytes(),
                    0,
                    0,
                );
            }
        } else {
            // Source: cc/trees/layer_tree_host_impl.cc:5713-5766.
            debug_assert_eq!(bitmap.GetFormat(), HostUIBitmapFormat::RGBA8);
            let canvas_scale_x = upload_size.width() as f32 / source_size.width() as f32;
            let canvas_scale_y = upload_size.height() as f32 / source_size.height() as f32;
            let info = context.services.MakeN32Premul(source_size);
            let mut source_bitmap = context.services.NewUIResourceSourceBitmap();
            source_bitmap.setInfo(info, bitmap.row_bytes());
            source_bitmap.setPixels(bitmap.GetPixels());
            let mut scaled_surface;
            if context.services.UIResourceContextProviderExists(sink) {
                let info = context.services.MakeN32Premul(upload_size);
                scaled_surface = context
                    .services
                    .RasterUIResourceSurface(info)
                    .expect("CHECK scaled_surface: OOM");
            } else {
                let dst_info = context.services.MakeN32Premul(upload_size);
                let row_bytes = dst_info.minRowBytes();
                scaled_surface = context
                    .services
                    .WrapUIResourcePixels(
                        dst_info,
                        shared_mapping.as_mut().unwrap().GetMemoryForPlane(0),
                        row_bytes,
                    )
                    .expect("CHECK scaled_surface: invalid parameters");
            }
            let scaled_canvas = scaled_surface.getCanvas();
            scaled_canvas.scale(canvas_scale_x, canvas_scale_y);
            scaled_canvas.clearTransparent();
            scaled_canvas.drawImage(&source_bitmap.asImage(), 0.0, 0.0);
            if context.services.UIResourceContextProviderExists(sink) {
                let pixmap = scaled_surface.peekPixels();
                client_shared_image = context.services.CreateUIResourceSharedImage(
                    sink,
                    format,
                    upload_size,
                    &color_space,
                    shared_image_usage,
                    "LayerTreeHostUIResourceScaledBitmap",
                    pixmap.pixels,
                );
                assert!(client_shared_image.is_some());
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5768-5805.
        let sync_token = client_shared_image.as_ref().unwrap().creation_sync_token();
        let transferable = context
            .services
            .MakeUITransferableResource(client_shared_image.as_ref().unwrap().clone(), sync_token);
        id = self
            .resource_provider_
            .as_mut()
            .expect("source resource_provider")
            .ImportResource(
                transferable,
                HostUIResourceReleaseTask::new(*self.owner_id_, uid, &self.weak_factory_),
            );
        let mut data = UIResourceData::<B>::default();
        data.opaque = bitmap.GetOpaque();
        data.shared_image = client_shared_image.take();
        data.resource_id_for_export = id;
        data.size = upload_size;
        let target = self.ui_resource_map_.entry(uid).or_default();
        AssignMoveUIResourceData(target, &mut data);
        self.MarkUIResourceNotEvicted(uid, context);
        if self
            .settings_
            .TreesInVizInClientProcess(context.services.SettingsFeatures())
        {
            self.ui_resource_changes_
                .entry(uid)
                .or_default()
                .resource_created = true;
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5807-5836.
    fn DeleteUIResource(
        &mut self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
        context: &mut HostImplContext<'_, B>,
    ) {
        context
            .services
            .TraceUIResourceEvent("LayerTreeHostImpl::DeleteUIResource");
        if let Some(data) = self.ui_resource_map_.get_mut(&uid) {
            let id = data.resource_id_for_export;
            let target = self.deleted_ui_resources_.entry(uid).or_default();
            AssignMoveUIResourceData(target, data);
            self.ui_resource_map_.remove(&uid);
            let mut client = UIResourceReleaseClient::<B> {
                owner: *self.owner_id_,
                deleted: self.deleted_ui_resources_,
            };
            self.resource_provider_
                .as_mut()
                .unwrap()
                .RemoveImportedResource(id, &mut client);
            if self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures())
            {
                let change = self.ui_resource_changes_.entry(uid).or_default();
                if change.resource_created {
                    change.resource_created = false;
                } else {
                    change.resource_deleted = true;
                }
            }
        }
        self.MarkUIResourceNotEvicted(uid, context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5925-5934.
    fn MarkUIResourceNotEvicted(
        &mut self,
        uid: crate::adapter::layer_tree_impl::UIResourceId,
        context: &mut HostImplContext<'_, B>,
    ) {
        if !self.evicted_ui_resources_.contains(&uid) {
            return;
        }
        self.evicted_ui_resources_.remove(&uid);
        if self.evicted_ui_resources_.is_empty() {
            let can_draw = self.CanDraw(context);
            context
                .services
                .OnCanDrawStateChanged(*self.delegate_, can_draw);
        }
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:801-842. Both direct owner and child
// UI-resource callbacks execute this one source body with live field borrows.
fn CanDrawForHost<B: HostImplTypes>(
    sink: &Option<HostImplFrameSinkId>,
    resourceless: &bool,
    active: Option<&LayerTreeImpl<B>>,
    evicted_surface: &<B as LayerTreeTypes>::LocalSurfaceId,
    external: &Rect,
    evicted_ui: &BTreeSet<crate::adapter::layer_tree_impl::UIResourceId>,
    services: &dyn HostImplServices<B>,
) -> bool {
    use crate::adapter::unbounded::LocalSurfaceId;
    if sink.is_none() {
        services.TraceCanDrawReason("LayerTreeHostImpl::CanDraw no LayerTreeFrameSink");
        return false;
    }
    if *resourceless {
        return true;
    }
    if active.unwrap().LayerListIsEmpty() {
        services.TraceCanDrawReason("LayerTreeHostImpl::CanDraw no root layer");
        return false;
    }
    if evicted_surface.is_valid() {
        services.TraceCanDrawReason(
            "LayerTreeHostImpl::CanDraw viz::Surface evicted and not recreated",
        );
        return false;
    }
    if active
        .unwrap()
        .GetDeviceViewportWithExternal(*external)
        .IsEmpty()
    {
        services.TraceCanDrawReason("LayerTreeHostImpl::CanDraw empty viewport");
        return false;
    }
    if !evicted_ui.is_empty() {
        services
            .TraceCanDrawReason("LayerTreeHostImpl::CanDraw UI resources evicted not recreated");
        return false;
    }
    true
}

pub(crate) struct HostImplScrollbarRegistrationClient<'a, 'c, B: HostImplTypes> {
    pub(crate) owner: HostImplWeak,
    pub(crate) mutator: &'a mut <B as LayerTreeTypes>::MutatorHost,
    pub(crate) context: &'a mut HostImplContext<'c, B>,
}
impl<B: HostImplTypes> crate::adapter::scrollbar::ScrollbarRegistrationHost
    for HostImplScrollbarRegistrationClient<'_, '_, B>
{
    fn PostDelayedAutoscrollTask(
        &mut self,
        task: crate::adapter::scrollbar::AutoscrollTask,
        delay: TimeDelta,
    ) {
        self.context
            .services
            .PostDelayedAutoscrollTask(self.owner.clone(), task, delay);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5025-5027.
    fn ScrollAnimationAbort(&mut self, id: ElementId) {
        self.context
            .services
            .MutatorScrollAnimationAbort(self.mutator, id);
    }
}

// Native source bodies extracted once for direct owner and temporary child
// facades. Owner replacements are listed in /tmp/tree_host_owner_reuse.md.

// Source: cc/trees/layer_tree_host_impl.cc:3492-3514.
fn RequestedMSAASampleCountForHost(
    settings: &LayerTreeSettings,
    is_low_end: impl FnOnce() -> bool,
    device_scale: impl FnOnce() -> f32,
) -> i32 {
    if settings.gpu_rasterization_msaa_sample_count == -1 {
        if is_low_end() {
            return 4;
        }
        let device_scale_factor = device_scale();
        return if device_scale_factor >= 2.0 { 4 } else { 8 };
    }
    settings.gpu_rasterization_msaa_sample_count
}
// Source: cc/trees/layer_tree_host_impl.cc:2009-2024.
fn RasterMSAASampleCountForHost(
    list: &impl HostDisplayItemList,
    caps: &RasterCapabilities,
    settings: &LayerTreeSettings,
    is_low_end: impl FnOnce() -> bool,
    device_scale: impl FnOnce() -> f32,
) -> i32 {
    if list.num_slow_paths_up_to_min_for_MSAA() < kMinNumberOfSlowPathsForMSAA {
        return 0;
    }
    if !caps.can_use_msaa {
        return 0;
    }
    if list.has_non_aa_paint() {
        return 0;
    }
    RequestedMSAASampleCountForHost(settings, is_low_end, device_scale)
}
// Source: cc/trees/layer_tree_host_impl.cc:1947-1968.
fn TargetColorParamsForHost<B: HostImplTypes>(
    sink: Option<HostImplFrameSinkId>,
    usage: ContentColorUsage,
    services: &(impl HostColorContextServices + ?Sized),
    display_spaces: impl FnOnce() -> B::DisplayColorSpaces,
) -> HostTargetColorParams<B::ColorSpace> {
    let mut params = HostTargetColorParams::default();
    if sink.is_none() || services.FrameSinkContextProvider(sink.unwrap()).is_none() {
        return params;
    }
    let display_cs = display_spaces();
    let raster_color_space = display_cs.GetRasterAndCompositeColorSpace(usage);
    if raster_color_space.IsValid() {
        params.color_space = raster_color_space;
    }
    if params.color_space.IsHDR() {
        params.hdr_headroom = Some(display_cs.GetHdrHeadroom());
    }
    params
}
// Source: cc/trees/layer_tree_host_impl.cc:5433-5464.
fn ScrollbarControllerElementIdForTree(
    tree: &dyn crate::adapter::layer_tree_impl::ScrollbarControllerTree,
    controllers: &HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    element: ElementId,
) -> Option<ElementId> {
    if let Some(inner) = tree.InnerViewportScrollNode() {
        debug_assert!(tree.OuterViewportScrollNode().is_some());
        if element == inner.element_id
            || element == tree.OuterViewportScrollNode().unwrap().element_id
        {
            if controllers.contains_key(&inner.element_id) {
                return Some(inner.element_id);
            }
            let outer = tree.OuterViewportScrollNode().unwrap().element_id;
            if controllers.contains_key(&outer) {
                return Some(outer);
            }
            return None;
        }
    }
    if !controllers.contains_key(&element) {
        return None;
    }
    Some(element)
}
// Source: cc/trees/layer_tree_host_impl.cc:1865-1878.
fn DidModifyTilePrioritiesForHost<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    dirty: &mut bool,
    manager: &mut B::TileManager,
    delegate: HostImplDelegateId,
    pending_update_tiles: bool,
    context: &mut HostImplContext<'_, B>,
) {
    if settings.trees_in_viz_in_viz_process {
        return;
    }
    if !pending_update_tiles {
        *dirty = true;
        B::ResidentTilePrioritiesModified(manager);
    }
    context
        .services
        .DelegateSetNeedsPrepareTilesOnImplThread(delegate);
}
// Source: cc/trees/layer_tree_host_impl.cc:1587-1590.
fn DidAnimateScrollOffsetForHost<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    delegate: HostImplDelegateId,
    context: &mut HostImplContext<'_, B>,
) {
    let reason = context.services.BeginMainFrameReasonScroll();
    RequestCommitForHost(settings, delegate, reason, context);
    context.services.DelegateRenewTreePriority(delegate);
}
// Source: cc/trees/layer_tree_host_impl.cc:4520-4532.
fn ClearCachesForHost<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    manager: &mut B::TileManager,
    holder: &mut Option<Box<ImageDecodeCacheHolder<B::ImageDecodeFactory>>>,
    images: &mut Option<Box<ImageAnimationController>>,
    context: &mut HostImplContext<'_, B>,
) {
    B::ClearResidentCheckerImages(manager, true, context);
    if let Some(holder) = holder.as_mut() {
        HostImplImageDecodeCache::ClearCache(holder.image_decode_cache_mut());
    }
    if let Some(controller) = images.as_mut() {
        assert!(!settings.trees_in_viz_in_viz_process);
        controller.set_did_navigate();
    }
}

// TreeHost forwarding status: complete (95 native callbacks + DrawServices live capability).
// Source nonowning child-to-owner calls use synchronous selected-receiver
// borrows; every state lookup addresses the original resident allocation.
impl<B: HostImplTypes> crate::adapter::layer_tree_impl::LayerTreeHost<B>
    for HostImplTreeHostFacade<'_, '_, B>
{
    fn DisconnectPictureTwin(
        &mut self,
        id: crate::cc::layers::picture_layer_impl::PictureLayerImplId,
    ) -> crate::cc::layers::picture_layer_impl::PictureLayerImplId {
        use crate::adapter::layer_tree_impl::{LayerTreeLayer, LayerTreePictureLayer};
        for slot in [
            &mut self.active_tree_,
            &mut self.pending_tree_,
            &mut self.recycle_tree_,
        ] {
            let tree = match slot {
                HostImplTreeSlot::Resident(slot) => slot.as_deref_mut(),
                HostImplTreeSlot::Direct(tree) => Some(&mut **tree),
                HostImplTreeSlot::Reserved(_) => None,
            };
            let Some(tree) = tree.filter(|tree| tree.id == id.tree) else {
                continue;
            };
            let layer = tree
                .LayerByIdMutable(id.layer)
                .expect("native resident PictureLayerImpl twin");
            let picture = layer
                .AsPictureLayerMut()
                .expect("native twin is a PictureLayerImpl");
            assert_eq!(picture.IdentityForDestruction(), id);
            return picture.ClearTwinForDestruction();
        }
        panic!("native PictureLayerImpl twin must belong to another resident tree")
    }
    fn WithLayerContextParts(
        &mut self,
        selected: LayerTreeId,
        callback: &mut dyn FnMut(
            &dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerTreeHost<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
            &mut dyn crate::adapter::layer_tree_impl::LayerTreeScrollbarRegistrationHost<B>,
        ),
    ) where
        B: crate::adapter::picture_layer_impl::PictureLayerImplTypes,
    {
        let HostImplTreeHostFacade {
            core,
            active_tree_,
            pending_tree_,
            recycle_tree_,
            input_delegate_,
            scrollbar_animation_controllers_,
            image_animation_controller_,
            context,
            ..
        } = self;
        macro_rules! invoke {
            ($input_controller:expr,$input_view:expr) => {{
                let fields = crate::adapter::host_picture::HostPictureFields {
                    owner_id: *core.owner_id_,
                    selected_id: selected,
                    active: active_tree_.PictureSlot(),
                    pending: pending_tree_.PictureSlot(),
                    recycle: recycle_tree_.PictureSlot(),
                    settings: core.settings_,
                    debug_state: core.debug_state_,
                    raster_caps: core.raster_caps_,
                    global_tile_state: core.global_tile_state_,
                    resource_pool: core.resource_pool_,
                    resource_provider: core.resource_provider_,
                    layer_context: core.layer_context_,
                    latency_monitors: core.latency_info_swap_promise_monitor_,
                    events_metrics: core.events_metrics_manager_,
                    tile_manager: core.tile_manager_,
                    image_animation_controller: image_animation_controller_,
                    paint_worklet_tracker: core.paint_worklet_tracker_,
                    requires_high_res_to_draw: core.requires_high_res_to_draw_,
                    external_viewport: core.external_viewport_,
                    viewport_rect_for_tile_priority: core.viewport_rect_for_tile_priority_,
                    layer_tree_frame_sink: core.layer_tree_frame_sink_,
                    current_begin_frame_tracker: core.current_begin_frame_tracker_,
                    delegate: *core.delegate_,
                    input_view: $input_view,
                    services: context.services,
                };
                let registration =
                    crate::adapter::host_picture::HostPictureRegistrationOwnerFields {
                        owner: HostImplWeak {
                            owner: *core.owner_id_,
                            alive: Arc::downgrade(core.weak_factory_),
                        },
                        settings: core.settings_,
                        input_controller: $input_controller,
                        controllers: scrollbar_animation_controllers_,
                        flashed: core.flashed_scrollbars_,
                        previously_visible: core.previously_visible_scrollable_elements_,
                        accumulated: core.accumulated_scroll_deltas_by_element_id_,
                        mutator: core.mutator_host_.as_mut(),
                    };
                B::WithLayerContextParts(fields, registration, callback)
            }};
        }
        if let Some(input) = input_delegate_.as_deref_mut() {
            input.WithRegistrationAndView(|controller, view| invoke!(Some(controller), Some(view)))
        } else {
            invoke!(None, None)
        }
    }
    fn NewLayerTraceValue(&self) -> B::LayerTraceValue {
        self.context.services.NewHostTraceValueJSON()
    }
    fn DrawServices(
        &self,
    ) -> &dyn crate::adapter::layer_tree_impl::draw_host::LayerTreeDrawServices {
        self.context.services
    }

    // Source: cc/trees/layer_tree_impl.cc:2308,2345; injected tick clock.
    fn MonotonicTimeNow(&self) -> TimeTicks {
        self.context.services.MonotonicTimeNow()
    }
    // Source: cc/trees/layer_tree_impl.h:860,910; native nonowning owner identity.
    fn identity(&self) -> crate::adapter::layer_tree_impl::LayerTreeHostId {
        *self.core.owner_id_
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5518-5520.
    fn GetTreePriority(&self) -> crate::adapter::scheduler::TreePriority {
        self.context
            .services
            .GlobalTileStateTreePriority(self.core.global_tile_state_)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6241-6243.
    fn IsReadyToActivate(&self) -> bool {
        self.context
            .services
            .IsReadyToActivate(*self.core.delegate_)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6245-6248.
    fn RequestImplSideInvalidationForRerasterTiling(&mut self) {
        self.context
            .services
            .SetNeedsImplSideInvalidation(*self.core.delegate_, true);
    }
    // Source: cc/trees/layer_tree_host_impl.h:708-710.
    fn global_tile_state(&self) -> &B::GlobalTileState {
        self.core.global_tile_state_
    }
    // Source: cc/trees/layer_tree_host_impl.h:862-864.
    fn paint_worklet_tracker(&mut self) -> &mut B::PaintWorkletTracker {
        self.core.paint_worklet_tracker_
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4704-4706.
    fn DrawTransform(&self) -> &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::Transform {
        self.core.external_transform_
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6237-6239.
    fn IsInSynchronousComposite(&self) -> bool {
        self.context
            .services
            .IsInSynchronousComposite(*self.core.delegate_)
    }
    // Source: cc/trees/layer_tree_host_impl.h:335-337.
    fn viewport_mobile_optimized(&self) -> bool {
        *self.core.is_viewport_mobile_optimized_
    }
    // Source: cc/trees/layer_tree_host_impl.h:784.
    fn ResetRequiresHighResToDraw(&mut self) {
        *self.core.requires_high_res_to_draw_ = false;
    }
    // Source: cc/trees/layer_tree_host_impl.h:785.
    fn RequiresHighResToDraw(&self) -> bool {
        *self.core.requires_high_res_to_draw_
    }
    // Source: cc/trees/layer_tree_host_impl.h:570.
    fn max_texture_size(&self) -> i32 {
        self.core.raster_caps_.max_texture_size
    }
    // Source: cc/trees/layer_tree_host_impl.h:719.
    fn debug_state(&self) -> &B::DebugState {
        self.core.debug_state_
    }
    // Source: cc/trees/layer_tree_host_impl.h:702-704.
    fn resource_provider(&self) -> Option<&B::ClientResourceProvider> {
        self.core.resource_provider_.as_deref()
    }
    // Source: cc/trees/layer_tree_host_impl.h:576.
    fn tile_manager(&self) -> &B::TileManager {
        self.core.tile_manager_
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3582-3586.
    fn GetImageDecodeCache(&self) -> Option<&B::ImageDecodeCache> {
        self.core
            .image_decode_cache_holder_
            .as_ref()
            .map(|holder| holder.image_decode_cache())
    }
    // Source: cc/trees/layer_tree_host_impl.h:584-586.
    fn image_animation_controller(&self) -> Option<&B::ImageAnimationController> {
        self.image_animation_controller_.as_deref()
    }
    // Source: cc/trees/layer_tree_host_impl.h:699.
    fn frame_sorter(&self) -> &B::FrameSorter {
        self.core.frame_sorter_
    }
    // Source: cc/trees/layer_tree_host_impl.h:700.
    fn memory_history(&self) -> &B::MemoryHistory {
        self.core
            .memory_history_
            .as_deref()
            .expect("source dereferenced memory history")
    }
    // Source: cc/trees/layer_tree_host_impl.h:701.
    fn debug_rect_history(&self) -> &B::DebugRectHistory {
        self.core
            .debug_rect_history_
            .as_deref()
            .expect("source dereferenced debug rect history")
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3980-3985.
    fn IsPinchGestureActive(&self) -> bool {
        self.input_delegate_
            .as_ref()
            .is_some_and(|input| input.pinch_gesture_active())
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5522-5526.
    fn CurrentBeginFrameArgs(&self) -> &crate::adapter::begin_frame::BeginFrameArgs {
        self.core
            .current_begin_frame_tracker_
            .DangerousMethodCurrentOrLast()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5528-5530.
    fn CurrentBeginFrameInterval(&self) -> TimeDelta {
        self.core.current_begin_frame_tracker_.Interval()
    }
    // Source: cc/trees/layer_tree_host_impl.h:768-770.
    fn viewport_rect_for_tile_priority(&self) -> Rect {
        *self.core.viewport_rect_for_tile_priority_
    }
    // Source: cc/trees/layer_tree_host_impl.h:578.
    fn raster_caps(&self) -> &RasterCapabilities {
        self.core.raster_caps_
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4274-4290.
    fn SetNeedsRedraw(
        &mut self,
        receiver: &mut LayerTreeImpl<B>,
        animation_only: bool,
        skip_if_inside_draw: bool,
    ) {
        let promises = RequestRedrawForHost(
            self.core.settings_,
            *self.core.delegate_,
            self.core.latency_info_swap_promise_monitor_,
            self.core.events_metrics_manager_,
            animation_only,
            skip_if_inside_draw,
            self.context,
        );
        let active = self
            .active_tree_
            .ResolveMut(receiver)
            .expect("source active tree");
        QueuePinnedLatencyPromisesOnActiveTree(active, promises);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5921-5923.
    fn EvictedUIResourcesExist(&self) -> bool {
        !self.core.evicted_ui_resources_.is_empty()
    }
    // Source: cc/trees/layer_tree_host_impl.h:716.
    fn mutator_host(&self) -> &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost {
        self.core.mutator_host_
    }
    // Source: cc/trees/layer_tree_impl.cc:1525-1532; current manager's live bounds.
    fn TopControlsShownRatioRange(&self) -> (f32, f32) {
        let manager = self
            .browser_controls_offset_manager_
            .as_ref()
            .expect("source controls manager");
        manager.TopControlsShownRatioRange()
    }
    // Source: cc/trees/layer_tree_impl.cc:1540-1547; current manager's live bounds.
    fn BottomControlsShownRatioRange(&self) -> (f32, f32) {
        let manager = self
            .browser_controls_offset_manager_
            .as_ref()
            .expect("source controls manager");
        manager.BottomControlsShownRatioRange()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:1592-1594.
    fn SetViewportDamage(&mut self, rect: Rect) {
        self.core.viewport_damage_rect_.Union(rect);
    }
    // Source: cc/trees/layer_tree_host_impl.h:764.
    fn external_viewport(&self) -> Rect {
        *self.core.external_viewport_
    }
    // Source: cc/trees/layer_tree_impl.cc:2150-2160.
    fn IsActiveTree(&self, id: crate::adapter::layer_tree_impl::LayerTreeId) -> bool {
        self.active_tree_.identity() == Some(id)
    }
    fn IsPendingTree(&self, id: crate::adapter::layer_tree_impl::LayerTreeId) -> bool {
        self.pending_tree_.identity() == Some(id)
    }
    fn IsRecycleTree(&self, id: crate::adapter::layer_tree_impl::LayerTreeId) -> bool {
        self.recycle_tree_.identity() == Some(id)
    }
    // Source: cc/trees/layer_tree_host_impl.h:658-660; cc/trees/layer_tree_host_impl.cc:6155-6157.
    fn IsSyncTree(&self, id: crate::adapter::layer_tree_impl::LayerTreeId) -> bool {
        if self.core.settings_.commit_to_active_tree {
            self.IsActiveTree(id)
        } else {
            self.IsPendingTree(id)
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:2026-2028.
    fn HasPendingTree(&self) -> bool {
        self.pending_tree_.identity().is_some()
    }
    // Source: cc/trees/layer_tree_host_impl.h:656; native pointer-presence query.
    fn HasRecycleTree(&self) -> bool {
        self.recycle_tree_.identity().is_some()
    }
    // Source: cc/trees/layer_tree_host_impl.h:439.
    fn settings(&self) -> &B::Settings {
        self.core.settings_
    }
    // Source: cc/trees/layer_tree_settings.h:36-44; canonical settings feature query.
    fn TreesInVizInClientProcess(&self) -> bool {
        self.core
            .settings_
            .TreesInVizInClientProcess(self.context.services.SettingsFeatures())
    }
    fn TreeAnimationsInVizInVizProcess(&self) -> bool {
        self.core
            .settings_
            .TreeAnimationsInVizInVizProcess(self.context.services.SettingsFeatures())
    }
    // Source: cc/trees/layer_tree_host_impl.h:567-569.
    fn HasLayerTreeFrameSink(&self) -> bool {
        self.core.layer_tree_frame_sink_.is_some()
    }
    // Source: cc/trees/layer_tree_impl.h:514-516; native host draw-mode state.
    fn IsResourcelessSoftwareDraw(&self) -> bool {
        *self.core.resourceless_software_draw_
    }
    // Source: cc/trees/layer_tree_host_impl.h:584-586.
    fn HasImageAnimationController(&self) -> bool {
        self.image_animation_controller_.is_some()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:6250-6255.
    fn RequestImplSideInvalidationForRasterInducingScroll(&mut self, id: ElementId) {
        self.core
            .pending_invalidation_raster_inducing_scrolls_
            .insert(id);
        self.context
            .services
            .SetNeedsImplSideInvalidation(*self.core.delegate_, true);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3954-3958.
    fn SetNeedsCommit(&mut self) {
        RequestCommitForHost(
            self.core.settings_,
            *self.core.delegate_,
            B::OtherBeginMainFrameReason(),
            self.context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4512-4514.
    fn ClearHistory(&mut self) {
        self.context
            .services
            .DelegateClearHistory(*self.core.delegate_);
    }

    // Source: cc/trees/layer_tree_impl.cc:3110-3111.
    fn SelectionEdgeVisibilityUsesFullEdge(&self) -> bool {
        self.context.services.SelectionEdgeVisibilityUsesFullEdge()
    }
    // Source: cc/trees/layer_tree_impl.cc:3343-3344.
    fn ViewTransitionCaptureAndDisplay(&self) -> bool {
        self.context.services.ViewTransitionCaptureAndDisplay()
    }
    // Source: cc/trees/layer_tree_impl.cc:3206-3209; host pointer resolves same external client identity.
    fn GetVideoFrameControllerClient(&self) -> &B::VideoFrameControllerClient {
        self.context
            .services
            .VideoFrameControllerClient(*self.core.owner_id_)
    }
    // Source: cc/trees/layer_tree_host_impl.h:712-714; native typed nonowning provider.
    fn task_runner_provider(&self) -> &B::TaskRunnerProvider {
        self.context
            .services
            .TaskRunnerProvider(*self.core.task_runner_provider_)
    }
    // Source: cc/trees/layer_tree_host_impl.h:567-569.
    fn WithLayerTreeFrameSink(&self, callback: &mut dyn FnMut(Option<&B::LayerTreeFrameSink>)) {
        if let Some(sink) = *self.core.layer_tree_frame_sink_ {
            self.context
                .services
                .WithLayerTreeFrameSink(sink, &mut |actual| callback(Some(actual)))
        } else {
            callback(None)
        }
    }
    // Source: cc/trees/layer_tree_impl.cc:2118-2120.
    fn context_provider(&self) -> Option<&B::RasterContextProvider> {
        self.core
            .layer_tree_frame_sink_
            .and_then(|sink| self.context.services.FrameSinkRasterContext(sink))
    }
    // Source: cc/trees/layer_tree_impl.cc:2750-2752,2788-2790,2826-2828,3043-3045.
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.context.services.IsCCSlimmingEnabled()
    }
    // Source: cc/trees/layer_tree_impl.cc:2089-2092; native trace-only callback.
    fn TraceDidActivateLayerTree(&self, source_frame_number: i32) {
        self.context
            .services
            .TraceDidActivateLayerTree(*self.core.owner_id_, source_frame_number);
    }
    // Source: cc/trees/layer_tree_impl.cc:1203-1208; native diagnostics only.
    fn MissingPropertyNodeForElementId(
        &self,
        mutation: crate::cc::trees::layer_tree_impl::PropertyMutation,
    ) {
        self.context
            .services
            .MissingPropertyNodeForElementId(mutation);
    }
    // Source: cc/trees/layer_tree_impl.cc:1085-1088; native trace-only callback.
    fn TraceDelegatedInkMetadataPush(&self, metadata: &B::DelegatedInkMetadata) {
        self.context
            .services
            .TraceDelegatedInkMetadataPush(metadata);
    }
    // Source: cc/trees/layer_tree_impl.cc:1983; native trace-only callback.
    fn TraceLayersUpdatedCount(&self, count: usize) {
        self.context.services.TraceLayersUpdatedCount(count);
    }
    // Source: cc/trees/layer_tree_impl.cc:448-452; native trace-only callback.
    fn TraceImageInvalidationCounts(
        &self,
        no_images: usize,
        no_invalidation: usize,
        invalidated: usize,
    ) {
        self.context
            .services
            .TraceImageInvalidationCounts(no_images, no_invalidation, invalidated);
    } // Source: cc/trees/layer_tree_impl.cc:2150-2160; native selected tree pointer.
    fn ActiveTreeLayerById<'s>(
        &'s self,
        receiver: &'s LayerTreeImpl<B>,
        id: i32,
    ) -> Option<&'s B::Layer> {
        self.active_tree_
            .Resolve(receiver)
            .and_then(|tree| tree.LayerById(id))
    }
    fn PendingTreeLayerById<'s>(
        &'s self,
        receiver: &'s LayerTreeImpl<B>,
        id: i32,
    ) -> Option<&'s B::Layer> {
        self.pending_tree_
            .Resolve(receiver)
            .and_then(|tree| tree.LayerById(id))
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3968-3970.
    fn OuterViewportScrollNode<'s>(
        &'s self,
        receiver: &'s LayerTreeImpl<B>,
    ) -> Option<&'s crate::cc::trees::scroll_node::ScrollNode> {
        self.active_tree_
            .Resolve(receiver)
            .unwrap()
            .OuterViewportScrollNode()
    }
    // Source: cc/trees/layer_tree_host_impl.cc:3987-3993; same canonical Input body.
    fn GetActivelyScrollingType(
        &self,
        receiver: &LayerTreeImpl<B>,
    ) -> crate::cc::input::actively_scrolling_type::ActivelyScrollingType {
        let Some(input) = self.input_delegate_.as_deref() else {
            return crate::cc::input::actively_scrolling_type::ActivelyScrollingType::kNone;
        };
        let active = self.active_tree_.Resolve(receiver).unwrap();
        input.View().GetActivelyScrollingTypeForTree(
            active.property_trees().scroll_tree(),
            self.core.settings_,
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:2009-2024,3492-3514.
    fn GetMSAASampleCountForRaster(
        &self,
        receiver: &LayerTreeImpl<B>,
        list: &B::DisplayItemList,
    ) -> i32 {
        RasterMSAASampleCountForHost(
            list,
            self.core.raster_caps_,
            self.core.settings_,
            || self.context.services.IsLowEndDevice(),
            || {
                if self.pending_tree_.identity().is_some() {
                    self.pending_tree_
                        .Resolve(receiver)
                        .unwrap()
                        .device_scale_factor()
                } else {
                    self.active_tree_
                        .Resolve(receiver)
                        .unwrap()
                        .device_scale_factor()
                }
            },
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:1922-1932,1947-1968.
    fn GetTargetColorParams(
        &self,
        receiver: &LayerTreeImpl<B>,
        usage: B::ContentColorUsage,
    ) -> B::TargetColorParams {
        TargetColorParamsForHost::<B>(
            *self.core.layer_tree_frame_sink_,
            usage,
            self.context.services,
            || {
                if self.pending_tree_.identity().is_some() {
                    self.pending_tree_
                        .Resolve(receiver)
                        .unwrap()
                        .display_color_spaces()
                        .clone()
                } else if self.active_tree_.identity().is_some() {
                    self.active_tree_
                        .Resolve(receiver)
                        .unwrap()
                        .display_color_spaces()
                        .clone()
                } else {
                    B::DisplayColorSpaces::default()
                }
            },
        )
    }
    // Source: cc/trees/layer_tree_impl.cc:2212-2237; factory body stays canonical.
    fn CreateScrollbarAnimationController(
        &mut self,
        receiver: &mut LayerTreeImpl<B>,
        id: ElementId,
        opacity: f32,
    ) -> Option<Box<ScrollbarAnimationController>> {
        let settings = self.core.settings_;
        let tree = self.active_tree_.ResolveMut(receiver).unwrap();
        let mut client = HostImplScrollbarClient {
            active_tree: tree,
            settings,
            delegate: *self.core.delegate_,
            events: self.core.events_metrics_manager_,
            latency_monitors: self.core.latency_info_swap_promise_monitor_,
            context: self.context,
        };
        LayerTreeImpl::<B>::CreateScrollbarAnimationControllerWithClient(
            settings,
            id,
            opacity,
            &mut client,
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5433-5464. The controller and
    // native client aliases remain inside the synchronous source operation.
    fn WithScrollbarAnimationController(
        &mut self,
        receiver: &mut LayerTreeImpl<B>,
        id: ElementId,
        callback: &mut dyn FnMut(
            Option<&mut ScrollbarAnimationController>,
            &mut dyn ScrollbarAnimationControllerClient<
                Transform = <B as LayerTreeTypes>::Transform,
            >,
        ),
    ) {
        let tree = self.active_tree_.ResolveMut(receiver).unwrap();
        let key =
            ScrollbarControllerElementIdForTree(tree, self.scrollbar_animation_controllers_, id);
        let mut client = HostImplScrollbarClient {
            active_tree: tree,
            settings: self.core.settings_,
            delegate: *self.core.delegate_,
            events: self.core.events_metrics_manager_,
            latency_monitors: self.core.latency_info_swap_promise_monitor_,
            context: self.context,
        };
        let controller = key
            .and_then(|key| self.scrollbar_animation_controllers_.get_mut(&key))
            .and_then(Option::as_deref_mut);
        callback(controller, &mut client);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:788-799;
    // cc/trees/layer_tree_impl.cc:1947-1951. This map is the native owned output,
    // not a snapshot used to stand in for the resident trees.
    fn UpdateImageAnimationStateFromDrivers(&mut self, receiver: &mut LayerTreeImpl<B>) {
        let drivers = GatherAnimatedImageDriverStateForTrees(
            self.active_tree_.Resolve(receiver).unwrap(),
            self.pending_tree_.Resolve(receiver),
            self.recycle_tree_.Resolve(receiver),
        );
        UpdateImageAnimationStateForHost(
            self.active_tree_.ResolveMut(receiver).unwrap(),
            self.core.settings_,
            self.core.impl_thread_phase_,
            *self.core.delegate_,
            self.core.latency_info_swap_promise_monitor_,
            self.core.events_metrics_manager_,
            self.image_animation_controller_,
            &drivers,
            self.context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:1865-1878; sole shared source body.
    fn DidModifyTilePriorities(&mut self, pending_update_tiles: bool) {
        DidModifyTilePrioritiesForHost(
            self.core.settings_,
            self.core.tile_priorities_dirty_,
            self.core.tile_manager_,
            *self.core.delegate_,
            pending_update_tiles,
            self.context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:1587-1590; sole shared source body.
    fn DidAnimateScrollOffset(&mut self) {
        DidAnimateScrollOffsetForHost(self.core.settings_, *self.core.delegate_, self.context);
    } // Source: cc/trees/layer_tree_host_impl.cc:4825-4830.
    fn UpdateImageDecodingHints(
        &mut self,
        map: BTreeMap<
            <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::PaintImageId,
            B::DecodingMode,
        >,
    ) {
        self.context
            .services
            .UpdateCheckerImageDecodingHints(self.core.tile_manager_, map);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:2354-2359.
    fn SetViewTransitionContentRect(
        &mut self,
        sequence: u32,
        id: &crate::adapter::property_types::ViewTransitionElementResourceId,
        rect: &crate::adapter::geometry::RectF,
    ) {
        SetViewTransitionContentRectForHost(
            self.core.view_transition_content_rects_,
            sequence,
            id,
            rect,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5898-5905.
    fn ResourceIdForUIResource(
        &self,
        id: crate::adapter::layer_tree_impl::UIResourceId,
    ) -> crate::adapter::layer_tree_impl::ResourceId {
        self.core.ui_resource_map_.get(&id).map_or(
            crate::adapter::layer_tree_impl::ResourceId::default(),
            |data| data.resource_id_for_export,
        )
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5907-5913.
    fn GetUIResourceSize(&self, id: crate::adapter::layer_tree_impl::UIResourceId) -> Size {
        self.core
            .ui_resource_map_
            .get(&id)
            .map_or(Size::default(), |data| data.size)
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5915-5919.
    fn IsUIResourceOpaque(&self, id: crate::adapter::layer_tree_impl::UIResourceId) -> bool {
        self.core
            .ui_resource_map_
            .get(&id)
            .expect("CHECK UIResource exists")
            .opaque
    }
    // Source: cc/trees/layer_tree_impl.cc:814-817. External mutator call; the
    // unsafe commit owner's handle is resolved synchronously by its owner.
    fn PushMutatorPropertiesFrom(
        &mut self,
        unsafe_state: &B::UnsafeCommitState,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
    ) {
        self.context
            .services
            .PushMutatorPropertiesTo(unsafe_state, self.core.mutator_host_, trees);
    }
    // Source: cc/trees/property_tree.cc:1630-1631,1647; RenderSurfaceImpl is external
    // layer-owned state; the factory returns its unique native owner.
    fn CreateRenderSurface(
        &mut self,
        tree: crate::adapter::layer_tree_impl::LayerTreeId,
        id: ElementId,
    ) -> Box<B::RenderSurface> {
        self.context.services.CreateRenderSurface(tree, id)
    }
    // Source: cc/trees/layer_tree_impl.cc:1438-1447. Process-scoped crash report
    // admission belongs to the external diagnostic sink, never a tree cache.
    fn ReportUnexpectedOopifPageScaleOnce(&mut self, scale: f32) {
        self.context
            .services
            .ReportUnexpectedOopifPageScaleOnce(scale);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4520-4532.
    fn ClearCaches(&mut self) {
        ClearCachesForHost(
            self.core.settings_,
            self.core.tile_manager_,
            self.core.image_decode_cache_holder_,
            self.image_animation_controller_,
            self.context,
        );
    }
    // Rust borrow spelling for cc/trees/layer_tree_impl.cc:989-1121: native
    // source and target are independent resident allocations. This callback is
    // invoked exactly once, synchronously, and no source borrow can escape.
    fn WithBorrowedTree(
        &mut self,
        source: &mut LayerTreeImpl<B>,
        callback: &mut dyn FnMut(&mut dyn crate::adapter::layer_tree_impl::LayerTreeHost<B>),
    ) {
        let id = source.id;
        let mut source = Some(source);
        let mut host = HostImplTreeHostFacade {
            core: self.core.reborrow(),
            active_tree_: self.active_tree_.ReborrowWithSource(id, &mut source),
            pending_tree_: self.pending_tree_.ReborrowWithSource(id, &mut source),
            recycle_tree_: self.recycle_tree_.ReborrowWithSource(id, &mut source),
            input_delegate_: &mut *self.input_delegate_,
            browser_controls_offset_manager_: self.browser_controls_offset_manager_.reborrow(),
            scrollbar_animation_controllers_: &mut *self.scrollbar_animation_controllers_,
            image_animation_controller_: &mut *self.image_animation_controller_,
            context: &mut *self.context,
        };
        assert!(source.is_none(), "source tree belongs to this host");
        callback(&mut host);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5611-5805; shared canonical UI body.
    fn CreateUIResource(
        &mut self,
        receiver: &LayerTreeImpl<B>,
        id: crate::adapter::layer_tree_impl::UIResourceId,
        bitmap: &B::UIResourceBitmap,
    ) {
        let active = self.active_tree_.Resolve(receiver);
        HostImplUIResourceState::new(&mut self.core, active).CreateUIResource(
            id,
            bitmap,
            self.context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5807-5836; shared canonical UI body.
    fn DeleteUIResource(
        &mut self,
        receiver: &LayerTreeImpl<B>,
        id: crate::adapter::layer_tree_impl::UIResourceId,
    ) {
        let active = self.active_tree_.Resolve(receiver);
        HostImplUIResourceState::new(&mut self.core, active).DeleteUIResource(id, self.context);
    }
    // Source: cc/trees/layer_tree_host_impl.cc:2393-2395.
    fn OnCanDrawStateChangedForTree(&mut self, receiver: &LayerTreeImpl<B>) {
        let active = self.active_tree_.Resolve(receiver);
        let can_draw = CanDrawForHost(
            self.core.layer_tree_frame_sink_,
            self.core.resourceless_software_draw_,
            active,
            self.core.evicted_local_surface_id_,
            self.core.external_viewport_,
            self.core.evicted_ui_resources_,
            self.context.services,
        );
        self.context
            .services
            .OnCanDrawStateChanged(*self.core.delegate_, can_draw);
    } // Source: cc/trees/layer_tree_host_impl.cc:5389-5408; shared source body.
    fn RegisterScrollbarAnimationController(
        &mut self,
        receiver: &mut LayerTreeImpl<B>,
        id: ElementId,
        opacity: f32,
    ) {
        let tree = self.active_tree_.ResolveMut(receiver).unwrap();
        RegisterScrollbarControllerForHost(
            self.core.settings_,
            tree,
            self.scrollbar_animation_controllers_,
            self.core.flashed_scrollbars_,
            self.core.previously_visible_scrollable_elements_,
            *self.core.delegate_,
            self.core.events_metrics_manager_,
            self.core.latency_info_swap_promise_monitor_,
            id,
            opacity,
            self.context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5410-5416.
    fn DidRegisterScrollbarLayer(
        &mut self,
        receiver: &mut LayerTreeImpl<B>,
        id: ElementId,
        orientation: ScrollbarOrientation,
    ) {
        debug_assert_eq!(self.active_tree_.identity(), Some(receiver.id));
        let owner = HostImplWeak {
            owner: *self.core.owner_id_,
            alive: std::sync::Arc::downgrade(self.core.weak_factory_),
        };
        NotifyRegisteredScrollbarForHost(
            self.input_delegate_.as_deref_mut(),
            self.core.mutator_host_.as_mut(),
            owner,
            id,
            orientation,
            self.context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5418-5431; shared source bodies.
    fn DidUnregisterScrollbarLayer(
        &mut self,
        receiver: &mut LayerTreeImpl<B>,
        id: ElementId,
        orientation: ScrollbarOrientation,
    ) {
        let tree = self.active_tree_.Resolve(receiver).unwrap();
        RemoveUnregisteredScrollbarStateForHost(
            tree,
            self.scrollbar_animation_controllers_,
            self.core.flashed_scrollbars_,
            self.core.previously_visible_scrollable_elements_,
            self.core.accumulated_scroll_deltas_by_element_id_,
            id,
        );
        let owner = HostImplWeak {
            owner: *self.core.owner_id_,
            alive: std::sync::Arc::downgrade(self.core.weak_factory_),
        };
        NotifyUnregisteredScrollbarForHost(
            self.input_delegate_.as_deref_mut(),
            self.core.mutator_host_.as_mut(),
            owner,
            id,
            orientation,
            self.context,
        );
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5466-5472; canonical source policy.
    fn OnPageScaleUpdated(&mut self, receiver: &mut LayerTreeImpl<B>) {
        let active = self.active_tree_.ResolveMut(receiver).unwrap();
        let mut client = HostImplScrollbarClient {
            active_tree: active,
            settings: self.core.settings_,
            delegate: *self.core.delegate_,
            events: self.core.events_metrics_manager_,
            latency_monitors: self.core.latency_info_swap_promise_monitor_,
            context: self.context,
        };
        OnPageScaleUpdatedForHost(
            self.core.settings_,
            self.scrollbar_animation_controllers_,
            self.core.flashed_scrollbars_,
            &mut client,
        );
    } // Source: cc/trees/layer_tree_impl.cc:1621-1626. The target's actual source
      // callback body is shared with all narrower scroll/page-scale hosts.
    fn PendingTreeDidUpdatePageScale(&mut self) {
        crate::adapter::layer_tree_impl::mutation::LayerTreeScaleHost::PendingTreeDidUpdatePageScale(&mut self.MutationHost());
    }
    fn RecycleTreeDidUpdatePageScale(&mut self) {
        crate::adapter::layer_tree_impl::mutation::LayerTreeScaleHost::RecycleTreeDidUpdatePageScale(&mut self.MutationHost());
    }
    // Source: cc/trees/layer_tree_impl.cc:271-280.
    fn PendingTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool) {
        crate::adapter::layer_tree_impl::mutation::LayerTreeScrollHost::PendingTreeDidUpdateScrollOffset(&mut self.MutationHost(),id,pushed);
    }
    fn RecycleTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool) {
        crate::adapter::layer_tree_impl::mutation::LayerTreeScrollHost::RecycleTreeDidUpdateScrollOffset(&mut self.MutationHost(),id,pushed);
    }
    // Source: cc/trees/layer_tree_impl.cc:3294-3303. Canonical manager owns all
    // animation/reset decisions; the client only borrows resident siblings.
    fn ResetBrowserControlsAnimations(&mut self, receiver: &mut LayerTreeImpl<B>) {
        self.WithControlsReceiver(receiver, |manager, client| manager.ResetAnimations(client));
    }
    // Source: cc/trees/layer_tree_impl.cc:1513-1519.
    fn OnBrowserControlsParamsChanged(
        &mut self,
        receiver: &mut LayerTreeImpl<B>,
        animate_height_changes: bool,
    ) {
        self.WithControlsReceiver(receiver, |manager, client| {
            manager.OnBrowserControlsParamsChanged(client, animate_height_changes)
        });
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4708-4716. Source body is shared
    // with canonical manager callbacks; no controls state is reconstructed.
    fn DidChangeBrowserControlsPosition(&mut self, receiver: &mut LayerTreeImpl<B>) {
        self.WithControlsReceiver(receiver,|manager,client|crate::cc::input::browser_controls_offset_manager_client::BrowserControlsOffsetManagerClient::DidChangeBrowserControlsPosition(client,manager));
    }
}

// Rust expression of native nonowning tree pointer lookup. A reserved slot is
// resolved with the actual current tree borrow, never treated as an empty slot.
impl<B: HostImplTypes> HostImplTreeSlot<'_, B> {
    fn Resolve<'s>(&'s self, receiver: &'s LayerTreeImpl<B>) -> Option<&'s LayerTreeImpl<B>> {
        match self {
            Self::Resident(slot) => slot.as_deref(),
            Self::Direct(tree) => Some(&**tree),
            Self::Reserved(id) => {
                assert_eq!(*id, receiver.id, "selected tree identity");
                Some(receiver)
            }
        }
    }
    fn ResolveMut<'s>(
        &'s mut self,
        receiver: &'s mut LayerTreeImpl<B>,
    ) -> Option<&'s mut LayerTreeImpl<B>> {
        match self {
            Self::Resident(slot) => slot.as_deref_mut(),
            Self::Direct(tree) => Some(&mut **tree),
            Self::Reserved(id) => {
                assert_eq!(*id, receiver.id, "selected tree identity");
                Some(receiver)
            }
        }
    }
}

impl<B: HostImplTypes> HostImplTreeSlot<'_, B> {
    fn ReborrowWithSource<'s>(
        &'s mut self,
        id: crate::adapter::layer_tree_impl::LayerTreeId,
        source: &mut Option<&'s mut LayerTreeImpl<B>>,
    ) -> HostImplTreeSlot<'s, B> {
        if matches!(self,Self::Reserved(reserved) if *reserved==id) {
            HostImplTreeSlot::Direct(source.take().expect("one selected native tree slot"))
        } else {
            self.reborrow()
        }
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:5389-5408.
fn RegisterScrollbarControllerForHost<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    tree: &mut LayerTreeImpl<B>,
    controllers: &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    flashed: &HashSet<ElementId>,
    previously_visible: &BTreeSet<ElementId>,
    delegate: HostImplDelegateId,
    events: &mut B::EventsMetricsManager,
    monitors: &BTreeSet<HostImplLatencyMonitorId>,
    element: ElementId,
    opacity: f32,
    context: &mut HostImplContext<'_, B>,
) {
    if ScrollbarControllerElementIdForTree(tree, controllers, element)
        .and_then(|key| controllers.get(&key).unwrap().as_ref())
        .is_some()
    {
        return;
    }
    let mut client = HostImplScrollbarClient {
        active_tree: tree,
        settings,
        delegate,
        events,
        latency_monitors: monitors,
        context,
    };
    RegisterScrollbarControllerAfterLookupForHost::<B>(
        settings,
        controllers,
        flashed,
        previously_visible,
        element,
        opacity,
        &mut client,
    );
}
// Source: cc/trees/layer_tree_host_impl.cc:5394-5408. Unique native creation
// body after the lookup; both whole-tree and selected-layer clients use it.
fn RegisterScrollbarControllerAfterLookupForHost<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    controllers: &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    flashed: &HashSet<ElementId>,
    previously_visible: &BTreeSet<ElementId>,
    element: ElementId,
    opacity: f32,
    client: &mut dyn crate::adapter::scrollbar::ScrollbarInitializationClient,
) {
    let controller = LayerTreeImpl::<B>::CreateScrollbarAnimationControllerWithClient(
        settings, element, opacity, client,
    );
    controllers.insert(element, controller);
    if settings.scrollbar_flash_once_visible_on_viewport {
        assert!(!flashed.contains(&element));
    }
    if settings.scrollbar_flash_once_visible_on_viewport {
        assert!(!previously_visible.contains(&element));
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:5418-5425.
fn RemoveUnregisteredScrollbarStateForHost<B: HostImplTypes>(
    tree: &LayerTreeImpl<B>,
    controllers: &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    flashed: &mut HashSet<ElementId>,
    previously_visible: &mut BTreeSet<ElementId>,
    accumulated: &mut HashMap<ElementId, Vector2dF>,
    element: ElementId,
) {
    RemoveUnregisteredScrollbarStateAfterQueryForHost(
        tree.ScrollbarsFor(element).is_empty(),
        controllers,
        flashed,
        previously_visible,
        accumulated,
        element,
    );
}
// Source: cc/trees/layer_tree_host_impl.cc:5418-5425. The query is evaluated
// at its original call point before lending the same owner fields.
fn RemoveUnregisteredScrollbarStateAfterQueryForHost(
    empty: bool,
    controllers: &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    flashed: &mut HashSet<ElementId>,
    previously_visible: &mut BTreeSet<ElementId>,
    accumulated: &mut HashMap<ElementId, Vector2dF>,
    element: ElementId,
) {
    if empty {
        controllers.remove(&element);
        flashed.remove(&element);
        previously_visible.remove(&element);
        accumulated.remove(&element);
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:5410-5416.
fn NotifyRegisteredScrollbarForHost<B: HostImplTypes>(
    input: Option<&mut B::InputHandler>,
    mutator: &mut <B as LayerTreeTypes>::MutatorHost,
    owner: HostImplWeak,
    element: ElementId,
    orientation: ScrollbarOrientation,
    context: &mut HostImplContext<'_, B>,
) {
    if let Some(input) = input {
        let mut client = HostImplScrollbarRegistrationClient {
            owner,
            mutator,
            context,
        };
        input.DidRegisterScrollbarWithHost(&mut client, element, orientation);
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:5427-5430.
fn NotifyUnregisteredScrollbarForHost<B: HostImplTypes>(
    input: Option<&mut B::InputHandler>,
    mutator: &mut <B as LayerTreeTypes>::MutatorHost,
    owner: HostImplWeak,
    element: ElementId,
    orientation: ScrollbarOrientation,
    context: &mut HostImplContext<'_, B>,
) {
    if let Some(input) = input {
        let mut client = HostImplScrollbarRegistrationClient {
            owner,
            mutator,
            context,
        };
        input.DidUnregisterScrollbarWithHost(&mut client, element, orientation);
    }
}

impl<B: HostImplTypes> HostImplInputTree<'_, B> {
    pub(crate) fn as_host_slot(&mut self) -> HostImplTreeSlot<'_, B> {
        match self {
            Self::Slot(slot) => HostImplTreeSlot::Resident(&mut **slot),
            Self::Direct(tree) => HostImplTreeSlot::Direct(&mut **tree),
        }
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:6324-6333.
fn FlashAllScrollbarsForHost(
    settings: &LayerTreeSettings,
    controllers: &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    did_scroll: bool,
    client: &mut dyn ScrollbarAnimationControllerClient<
        Transform = impl crate::adapter::scrollbar::ScrollbarTransform,
    >,
) {
    assert!(settings.scrollbar_flash_after_any_scroll_update);
    for controller in controllers.values_mut() {
        let controller = controller
            .as_mut()
            .expect("source scrollbar controller dereference");
        if did_scroll {
            controller.DidScrollUpdate(client);
        } else {
            controller.WillUpdateScroll(client);
        }
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:6335-6338.
fn EraseFlashedScrollbarsForHost(settings: &LayerTreeSettings, flashed: &mut HashSet<ElementId>) {
    assert!(settings.scrollbar_flash_once_after_scroll_update);
    flashed.clear();
}
// Source: cc/trees/layer_tree_host_impl.cc:6340-6363.
fn MaybeFlashAllScrollbarsOnceForHost(
    settings: &LayerTreeSettings,
    controllers: &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    flashed: &mut HashSet<ElementId>,
    tracking: ElementId,
    did_scroll: bool,
    client: &mut dyn ScrollbarAnimationControllerClient<
        Transform = impl crate::adapter::scrollbar::ScrollbarTransform,
    >,
) -> bool {
    assert!(settings.scrollbar_flash_once_after_scroll_update);
    let mut flashed_tracking_element_id = false;
    for (&element_id, controller) in controllers.iter_mut() {
        if flashed.contains(&element_id) {
            continue;
        }
        let controller = controller
            .as_mut()
            .expect("source scrollbar controller dereference");
        if did_scroll {
            controller.DidScrollUpdate(client);
        } else {
            controller.WillUpdateScroll(client);
        }
        flashed.insert(element_id);
        flashed_tracking_element_id = element_id == tracking;
    }
    flashed_tracking_element_id
}
// Source: cc/trees/layer_tree_host_impl.cc:6306-6322.
fn MaybeFlashAllScrollbarsForHost(
    settings: &LayerTreeSettings,
    controllers: &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    flashed: &mut HashSet<ElementId>,
    tracking: ElementId,
    did_scroll: bool,
    client: &mut dyn ScrollbarAnimationControllerClient<
        Transform = impl crate::adapter::scrollbar::ScrollbarTransform,
    >,
) -> bool {
    let mut has_tracking_element_id_flashed = false;
    if settings.scrollbar_flash_after_any_scroll_update {
        FlashAllScrollbarsForHost(settings, controllers, did_scroll, client);
        has_tracking_element_id_flashed = true;
    } else if settings.scrollbar_flash_once_after_scroll_update {
        has_tracking_element_id_flashed = MaybeFlashAllScrollbarsOnceForHost(
            settings,
            controllers,
            flashed,
            tracking,
            did_scroll,
            client,
        );
    }
    has_tracking_element_id_flashed
}
// Source: cc/trees/layer_tree_host_impl.cc:5466-5472.
fn OnPageScaleUpdatedForHost(
    settings: &LayerTreeSettings,
    controllers: &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
    flashed: &mut HashSet<ElementId>,
    client: &mut dyn ScrollbarAnimationControllerClient<
        Transform = impl crate::adapter::scrollbar::ScrollbarTransform,
    >,
) {
    if settings.scrollbar_flash_once_after_scroll_update {
        EraseFlashedScrollbarsForHost(settings, flashed);
    }
    MaybeFlashAllScrollbarsForHost(
        settings,
        controllers,
        flashed,
        ElementId::default(),
        false,
        client,
    );
}

// Source input/runtime field borrows; no long-lived owner references.
mod source_input_forwarding {
    // Source-aligned live mutation capabilities; selected receiver is borrowed out.
    use super::*;
    use crate::adapter::layer_tree_impl::mutation::{LayerTreeScaleHost, LayerTreeScrollHost};
    use crate::adapter::layer_tree_impl::LayerTreeId;
    pub(crate) struct HostImplMutationFacade<'a, 'c, B: HostImplTypes> {
        pub(crate) core: HostImplRuntimeCore<'a, B>,
        pub(crate) active_tree_: HostImplTreeSlot<'a, B>,
        pub(crate) pending_tree_: HostImplTreeSlot<'a, B>,
        pub(crate) recycle_tree_: HostImplTreeSlot<'a, B>,
        pub(crate) browser_controls_offset_manager_: HostImplControlsSlot<'a>,
        pub(crate) scrollbar_animation_controllers_:
            &'a mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
        pub(crate) image_animation_controller_: &'a mut Option<Box<ImageAnimationController>>,
        pub(crate) context: &'a mut HostImplContext<'c, B>,
    }
    impl<B: HostImplTypes> HostImplInputTree<'_, B> {
        pub(crate) fn MutationSlot(&mut self) -> HostImplTreeSlot<'_, B> {
            match self {
                Self::Slot(slot) => HostImplTreeSlot::Resident(&mut **slot),
                Self::Direct(tree) => HostImplTreeSlot::Direct(&mut **tree),
            }
        }
    }
    impl<B: HostImplTypes> HostImplMutationFacade<'_, '_, B> {
        pub(crate) fn WithActiveTree<R>(
            &mut self,
            callback: impl FnOnce(&mut LayerTreeImpl<B>, &mut HostImplMutationFacade<'_, '_, B>) -> R,
        ) -> R {
            let Self {
                core,
                active_tree_,
                pending_tree_,
                recycle_tree_,
                browser_controls_offset_manager_,
                scrollbar_animation_controllers_,
                image_animation_controller_,
                context,
            } = self;
            let tree = match active_tree_ {
                HostImplTreeSlot::Resident(slot) => {
                    slot.as_deref_mut().expect("source nonnull active tree")
                }
                HostImplTreeSlot::Direct(tree) => &mut **tree,
                HostImplTreeSlot::Reserved(_) => {
                    panic!("selected tree must be supplied as a synchronous Direct receiver")
                }
            };
            let selected_id = tree.id;
            let mut host = HostImplMutationFacade {
                core: core.reborrow(),
                active_tree_: HostImplTreeSlot::Reserved(selected_id),
                pending_tree_: pending_tree_.reborrow(),
                recycle_tree_: recycle_tree_.reborrow(),
                browser_controls_offset_manager_: browser_controls_offset_manager_.reborrow(),
                scrollbar_animation_controllers_: &mut **scrollbar_animation_controllers_,
                image_animation_controller_: &mut **image_animation_controller_,
                context: &mut **context,
            };
            callback(tree, &mut host)
        }
        pub(crate) fn WithPendingTree<R>(
            &mut self,
            callback: impl FnOnce(&mut LayerTreeImpl<B>, &mut HostImplMutationFacade<'_, '_, B>) -> R,
        ) -> R {
            let Self {
                core,
                active_tree_,
                pending_tree_,
                recycle_tree_,
                browser_controls_offset_manager_,
                scrollbar_animation_controllers_,
                image_animation_controller_,
                context,
            } = self;
            let tree = match pending_tree_ {
                HostImplTreeSlot::Resident(slot) => {
                    slot.as_deref_mut().expect("source nonnull pending tree")
                }
                HostImplTreeSlot::Direct(tree) => &mut **tree,
                HostImplTreeSlot::Reserved(_) => {
                    panic!("selected tree must be supplied as a synchronous Direct receiver")
                }
            };
            let selected_id = tree.id;
            let mut host = HostImplMutationFacade {
                core: core.reborrow(),
                active_tree_: active_tree_.reborrow(),
                pending_tree_: HostImplTreeSlot::Reserved(selected_id),
                recycle_tree_: recycle_tree_.reborrow(),
                browser_controls_offset_manager_: browser_controls_offset_manager_.reborrow(),
                scrollbar_animation_controllers_: &mut **scrollbar_animation_controllers_,
                image_animation_controller_: &mut **image_animation_controller_,
                context: &mut **context,
            };
            callback(tree, &mut host)
        }
        pub(crate) fn WithRecycleTree<R>(
            &mut self,
            callback: impl FnOnce(&mut LayerTreeImpl<B>, &mut HostImplMutationFacade<'_, '_, B>) -> R,
        ) -> R {
            let Self {
                core,
                active_tree_,
                pending_tree_,
                recycle_tree_,
                browser_controls_offset_manager_,
                scrollbar_animation_controllers_,
                image_animation_controller_,
                context,
            } = self;
            let tree = match recycle_tree_ {
                HostImplTreeSlot::Resident(slot) => {
                    slot.as_deref_mut().expect("source nonnull recycle tree")
                }
                HostImplTreeSlot::Direct(tree) => &mut **tree,
                HostImplTreeSlot::Reserved(_) => {
                    panic!("selected tree must be supplied as a synchronous Direct receiver")
                }
            };
            let selected_id = tree.id;
            let mut host = HostImplMutationFacade {
                core: core.reborrow(),
                active_tree_: active_tree_.reborrow(),
                pending_tree_: pending_tree_.reborrow(),
                recycle_tree_: HostImplTreeSlot::Reserved(selected_id),
                browser_controls_offset_manager_: browser_controls_offset_manager_.reborrow(),
                scrollbar_animation_controllers_: &mut **scrollbar_animation_controllers_,
                image_animation_controller_: &mut **image_animation_controller_,
                context: &mut **context,
            };
            callback(tree, &mut host)
        }

        pub(crate) fn WithScrollbarController<R>(
            &mut self,
            receiver: &mut LayerTreeImpl<B>,
            id: ElementId,
            callback: impl FnOnce(
                Option<&mut ScrollbarAnimationController>,
                &mut dyn ScrollbarAnimationControllerClient<
                    Transform = <B as LayerTreeTypes>::Transform,
                >,
            ) -> R,
        ) -> R {
            assert_eq!(self.active_tree_.identity(), Some(receiver.id));
            let key = ScrollbarControllerElementIdForTree(
                receiver,
                self.scrollbar_animation_controllers_,
                id,
            );
            let mut client = HostImplScrollbarClient {
                active_tree: receiver,
                settings: self.core.settings_,
                delegate: *self.core.delegate_,
                events: self.core.events_metrics_manager_,
                latency_monitors: self.core.latency_info_swap_promise_monitor_,
                context: self.context,
            };
            let controller = key
                .and_then(|key| self.scrollbar_animation_controllers_.get_mut(&key))
                .and_then(Option::as_deref_mut);
            callback(controller, &mut client)
        }
    }
    impl<B: HostImplTypes> LayerTreeScrollHost<B> for HostImplMutationFacade<'_, '_, B> {
        fn settings(&self) -> &LayerTreeSettings {
            self.core.settings_
        }
        fn IsCCSlimmingEnabled(&self) -> bool {
            self.context.services.IsCCSlimmingEnabled()
        }
        fn TreesInVizInClientProcess(&self) -> bool {
            self.core
                .settings_
                .TreesInVizInClientProcess(self.context.services.SettingsFeatures())
        }
        fn IsActiveTree(&self, id: LayerTreeId) -> bool {
            self.active_tree_.identity() == Some(id)
        }
        fn HasPendingTree(&self) -> bool {
            self.pending_tree_.identity().is_some()
        }
        fn HasRecycleTree(&self) -> bool {
            self.recycle_tree_.identity().is_some()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:6250-6255.
        fn RequestImplSideInvalidationForRasterInducingScroll(&mut self, id: ElementId) {
            self.core
                .pending_invalidation_raster_inducing_scrolls_
                .insert(id);
            self.context
                .services
                .SetNeedsImplSideInvalidation(*self.core.delegate_, true);
        }
        // Source: cc/trees/layer_tree_impl.cc:268-272.
        fn PendingTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool) {
            self.WithPendingTree(|tree, host| tree.DidUpdateScrollOffsetWithHost(id, pushed, host));
        }
        // Source: cc/trees/layer_tree_impl.cc:275-277.
        fn RecycleTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool) {
            self.WithRecycleTree(|tree, host| tree.DidUpdateScrollOffsetWithHost(id, pushed, host));
        }
        // Source: cc/trees/layer_tree_host_impl.cc:1587-1590.
        fn DidAnimateScrollOffset(&mut self) {
            let reason = self.context.services.BeginMainFrameReasonScroll();
            RequestCommitForHost(
                self.core.settings_,
                *self.core.delegate_,
                reason,
                self.context,
            );
            self.context
                .services
                .DelegateRenewTreePriority(*self.core.delegate_);
        }
    }
    impl<B: HostImplTypes> LayerTreeScaleHost<B> for HostImplMutationFacade<'_, '_, B> {
        fn ReportUnexpectedOopifPageScaleOnce(&mut self, scale: f32) {
            self.context
                .services
                .ReportUnexpectedOopifPageScaleOnce(scale)
        }
        // Source: cc/trees/layer_tree_impl.cc:1619-1622.
        fn PendingTreeDidUpdatePageScale(&mut self) {
            self.WithPendingTree(|tree, host| tree.DidUpdatePageScaleWithHost(host));
        }
        // Source: cc/trees/layer_tree_impl.cc:1623-1626.
        fn RecycleTreeDidUpdatePageScale(&mut self) {
            self.WithRecycleTree(|tree, host| tree.DidUpdatePageScaleWithHost(host));
        }
        fn OuterViewportScrollNode<'a>(
            &'a self,
            receiver: &'a LayerTreeImpl<B>,
        ) -> Option<&'a ScrollNode> {
            receiver.OuterViewportScrollNode()
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5466-5472.
        fn OnPageScaleUpdated(&mut self, receiver: &mut LayerTreeImpl<B>) {
            assert_eq!(self.active_tree_.identity(), Some(receiver.id));
            let mut client = HostImplScrollbarClient {
                active_tree: receiver,
                settings: self.core.settings_,
                delegate: *self.core.delegate_,
                events: self.core.events_metrics_manager_,
                latency_monitors: self.core.latency_info_swap_promise_monitor_,
                context: self.context,
            };
            OnPageScaleUpdatedForHost(
                self.core.settings_,
                self.scrollbar_animation_controllers_,
                self.core.flashed_scrollbars_,
                &mut client,
            );
        }
        fn WithScrollbarAnimationController(
            &mut self,
            receiver: &mut LayerTreeImpl<B>,
            id: ElementId,
            callback: &mut dyn FnMut(
                Option<&mut ScrollbarAnimationController>,
                &mut dyn ScrollbarAnimationControllerClient<
                    Transform = <B as LayerTreeTypes>::Transform,
                >,
            ),
        ) {
            self.WithScrollbarController(receiver, id, |controller, client| {
                callback(controller, client)
            })
        }
    }
}
pub(crate) use source_input_forwarding::HostImplMutationFacade;

mod source_input_mutation_bridges {
    use super::*;
    impl<'a, 'c, B: HostImplTypes> HostImplTreeHostFacade<'a, 'c, B> {
        pub(crate) fn MutationHost(&mut self) -> HostImplMutationFacade<'_, 'c, B> {
            HostImplMutationFacade {
                core: self.core.reborrow(),
                active_tree_: self.active_tree_.reborrow(),
                pending_tree_: self.pending_tree_.reborrow(),
                recycle_tree_: self.recycle_tree_.reborrow(),
                browser_controls_offset_manager_: self.browser_controls_offset_manager_.reborrow(),
                scrollbar_animation_controllers_: &mut *self.scrollbar_animation_controllers_,
                image_animation_controller_: &mut *self.image_animation_controller_,
                context: &mut *self.context,
            }
        }
    }
    impl<B: HostImplTypes> HostImplInputFacade<'_, B> {
        pub(crate) fn WithActiveMutation<R>(
            &mut self,
            callback: impl FnOnce(&mut LayerTreeImpl<B>, &mut HostImplMutationFacade<'_, '_, B>) -> R,
        ) -> R {
            let tree = self.active_tree_.as_mut().expect("source active tree");
            let id = tree.id;
            let mut host = HostImplMutationFacade {
                core: self.core.reborrow(),
                active_tree_: HostImplTreeSlot::Reserved(id),
                pending_tree_: self.pending_tree_.MutationSlot(),
                recycle_tree_: self.recycle_tree_.MutationSlot(),
                browser_controls_offset_manager_: self.browser_controls_offset_manager_.reborrow(),
                scrollbar_animation_controllers_: &mut *self.scrollbar_animation_controllers_,
                image_animation_controller_: &mut *self.image_animation_controller_,
                context: &mut self.context,
            };
            callback(tree, &mut host)
        }
        // Source: cc/trees/property_tree.cc:2371-2395; cc/input/viewport.cc:190-192.
        pub(crate) fn ScrollNodeByForViewport(&mut self, id: i32, delta: &Vector2dF) -> Vector2dF {
            self.WithActiveMutation(|tree, host| tree.ScrollNodeByWithHost(id, delta, host))
        }
        // Source: cc/trees/property_tree.cc:1946-1956; cc/input/viewport.cc:308-314.
        pub(crate) fn ClampScrollNodeForViewport(&mut self, id: i32) -> Vector2dF {
            self.WithActiveMutation(|tree, host| {
                tree.ClampScrollNodeToMaxScrollOffsetWithHost(id, host)
            })
        }
        // Source: cc/trees/layer_tree_impl.cc:1433-1450; cc/input/viewport.cc:326.
        pub(crate) fn SetPageScaleForViewport(&mut self, scale: f32) {
            self.WithActiveMutation(|tree, host| {
                tree.SetPageScaleOnActiveTreeWithHost(scale, host)
            });
        }
    }
}

impl<'a, 'c, B: HostImplTypes> HostImplTreeHostFacade<'a, 'c, B> {
    fn WithControlsReceiver<R>(
        &mut self,
        receiver: &mut LayerTreeImpl<B>,
        callback: impl FnOnce(
            &mut BrowserControlsOffsetManager,
            &mut HostImplBrowserControlsClient<'_, '_, B>,
        ) -> R,
    ) -> R {
        let active = self
            .active_tree_
            .ResolveMut(receiver)
            .expect("source nonnull active tree");
        let mut client = HostImplBrowserControlsClient {
            core: self.core.reborrow(),
            active_tree_: HostImplInputTree::Direct(active),
            pending_tree_: self.pending_tree_.RequireLiveTreeSlot(),
            recycle_tree_: self.recycle_tree_.RequireLiveTreeSlot(),
            scrollbar_animation_controllers_: &mut *self.scrollbar_animation_controllers_,
            image_animation_controller_: &mut *self.image_animation_controller_,
            context: &mut *self.context,
        };
        let manager = self
            .browser_controls_offset_manager_
            .as_mut()
            .expect("source browser controls manager");
        callback(manager, &mut client)
    }
}
impl<B: HostImplTypes> HostImplTreeSlot<'_, B> {
    // Borrow precondition of the native two-raw-pointer call chain: the current
    // receiver is supplied explicitly, and PushPropertiesTo lends the other
    // source via WithBorrowedTree before invoking a target callback. Remaining
    // roles therefore have their real Resident/Direct allocation available.
    // Reserved here is a violated scope contract, never an absent native tree.
    fn RequireLiveTreeSlot(&mut self) -> HostImplInputTree<'_, B> {
        match self {
            Self::Resident(slot) => HostImplInputTree::Slot(&mut **slot),
            Self::Direct(tree) => HostImplInputTree::Direct(&mut **tree),
            Self::Reserved(id) => panic!(
                "tree {:?} must be supplied through synchronous WithBorrowedTree before sibling callback",
                id
            ),
        }
    }
}

mod source_input_host {
    use crate::cc::input::input_handler::InputHandlerView;
    impl<B: HostImplTypes> HostImplInputFacade<'_, B> {
        pub(crate) fn WithViewport<R>(
            &mut self,
            input_view: Option<&InputHandlerView<'_>>,
            callback: impl FnOnce(
                &mut Viewport,
                &mut crate::adapter::viewport::ViewportContext<'_, B>,
            ) -> R,
        ) -> R {
            let viewport = self.viewport_.as_mut().expect("source viewport");
            let mut runtime = HostImplViewportRuntime {
                animation_reentry: self.animation_reentry.as_mut().map(|receiver| {
                    &mut **receiver
                        as &mut dyn crate::adapter::animation_host::ScrollAnimationReentry
                }),
                core: self.core.reborrow(),
                active_tree_: self.active_tree_.reborrow(),
                pending_tree_: self.pending_tree_.reborrow(),
                recycle_tree_: self.recycle_tree_.reborrow(),
                browser_controls_offset_manager_: self.browser_controls_offset_manager_.reborrow(),
                scrollbar_animation_controllers_: &mut *self.scrollbar_animation_controllers_,
                image_animation_controller_: &mut *self.image_animation_controller_,
                input_view: input_view.map(InputHandlerView::reborrow),
                context: &mut self.context,
            };
            let mut context = crate::adapter::viewport::ViewportContext {
                runtime: &mut runtime,
            };
            callback(viewport, &mut context)
        }
        fn WithScrollbarFlashControllers<R>(
            &mut self,
            callback: impl FnOnce(
                &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
                &mut HashSet<ElementId>,
                &mut BTreeSet<ElementId>,
                &mut dyn ScrollbarAnimationControllerClient<
                    Transform = <B as LayerTreeTypes>::Transform,
                >,
            ) -> R,
        ) -> R {
            let mut client = HostImplScrollbarClient {
                active_tree: self.active_tree_.as_mut().unwrap(),
                settings: self.core.settings_,
                delegate: *self.core.delegate_,
                events: self.core.events_metrics_manager_,
                latency_monitors: self.core.latency_info_swap_promise_monitor_,
                context: &mut self.context,
            };
            callback(
                self.scrollbar_animation_controllers_,
                self.core.flashed_scrollbars_,
                self.core.previously_visible_scrollable_elements_,
                &mut client,
            )
        }
        fn MaybeFlashAllScrollbars(&mut self, tracking: ElementId, did_scroll: bool) -> bool {
            let settings = self.core.settings_;
            self.WithScrollbarFlashControllers(|controllers, flashed, _previous, client| {
                MaybeFlashAllScrollbarsForHost(
                    settings,
                    controllers,
                    flashed,
                    tracking,
                    did_scroll,
                    client,
                )
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:6365-6438.
        pub(crate) fn MaybeFlashEnteredViewportScrollbars(
            &mut self,
            element_id: ElementId,
            scroll_delta: &Vector2dF,
        ) {
            if !self.core.settings_.scrollbar_flash_once_visible_on_viewport {
                return;
            }
            let last_scroll_delta = *self
                .core
                .accumulated_scroll_deltas_by_element_id_
                .entry(element_id)
                .or_default();
            let new_scroll_delta = last_scroll_delta + *scroll_delta;
            if last_scroll_delta.Length() > new_scroll_delta.Length() {
                self.core
                    .accumulated_scroll_deltas_by_element_id_
                    .insert(element_id, *scroll_delta);
            } else {
                self.core
                    .accumulated_scroll_deltas_by_element_id_
                    .insert(element_id, new_scroll_delta);
            }
            let accumulated = self.core.accumulated_scroll_deltas_by_element_id_[&element_id];
            if accumulated.x().abs() < kScrollDeltaThreshold
                && accumulated.y().abs() < kScrollDeltaThreshold
            {
                return;
            }
            self.core
                .accumulated_scroll_deltas_by_element_id_
                .insert(element_id, Vector2dF::default());
            let visible_viewport_rect = self
                .active_tree_
                .as_ref()
                .unwrap()
                .GetDeviceViewportWithExternal(*self.core.external_viewport_);
            self.WithScrollbarFlashControllers(
                |controllers, _flashed, previous_visible, client| {
                    for (&scroll_element_id, controller) in controllers.iter_mut() {
                        if scroll_element_id == element_id {
                            continue;
                        }
                        let scrollbars = client.ScrollbarsFor(scroll_element_id);
                        if scrollbars.is_empty() {
                            continue;
                        }
                        let mut combined_bounds_in_screen = Rect::default();
                        for scrollbar_layer in scrollbars {
                            let transform = client.Scrollbar_ScreenSpaceTransform(&scrollbar_layer);
                            let bounds = client.Scrollbar_bounds(&scrollbar_layer);
                            let scrollbar_bounds_in_screen = MathUtil::MapEnclosingClippedRect(
                                &transform,
                                Rect::new(0, 0, bounds.width(), bounds.height()),
                            );
                            if combined_bounds_in_screen.IsEmpty() {
                                combined_bounds_in_screen = scrollbar_bounds_in_screen;
                            } else {
                                combined_bounds_in_screen.Union(scrollbar_bounds_in_screen);
                            }
                        }
                        let was_visible = previous_visible.contains(&scroll_element_id);
                        let mut intersection = visible_viewport_rect;
                        intersection.Intersect(combined_bounds_in_screen);
                        // Source multiplies integer dimensions before its float comparison.
                        let is_visible = (intersection.width * intersection.height) as f32
                            >= (combined_bounds_in_screen.width * combined_bounds_in_screen.height)
                                as f32
                                * kIntersectionThresholdAreaPercentage;
                        if is_visible && !was_visible {
                            controller
                                .as_mut()
                                .expect("source scrollbar controller dereference")
                                .DidScrollUpdate(client);
                            previous_visible.insert(scroll_element_id);
                        } else if !is_visible && was_visible {
                            previous_visible.remove(&scroll_element_id);
                        }
                    }
                },
            );
        }
    }

    use super::*;
    use crate::adapter::geometry::{PointF, Rect, SizeF, Vector2dF};
    use crate::adapter::input_handler::*;
    use crate::adapter::layer_tree_impl::{LayerTreeId, LayerTreeLayer, LayerTreeScrollbar};
    use crate::cc::input::{
        browser_controls_offset_tag_modifications::BrowserControlsOffsetTagModifications,
        browser_controls_state::BrowserControlsState,
        event_listener_properties::{EventListenerClass, EventListenerProperties},
        overscroll_behavior::OverscrollBehavior,
        scroll_state::ScrollState,
    };
    impl<B: HostImplTypes> HostImplInputFacade<'_, B> {
        fn InputActiveTree(&self) -> &LayerTreeImpl<B> {
            self.active_tree_
                .as_ref()
                .expect("source active_tree is nonnull")
        }
        fn InputActiveTreeMut(&mut self) -> &mut LayerTreeImpl<B> {
            self.active_tree_
                .as_mut()
                .expect("source active_tree is nonnull")
        }
        fn InputLayer(&self, layer: &InputLayerId) -> &B::Layer {
            let tree = self.InputActiveTree();
            assert_eq!(
                layer.tree, tree.id,
                "input layer belongs to current active tree"
            );
            tree.LayerById(layer.id)
                .expect("source live LayerImpl pointer")
        }
        fn AsWeakPtr(&self) -> HostImplWeak {
            HostImplWeak {
                owner: *self.core.owner_id_,
                alive: Arc::downgrade(&*self.core.weak_factory_),
            }
        }
        pub(crate) fn SetNeedsOneBeginImplFrame(&mut self) {
            let promises = RequestBeginImplFrameForHost(
                *self.core.delegate_,
                self.core.latency_info_swap_promise_monitor_,
                self.core.events_metrics_manager_,
                &mut self.context,
            );
            QueuePinnedLatencyPromisesOnActiveTree(self.InputActiveTreeMut(), promises);
        }
        fn RenewTreePriority(&mut self) {
            self.context
                .services
                .DelegateRenewTreePriority(*self.core.delegate_)
        }
        fn SetNeedsRedraw(&mut self, animation_only: bool, skip: bool) {
            let promises = RequestRedrawForHost(
                self.core.settings_,
                *self.core.delegate_,
                self.core.latency_info_swap_promise_monitor_,
                self.core.events_metrics_manager_,
                animation_only,
                skip,
                &mut self.context,
            );
            QueuePinnedLatencyPromisesOnActiveTree(self.InputActiveTreeMut(), promises);
        }
        fn WithBrowserControls<R>(
            &mut self,
            callback: impl FnOnce(
                &mut BrowserControlsOffsetManager,
                &mut dyn BrowserControlsOffsetManagerClient<
                    BeginMainFrameReason = <B as InputHostTypes>::BeginMainFrameReason,
                >,
            ) -> R,
        ) -> R {
            let manager = self
                .browser_controls_offset_manager_
                .as_mut()
                .expect("source browser controls manager");
            let mut client = HostImplBrowserControlsClient {
                core: self.core.reborrow(),
                active_tree_: self.active_tree_.reborrow(),
                pending_tree_: self.pending_tree_.reborrow(),
                recycle_tree_: self.recycle_tree_.reborrow(),
                scrollbar_animation_controllers_: &mut *self.scrollbar_animation_controllers_,
                image_animation_controller_: &mut *self.image_animation_controller_,
                context: &mut self.context,
            };
            callback(manager, &mut client)
        }
        fn WithScrollbarAnimationController<R>(
            &mut self,
            id: ElementId,
            callback: impl FnOnce(
                Option<&mut ScrollbarAnimationController>,
                &mut dyn ScrollbarAnimationControllerClient<
                    Transform = <B as LayerTreeTypes>::Transform,
                >,
            ) -> R,
        ) -> R {
            self.WithActiveMutation(|tree, host| host.WithScrollbarController(tree, id, callback))
        }
        fn WithScrollbarAnimationControllers<R>(
            &mut self,
            callback: impl FnOnce(
                &mut HashMap<ElementId, Option<Box<ScrollbarAnimationController>>>,
                &mut dyn ScrollbarAnimationControllerClient<
                    Transform = <B as LayerTreeTypes>::Transform,
                >,
            ) -> R,
        ) -> R {
            let mut client = HostImplScrollbarClient {
                active_tree: self.active_tree_.as_mut().unwrap(),
                settings: self.core.settings_,
                delegate: *self.core.delegate_,
                events: self.core.events_metrics_manager_,
                latency_monitors: self.core.latency_info_swap_promise_monitor_,
                context: &mut self.context,
            };
            callback(self.scrollbar_animation_controllers_, &mut client)
        }
    }
    impl<B: HostImplTypes> InputHost<B> for HostImplInputFacade<'_, B> {
        fn WithScrollbarHost<R>(
            &mut self,
            view: &InputHandlerView<'_>,
            last_scroll_update_state: Option<&ScrollState>,
            callback: impl FnOnce(&mut InputScrollbarHost<'_, B>) -> R,
        ) -> R {
            self.WithScrollbarHostForInput(view, last_scroll_update_state, callback)
        }
        fn StretchAmount(&self, helper: &B::Elasticity, id: ElementId) -> Vector2dF {
            self.StretchAmountForInput(helper, id)
        }
        fn CreateScrollElasticityHelper(&mut self) -> B::Elasticity {
            self.CreateScrollElasticityHelperForInput()
        }
        fn ResetStretchAmounts(&mut self, helper: &mut B::Elasticity) {
            self.ResetStretchAmountsForInput(helper)
        }
        fn ApplyStretchAmountsToPending(&mut self, helper: &mut B::Elasticity) {
            self.ApplyStretchAmountsToPendingForInput(helper)
        }
        fn ApplyStretchAmountsToActive(&mut self, helper: &mut B::Elasticity) {
            self.ApplyStretchAmountsToActiveForInput(helper)
        }

        // Source: cc/input/input_handler.cc:2501-2502; cc/trees/layer_tree_impl.cc:3040-3063.
        fn ActiveTree_FindFrameElementIdAtPoint(
            &mut self,
            view: &InputHandlerView<'_>,
            point: PointF,
        ) -> ElementId {
            self.WithActiveMutation(|tree, host| {
                let mut draw_host = HostImplDrawFacade { host, view };
                tree.FindFrameElementIdAtPointWithHost(&point, &mut draw_host)
            })
        }
        // Source: cc/trees/layer_tree_impl.cc:2783-2818. Only the native nonowning
        // layer identity leaves this synchronous resident-tree borrow.
        fn ActiveTree_FindLayerThatIsHitByPointInWheelEventHandlerRegion(
            &mut self,
            view: &InputHandlerView<'_>,
            point: PointF,
        ) -> Option<InputLayerId> {
            self.WithActiveMutation(|tree, host| {
                let mut draw_host = HostImplDrawFacade { host, view };
                let id = tree.id;
                tree.FindLayerThatIsHitByPointInWheelEventHandlerRegionWithHost(
                    &point,
                    &mut draw_host,
                )
                .map(|layer| InputLayerId {
                    tree: id,
                    id: <B::Layer as crate::adapter::layer_tree_impl::LayerTreeLayer<B>>::id(layer),
                })
            })
        }
        fn ActiveTree_FindLayerThatIsHitByPointInTouchHandlerRegion(
            &mut self,
            view: &InputHandlerView<'_>,
            rect: RectF,
        ) -> Option<InputLayerId> {
            self.WithActiveMutation(|tree, host| {
                let mut draw_host = HostImplDrawFacade { host, view };
                let id = tree.id;
                tree.FindLayerThatIsHitByPointInTouchHandlerRegionWithHost(&rect, &mut draw_host)
                    .map(|layer| InputLayerId {
                        tree: id,
                        id: <B::Layer as crate::adapter::layer_tree_impl::LayerTreeLayer<B>>::id(
                            layer,
                        ),
                    })
            })
        }
        // Source: cc/trees/layer_tree_impl.cc:2747-2781.
        fn ActiveTree_FindLayerThatIsHitByPoint(
            &mut self,
            view: &InputHandlerView<'_>,
            point: PointF,
        ) -> Option<InputLayerId> {
            self.WithActiveMutation(|tree, host| {
                let mut draw_host = HostImplDrawFacade { host, view };
                let id = tree.id;
                tree.FindLayerThatIsHitByPointWithHost(&point, &mut draw_host)
                    .map(|layer| InputLayerId {
                        tree: id,
                        id: <B::Layer as crate::adapter::layer_tree_impl::LayerTreeLayer<B>>::id(
                            layer,
                        ),
                    })
            })
        }
        // Source: cc/trees/layer_tree_impl.cc:2820-2899. This Vec is the source's
        // ordered hit-test output, containing only stable nonowning layer IDs.
        fn ActiveTree_FindLayersUpToFirstScrollableOrOpaqueToHitTest(
            &mut self,
            view: &InputHandlerView<'_>,
            point: PointF,
        ) -> Vec<InputLayerId> {
            self.WithActiveMutation(|tree, host| {
                let mut draw_host = HostImplDrawFacade { host, view };
                let id = tree.id;
                tree.FindLayersUpToFirstScrollableOrOpaqueToHitTestWithHost(&point, &mut draw_host)
                    .into_iter()
                    .map(|layer| InputLayerId {
                        tree: id,
                        id: <B::Layer as crate::adapter::layer_tree_impl::LayerTreeLayer<B>>::id(
                            layer,
                        ),
                    })
                    .collect()
            })
        }

        // Source: cc/trees/layer_tree_host_impl.cc:4791-4823.
        fn ScrollAnimationCreate(
            &mut self,
            scroll_node: &InputScrollNodeId,
            delta: &Vector2dF,
            delayed_by: TimeDelta,
        ) -> bool {
            let tree = self.active_tree_.as_ref().unwrap();
            assert_eq!(tree.id, scroll_node.tree);
            let kEpsilon = 0.1_f32;
            let scroll_animated = delta.x().abs() > kEpsilon || delta.y().abs() > kEpsilon;
            if !scroll_animated {
                self.WithActiveMutation(|tree, host| {
                    tree.ScrollNodeByWithHost(scroll_node.index, delta, host)
                });
                self.context
                    .services
                    .TraceHostImplGEvent("cc", "no scroll animation due to small delta");
                return false;
            }
            let tree = self.active_tree_.as_ref().unwrap();
            let trees = tree.property_trees();
            let scroll_tree = trees.scroll_tree();
            let node = scroll_tree.Node(scroll_node.index);
            let current_offset = scroll_tree.current_scroll_offset(node.element_id, trees.state());
            let target_offset = scroll_tree.ClampScrollOffsetToLimits(
                current_offset + *delta,
                node,
                trees.transform_tree(),
                trees.state(),
            );
            let animation_start_offset = self.core.current_begin_frame_tracker_.Current().interval;
            if let Some(receiver) = self
                .animation_reentry
                .as_mut()
                .filter(|r| r.MatchesElement(node.element_id))
            {
                receiver.CreateMouseWheelScrollAnimation(
                    node.element_id,
                    target_offset,
                    current_offset,
                    delayed_by,
                    animation_start_offset,
                );
            } else {
                self.context.services.MutatorImplOnlyScrollAnimationCreate(
                    self.core.mutator_host_,
                    node.element_id,
                    target_offset,
                    current_offset,
                    delayed_by,
                    animation_start_offset,
                );
            }
            self.SetNeedsOneBeginImplFrame();
            true
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4844-4862.
        fn WillScrollContent(&mut self, element_id: ElementId) {
            if !self.MaybeFlashAllScrollbars(element_id, false) {
                self.WithScrollbarAnimationController(element_id, |controller, client| {
                    if let Some(controller) = controller {
                        controller.WillUpdateScroll(client);
                    }
                });
            }
            self.context
                .services
                .SaveActiveEventMetrics(self.core.events_metrics_manager_);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4864-4970.
        fn DidScrollContent(
            &mut self,
            view: &crate::cc::input::input_handler::InputHandlerView<'_>,
            element_id: ElementId,
            animated: bool,
            scroll_delta: &Vector2dF,
        ) {
            if self.core.settings_.enable_scroll_performance_timing
                && !animated
                && element_id != ElementId::default()
                && !scroll_delta.IsZero()
            {
                self.context
                    .services
                    .RecordAppliedScrollObservation(self.core.events_metrics_manager_, element_id);
            }
            *self.core.scroll_accumulated_this_frame_ += *scroll_delta;
            *self.core.frame_max_scroll_delta_ =
                crate::adapter::geometry::cpp_max(scroll_delta.x().abs(), scroll_delta.y().abs());
            if !self.MaybeFlashAllScrollbars(element_id, true) {
                self.WithScrollbarAnimationController(element_id, |controller, client| {
                    if let Some(controller) = controller {
                        controller.DidScrollUpdate(client);
                    }
                });
            }
            self.MaybeFlashEnteredViewportScrollbars(element_id, scroll_delta);
            let is_currently_scrolling = view
                .IsCurrentlyScrollingForTree(self.InputActiveTree().property_trees().scroll_tree());
            if is_currently_scrolling {
                if !self.core.settings_.single_thread_proxy_scheduler {
                    self.context
                        .services
                        .DelegateSetWaitingForScrollEvent(*self.core.delegate_, false);
                }
                if self
                    .context
                    .services
                    .NewContentForCheckerboardedScrollsEnabled()
                    && self
                        .context
                        .services
                        .NewContentForCheckerboardedScrollsPerFrame()
                {
                    *self.core.prioritize_new_content_due_to_checkerboarding_ = false;
                    let tree = self.active_tree_.as_ref().unwrap();
                    let trees = tree.property_trees();
                    let scroll_tree = trees.scroll_tree();
                    if let Some(cull_rect) = scroll_tree.ScrollingContentsCullRect(element_id) {
                        if let Some(scroll_node) = scroll_tree.FindNodeFromElementId(element_id) {
                            if scroll_node.transform_id != kInvalidPropertyNodeId {
                                let bounds =
                                    scroll_tree.container_bounds(scroll_node.id, trees.state());
                                let mut visible_rect =
                                    crate::adapter::geometry::RectF::from(Rect::new(
                                        scroll_node.container_origin.x(),
                                        scroll_node.container_origin.y(),
                                        bounds.width(),
                                        bounds.height(),
                                    ));
                                let offset = scroll_tree
                                    .current_scroll_offset(element_id, trees.state())
                                    .OffsetFromOrigin();
                                visible_rect.Offset(offset.x(), offset.y());
                                let transform_to_screen =
                                    trees.transform_tree().ToScreen(scroll_node.transform_id);
                                visible_rect =
                                    MathUtil::MapClippedRect(transform_to_screen, visible_rect);
                                visible_rect.Intersect(
                                    HostScrollRenderSurface::DrawableContentRect(
                                        tree.RootRenderSurface().unwrap(),
                                        tree.property_trees(),
                                    ),
                                );
                                let to_screen_cull_rect = MathUtil::MapClippedRect(
                                    transform_to_screen,
                                    crate::adapter::geometry::RectF::from(*cull_rect),
                                );
                                if !to_screen_cull_rect.Contains(visible_rect) {
                                    *self.core.prioritize_new_content_due_to_checkerboarding_ =
                                        true;
                                }
                            }
                        }
                    }
                }
                self.RenewTreePriority();
            }
            if !animated {
                self.SetNeedsRedraw(false, false);
            }
        }
        fn LayerGetAllowedTouchAction(
            &mut self,
            layer: &InputLayerId,
            rect: Rect,
        ) -> crate::cc::input::touch_action::TouchAction {
            self.InputLayer(layer).GetAllowedTouchAction(rect)
        }
        fn LayerScrollTreeIndex(&mut self, layer: &InputLayerId) -> i32 {
            crate::adapter::tree_sync::SyncImplLayer::<B>::scroll_tree_index(self.InputLayer(layer))
        }
        // Source: cc/trees/property_tree.cc:2073-2100.
        fn CollectScrollDeltas(
            &mut self,
            data: &mut CompositorCommitData,
            inner: ElementId,
            fractional: bool,
            snapped: BTreeMap<
                ElementId,
                crate::cc::input::scroll_snap_data::TargetSnapAreaElementIds,
            >,
            mutator: Option<&<B as InputHostTypes>::MutatorHost>,
        ) {
            let trees = self.InputActiveTree().property_trees();
            trees.scroll_tree().CollectScrollDeltas(
                data,
                inner,
                fractional,
                &snapped,
                mutator.map(|m| m as &dyn crate::adapter::property_tree::PropertyTreeMutatorHost),
                trees.state(),
            );
        }
        // Source: cc/layers/viewport.cc:41-80.
        fn ViewportScrollBy(
            &mut self,
            view: &InputHandlerView<'_>,
            delta: Vector2dF,
            point: Point,
            direct: bool,
            controls: bool,
            outer: bool,
            inertial: bool,
        ) -> crate::cc::input::input_handler::ViewportScrollResult {
            self.WithViewport(Some(view), |viewport, c| {
                viewport.ScrollBy(delta, point, direct, controls, outer, inertial, c)
            })
        }
        // Source: cc/layers/viewport.cc:199-225.
        fn ViewportScrollAnimated(
            &mut self,
            view: &InputHandlerView<'_>,
            delta: Vector2dF,
            delayed_by: TimeDelta,
        ) -> crate::cc::input::input_handler::ViewportScrollResult {
            self.WithViewport(Some(view), |viewport, c| {
                viewport.ScrollAnimated(delta, delayed_by, c)
            })
        }
        fn ViewportPinchUpdate(
            &mut self,
            view: &InputHandlerView<'_>,
            magnify: f32,
            anchor: Point,
        ) {
            self.WithViewport(Some(view), |viewport, c| {
                viewport.PinchUpdate(magnify, anchor, c)
            });
        }
        fn ViewportPinchEnd(
            &mut self,
            view: &InputHandlerView<'_>,
            anchor: Point,
            snap_to_min: bool,
        ) {
            self.WithViewport(Some(view), |viewport, c| {
                viewport.PinchEnd(anchor, snap_to_min, c)
            });
        }
        fn Viewport_ShouldScroll(&mut self, node: &InputScrollNodeId) -> bool {
            self.WithViewport(None, |viewport, c| viewport.ShouldScroll(node, c))
        }
        fn Viewport_CanScroll(
            &mut self,
            view: &InputHandlerView<'_>,
            node: &InputScrollNodeId,
            state: &ScrollState,
        ) -> bool {
            self.WithViewport(Some(view), |viewport, c| viewport.CanScroll(node, state, c))
        }
        fn Viewport_TotalScrollOffset(&mut self) -> PointF {
            Viewport::TotalScrollOffsetForTree(self.InputActiveTree())
        }

        // Source: cc/trees/layer_tree_impl.h:138; cc/trees/property_tree.h:70-73.
        fn ActiveTreeId(&mut self) -> LayerTreeId {
            self.InputActiveTree().id
        }
        // Source: cc/trees/property_tree.h:87; cc/trees/property_tree.h:103; cc/trees/property_tree.h:127.
        fn ReadScrollNode<R>(
            &mut self,
            node: &InputScrollNodeId,
            read: impl FnOnce(&ScrollNode) -> R,
        ) -> R {
            let tree = self.InputActiveTree();
            assert_eq!(
                node.tree, tree.id,
                "input node belongs to current active tree"
            );
            read(tree.property_trees().scroll_tree().Node(node.index))
        }
        // Source: cc/trees/property_tree.h:87; cc/trees/property_tree.h:103; cc/trees/property_tree.h:127.
        fn WriteScrollNode<R>(
            &mut self,
            node: &InputScrollNodeId,
            write: impl FnOnce(&mut ScrollNode) -> R,
        ) -> R {
            let tree = self.InputActiveTreeMut();
            assert_eq!(
                node.tree, tree.id,
                "input node belongs to current active tree"
            );
            write(
                tree.property_trees_mut()
                    .scroll_tree_mutable()
                    .MutableNode(node.index),
            )
        }
        // Source: cc/layers/layer_impl.h:182.
        fn LayerOpaqueToHitTest(&mut self, layer: &InputLayerId) -> bool {
            self.InputLayer(layer)
                .OpaqueToHitTest(self.InputActiveTree().property_trees())
        }
        // Source: cc/layers/layer_impl.h:173.
        fn LayerIsScrollerOrScrollbar(&mut self, layer: &InputLayerId) -> bool {
            self.InputLayer(layer)
                .IsScrollerOrScrollbar(self.InputActiveTree().property_trees())
        }

        fn LayerBelongsToActiveTree(&mut self, layer: &InputLayerId) -> bool {
            layer.tree == self.InputActiveTree().id
        }
        // Source: cc/layers/layer_impl.h:171.
        fn LayerIsScrollbarLayer(&mut self, layer: &InputLayerId) -> bool {
            self.InputLayer(layer).AsScrollbarLayer().is_some()
        }

        fn LayerScrollbarScrollElementId(&mut self, layer: &InputLayerId) -> ElementId {
            self.InputLayer(layer)
                .AsScrollbarLayer()
                .expect("source ToScrollbarLayer")
                .scroll_element_id()
        }
        // Source: cc/trees/layer_tree_impl.h:341; cc/trees/layer_tree_impl.h:342; cc/trees/layer_tree_impl.h:344.
        fn ActiveTree_InnerViewportScrollNode(&mut self) -> Option<InputScrollNodeId> {
            let tree = self.InputActiveTree();
            tree.InnerViewportScrollNode()
                .map(|node| InputScrollNodeId {
                    tree: tree.id,
                    index: node.id,
                })
        }
        // Source: cc/trees/layer_tree_impl.h:351; cc/trees/layer_tree_impl.h:352; cc/trees/layer_tree_impl.h:354.
        fn ActiveTree_OuterViewportScrollNode(&mut self) -> Option<InputScrollNodeId> {
            let tree = self.InputActiveTree();
            tree.OuterViewportScrollNode()
                .map(|node| InputScrollNodeId {
                    tree: tree.id,
                    index: node.id,
                })
        }
        // Source: cc/trees/layer_tree_impl.h:402; cc/trees/property_tree.h:250.
        fn ActiveTree_device_scale_factor(&mut self) -> f32 {
            self.InputActiveTree().device_scale_factor()
        }
        // Source: cc/trees/layer_tree_impl.h:407.
        fn ActiveTree_painted_device_scale_factor(&mut self) -> f32 {
            self.InputActiveTree().painted_device_scale_factor()
        }
        // Source: cc/trees/layer_tree_impl.h:454; cc/trees/property_tree.h:257; cc/trees/property_tree.h:401.
        fn ActiveTree_external_page_scale_factor(&mut self) -> f32 {
            self.InputActiveTree().external_page_scale_factor()
        }
        // Source: cc/trees/layer_tree_impl.h:393.
        fn ActiveTree_min_page_scale_factor(&mut self) -> f32 {
            self.InputActiveTree().min_page_scale_factor()
        }
        // Source: cc/trees/layer_tree_impl.h:394.
        fn ActiveTree_max_page_scale_factor(&mut self) -> f32 {
            self.InputActiveTree().max_page_scale_factor()
        }
        // Source: cc/trees/layer_tree_impl.h:709.
        fn ActiveTree_overscroll_behavior(&mut self) -> OverscrollBehavior {
            self.InputActiveTree().overscroll_behavior()
        }
        // Source: cc/trees/layer_tree_impl.h:309; cc/trees/layer_tree_impl.h:310.
        fn ActiveTree_TotalScrollOffset(&mut self) -> PointF {
            self.InputActiveTree().TotalScrollOffset()
        }
        // Source: cc/trees/layer_tree_impl.h:311; cc/trees/layer_tree_impl.h:312.
        fn ActiveTree_TotalMaxScrollOffset(&mut self) -> PointF {
            self.InputActiveTree().TotalMaxScrollOffset()
        }
        // Source: cc/trees/layer_tree_impl.h:537.
        fn ActiveTree_ScrollableSize(&mut self) -> SizeF {
            self.InputActiveTree().ScrollableSize()
        }
        // Source: cc/trees/layer_tree_impl.h:193.
        fn ActiveTree_LayerListIsEmpty(&mut self) -> bool {
            self.InputActiveTree().LayerListIsEmpty()
        }
        // Source: cc/trees/layer_tree_impl.h:390; cc/trees/layer_tree_impl.h:465; cc/trees/layer_tree_impl.h:467.
        fn ActiveTree_current_page_scale_factor(&mut self) -> f32 {
            self.InputActiveTree()
                .current_page_scale_factor_for_tree(true)
        }
        // Source: cc/trees/layer_tree_impl.h:463.
        fn ActiveTree_page_scale_factor_for_scroll(&mut self) -> f32 {
            self.InputActiveTree()
                .page_scale_factor_for_scroll_for_tree(
                    true,
                    self.core.settings_.is_for_scalable_page,
                )
        }
        // Source: cc/trees/layer_tree_impl.h:741.
        fn ActiveTree_event_listener_properties(
            &mut self,
            class: EventListenerClass,
        ) -> EventListenerProperties {
            self.InputActiveTree().event_listener_properties(class)
        }
        // Source: cc/trees/layer_tree_impl.h:649.
        fn ActiveTree_PointHitsMainThreadScrollHitTestRegion(
            &mut self,
            point: PointF,
            layer: &InputLayerId,
        ) -> bool {
            self.InputActiveTree()
                .PointHitsMainThreadScrollHitTestRegion(&point, self.InputLayer(layer))
        }
        // Source: cc/trees/layer_tree_impl.h:652.
        fn ActiveTree_PointHitsNonCompositedScroll(
            &mut self,
            point: PointF,
            layer: &InputLayerId,
        ) -> ElementId {
            self.InputActiveTree()
                .PointHitsNonCompositedScroll(&point, self.InputLayer(layer))
        }
        // Source: cc/trees/layer_tree_impl.h:363; cc/trees/layer_tree_impl.h:364; cc/trees/property_tree.h:620; cc/trees/property_tree.h:621.
        fn ScrollTree_CurrentlyScrollingNode(&mut self) -> Option<InputScrollNodeId> {
            let tree = self.InputActiveTree();
            tree.property_trees()
                .scroll_tree()
                .CurrentlyScrollingNode()
                .map(|node| InputScrollNodeId {
                    tree: tree.id,
                    index: node.id,
                })
        }
        // Source: cc/trees/property_tree.h:113.
        fn ScrollTree_MutableFindNodeFromElementId(
            &mut self,
            id: ElementId,
        ) -> Option<InputScrollNodeId> {
            let tree = self.InputActiveTreeMut();
            let tree_id = tree.id;
            tree.property_trees_mut()
                .scroll_tree_mutable()
                .MutableFindNodeFromElementId(id)
                .map(|node| InputScrollNodeId {
                    tree: tree_id,
                    index: node.id,
                })
        }
        // Source: cc/trees/property_tree.h:93; cc/trees/property_tree.h:105; cc/trees/property_tree.h:119.
        fn ScrollTree_MutableNode(&mut self, id: i32) -> InputScrollNodeId {
            let tree = self.InputActiveTreeMut();
            let tree_id = tree.id;
            let node = tree
                .property_trees_mut()
                .scroll_tree_mutable()
                .MutableNode(id);
            InputScrollNodeId {
                tree: tree_id,
                index: node.id,
            }
        }
        // Source: cc/trees/property_tree.h:99.
        fn ScrollTree_HasParent(&mut self, node: &InputScrollNodeId) -> bool {
            let tree = self.InputActiveTree();
            assert_eq!(node.tree, tree.id);
            let scroll = tree.property_trees().scroll_tree();
            scroll.HasParent(scroll.Node(node.index))
        }
        // Source: cc/trees/property_tree.h:105.
        fn ScrollTree_MutableParent(&mut self, node: &InputScrollNodeId) -> InputScrollNodeId {
            let tree = self.InputActiveTreeMut();
            assert_eq!(node.tree, tree.id);
            let tree_id = tree.id;
            let scroll = tree.property_trees_mut().scroll_tree_mutable();
            // Both source accessors validate the resident parent, without cloning it.
            let parent_id = scroll.Node(node.index).parent_id;
            let parent = scroll.MutableNode(parent_id);
            InputScrollNodeId {
                tree: tree_id,
                index: parent.id,
            }
        }
        // Source: cc/trees/property_tree.h:741.
        fn ScrollTree_CanRealizeScrollsOnPendingTree(&mut self, node: &InputScrollNodeId) -> bool {
            let tree = self.InputActiveTree();
            assert_eq!(node.tree, tree.id);
            let scroll = tree.property_trees().scroll_tree();
            scroll.CanRealizeScrollsOnPendingTree(scroll.Node(node.index))
        }
        // Source: cc/trees/property_tree.h:742.
        fn ScrollTree_ShouldRealizeScrollsOnMain(&mut self, node: &InputScrollNodeId) -> bool {
            let tree = self.InputActiveTree();
            assert_eq!(node.tree, tree.id);
            let scroll = tree.property_trees().scroll_tree();
            scroll.ShouldRealizeScrollsOnMain(scroll.Node(node.index))
        }
        // Source: cc/trees/property_tree.h:635.
        fn ScrollTree_current_scroll_offset(&mut self, id: ElementId) -> PointF {
            let (scroll, _, state) = self
                .InputActiveTree()
                .property_trees()
                .scroll_tree_read_context();
            scroll.current_scroll_offset(id, state)
        }
        // Source: cc/trees/property_tree.h:714.
        fn ScrollTree_ClampScrollOffsetToLimits(
            &mut self,
            offset: PointF,
            node: &InputScrollNodeId,
        ) -> PointF {
            let tree = self.InputActiveTree();
            assert_eq!(node.tree, tree.id);
            let (scroll, transform, state) = tree.property_trees().scroll_tree_read_context();
            scroll.ClampScrollOffsetToLimits(offset, scroll.Node(node.index), transform, state)
        }
        // Source: cc/trees/property_tree.h:961; cc/trees/property_tree.h:963.
        fn ScrollTree_outer_viewport_container_bounds_delta(&mut self) -> Vector2dF {
            self.InputActiveTree()
                .property_trees()
                .outer_viewport_container_bounds_delta()
        }
        // Source: cc/trees/layer_tree_impl.h:365.
        fn SetCurrentlyScrollingNode(&mut self, node: InputScrollNodeId) {
            let tree = self.InputActiveTreeMut();
            assert_eq!(node.tree, tree.id);
            tree.property_trees().scroll_tree().Node(node.index);
            tree.property_trees_mut()
                .scroll_tree_mutable()
                .set_currently_scrolling_node(node.index)
        }
        // Source: cc/trees/layer_tree_impl.h:366.
        fn ClearCurrentlyScrollingNode(&mut self) {
            self.InputActiveTreeMut()
                .property_trees_mut()
                .scroll_tree_mutable()
                .set_currently_scrolling_node(
                    crate::cc::trees::property_ids::kInvalidPropertyNodeId,
                )
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5029-5031.
        fn GetBrowserControlsTopOffset(&self) -> f32 {
            let client = HostImplBrowserControlsReadClient {
                active_tree: self.InputActiveTree(),
                frame_tracker: self.core.current_begin_frame_tracker_,
                services: self.context.services,
            };
            self.browser_controls_offset_manager_
                .as_ref()
                .expect("source browser_controls manager")
                .ControlsTopOffset(&client)
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5105-5110.
        fn PredictViewportBoundsDelta(&self, current: f64, distance: Vector2dF) -> f64 {
            let client = HostImplBrowserControlsReadClient {
                active_tree: self.InputActiveTree(),
                frame_tracker: self.core.current_begin_frame_tracker_,
                services: self.context.services,
            };
            self.browser_controls_offset_manager_
                .as_ref()
                .expect("source browser_controls manager")
                .PredictViewportBoundsDelta(&client, current, distance)
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5112-5115.
        fn ElementHasImplOnlyScrollAnimation(&self, id: ElementId) -> bool {
            if let Some(receiver) = self
                .animation_reentry
                .as_ref()
                .filter(|r| r.MatchesElement(id))
            {
                return receiver.ElementHasImplOnlyScrollAnimation(id);
            }
            self.context
                .services
                .MutatorElementHasImplOnlyScrollAnimation(self.core.mutator_host_, id)
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5128-5130.
        fn HasAnimatedScrollbars(&self) -> bool {
            !self.scrollbar_animation_controllers_.is_empty()
        }
        // Source: cc/trees/layer_tree_host_impl.h:580.
        fn GetSettings(
            &self,
        ) -> &crate::cc::trees::layer_tree_settings::LayerTreeSettings {
            self.core.settings_
        }
        // Source: cc/trees/layer_tree_host_impl.cc:562-564.
        fn IsInHighLatencyMode(&self) -> bool {
            *self.core.impl_thread_phase_ == ImplThreadPhase::IDLE
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5025-5027.
        fn ScrollAnimationAbort(&mut self, element_id: ElementId) {
            if let Some(receiver) = self
                .animation_reentry
                .as_mut()
                .filter(|r| r.MatchesElement(element_id))
            {
                receiver.AbortScrollAnimation(element_id, false);
                return;
            }
            self.context
                .services
                .MutatorScrollAnimationAbort(self.core.mutator_host_, element_id);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5033-5035.
        fn ScrollBegin(&mut self) {
            self.WithBrowserControls(|manager, client| manager.ScrollBegin(client));
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5037-5040.
        fn ScrollEnd(&mut self, delta: &Vector2dF) {
            self.WithBrowserControls(|manager, client| manager.ScrollEnd(client, delta));
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5042-5046.
        fn StartScrollSequence(
            &mut self,
            sequence: B::FrameSequenceTrackerType,
            thread: B::SmoothEffectDrivingThread,
        ) {
            self.context.services.FrameTrackersStartScrollSequence(
                self.core.frame_trackers_,
                sequence,
                thread,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5048-5050.
        fn StopSequence(&mut self, sequence: B::FrameSequenceTrackerType) {
            self.context
                .services
                .FrameTrackersStopSequence(self.core.frame_trackers_, sequence);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5060-5067.
        fn ScrollbarAnimationMouseLeave(&mut self, id: ElementId) {
            self.WithScrollbarAnimationController(id, |controller, client| {
                if let Some(controller) = controller {
                    controller.DidMouseLeave(client);
                }
            });
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5069-5077.
        fn ScrollbarAnimationMouseMove(&mut self, id: ElementId, point: PointF) {
            self.WithScrollbarAnimationController(id, |controller, client| {
                if let Some(controller) = controller {
                    controller.DidMouseMove(client, point);
                }
            });
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5079-5088.
        fn ScrollbarAnimationMouseDown(&mut self, id: ElementId) -> bool {
            self.WithScrollbarAnimationController(id, |controller, client| {
                if let Some(controller) = controller {
                    controller.DidMouseDown(client);
                    return true;
                }
                false
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5090-5098.
        fn ScrollbarAnimationMouseUp(&mut self, id: ElementId) -> bool {
            self.WithScrollbarAnimationController(id, |controller, client| {
                if let Some(controller) = controller {
                    controller.DidMouseUp(client);
                    return true;
                }
                false
            })
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5052-5054.
        fn PinchBegin(&mut self) {
            self.WithBrowserControls(|manager, client| manager.PinchBegin(client));
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5056-5058.
        fn PinchEnd(&mut self) {
            self.WithBrowserControls(|manager, client| manager.PinchEnd(client));
        }
        // Source: cc/trees/layer_tree_host_impl.cc:943-945.
        fn SetNeedsAnimateInput(&mut self) {
            self.SetNeedsOneBeginImplFrame();
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5100-5103.
        fn TickScrollAnimations(&mut self) {
            let time = self.core.current_begin_frame_tracker_.Current().frame_time;
            self.context.services.MutatorTickScrollAnimations(
                self.core.mutator_host_,
                time,
                self.active_tree_.as_ref().unwrap().property_trees(),
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:947-951.
        fn CreateLatencyInfoSwapPromiseMonitor(
            &mut self,
            latency: Option<&mut B::LatencyInfo>,
        ) -> Option<Box<B::LatencyInfoSwapPromiseMonitor>> {
            let monitor = self
                .context
                .services
                .CreateLatencyInfoSwapPromiseMonitor(latency, self.AsWeakPtr());
            self.core
                .latency_info_swap_promise_monitor_
                .insert(monitor.identity());
            Some(monitor)
        }
        // Source: cc/trees/layer_tree_host_impl.cc:953-957.
        fn GetScopedEventMetricsMonitor(
            &mut self,
            done_callback: B::EventMetricsDoneCallback,
        ) -> Option<Box<B::EventMetricsScopedMonitor>> {
            Some(
                self.context
                    .services
                    .GetScopedEventMetricsMonitor(self.core.events_metrics_manager_, done_callback),
            )
        }
        // Source: cc/trees/layer_tree_host_impl.cc:959-961.
        fn DidScrollForMetrics(&mut self) {
            self.context
                .services
                .SetDidScrollForMetrics(self.core.events_metrics_manager_, true);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:963-967.
        fn NotifyInputEvent(&mut self, is_fling: bool) {
            NotifyInputEventForHost(
                self.core.has_input_for_frame_interval_,
                self.core.has_input_resetter_,
                self.core.has_non_fling_input_since_last_frame_,
                is_fling,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:5117-5126.
        fn UpdateImplAnimationScrollTargetWithDelta(
            &mut self,
            delta: Vector2dF,
            node_id: i32,
            delayed_by: TimeDelta,
            id: ElementId,
        ) -> Option<PointF> {
            let trees = self.active_tree_.as_ref().unwrap().property_trees();
            let max_offset =
                trees
                    .scroll_tree()
                    .MaxScrollOffset(node_id, trees.transform_tree(), trees.state());
            let time = self.core.current_begin_frame_tracker_.Current().frame_time;
            if let Some(receiver) = self
                .animation_reentry
                .as_mut()
                .filter(|r| r.MatchesElement(id))
            {
                return receiver
                    .UpdateScrollAnimationTarget(delta, max_offset, time, delayed_by, id);
            }
            self.context
                .services
                .MutatorImplOnlyScrollAnimationUpdateTarget(
                    self.core.mutator_host_,
                    delta,
                    max_offset,
                    time,
                    delayed_by,
                    id,
                )
        }
        // Source: cc/trees/layer_tree_host_impl.cc:543-546.
        fn SetDeferBeginMainFrame(&mut self, defer: bool) {
            self.context
                .services
                .DelegateSetDeferBeginMainFrameFromImpl(*self.core.delegate_, defer);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:472-482.
        fn DidUpdateScrollAnimationCurve(&mut self) {
            NotifyLatencyMonitorsForHost(
                self.core.latency_info_swap_promise_monitor_,
                &mut self.context,
            );
            self.context
                .services
                .SaveActiveEventMetrics(self.core.events_metrics_manager_);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:484-487.
        fn DidStartPinchZoom(&mut self) {
            self.RenewTreePriority();
            self.context
                .services
                .TrackersStartPinchZoom(self.core.frame_trackers_);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:498-501.
        fn DidUpdatePinchZoom(&mut self) {
            self.SetNeedsRedraw(false, false);
            self.RenewTreePriority();
        }
        // Source: cc/trees/layer_tree_host_impl.cc:489-496.
        fn DidEndPinchZoom(&mut self) {
            self.active_tree_
                .as_mut()
                .unwrap()
                .set_needs_update_draw_properties();
            self.SetNeedsRedraw(false, false);
            self.context
                .services
                .TrackersStopPinchZoom(self.core.frame_trackers_);
        }
        // Source: cc/trees/layer_tree_host_impl.cc:503-508.
        fn DidStartScroll(&mut self) {
            if !self.core.settings_.single_thread_proxy_scheduler {
                self.context
                    .services
                    .DelegateSetHasActiveThreadedScroll(*self.core.delegate_, true);
            }
            self.RenewTreePriority();
        }
        // Source: cc/trees/layer_tree_host_impl.cc:510-528.
        fn DidEndScroll(&mut self) {
            if self
                .context
                .services
                .NewContentForCheckerboardedScrollsEnabled()
                && self
                    .context
                    .services
                    .NewContentForCheckerboardedScrollsPerScroll()
            {
                *self.core.prioritize_new_content_due_to_checkerboarding_ = false;
            }
            if !self.core.settings_.single_thread_proxy_scheduler {
                self.context
                    .services
                    .DelegateSetHasActiveThreadedScroll(*self.core.delegate_, false);
                self.context
                    .services
                    .DelegateSetWaitingForScrollEvent(*self.core.delegate_, false);
            }
            #[cfg(any(target_os = "android", target_os = "ios"))]
            if let Some(observer) = self.core.render_frame_metadata_observer_.as_mut() {
                self.context
                    .services
                    .RenderFrameMetadataObserverDidEndScroll(observer);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:530-535.
        fn DidMouseEnterNonViewportScroller(&mut self, element_id: ElementId) {
            self.WithScrollbarAnimationController(element_id, |controller, client| {
                if let Some(controller) = controller {
                    controller.DidScrollUpdate(client);
                }
            });
        }
        // Source: cc/trees/layer_tree_host_impl.cc:537-541.
        fn DidMouseLeave(&mut self) {
            self.WithScrollbarAnimationControllers(|controllers, client| {
                for controller in controllers.values_mut() {
                    controller.as_mut().unwrap().DidMouseLeave(client);
                }
            });
        }
        // Source: cc/trees/layer_tree_host_impl.cc:558-560.
        fn HasScrollLinkedAnimation(&self, for_scroller: ElementId) -> bool {
            self.context
                .services
                .MutatorHasScrollLinkedAnimation(&*self.core.mutator_host_, for_scroller)
        }
        // Source: cc/trees/layer_tree_host_impl.cc:548-556.
        fn UpdateBrowserControlsState(
            &mut self,
            constraints: crate::cc::input::browser_controls_state::BrowserControlsState,
            current: crate::cc::input::browser_controls_state::BrowserControlsState,
            animate: bool,
            offset_tag_modifications:Option<&crate::cc::input::browser_controls_offset_tag_modifications::BrowserControlsOffsetTagModifications>,
        ) {
            self.WithBrowserControls(|manager, client| {
                manager.UpdateBrowserControlsState(
                    client,
                    constraints,
                    current,
                    animate,
                    offset_tag_modifications,
                )
            });
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn PageScaleFactor(&self) -> f32 {
            self.InputActiveTree()
                .page_scale_factor_for_scroll_for_tree(
                    true,
                    self.core.settings_.is_for_scalable_page,
                )
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn VisualDeviceViewportSize(&self) -> Size {
            *self.core.visual_device_viewport_size_
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn SetNeedsCommit(&mut self, reason: <B as InputHostTypes>::BeginMainFrameReason) {
            RequestCommitForHost(
                self.core.settings_,
                *self.core.delegate_,
                reason,
                &mut self.context,
            )
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn input_handler_id(&self) -> u64 {
            self.core.owner_id_.0
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn Now(&self) -> TimeTicks {
            self.context.services.MonotonicTimeNow()
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn feature(&self, feature: InputFeature) -> bool {
            self.context.services.InputFeatureEnabled(feature)
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn scroll_event_dispatch_mode(&self) -> String {
            self.context.services.InputScrollEventDispatchMode()
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn late_scroll_deadline_ratio(&self) -> f64 {
            self.context.services.InputLateScrollDeadlineRatio()
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn TrackerType(
            &self,
            kind: ScrollTrackingType,
        ) -> <B as InputHostTypes>::FrameSequenceTrackerType {
            self.context.services.InputTrackerType(kind)
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn DrivingThread(
            &self,
            thread: ScrollingThread,
        ) -> <B as InputHostTypes>::SmoothEffectDrivingThread {
            self.context.services.InputDrivingThread(thread)
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn ScrollBeginMainFrameReason(&self) -> <B as InputHostTypes>::BeginMainFrameReason {
            self.context.services.BeginMainFrameReasonScroll()
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn ShouldRecordSubsampledMetric(&self, probability: f64) -> bool {
            self.context
                .services
                .ShouldRecordSubsampledMetric(probability)
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn HistogramCounts(&self, name: &str, value: f32) {
            self.context.services.InputHistogramCounts(name, value)
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn HistogramTimes(&self, name: &str, value: TimeDelta) {
            self.context.services.InputHistogramTimes(name, value)
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn BeginTrace(
            &self,
            event: &str,
            fields: &[InputTraceField],
        ) -> <B as InputHandlerTypes>::TraceSpan {
            self.context.services.InputBeginTrace(event, fields)
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn InstantTrace(&self, event: &str, fields: &[InputTraceField]) {
            self.context.services.InputInstantTrace(event, fields)
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn LayerScreenSpaceTransform(
            &mut self,
            layer: &InputLayerId,
        ) -> <B as InputHandlerTypes>::Transform {
            LayerTreeLayer::<B>::ScreenSpaceTransform(
                self.InputLayer(layer),
                self.InputActiveTree().property_trees(),
            )
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn ScrollTree_ScreenSpaceTransform(
            &mut self,
            id: i32,
        ) -> <B as InputHandlerTypes>::Transform {
            let tree = self.InputActiveTree().property_trees();
            tree.scroll_tree()
                .ScreenSpaceTransform(id, tree.transform_tree())
        }
        // Source callback: cc/input/compositor_input_interfaces.h:135-202.
        fn ScrollBy(&mut self, node: &InputScrollNodeId, delta: Vector2dF) {
            assert_eq!(node.tree, self.InputActiveTree().id);
            self.ScrollNodeByForViewport(node.index, &delta);
        }
    }
}

// Source: cc/trees/layer_tree_impl.cc:1849-1958,1960-1985,2750-2752,
// 2788-2790,2826-2828,3043-3045. This temporary draw host holds only the
// existing mutation facade's live resident references and external services.
/// Per-call draw host: actual resident host field borrows plus the current
/// input owner's readonly output. No input flag/map or tree is copied.
struct HostImplDrawFacade<'a, 'h, 'c, 'v, B: HostImplTypes> {
    host: &'a mut HostImplMutationFacade<'h, 'c, B>,
    view: &'a crate::cc::input::input_handler::InputHandlerView<'v>,
}
impl<'a, 'h, 'c, 'v, B: HostImplTypes> std::ops::Deref for HostImplDrawFacade<'a, 'h, 'c, 'v, B> {
    type Target = HostImplMutationFacade<'h, 'c, B>;
    fn deref(&self) -> &Self::Target {
        self.host
    }
}
impl<B: HostImplTypes> std::ops::DerefMut for HostImplDrawFacade<'_, '_, '_, '_, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.host
    }
}
impl<B: HostImplTypes> crate::adapter::layer_tree_impl::draw_host::LayerTreeDrawHost<B>
    for HostImplDrawFacade<'_, '_, '_, '_, B>
{
    fn DrawServices(
        &self,
    ) -> &dyn crate::adapter::layer_tree_impl::draw_host::LayerTreeDrawServices {
        self.host.context.services
    }
    fn settings(&self) -> &B::Settings {
        self.host.core.settings_
    }
    fn IsActiveTree(&self, id: crate::adapter::layer_tree_impl::LayerTreeId) -> bool {
        self.host.active_tree_.identity() == Some(id)
    }
    fn HasLayerTreeFrameSink(&self) -> bool {
        self.host.core.layer_tree_frame_sink_.is_some()
    }
    fn IsResourcelessSoftwareDraw(&self) -> bool {
        *self.host.core.resourceless_software_draw_
    }
    fn HasImageAnimationController(&self) -> bool {
        self.host.image_animation_controller_.is_some()
    }
    fn ViewTransitionCaptureAndDisplay(&self) -> bool {
        self.host.context.services.ViewTransitionCaptureAndDisplay()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.host.context.services.IsCCSlimmingEnabled()
    }
    fn DrawTransform(&self) -> &<B as LayerTreeTypes>::Transform {
        self.host.core.external_transform_
    }
    fn external_viewport(&self) -> Rect {
        *self.host.core.external_viewport_
    }
    fn max_texture_size(&self) -> i32 {
        self.host.core.raster_caps_.max_texture_size
    }
    fn SetViewTransitionContentRect(
        &mut self,
        sequence: u32,
        id: &crate::adapter::property_types::ViewTransitionElementResourceId,
        rect: &RectF,
    ) {
        SetViewTransitionContentRectForHost(
            self.host.core.view_transition_content_rects_,
            sequence,
            id,
            rect,
        );
    }
    fn UpdateImageAnimationStateFromDrivers(&mut self, receiver: &mut LayerTreeImpl<B>) {
        let drivers = GatherAnimatedImageDriverStateForTrees(
            self.host.active_tree_.Resolve(receiver).unwrap(),
            self.host.pending_tree_.Resolve(receiver),
            self.host.recycle_tree_.Resolve(receiver),
        );
        UpdateImageAnimationStateForHost(
            self.host.active_tree_.ResolveMut(receiver).unwrap(),
            self.host.core.settings_,
            self.host.core.impl_thread_phase_,
            *self.host.core.delegate_,
            self.host.core.latency_info_swap_promise_monitor_,
            self.host.core.events_metrics_manager_,
            self.host.image_animation_controller_,
            &drivers,
            self.host.context,
        );
    }
    fn DidModifyTilePriorities(&mut self, pending_update_tiles: bool) {
        DidModifyTilePrioritiesForHost(
            self.host.core.settings_,
            self.host.core.tile_priorities_dirty_,
            self.host.core.tile_manager_,
            *self.host.core.delegate_,
            pending_update_tiles,
            self.host.context,
        );
    }
    fn TraceLayersUpdatedCount(&self, count: usize) {
        self.host.context.services.TraceLayersUpdatedCount(count);
    }
}
// Borrowed native receiver/owner slots for source LayerTreeImpl::IsRecycleTree.
struct HostImplPictureReadContext<'a, B: HostImplTypes> {
    receiver: &'a LayerTreeImpl<B>,
    recycle: Option<&'a LayerTreeImpl<B>>,
}
impl<B: HostImplTypes> crate::adapter::picture_layer_impl::PictureLayerReadContext
    for HostImplPictureReadContext<'_, B>
{
    fn IsRecycleTree(&self) -> bool {
        self.recycle.is_some_and(|tree| tree.id == self.receiver.id)
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:788-799.
fn GatherAnimatedImageDriverStateForTrees<B: HostImplTypes>(
    active: &LayerTreeImpl<B>,
    pending: Option<&LayerTreeImpl<B>>,
    recycle: Option<&LayerTreeImpl<B>>,
) -> crate::adapter::image_animation::AnimatedImageDriverMap {
    let mut result = crate::adapter::image_animation::AnimatedImageDriverMap::default();
    active.AnnotateAnimatedImages(
        &mut result,
        &HostImplPictureReadContext {
            receiver: active,
            recycle,
        },
    );
    if let Some(tree) = pending {
        tree.AnnotateAnimatedImages(
            &mut result,
            &HostImplPictureReadContext {
                receiver: tree,
                recycle,
            },
        );
    }
    if let Some(tree) = recycle {
        tree.AnnotateAnimatedImages(
            &mut result,
            &HostImplPictureReadContext {
                receiver: tree,
                recycle,
            },
        );
    }
    result
}
// Source: cc/trees/layer_tree_impl.cc:1947-1951; live client required by native
// ImageAnimationController's retained scheduler, without any input owner.
fn UpdateImageAnimationStateForHost<B: HostImplTypes>(
    active_tree: &mut LayerTreeImpl<B>,
    settings: &LayerTreeSettings,
    phase: &ImplThreadPhase,
    delegate: HostImplDelegateId,
    monitors: &BTreeSet<HostImplLatencyMonitorId>,
    events: &mut B::EventsMetricsManager,
    images: &mut Option<Box<ImageAnimationController>>,
    drivers: &crate::adapter::image_animation::AnimatedImageDriverMap,
    context: &mut HostImplContext<'_, B>,
) {
    let mut client = HostImplImageAnimationClient {
        active_tree,
        settings,
        phase,
        delegate,
        monitors,
        events,
        context,
    };
    images
        .as_mut()
        .unwrap()
        .UpdateStateFromDrivers(drivers, &mut client);
}
// Source: cc/trees/layer_tree_host_impl.cc:2354-2359.
fn SetViewTransitionContentRectForHost(
    rects: &mut HashMap<u32, HostViewTransitionElementResourceRects>,
    sequence: u32,
    id: &crate::adapter::property_types::ViewTransitionElementResourceId,
    rect: &RectF,
) {
    rects
        .entry(sequence)
        .or_default()
        .entry(*id)
        .or_insert(*rect);
}

mod source_input_scrollbar {
    use super::*;
    use crate::adapter::{
        geometry::{PointF, Rect, SizeF, Vector2dF},
        input_handler::{InputHost, InputScrollNodeId, InputScrollbarHost},
        scrollbar::{AutoscrollTask, ScrollbarHost, ScrollbarLayerAccess},
    };
    use crate::cc::input::{
        input_handler::{InputHandler, InputHandlerView},
        scroll_state::ScrollState,
    };
    struct HostImplInputScrollbarHost<'a, 'h, B: HostImplTypes> {
        host: &'a mut HostImplInputFacade<'h, B>,
        last_scroll_update_state: Option<&'a ScrollState>,
        view: &'a InputHandlerView<'a>,
    }
    impl<B: HostImplTypes> HostImplInputScrollbarHost<'_, '_, B> {
        fn WithLayerClient<R>(
            &mut self,
            callback: impl FnOnce(&mut HostImplScrollbarClient<'_, '_, B>) -> R,
        ) -> R {
            let host = &mut *self.host;
            let mut client = HostImplScrollbarClient {
                active_tree: host.active_tree_.as_mut().unwrap(),
                settings: host.core.settings_,
                delegate: *host.core.delegate_,
                events: host.core.events_metrics_manager_,
                latency_monitors: host.core.latency_info_swap_promise_monitor_,
                context: &mut host.context,
            };
            callback(&mut client)
        }
    }
    impl<B: HostImplTypes> ScrollbarLayerAccess for HostImplInputScrollbarHost<'_, '_, B> {
        type Transform = <B as LayerTreeTypes>::Transform;
        fn Scrollbar_orientation(&mut self, layer: &InputLayerId) -> ScrollbarOrientation {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_orientation(client, layer)
            })
        }
        fn Scrollbar_scroll_element_id(&mut self, layer: &InputLayerId) -> ElementId {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_scroll_element_id(client, layer)
            })
        }
        fn Scrollbar_OverlayScrollbarOpacity(&mut self, layer: &InputLayerId) -> f32 {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_OverlayScrollbarOpacity(client, layer)
            })
        }
        fn Scrollbar_UsesAuraOverlayAnimator(&mut self, layer: &InputLayerId) -> bool {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_UsesAuraOverlayAnimator(client, layer)
            })
        }
        fn Scrollbar_has_find_in_page_tickmarks(&mut self, layer: &InputLayerId) -> bool {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_has_find_in_page_tickmarks(client, layer)
            })
        }
        fn Scrollbar_JumpOnTrackClick(&mut self, layer: &InputLayerId) -> bool {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_JumpOnTrackClick(client, layer)
            })
        }
        fn Scrollbar_current_pos(&mut self, layer: &InputLayerId) -> f32 {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_current_pos(client, layer)
            })
        }
        fn Scrollbar_scroll_layer_length(&mut self, layer: &InputLayerId) -> f32 {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_scroll_layer_length(client, layer)
            })
        }
        fn Scrollbar_ComputeThumbQuadRect(&mut self, layer: &InputLayerId) -> Rect {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_ComputeThumbQuadRect(client, layer)
            })
        }
        fn Scrollbar_SupportsDragSnapBack(&mut self, layer: &InputLayerId) -> bool {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_SupportsDragSnapBack(client, layer)
            })
        }
        fn Scrollbar_ForwardTrackRect(&mut self, layer: &InputLayerId) -> Rect {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_ForwardTrackRect(client, layer)
            })
        }
        fn Scrollbar_BackTrackRect(&mut self, layer: &InputLayerId) -> Rect {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_BackTrackRect(client, layer)
            })
        }
        fn Scrollbar_BackButtonRect(&mut self, layer: &InputLayerId) -> Rect {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_BackButtonRect(client, layer)
            })
        }
        fn Scrollbar_ForwardButtonRect(&mut self, layer: &InputLayerId) -> Rect {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_ForwardButtonRect(client, layer)
            })
        }
        fn Scrollbar_ThumbThickness(&mut self, layer: &InputLayerId) -> i32 {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_ThumbThickness(client, layer)
            })
        }
        fn Scrollbar_ThumbLength(&mut self, layer: &InputLayerId) -> i32 {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_ThumbLength(client, layer)
            })
        }
        fn Scrollbar_TrackLength(&mut self, layer: &InputLayerId) -> f32 {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_TrackLength(client, layer)
            })
        }
        fn Scrollbar_ScreenSpaceTransform(&mut self, layer: &InputLayerId) -> Self::Transform {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_ScreenSpaceTransform(client, layer)
            })
        }
        fn Scrollbar_IdentifyScrollbarPart(
            &mut self,
            layer: &InputLayerId,
            position: PointF,
        ) -> ScrollbarPart {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_IdentifyScrollbarPart(client, layer, position)
            })
        }
        fn Scrollbar_is_overlay_scrollbar(&mut self, layer: &InputLayerId) -> bool {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_is_overlay_scrollbar(client, layer)
            })
        }
        fn Scrollbar_bounds(&mut self, layer: &InputLayerId) -> geometry::Size {
            self.WithLayerClient(|client| ScrollbarLayerAccess::Scrollbar_bounds(client, layer))
        }
        fn Scrollbar_device_scale_factor(&mut self, layer: &InputLayerId) -> f32 {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_device_scale_factor(client, layer)
            })
        }
        fn Scrollbar_ComputeHitTestableExpandedThumbQuadRect(
            &mut self,
            layer: &InputLayerId,
        ) -> Rect {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_ComputeHitTestableExpandedThumbQuadRect(
                    client, layer,
                )
            })
        }
        fn Scrollbar_thumb_thickness_scale_factor(&mut self, layer: &InputLayerId) -> f32 {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_thumb_thickness_scale_factor(client, layer)
            })
        }
        fn Scrollbar_SetThumbThicknessScaleFactor(&mut self, layer: &InputLayerId, scale: f32) {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_SetThumbThicknessScaleFactor(client, layer, scale)
            })
        }
        fn Scrollbar_CanScrollOrientation(&mut self, layer: &InputLayerId) -> bool {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_CanScrollOrientation(client, layer)
            })
        }
        fn Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
            &mut self,
            layer: &InputLayerId,
            opacity: f32,
            fade_out_animation: bool,
        ) {
            self.WithLayerClient(|client| {
                ScrollbarLayerAccess::Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
                    client,
                    layer,
                    opacity,
                    fade_out_animation,
                )
            })
        }
    }
    impl<B: HostImplTypes> ScrollbarHost for HostImplInputScrollbarHost<'_, '_, B> {
        fn ScrollbarsFor(&mut self, element: ElementId) -> Vec<InputLayerId> {
            let tree = self.host.active_tree_.as_ref().unwrap();
            tree.ScrollbarsFor(element)
                .iter()
                .map(|layer| InputLayerId {
                    tree: tree.id,
                    id: layer.layer_id,
                })
                .collect()
        }
        fn IsScrollbarLayer(&mut self, layer: &InputLayerId) -> bool {
            self.host.LayerIsScrollbarLayer(layer)
        }
        fn FindLayerThatIsHitByPoint(&mut self, point: PointF) -> Option<InputLayerId> {
            self.host
                .ActiveTree_FindLayerThatIsHitByPoint(self.view, point)
        }
        fn device_scale_factor(&mut self) -> f32 {
            self.host.ActiveTree_device_scale_factor()
        }
        fn painted_device_scale_factor(&mut self) -> f32 {
            self.host.ActiveTree_painted_device_scale_factor()
        }
        fn page_scale_factor_for_scroll(&mut self) -> f32 {
            self.host.ActiveTree_page_scale_factor_for_scroll()
        }
        fn FindNodeFromElementId(&mut self, element: ElementId) -> Option<InputScrollNodeId> {
            self.host.ScrollTree_MutableFindNodeFromElementId(element)
        }
        fn current_scroll_offset(&mut self, element: ElementId) -> PointF {
            self.host.ScrollTree_current_scroll_offset(element)
        }
        fn scrolls_inner_viewport(&mut self, node: &InputScrollNodeId) -> bool {
            self.host
                .ReadScrollNode(node, |node| node.scrolls_inner_viewport)
        }
        fn scrolls_outer_viewport(&mut self, node: &InputScrollNodeId) -> bool {
            self.host
                .ReadScrollNode(node, |node| node.scrolls_outer_viewport)
        }
        fn element_id(&mut self, node: &InputScrollNodeId) -> ElementId {
            self.host.ReadScrollNode(node, |node| node.element_id)
        }
        fn container_bounds(&mut self, node: &InputScrollNodeId) -> Size {
            let tree = self.host.active_tree_.as_ref().unwrap();
            assert_eq!(tree.id, node.tree);
            let properties = tree.property_trees();
            properties
                .scroll_tree()
                .container_bounds(node.index, properties.state())
        }
        fn SnapContainerRect(
            &mut self,
            node: &InputScrollNodeId,
        ) -> Option<crate::adapter::geometry::RectF> {
            self.host.ReadScrollNode(node, |node| {
                node.snap_container_data.as_ref().map(|data| data.rect())
            })
        }
        fn ComputeClampedDelta(&mut self, delta: &Vector2dF) -> Vector2dF {
            Viewport::ComputeClampedDeltaWith(self, *delta)
        }
        fn GetInnerViewportSizeExcludingScrollbars(&mut self) -> SizeF {
            self.host.WithViewport(None, |viewport, c| {
                viewport.GetInnerViewportSizeExcludingScrollbars(c)
            })
        }
        fn ComputeScrollDelta(&mut self, node: &InputScrollNodeId, delta: &Vector2dF) -> Vector2dF {
            InputHandler::<B>::ComputeScrollDeltaForState(
                self.host,
                self.last_scroll_update_state,
                node,
                *delta,
                None,
            )
        }
        fn ScrollAnimationAbort(&mut self, element: ElementId) {
            InputHost::<B>::ScrollAnimationAbort(self.host, element)
        }
        fn IsElementAnimating(&mut self, element: ElementId) -> bool {
            InputHost::<B>::ElementHasImplOnlyScrollAnimation(self.host, element)
        }
        // Source: cc/trees/layer_tree_host_impl.cc:4771-4789.
        fn AutoScrollAnimationCreate(
            &mut self,
            node: &InputScrollNodeId,
            target: PointF,
            velocity: f32,
        ) {
            let host = &mut *self.host;
            let animation_start_offset = host.core.current_begin_frame_tracker_.Current().interval;
            let tree = host.active_tree_.as_ref().unwrap();
            assert_eq!(tree.id, node.tree);
            let properties = tree.property_trees();
            let scroll = properties.scroll_tree();
            let element = scroll.Node(node.index).element_id;
            let current = scroll.current_scroll_offset(element, properties.state());
            if let Some(receiver) = host
                .animation_reentry
                .as_mut()
                .filter(|r| r.MatchesElement(element))
            {
                receiver.CreateAutoScrollAnimation(
                    element,
                    target,
                    current,
                    velocity,
                    animation_start_offset,
                );
            } else {
                host.context
                    .services
                    .MutatorImplOnlyAutoScrollAnimationCreate(
                        host.core.mutator_host_,
                        element,
                        &target,
                        current,
                        velocity,
                        animation_start_offset,
                    );
            }
            host.SetNeedsOneBeginImplFrame();
        }
        fn controller_id(&self) -> u64 {
            self.host.core.owner_id_.0
        }
        fn PostDelayedAutoscrollTask(&mut self, task: AutoscrollTask, delay: TimeDelta) {
            let target = HostImplWeak {
                owner: *self.host.core.owner_id_,
                alive: Arc::downgrade(&*self.host.core.weak_factory_),
            };
            self.host
                .context
                .services
                .PostDelayedAutoscrollTask(target, task, delay);
        }
    }
    impl<B: HostImplTypes> crate::cc::layers::viewport::ClampedDeltaContext
        for HostImplInputScrollbarHost<'_, '_, B>
    {
        fn InnerDelta(&mut self, delta: Vector2dF) -> Vector2dF {
            let node = self.host.ActiveTree_InnerViewportScrollNode().unwrap();
            InputHandler::<B>::ComputeScrollDeltaForState(
                self.host,
                self.last_scroll_update_state,
                &node,
                delta,
                None,
            )
        }
        fn PageScaleFactorForScroll(&self) -> f32 {
            self.host
                .active_tree_
                .as_ref()
                .unwrap()
                .page_scale_factor_for_scroll_for_tree(
                    true,
                    self.host.core.settings_.is_for_scalable_page,
                )
        }
        fn OuterDelta(&mut self, delta: Vector2dF) -> Vector2dF {
            let node = self.host.ActiveTree_OuterViewportScrollNode().unwrap();
            InputHandler::<B>::ComputeScrollDeltaForState(
                self.host,
                self.last_scroll_update_state,
                &node,
                delta,
                None,
            )
        }
    }
    impl<B: HostImplTypes> HostImplInputFacade<'_, B> {
        pub(crate) fn WithScrollbarHostForInput<R>(
            &mut self,
            view: &InputHandlerView<'_>,
            last_scroll_update_state: Option<&ScrollState>,
            callback: impl FnOnce(&mut InputScrollbarHost<'_, B>) -> R,
        ) -> R {
            let mut host = HostImplInputScrollbarHost {
                host: self,
                last_scroll_update_state,
                view,
            };
            callback(&mut host)
        }
    }
}

mod source_input_owner_forwarding {
    use super::*;
    // Source: cc/trees/layer_tree_host_impl.cc:963-967. Schedules the same
    // canonical notifier and mutates only resident owner flags.
    pub(super) fn NotifyInputEventForHost(
        has_input: &mut bool,
        notifier: &Arc<crate::cc::base::delayed_unique_notifier::DelayedUniqueNotifier>,
        has_non_fling: &mut bool,
        is_fling: bool,
    ) {
        *has_input = true;
        notifier.Schedule();
        *has_non_fling |= !is_fling;
    }
    impl<B: HostImplTypes> LayerTreeHostImpl<B> {
        pub(super) fn WithInputRuntime<R>(
            &mut self,
            context: &mut HostImplContext<'_, B>,
            callback: impl FnOnce(&mut HostImplInputFacade<'_, B>) -> R,
        ) -> R {
            let HostImplRuntimeParts {
                core,
                active_tree_,
                pending_tree_,
                recycle_tree_,
                input_delegate_: _,
                browser_controls_offset_manager_,
                viewport_,
                scrollbar_animation_controllers_,
                image_animation_controller_,
            } = self.BorrowRuntime();
            let mut host = HostImplInputFacade {
                animation_reentry: None,
                core,
                active_tree_: HostImplInputTree::Slot(active_tree_),
                pending_tree_: HostImplInputTree::Slot(pending_tree_),
                recycle_tree_: HostImplInputTree::Slot(recycle_tree_),
                browser_controls_offset_manager_: HostImplControlsSlot::Resident(
                    browser_controls_offset_manager_,
                ),
                viewport_: HostImplViewportSlot::Resident(viewport_),
                scrollbar_animation_controllers_,
                image_animation_controller_,
                context: HostImplContext {
                    services: context.services,
                },
            };
            callback(&mut host)
        }
        pub(super) fn WithInputRuntimeView<R>(
            &mut self,
            context: &mut HostImplContext<'_, B>,
            callback: impl FnOnce(
                &mut HostImplInputFacade<'_, B>,
                &crate::cc::input::input_handler::InputHandlerView<'_>,
            ) -> R,
        ) -> R {
            let HostImplRuntimeParts {
                core,
                active_tree_,
                pending_tree_,
                recycle_tree_,
                input_delegate_,
                browser_controls_offset_manager_,
                viewport_,
                scrollbar_animation_controllers_,
                image_animation_controller_,
            } = self.BorrowRuntime();
            let view = input_delegate_
                .as_ref()
                .expect("source GetInputHandler")
                .View();
            let mut host = HostImplInputFacade {
                animation_reentry: None,
                core,
                active_tree_: HostImplInputTree::Slot(active_tree_),
                pending_tree_: HostImplInputTree::Slot(pending_tree_),
                recycle_tree_: HostImplInputTree::Slot(recycle_tree_),
                browser_controls_offset_manager_: HostImplControlsSlot::Resident(
                    browser_controls_offset_manager_,
                ),
                viewport_: HostImplViewportSlot::Resident(viewport_),
                scrollbar_animation_controllers_,
                image_animation_controller_,
                context: HostImplContext {
                    services: context.services,
                },
            };
            callback(&mut host, &view)
        }
    }
}
use source_input_owner_forwarding::NotifyInputEventForHost;

// Source: cc/trees/layer_tree_host_impl.cc:5228-5237,1592-1594.
// Source Rect return values are read at each call, never retained as host state.
fn SetFullViewportDamageForViewport<B: HostImplTypes>(
    internal: Rect,
    external: Rect,
    damage: &mut Rect,
) {
    let origin = internal.origin();
    let viewport = LayerTreeImpl::<B>::GetDeviceViewportForRects(internal, external);
    damage.Union(Rect::new(
        origin.x(),
        origin.y(),
        viewport.width,
        viewport.height,
    ));
}
// Source: cc/input/scroll_elasticity_helper.cc:97-108,119-330.
// All contexts below are borrowed for one helper call. The stored helper owns
// only the source non-owning host identity; no host/tree state is retained.
mod source_elasticity_runtime {
    use super::*;
    use crate::adapter::elasticity_context::*;
    use crate::cc::input::scroll_elasticity_helper::{self, ScrollElasticityHelper};
    struct ElasticityPolicy<'a>(&'a LayerTreeSettings);
    impl crate::adapter::scroll_elasticity::LayerTreeHostImpl for ElasticityPolicy<'_> {
        fn CommitsToActiveTree(&self) -> bool {
            self.0.commit_to_active_tree
        }
    }
    struct TreeEffects<'a>(&'a mut bool);
    impl ElasticityTreeEffects for TreeEffects<'_> {
        fn set_needs_update_draw_properties(&mut self) {
            *self.0 = true;
        }
    }
    struct HostEffects<'a, 'c, B: HostImplTypes> {
        core: HostImplRuntimeCore<'a, B>,
        internal_viewport: &'a Rect,
        latency_promises: &'a mut Vec<Box<B::SwapPromise>>,
        context: &'a mut HostImplContext<'c, B>,
    }
    impl<B: HostImplTypes> crate::adapter::scroll_elasticity::LayerTreeHostImpl
        for HostEffects<'_, '_, B>
    {
        fn CommitsToActiveTree(&self) -> bool {
            self.core.settings_.commit_to_active_tree
        }
    }
    impl<B: HostImplTypes> ElasticityHostEffects for HostEffects<'_, '_, B> {
        fn SetNeedsRedraw(&mut self, animation_only: bool, skip_if_inside_draw: bool) {
            self.latency_promises.extend(RequestRedrawForHost(
                self.core.settings_,
                *self.core.delegate_,
                self.core.latency_info_swap_promise_monitor_,
                self.core.events_metrics_manager_,
                animation_only,
                skip_if_inside_draw,
                self.context,
            ));
        }
        fn SetFullViewportDamage(&mut self) {
            SetFullViewportDamageForViewport::<B>(
                *self.internal_viewport,
                *self.core.external_viewport_,
                self.core.viewport_damage_rect_,
            );
        }
        fn RequestImplSideInvalidationForRasterInducingScroll(&mut self, id: ElementId) {
            self.core
                .pending_invalidation_raster_inducing_scrolls_
                .insert(id);
            self.context
                .services
                .SetNeedsImplSideInvalidation(*self.core.delegate_, true);
        }
    }
    struct ReadContext<'a, 'h, B: HostImplTypes> {
        host: &'a HostImplInputFacade<'h, B>,
    }
    struct Context<'a, 'h, B: HostImplTypes> {
        host: &'a mut HostImplInputFacade<'h, B>,
    }
    macro_rules! queries {
        ($type:ident) => {
            impl<B: HostImplTypes> ElasticityOwner for $type<'_, '_, B> {
                fn identity(&self) -> LayerTreeHostId {
                    *self.host.core.owner_id_
                }
            }
            impl<B: HostImplTypes> crate::adapter::scroll_elasticity::LayerTreeHostImpl
                for $type<'_, '_, B>
            {
                fn CommitsToActiveTree(&self) -> bool {
                    self.host.core.settings_.commit_to_active_tree
                }
            }
            impl<B: HostImplTypes> ElasticityQueries for $type<'_, '_, B> {
                fn GetScrollTree(
                    &self,
                ) -> &crate::cc::trees::property_tree::ScrollTree {
                    self.host
                        .active_tree_
                        .as_ref()
                        .unwrap()
                        .property_trees()
                        .scroll_tree()
                }
                fn InnerViewportScrollNodeId(&self) -> Option<i32> {
                    self.host
                        .active_tree_
                        .as_ref()
                        .unwrap()
                        .InnerViewportScrollNode()
                        .map(|n| n.id)
                }
                fn OuterViewportScrollNodeId(&self) -> Option<i32> {
                    self.host
                        .active_tree_
                        .as_ref()
                        .unwrap()
                        .OuterViewportScrollNode()
                        .map(|n| n.id)
                }
                fn enable_elastic_overscroll_on_root(&self) -> bool {
                    self.host.core.settings_.enable_elastic_overscroll_on_root
                }
                fn enable_elastic_overscroll_for_subscroll(&self) -> bool {
                    self.host
                        .core
                        .settings_
                        .enable_elastic_overscroll_for_subscroll
                }
                fn active_scroll_tree(
                    &self,
                ) -> &crate::cc::trees::property_tree::ScrollTree {
                    self.GetScrollTree()
                }
                fn TotalScrollOffset(&self, id: Option<ElementId>) -> PointF {
                    let tree = self.host.active_tree_.as_ref().unwrap();
                    if let Some(id) = id {
                        tree.TotalScrollOffsetForElement(id)
                    } else {
                        tree.TotalScrollOffset()
                    }
                }
                fn TotalMaxScrollOffset(&self, id: Option<ElementId>) -> PointF {
                    let tree = self.host.active_tree_.as_ref().unwrap();
                    if let Some(id) = id {
                        tree.TotalMaxScrollOffsetForElement(id)
                    } else {
                        tree.TotalMaxScrollOffset()
                    }
                }
            }
        };
    }
    queries!(ReadContext);
    queries!(Context);
    impl<B: HostImplTypes> ElasticityContext for Context<'_, '_, B> {
        type Transform = <B as LayerTreeTypes>::Transform;
        fn WithActiveTreeMutation<R>(
            &mut self,
            visit: impl FnOnce(ElasticityTreeMutation<'_, Self::Transform>) -> R,
        ) -> R {
            let policy = ElasticityPolicy(self.host.core.settings_);
            let active = self.host.active_tree_.as_mut().unwrap();
            let mut latency_promises = Vec::new();
            let result =
                active.WithElasticityMutationParts(|scroll_tree, mutation, dirty, internal| {
                    let mut tree_effects = TreeEffects(dirty);
                    let mut host_effects = HostEffects {
                        core: self.host.core.reborrow(),
                        internal_viewport: internal,
                        latency_promises: &mut latency_promises,
                        context: &mut self.host.context,
                    };
                    visit(ElasticityTreeMutation {
                        scroll_tree,
                        context: mutation,
                        tree_effects: &mut tree_effects,
                        host_policy: &policy,
                        host_effects: Some(&mut host_effects),
                        is_active_tree: true,
                    })
                });
            QueuePinnedLatencyPromisesOnActiveTree(active, latency_promises);
            result
        }
        fn WithActiveAndPendingMutation<R>(
            &mut self,
            visit: impl FnOnce(
                &crate::cc::trees::property_tree::ScrollTree,
                ElasticityTreeMutation<'_, Self::Transform>,
            ) -> R,
        ) -> R {
            let policy = ElasticityPolicy(self.host.core.settings_);
            let active = self
                .host
                .active_tree_
                .as_ref()
                .unwrap()
                .property_trees()
                .scroll_tree();
            let pending = self
                .host
                .pending_tree_
                .as_mut()
                .expect("source pending tree");
            let mut latency_promises = Vec::new();
            let result =
                pending.WithElasticityMutationParts(|scroll_tree, mutation, dirty, internal| {
                    let mut tree_effects = TreeEffects(dirty);
                    let mut host_effects = HostEffects {
                        core: self.host.core.reborrow(),
                        internal_viewport: internal,
                        latency_promises: &mut latency_promises,
                        context: &mut self.host.context,
                    };
                    visit(
                        active,
                        ElasticityTreeMutation {
                            scroll_tree,
                            context: mutation,
                            tree_effects: &mut tree_effects,
                            host_policy: &policy,
                            host_effects: Some(&mut host_effects),
                            is_active_tree: false,
                        },
                    )
                });
            QueuePinnedLatencyPromisesOnActiveTree(
                self.host.active_tree_.as_mut().expect("source active tree"),
                latency_promises,
            );
            result
        }
        fn ScrollByNodeId(&mut self, id: i32, delta: &Vector2dF) {
            self.host.ScrollNodeByForViewport(id, delta);
        }
        fn SetNeedsOneBeginImplFrame(&mut self) {
            let promises = RequestBeginImplFrameForHost(
                *self.host.core.delegate_,
                self.host.core.latency_info_swap_promise_monitor_,
                self.host.core.events_metrics_manager_,
                &mut self.host.context,
            );
            QueuePinnedLatencyPromisesOnActiveTree(
                self.host.active_tree_.as_mut().expect("source active tree"),
                promises,
            );
        }
    }
    impl<B: HostImplTypes> HostImplInputFacade<'_, B> {
        pub(crate) fn CreateScrollElasticityHelperForInput(&mut self) -> B::Elasticity {
            scroll_elasticity_helper::CreateForLayerTreeHostImpl(&ReadContext { host: self })
        }
        pub(crate) fn ResetStretchAmountsForInput(&mut self, helper: &mut B::Elasticity) {
            helper.ResetStretchAmounts(&mut Context { host: self });
        }
        pub(crate) fn ApplyStretchAmountsToPendingForInput(&mut self, helper: &mut B::Elasticity) {
            helper.ApplyStretchAmountsToPending(&mut Context { host: self });
        }
        pub(crate) fn ApplyStretchAmountsToActiveForInput(&mut self, helper: &mut B::Elasticity) {
            helper.ApplyStretchAmountsToActive(&mut Context { host: self });
        }
        pub(crate) fn StretchAmountForInput(
            &self,
            helper: &B::Elasticity,
            id: ElementId,
        ) -> Vector2dF {
            helper.StretchAmount(&ReadContext { host: self }, id)
        }
    }
}

mod source_chunk_d {
    use super::*;
    // Translation status: complete
    // Source: cc/trees/layer_tree_host_impl.cc:2396-3491.
    // Canonical source control flow; external Viz values stay move-owned and typed.
    use crate::adapter::layer_tree_impl::LayerTreeViewTransitionRequest;
    use crate::adapter::{damage_context::DamageContext, geometry::SizeF};
    use crate::cc::layers::draw_mode::DrawMode;
    impl<B: HostImplTypes> LayerTreeHostImpl<B> {
        // Source: cc/trees/layer_tree_host_impl.cc:2397-2465.
        pub fn CollectTrackedElementRects(
            &self,
            is_for_compositor_frame_metadata: bool,
            need_occlusion: bool,
        ) -> <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects
        {
            let mut rects = <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects::default();
            let tree = self.active_tree_.as_deref().unwrap();
            let inner_scroll_id = tree
                .InnerViewportScrollNode()
                .map_or(kInvalidPropertyNodeId, |node| node.transform_id);
            let outer_scroll_id = tree
                .OuterViewportScrollNode()
                .map_or(kInvalidPropertyNodeId, |node| node.transform_id);
            let transform_tree = tree.property_trees().transform_tree();
            let mut sticky_or_viewport_scroll_cache = BTreeMap::new();
            let mut front_occluders = Vec::new();
            for layer in tree.begin().rev() {
                if let Some(tracked) = HostMetadataLayer::<B>::tracked_element_rects_value(layer)
                    .filter(|tracked| !tracked.is_empty())
                {
                    for (&feature, tracked_element_list) in tracked.iter() {
                        for rect_data in tracked_element_list {
                            if rect_data.should_add_to_compositor_frame_metadata()
                                != is_for_compositor_frame_metadata
                            {
                                continue;
                            }
                            assert!(
                                !rect_data.should_exclude_fixed_and_sticky_occlusions()
                                    || need_occlusion
                            );
                            let mut transformed_rect = rect_data.clone();
                            transformed_rect.set_visible_bounds(CalculateVisibleBounds(
                                layer,
                                tree.property_trees(),
                                rect_data,
                                &front_occluders,
                                transform_tree,
                                inner_scroll_id,
                                outer_scroll_id,
                                &mut sticky_or_viewport_scroll_cache,
                            ));
                            rects.Append(feature, transformed_rect);
                        }
                    }
                }
                if need_occlusion
                    && HostImplLayer::draws_content(layer)
                    && HostMetadataLayer::<B>::draw_opacity(layer) > 0.05
                {
                    let transform_id = HostImplLayer::transform_tree_index(layer);
                    if IsFixedOrStickyOccluderTransform(
                        transform_id,
                        transform_tree,
                        inner_scroll_id,
                        outer_scroll_id,
                        &mut sticky_or_viewport_scroll_cache,
                    ) {
                        let layer_visible_rect =
                            HostImplLayer::VisibleLayerRect(layer, tree.property_trees());
                        if !layer_visible_rect.IsEmpty() {
                            let occluder_screen_rect = MathUtil::ProjectEnclosingClippedRect(
                                &HostImplLayer::ScreenSpaceTransform(layer, tree.property_trees()),
                                layer_visible_rect,
                            );
                            if !occluder_screen_rect.IsEmpty() {
                                front_occluders.push(HeuristicFixedOrStickyOccluder {
                                    rect: occluder_screen_rect,
                                    transform_tree_index: HostImplLayer::transform_tree_index(
                                        layer,
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            rects
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2467-2492.
        pub fn CollectRegionCaptureBounds(
            &self,
        ) -> <B as crate::adapter::layer_tree_host_impl::HostImplDTypes>::RegionCaptureBounds
        {
            let mut bounds = <B as crate::adapter::layer_tree_host_impl::HostImplDTypes>::RegionCaptureBounds::default();
            let tree = self.active_tree_.as_deref().unwrap();
            for layer in tree.begin().rev() {
                let Some(capture) = HostMetadataLayer::<B>::capture_bounds(layer) else {
                    continue;
                };
                for (&id, rect) in capture.bounds() {
                    let mut screen = MathUtil::ProjectEnclosingClippedRect(
                        &HostImplLayer::ScreenSpaceTransform(layer, tree.property_trees()),
                        *rect,
                    );
                    let root = tree.RootRenderSurface().unwrap();
                    let content=crate::adapter::geometry::ToEnclosedRect(MathUtil::MapClippedRect(&crate::adapter::draw_property::DrawSurface::screen_space_transform(root),crate::adapter::layer_tree_host_impl::HostScrollRenderSurface::DrawableContentRect(root, tree.property_trees())));
                    screen.Intersect(content);
                    bounds.Set(id, screen);
                }
            }
            bounds
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2494-2661.
        pub fn MakeCompositorFrameMetadata(
            &mut self,
            frame: &FrameData<B>,
            context: &mut HostImplContext<'_, B>,
        ) -> B::CompositorFrameMetadata {
            let mut metadata = B::CompositorFrameMetadata::default();
            if self.settings_.trees_in_viz_in_viz_process {
                metadata.set_frame_token(self.next_frame_token_from_client_);
            } else {
                metadata.set_frame_token(
                    context
                        .services
                        .IncrementFrameToken(&mut self.next_frame_token_),
                );
            }
            let tree = self.active_tree_.as_deref().unwrap();
            metadata.set_device_scale_factor(
                tree.painted_device_scale_factor() * tree.device_scale_factor(),
            );
            metadata.set_page_scale_factor(tree.current_page_scale_factor_for_tree(true));
            metadata.set_scrollable_viewport_size(
                self.WithActiveTree(context, |tree, c| tree.ScrollableViewportSize(c)),
            );
            if let Some(inner) = self
                .active_tree_
                .as_deref()
                .unwrap()
                .InnerViewportScrollNode()
            {
                metadata.set_visible_viewport_size(inner.container_bounds);
            }
            metadata.set_root_background_color(
                self.active_tree_.as_deref().unwrap().background_color(),
            );
            metadata.set_may_throttle_if_undrawn_frames(self.may_throttle_if_undrawn_frames_);
            self.presentation_time_callbacks_
                .RegisterMainThreadCallbacks(
                    metadata.frame_token(),
                    self.active_tree_
                        .as_deref_mut()
                        .unwrap()
                        .TakePresentationCallbacks(),
                );
            self.presentation_time_callbacks_
                .RegisterMainThreadSuccessfulCallbacks(
                    metadata.frame_token(),
                    self.active_tree_
                        .as_deref_mut()
                        .unwrap()
                        .TakeSuccessfulPresentationCallbacks(),
                );
            metadata.set_is_handling_interaction(self.IsHandlingInteraction(context));
            let active_types = context
                .services
                .FrameSequenceTrackerActiveTypes(&mut self.frame_trackers_);
            metadata.set_is_handling_animation(
                context.services.HasMainThreadAnimation(&active_types)
                    || context.services.HasCompositorThreadAnimation(&active_types),
            );
            let referenced = self.active_tree_.as_deref().unwrap().SurfaceRanges();
            for range in referenced {
                metadata.referenced_surfaces_mut().push(range.clone());
            }
            if &self.last_draw_referenced_surfaces_ != referenced {
                self.last_draw_referenced_surfaces_ = referenced.clone();
            }
            metadata.set_min_page_scale_factor(
                self.active_tree_
                    .as_deref()
                    .unwrap()
                    .min_page_scale_factor(),
            );
            if self
                .WithBrowserControls(context, |manager, client| manager.TopControlsHeight(client))
                > 0.0
            {
                let visible = self.WithBrowserControls(context, |manager, client| {
                    manager.TopControlsHeight(client)
                }) * self.WithBrowserControls(context, |manager, client| {
                    manager.TopControlsShownRatio(client)
                });
                metadata.set_top_controls_visible_height(visible);
                // Source: cc/trees/layer_tree_host_impl.cc:2560-2591.
                #[cfg(target_os = "android")]
                {
                    let top_tag = self
                        .browser_controls_offset_manager_
                        .as_deref()
                        .unwrap()
                        .TopControlsOffsetTag();
                    let content_tag = self
                        .browser_controls_offset_manager_
                        .as_deref()
                        .unwrap()
                        .ContentOffsetTag();
                    if !top_tag.IsEmpty() {
                        assert!(!content_tag.IsEmpty());
                        let mut offset = self.WithBrowserControls(context, |manager, client| {
                            manager.TopControlsHeight(client)
                        }) - visible;
                        if visible == 0.0 {
                            offset += self
                                .browser_controls_offset_manager_
                                .as_deref()
                                .unwrap()
                                .TopControlsAdditionalHeight()
                                as f32;
                        }
                        metadata
                            .AppendOffsetTagValue(top_tag, Vector2dF::new(0.0, -offset.round()));
                    }
                    if !content_tag.IsEmpty() {
                        let offset = self.WithBrowserControls(context, |manager, client| {
                            manager.TopControlsHeight(client)
                        }) - visible;
                        metadata.AppendOffsetTagValue(
                            content_tag,
                            Vector2dF::new(0.0, -offset.round()),
                        );
                    }
                }
            }
            // Source: cc/trees/layer_tree_host_impl.cc:2594-2621.
            #[cfg(target_os = "android")]
            if self.WithBrowserControls(context, |manager, client| {
                manager.BottomControlsHeight(client)
            }) > 0.0
            {
                let tag = self
                    .browser_controls_offset_manager_
                    .as_deref()
                    .unwrap()
                    .BottomControlsOffsetTag();
                if !tag.IsEmpty() {
                    let visible = self.WithBrowserControls(context, |manager, client| {
                        manager.BottomControlsHeight(client)
                    }) * self.WithBrowserControls(context, |manager, client| {
                        manager.BottomControlsShownRatio(client)
                    });
                    let mut offset = self.WithBrowserControls(context, |manager, client| {
                        manager.BottomControlsHeight(client)
                    }) - visible;
                    if visible == 0.0 {
                        offset += self
                            .browser_controls_offset_manager_
                            .as_deref()
                            .unwrap()
                            .BottomControlsAdditionalHeight()
                            as f32;
                    }
                    metadata.AppendOffsetTagValue(tag, Vector2dF::new(0.0, offset.round()));
                }
            }
            if self
                .active_tree_
                .as_deref()
                .unwrap()
                .InnerViewportScrollNode()
                .is_some()
            {
                metadata.set_root_scroll_offset(
                    self.active_tree_.as_deref().unwrap().TotalScrollOffset(),
                );
            }
            metadata.set_display_transform_hint(
                self.active_tree_
                    .as_deref()
                    .unwrap()
                    .display_transform_hint(),
            );
            metadata.set_is_mobile_optimized(self.WithActiveTree(context, |tree, c| {
                IsMobileOptimized(&HostNativeTreeView { tree, context: c })
            }));
            if let Some(ink) = self
                .active_tree_
                .as_deref()
                .unwrap()
                .delegated_ink_metadata()
            {
                let mut copy = context.services.CloneDelegatedInkMetadata(ink);
                context
                    .services
                    .SetDelegatedInkFrameTime(&mut copy, self.CurrentBeginFrameArgs().frame_time);
                context.services.TraceDelegatedInkOnCompositorFrame(&copy);
                metadata.set_delegated_ink_metadata(copy);
            }
            metadata.set_capture_bounds(self.CollectRegionCaptureBounds());
            if self.settings_.trees_in_viz_in_viz_process {
                metadata.set_tracked_element_rects(self.tracked_element_rects_from_client_.clone());
            } else {
                metadata.set_tracked_element_rects(self.CollectTrackedElementRects(true,frame.tracked_element_status==crate::cc::trees::frame_data::TrackedElementStatus::kHasTrackedElementsNeedingOcclusion));
            }
            if !self.screenshot_destination_.is_empty() {
                metadata.set_screenshot_destination(self.screenshot_destination_);
                if !self
                    .settings_
                    .TreesInVizInClientProcess(context.services.SettingsFeatures())
                {
                    self.screenshot_destination_ = UnguessableToken::default();
                }
            }
            metadata.set_is_software(self.GetDrawMode(context) != DrawMode::DRAW_MODE_HARDWARE);
            metadata
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2663-2810.
        pub fn MakeRenderFrameMetadata(
            &mut self,
            frame: &FrameData<B>,
            context: &mut HostImplContext<'_, B>,
        ) -> crate::cc::trees::render_frame_metadata::RenderFrameMetadata<B> {
            use crate::adapter::unbounded::LocalSurfaceId;
            use crate::cc::trees::render_frame_metadata::RenderFrameMetadata;
            let mut metadata = RenderFrameMetadata::<B>::default();
            metadata.root_scroll_offset =
                Some(self.active_tree_.as_deref().unwrap().TotalScrollOffset());
            metadata.root_background_color =
                self.active_tree_.as_deref().unwrap().background_color();
            metadata.is_scroll_offset_at_top = self
                .active_tree_
                .as_deref()
                .unwrap()
                .TotalScrollOffset()
                .y()
                == 0.0;
            metadata.device_scale_factor = self
                .active_tree_
                .as_deref()
                .unwrap()
                .painted_device_scale_factor()
                * self.active_tree_.as_deref().unwrap().device_scale_factor();
            self.WithActiveTree(context, |tree, c| {
                tree.GetViewportSelection(&mut metadata.selection, c)
            });
            metadata.is_mobile_optimized = self.WithActiveTree(context, |tree, c| {
                IsMobileOptimized(&HostNativeTreeView { tree, context: c })
            });
            let viewport = self
                .active_tree_
                .as_deref()
                .unwrap()
                .GetDeviceViewportWithExternal(self.external_viewport_);
            metadata.viewport_size_in_pixels = Size::new(viewport.width, viewport.height);
            metadata.page_scale_factor = self
                .active_tree_
                .as_deref()
                .unwrap()
                .current_page_scale_factor_for_tree(true);
            metadata.external_page_scale_factor = self
                .active_tree_
                .as_deref()
                .unwrap()
                .external_page_scale_factor();
            metadata.top_controls_height = self
                .WithBrowserControls(context, |manager, client| manager.TopControlsHeight(client));
            metadata.top_controls_shown_ratio = self
                .WithBrowserControls(context, |manager, client| {
                    manager.TopControlsShownRatio(client)
                });
            // Source: cc/trees/layer_tree_host_impl.cc:2687-2711.
            #[cfg(any(target_os = "android", target_os = "ios"))]
            {
                metadata.bottom_controls_height = self
                    .WithBrowserControls(context, |manager, client| {
                        manager.BottomControlsHeight(client)
                    });
                metadata.bottom_controls_shown_ratio = self
                    .WithBrowserControls(context, |manager, client| {
                        manager.BottomControlsShownRatio(client)
                    });
                metadata.top_controls_min_height_offset = self
                    .browser_controls_offset_manager_
                    .as_deref()
                    .unwrap()
                    .TopControlsMinHeightOffset();
                metadata.bottom_controls_min_height_offset = self
                    .browser_controls_offset_manager_
                    .as_deref()
                    .unwrap()
                    .BottomControlsMinHeightOffset();
                metadata.scrollable_viewport_size =
                    self.WithActiveTree(context, |tree, c| tree.ScrollableViewportSize(c));
                metadata.min_page_scale_factor = self
                    .active_tree_
                    .as_deref()
                    .unwrap()
                    .min_page_scale_factor();
                metadata.max_page_scale_factor = self
                    .active_tree_
                    .as_deref()
                    .unwrap()
                    .max_page_scale_factor();
                metadata.root_layer_size = self.active_tree_.as_deref().unwrap().ScrollableSize();
                if let Some(inner) = self
                    .active_tree_
                    .as_deref()
                    .unwrap()
                    .InnerViewportScrollNode()
                {
                    debug_assert!(self
                        .active_tree_
                        .as_deref()
                        .unwrap()
                        .OuterViewportScrollNode()
                        .is_some());
                    metadata.root_overflow_y_hidden = !self
                        .active_tree_
                        .as_deref()
                        .unwrap()
                        .OuterViewportScrollNode()
                        .unwrap()
                        .user_scrollable_vertical
                        || !inner.user_scrollable_vertical;
                }
                metadata.has_transparent_background = frame
                    .render_passes
                    .last()
                    .unwrap()
                    .has_transparent_background();
                metadata.has_offset_tag = self
                    .browser_controls_offset_manager_
                    .as_deref()
                    .unwrap()
                    .HasOffsetTag();
            }
            let mut allocate_new_local_surface_id = false;
            if frame.tracked_element_status
                != crate::cc::trees::frame_data::TrackedElementStatus::kNone
            {
                metadata.tracked_element_rects=self.CollectTrackedElementRects(false,frame.tracked_element_status==crate::cc::trees::frame_data::TrackedElementStatus::kHasTrackedElementsNeedingOcclusion);
            }
            if let Some(last) = self.last_draw_render_frame_metadata_.as_ref() {
                let old_y = last.root_scroll_offset.unwrap_or_default().y();
                let new_y = metadata.root_scroll_offset.unwrap().y();
                if !MathUtil::IsWithinEpsilon(old_y, new_y) {
                    let direction = if old_y < new_y {
                        VerticalScrollDirection::kDown
                    } else {
                        VerticalScrollDirection::kUp
                    };
                    if self.last_vertical_scroll_direction_ != direction {
                        metadata.new_vertical_scroll_direction = direction;
                    }
                }
                // Source: cc/trees/layer_tree_host_impl.cc:2747-2792.
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                {
                    allocate_new_local_surface_id = last.top_controls_height
                        != metadata.top_controls_height
                        || last.top_controls_shown_ratio != metadata.top_controls_shown_ratio
                        || last.tracked_element_rects != metadata.tracked_element_rects;
                }
                #[cfg(target_os = "android")]
                {
                    allocate_new_local_surface_id = last.top_controls_height
                        != metadata.top_controls_height
                        || last.bottom_controls_height != metadata.bottom_controls_height
                        || last.selection != metadata.selection
                        || last.has_transparent_background != metadata.has_transparent_background;
                    allocate_new_local_surface_id |=
                        last.has_offset_tag && !metadata.has_offset_tag;
                    if !self
                        .browser_controls_offset_manager_
                        .as_deref()
                        .unwrap()
                        .HasOffsetTag()
                    {
                        allocate_new_local_surface_id |= last.top_controls_shown_ratio
                            != metadata.top_controls_shown_ratio
                            || last.bottom_controls_shown_ratio
                                != metadata.bottom_controls_shown_ratio;
                    }
                }
                #[cfg(target_os = "ios")]
                {
                    allocate_new_local_surface_id = last.top_controls_height
                        != metadata.top_controls_height
                        || last.top_controls_shown_ratio != metadata.top_controls_shown_ratio
                        || last.bottom_controls_height != metadata.bottom_controls_height
                        || last.bottom_controls_shown_ratio != metadata.bottom_controls_shown_ratio
                        || last.selection != metadata.selection
                        || last.has_transparent_background != metadata.has_transparent_background;
                }
            }
            if self.GetCurrentLocalSurfaceId(context).is_valid() {
                if allocate_new_local_surface_id && !self.settings_.trees_in_viz_in_viz_process {
                    self.AllocateLocalSurfaceId();
                }
                metadata.local_surface_id = Some(self.GetCurrentLocalSurfaceId(context).clone());
            }
            metadata.primary_main_frame_item_sequence_number = self
                .active_tree_
                .as_deref()
                .unwrap()
                .primary_main_frame_item_sequence_number();
            metadata
        }
        // Source: cc/trees/layer_tree_host_impl.cc:2812-3072.
        pub fn DrawLayers(
            &mut self,
            frame: &mut FrameData<B>,
            context: &mut HostImplContext<'_, B>,
        ) -> Option<crate::cc::metrics::submit_info::SubmitInfo<B::EventMetricsSet>>
        {
            debug_assert!(self.CanDraw(context));
            debug_assert_eq!(frame.has_no_damage, frame.render_passes.is_empty());
            self.ResetRequiresHighResToDraw();
            if frame.has_no_damage {
                debug_assert!(!self.resourceless_software_draw_);
                context.services.TraceNoDamageEarlyOut();
                self.WithActiveTree(context, |tree, c| {
                    tree.BreakSwapPromises(
                        crate::adapter::layer_tree_impl::DidNotSwapReason::SWAP_FAILS,
                        c,
                    )
                });
                if self
                    .settings_
                    .TreesInVizInClientProcess(context.services.SettingsFeatures())
                {
                    self.UpdateDisplayTree(
                        frame,
                        Vec::new(),
                        <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects::default(),
                        false,
                        context,
                    );
                }
                self.active_tree_
                    .as_deref_mut()
                    .unwrap()
                    .ResetAllChangeTracking();
                if self.settings_.is_layer_tree_for_ui {
                    drop(
                        self.active_tree_
                            .as_deref_mut()
                            .unwrap()
                            .TakeEventsMetrics(),
                    );
                    drop(
                        self.active_tree_
                            .as_deref_mut()
                            .unwrap()
                            .TakeRasterEventsMetrics(),
                    );
                    drop(
                        context
                            .services
                            .TakeSavedEventsMetrics(&mut self.events_metrics_manager_),
                    );
                }
                return None;
            }
            context.services.FrameSinkSetSourceFrameNumber(
                self.layer_tree_frame_sink_.unwrap(),
                self.active_tree_.as_deref().unwrap().source_frame_number(),
            );
            let mut drawn_with_new_layer_tree = true;
            if self.last_draw_active_tree_begin_frame_args_.IsValid() {
                drawn_with_new_layer_tree = self
                    .active_tree_
                    .as_deref()
                    .unwrap()
                    .CreatedBeginFrameArgs()
                    .frame_id
                    != self.last_draw_active_tree_begin_frame_args_.frame_id;
            }
            self.last_draw_active_tree_begin_frame_args_ = self
                .active_tree_
                .as_deref()
                .unwrap()
                .CreatedBeginFrameArgs()
                .clone();
            let mut compositor_frame = self.GenerateCompositorFrame(frame, context);
            let frame_token = compositor_frame.metadata().frame_token();
            frame.frame_token = frame_token;
            if self.dump_compositor_frame_
                && frame_token >= self.dump_compositor_frame_begin_
                && frame_token <= self.dump_compositor_frame_end_
            {
                context.services.PostCompositorFrameDump(
                    &compositor_frame,
                    self.settings_
                        .TreesInVizInClientProcess(context.services.SettingsFeatures()),
                    self.settings_.trees_in_viz_in_viz_process,
                    frame_token,
                );
            }
            let mut top_controls_moved = false;
            let current_top_controls = compositor_frame
                .metadata()
                .top_controls_visible_height()
                .unwrap_or(0.);
            if current_top_controls != self.top_controls_visible_height_ {
                top_controls_moved = true;
                self.top_controls_visible_height_ = current_top_controls;
            }
            #[cfg(debug_assertions)]
            let begin_frame_ack_frame_id = compositor_frame.metadata().begin_frame_ack().frame_id();
            let main_events = self
                .active_tree_
                .as_deref_mut()
                .unwrap()
                .TakeEventsMetrics();
            let saved_events = context
                .services
                .TakeSavedEventsMetrics(&mut self.events_metrics_manager_);
            let raster_events = self
                .active_tree_
                .as_deref_mut()
                .unwrap()
                .TakeRasterEventsMetrics();
            let events_metrics =
                context
                    .services
                    .MakeEventMetricsSet(main_events, saved_events, raster_events);
            context.services.CollectScrollEventsFromFrame(
                &mut self.lag_tracking_manager_,
                frame_token,
                &events_metrics,
            );
            let mut normalized_invalidated_area = None;
            if self.settings_.is_layer_tree_for_ui {
                assert!(self.total_invalidated_area_.is_some());
                let output = crate::adapter::draw_property::DrawSurface::content_rect(
                    self.active_tree_
                        .as_deref()
                        .unwrap()
                        .RootRenderSurface()
                        .unwrap(),
                );
                let area = self.total_invalidated_area_.unwrap();
                normalized_invalidated_area = Some(
                    if area.IsValid() {
                        area.ValueOrDie()
                    } else {
                        u32::MAX
                    } as f32
                        / (output.width * output.height) as f32,
                );
                self.total_invalidated_area_ = Some(CheckedU32::FromValue(0));
            }
            if VerboseLogEnabled(context.services) {
                context
                    .services
                    .VerboseLogCompositorFrame(&compositor_frame);
            }
            let submit_time = context.services.MonotonicTimeNow();
            let mut trees_in_viz_submit_time = TimeTicks::default();
            if self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures())
            {
                self.send_frame_token_to_embedder_ =
                    compositor_frame.metadata().send_frame_token_to_embedder();
                let latency = compositor_frame.metadata_mut().TakeLatencyInfo();
                let tracked = compositor_frame.metadata_mut().TakeTrackedElementRects();
                trees_in_viz_submit_time =
                    self.UpdateDisplayTree(frame, latency, tracked, false, context);
                context
                    .services
                    .FrameSinkExportFrameTiming(self.layer_tree_frame_sink_.unwrap());
            } else {
                context.services.TraceSubmitCompositorFrame(
                    self.CurrentBeginFrameArgs().trace_id,
                    compositor_frame.metadata().latency_info(),
                );
                if self.settings_.trees_in_viz_in_viz_process {
                    compositor_frame
                        .metadata_mut()
                        .set_submit_compositor_frame_time(submit_time);
                }
                context.services.FrameSinkSubmitCompositorFrame(
                    self.layer_tree_frame_sink_.unwrap(),
                    compositor_frame,
                    false,
                );
                if self.new_local_surface_id_expected_ {
                    context
                        .services
                        .FrameSinkNotifyNewLocalSurfaceIdExpectedWhilePaused(
                            self.layer_tree_frame_sink_.unwrap(),
                        );
                }
            }
            #[cfg(debug_assertions)]
            if !self.doing_sync_draw_ {
                debug_assert_eq!(
                    self.current_begin_frame_tracker_.Current().frame_id,
                    begin_frame_ack_frame_id
                );
            }
            if !context
                .services
                .MutatorNextFrameHasPendingRAF(&self.mutator_host_)
            {
                context
                    .services
                    .StopDrawSequence(&mut self.frame_trackers_, HostDrawSequence::kRAF);
            }
            if !context
                .services
                .MutatorHasCanvasInvalidation(&self.mutator_host_)
            {
                context.services.StopDrawSequence(
                    &mut self.frame_trackers_,
                    HostDrawSequence::kCanvasAnimation,
                );
            }
            if !context
                .services
                .MutatorNextFrameHasPendingRAF(&self.mutator_host_)
                && !context.services.MutatorHasJSAnimation(&self.mutator_host_)
            {
                context
                    .services
                    .StopDrawSequence(&mut self.frame_trackers_, HostDrawSequence::kJSAnimation);
            }
            if context
                .services
                .MutatorMainThreadAnimationsCount(&self.mutator_host_)
                == 0
                && !context
                    .services
                    .MutatorHasSmilAnimation(&self.mutator_host_)
            {
                context.services.StopDrawSequence(
                    &mut self.frame_trackers_,
                    HostDrawSequence::kMainThreadAnimation,
                );
                context.services.StopDrawSequence(
                    &mut self.frame_trackers_,
                    HostDrawSequence::kSETMainThreadAnimation,
                );
            } else if !context
                .services
                .MutatorHasViewTransition(&self.mutator_host_)
            {
                context.services.StopDrawSequence(
                    &mut self.frame_trackers_,
                    HostDrawSequence::kSETMainThreadAnimation,
                );
            }
            self.active_tree_
                .as_deref_mut()
                .unwrap()
                .ClearSwapPromises();
            if frame.has_copy_requests {
                self.active_tree_
                    .as_deref_mut()
                    .unwrap()
                    .property_trees_mut()
                    .effect_tree_mutable()
                    .ClearCopyRequests();
                self.active_tree_
                    .as_deref_mut()
                    .unwrap()
                    .set_needs_update_draw_properties();
                self.SetNeedsCommit(context);
            }
            assert_eq!(
                frame.render_surface_list,
                Some(self.active_tree_.as_deref().unwrap().id)
            );
            self.WithActiveTreeDamage(context, |damage| {
                for effect_id in damage.GetRenderSurfaceList() {
                    damage
                        .damage_tracker(damage.GetRenderSurface(effect_id).unwrap())
                        .DidDrawDamagedArea();
                }
            });
            if self
                .active_tree_
                .as_deref()
                .unwrap()
                .RootRenderSurface()
                .is_some()
            {
                self.viewport_damage_rect_ = Rect::default();
                self.root_layer_damage_rect_ = Rect::default();
            }
            self.active_tree_
                .as_deref_mut()
                .unwrap()
                .ResetAllChangeTracking();
            if !self.settings_.is_layer_tree_for_ui {
                context.services.DidDrawDevtoolsFrame(
                    self.id_,
                    frame.begin_frame_ack.frame_id().sequence_number,
                );
            }
            if !self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures())
            {
                context
                    .services
                    .IssueImplThreadRenderingStatsEvent(self.rendering_stats_instrumentation_);
            }
            if self.settings_.enable_compositing_based_throttling
                && self.throttle_decider_.HasThrottlingChanged()
            {
                context
                    .services
                    .FrameSinksToThrottleUpdated(self.delegate_, self.throttle_decider_.ids());
            }
            if context.services.NewContentForCheckerboardedScrollsEnabled() {
                let param = context.services.NewContentForCheckerboardedScrollsMode();
                if param == HostCheckerboardMode::kPerScroll {
                    if self.GetActivelyScrollingType(context)!=crate::cc::input::actively_scrolling_type::ActivelyScrollingType::kNone&&frame.checkerboarded_needs_record {self.prioritize_new_content_due_to_checkerboarding_=true;}
                } else if param == HostCheckerboardMode::kPerFrame {
                    self.prioritize_new_content_due_to_checkerboarding_=self.GetActivelyScrollingType(context)!=crate::cc::input::actively_scrolling_type::ActivelyScrollingType::kNone&&frame.checkerboarded_needs_record;
                }
            }
            Some(
                crate::cc::metrics::submit_info::SubmitInfo::new(
                    frame_token,
                    submit_time,
                    frame.checkerboarded_needs_raster,
                    frame.checkerboarded_needs_record,
                    top_controls_moved,
                    events_metrics,
                    drawn_with_new_layer_tree,
                    self.active_tree_
                        .as_deref()
                        .unwrap()
                        .did_raster_inducing_scroll(),
                    normalized_invalidated_area,
                    trees_in_viz_submit_time,
                ),
            )
        }
        // Source: cc/trees/layer_tree_host_impl.cc:3074-3419.
        pub fn GenerateCompositorFrame(
            &mut self,
            frame: &mut FrameData<B>,
            context: &mut HostImplContext<'_, B>,
        ) -> B::Frame {
            if !self.settings_.trees_in_viz_in_viz_process {
                self.memory_history_
                    .as_deref_mut()
                    .unwrap()
                    .SaveEntry(context.services.TileManagerMemoryStats(&self.tile_manager_));
            }
            if self.debug_state_.ShowDebugRects() {
                let tree = self.active_tree_.as_deref_mut().unwrap();
                let hud = tree
                    .hud_layer()
                    .map(|layer| crate::adapter::layer_tree_impl::LayerTreeLayer::id(layer));
                let have_scroll_event_handlers = tree.have_scroll_event_handlers();
                let state = tree.RenderPassState();
                let mut debug_context = B::MakeDebugDrawContext(HostDebugDrawState {
                    layers: state.layer_list,
                    have_scroll_event_handlers,
                });
                self.debug_rect_history_
                    .as_deref_mut()
                    .unwrap()
                    .SaveDebugRectsForCurrentFrame(
                        &mut debug_context,
                        state.property_trees,
                        hud,
                        state.render_surface_list,
                        &self.debug_state_,
                    );
            }
            context
                .services
                .TraceScrollDeltaThisFrame(self.scroll_accumulated_this_frame_);
            self.scroll_accumulated_this_frame_ = Vector2dF::default();
            if context.services.IsNewTrace() {
                if let Some(tree) = self.pending_tree_.as_deref_mut() {
                    for layer in tree.begin_mut() {
                        HostMetadataLayer::<B>::DidBeginTracing(layer);
                    }
                }
                for layer in self.active_tree_.as_deref_mut().unwrap().begin_mut() {
                    HostMetadataLayer::<B>::DidBeginTracing(layer);
                }
            }
            {
                let trace=context.services.HostTraceCategoryEnabled("disabled-by-default-cc.debug,disabled-by-default-viz.quads,disabled-by-default-devtools.timeline.layers");
                if trace {
                    let value = self.AsValueWithFrame(Some(frame), context);
                    context.services.TraceFrameViewerSnapshot(self.id_, &value);
                }
            }
            let draw_mode = self.GetDrawMode(context);
            if let Some(hud) = self
                .active_tree_
                .as_deref()
                .unwrap()
                .hud_layer()
                .map(|layer| crate::adapter::layer_tree_impl::LayerTreeLayer::id(layer))
            {
                let layer = self
                    .active_tree_
                    .as_deref_mut()
                    .unwrap()
                    .LayerByIdMutable(hud)
                    .unwrap();
                context.services.UpdateHudTexture(
                    layer,
                    draw_mode,
                    self.layer_tree_frame_sink_.unwrap(),
                    self.resource_provider_.as_deref_mut(),
                    &self.raster_caps_,
                    &mut frame.render_passes,
                );
            }
            let mut metadata = self.MakeCompositorFrameMetadata(frame, context);
            let mut has_view_transition_with_animate = false;
            let mut delay_layer_tree_view_deletion = false;
            if !self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures())
            {
                let mut elements = BTreeMap::new();
                let tokens =
                    self.WithActiveTree(context, |tree, c| tree.GetCaptureViewTransitionTokens(c));
                let tree = self.active_tree_.as_deref().unwrap();
                for &effect_id in frame.RenderSurfaceList(tree).unwrap() {
                    let surface = tree.GetRenderSurface(effect_id).unwrap();
                    let id = tree
                        .property_trees()
                        .effect_tree()
                        .Node(effect_id)
                        .view_transition_element_resource_id;
                    if !id.IsValid() {
                        continue;
                    }
                    debug_assert!(
                        !elements.contains_key(&id),
                        "Cannot map a view-transition resource to two render passes"
                    );
                    if id.MatchesToken(&tokens) {
                        elements.insert(id, surface.view_transition_capture_render_pass_id());
                    } else {
                        elements.insert(id, surface.render_pass_id());
                    }
                }
                let display_colors = self.GetDisplayColorSpaces();
                // The native range-for extends the returned owning vector through
                // the whole loop. Borrow its unique_ptr elements so every request
                // is destroyed at that same end-of-loop scope, not each iteration.
                let requests = self
                    .active_tree_
                    .as_deref_mut()
                    .unwrap()
                    .TakeViewTransitionRequests(true);
                for request in &requests {
                    if self.resourceless_software_draw_ {
                        self.OnCompositorFrameTransitionDirectiveProcessed(
                            request.sequence_id(),
                            context,
                        );
                    } else {
                        metadata.transition_directives_mut().push(
                            context.services.ConstructTransitionDirective(
                                request,
                                &elements,
                                &display_colors,
                                HostMetadataTransitionRequest::delay_layer_tree_view_deletion(
                                    request.as_ref(),
                                ),
                            ),
                        );
                        if HostMetadataTransitionRequest::delay_layer_tree_view_deletion(
                            request.as_ref(),
                        ) {
                            self.OnCompositorFrameTransitionDirectiveProcessed(
                                request.sequence_id(),
                                context,
                            );
                            if HostMetadataTransitionRequest::IsAnimateRenderer(request.as_ref()) {
                                has_view_transition_with_animate = true;
                            }
                            delay_layer_tree_view_deletion = true;
                        }
                    }
                }
            } else {
                if self
                    .active_tree_
                    .as_deref()
                    .unwrap()
                    .HasViewTransitionRequests()
                {
                    self.active_tree_
                        .as_deref_mut()
                        .unwrap()
                        .set_needs_update_draw_properties();
                }
                // Native request objects remain resident in the Viz-client branch.
                for index in 0..self
                    .active_tree_
                    .as_deref()
                    .unwrap()
                    .view_transition_requests()
                    .len()
                {
                    let request = &self
                        .active_tree_
                        .as_deref()
                        .unwrap()
                        .view_transition_requests()[index];
                    if HostMetadataTransitionRequest::delay_layer_tree_view_deletion(
                        request.as_ref(),
                    ) {
                        let sequence = request.sequence_id();
                        let animate =
                            HostMetadataTransitionRequest::IsAnimateRenderer(request.as_ref());
                        self.OnCompositorFrameTransitionDirectiveProcessed(sequence, context);
                        if animate {
                            has_view_transition_with_animate = true;
                        }
                        delay_layer_tree_view_deletion = true;
                    }
                }
            }
            self.WithActiveTree(context, |tree, c| {
                PopulateMetadataContentColorUsage(
                    &HostNativeTreeView { tree, context: c },
                    frame,
                    &mut metadata,
                )
            });
            metadata.set_has_shared_element_resources(frame.has_shared_element_resources);
            let mut frame_deadline = frame.deadline_in_frames.unwrap_or(0);
            if delay_layer_tree_view_deletion && has_view_transition_with_animate {
                frame_deadline = 240;
            }
            metadata.set_deadline(context.services.MakeFrameDeadline(
                self.CurrentBeginFrameArgs().frame_time,
                frame_deadline,
                self.CurrentBeginFrameArgs().interval,
                frame.use_default_lower_bound_deadline,
            ));
            metadata.set_interval_frame_time(self.CurrentBeginFrameArgs().frame_time);
            metadata.set_interval_has_input(self.has_input_for_frame_interval_);
            metadata.set_interval_has_user_input(self.has_non_fling_input_since_last_frame_);
            self.has_non_fling_input_since_last_frame_ = false;
            use crate::cc::trees::damage_reason::DamageReason;
            if frame.damage_reasons.Has(DamageReason::kCompositorScroll) {
                if self.begin_frame_time_delta_.InMicroseconds() < 100
                    || self.begin_frame_time_delta_.InMicroseconds() > 1000000
                {
                    metadata.set_major_scroll_speed(0.);
                } else {
                    metadata.set_major_scroll_speed(
                        (self.frame_max_scroll_delta_ as f64
                            / self.begin_frame_time_delta_.InSecondsF())
                            as f32,
                    );
                }
                metadata.AppendContentInterval(
                    HostContentFrameIntervalType::kCompositorScroll,
                    TimeDelta::default(),
                    1,
                );
                frame.damage_reasons.Remove(DamageReason::kCompositorScroll);
            }
            if !frame.video_layer_preferred_intervals.is_empty()
                && frame.damage_reasons.Has(DamageReason::kVideoLayer)
            {
                for (&interval, &count) in &frame.video_layer_preferred_intervals {
                    metadata.AppendContentInterval(
                        HostContentFrameIntervalType::kVideo,
                        interval,
                        count - 1,
                    );
                }
                frame.damage_reasons.Remove(DamageReason::kVideoLayer);
            }
            if frame.damage_reasons.Has(DamageReason::kAnimatedImage) {
                if !self.settings_.trees_in_viz_in_viz_process {
                    let duration = self
                        .image_animation_controller_
                        .as_mut()
                        .expect("image animation controller")
                        .GetConsistentContentFrameDuration();
                    if let Some(duration) = duration {
                        metadata.AppendContentInterval(
                            HostContentFrameIntervalType::kAnimatingImage,
                            duration.frame_duration,
                            duration.num_images - 1,
                        );
                        frame.damage_reasons.Remove(DamageReason::kAnimatedImage);
                    }
                }
            }
            if frame
                .damage_reasons
                .Has(DamageReason::kScrollbarFadeOutAnimation)
            {
                metadata.AppendContentInterval(
                    HostContentFrameIntervalType::kScrollBarFadeOutAnimation,
                    TimeDelta::Microseconds(50_000),
                    0,
                );
                frame
                    .damage_reasons
                    .Remove(DamageReason::kScrollbarFadeOutAnimation);
            }
            if frame.trees_in_viz_timing_details.is_some()
                && self.settings_.trees_in_viz_in_viz_process
            {
                // Native TreesInVizTiming is four TimeTicks: its implicit move copies
                // these scalar values and leaves the source optional engaged.
                metadata.set_trees_in_viz_timing_details(
                    frame.trees_in_viz_timing_details.as_ref().unwrap().clone(),
                );
            }
            metadata.set_has_only_content_frame_interval_updates(frame.damage_reasons.empty());
            metadata
                .set_activation_dependencies(std::mem::take(&mut frame.activation_dependencies));
            self.active_tree_
                .as_deref_mut()
                .unwrap()
                .FinishSwapPromises(&mut metadata);
            debug_assert_eq!(metadata.frame_token(), self.next_frame_token(context));
            if self.render_frame_metadata_observer_.is_some()
                || self.settings_.trees_in_viz_in_viz_process
            {
                self.last_draw_render_frame_metadata_ =
                    Some(self.MakeRenderFrameMetadata(frame, context));
                if let Some(ink) = metadata.delegated_ink_metadata() {
                    self.last_draw_render_frame_metadata_.as_mut().unwrap().delegated_ink_metadata=Some(crate::cc::trees::render_frame_metadata::DelegatedInkBrowserMetadata::new(context.services.DelegatedInkIsHovering(ink)));
                }
                if self
                    .last_draw_render_frame_metadata_
                    .as_ref()
                    .unwrap()
                    .new_vertical_scroll_direction
                    != crate::adapter::render_metadata::VerticalScrollDirection::kNull
                {
                    self.last_vertical_scroll_direction_ = self
                        .last_draw_render_frame_metadata_
                        .as_ref()
                        .unwrap()
                        .new_vertical_scroll_direction;
                }
                if let Some(observer) = self.render_frame_metadata_observer_.as_deref_mut() {
                    debug_assert!(!self.settings_.trees_in_viz_in_viz_process);
                    context.services.ObserveRenderFrameSubmission(
                        observer,
                        self.last_draw_render_frame_metadata_.as_ref().unwrap(),
                        &mut metadata,
                        self.active_tree_
                            .as_deref_mut()
                            .unwrap()
                            .TakeForceSendMetadataRequest(),
                    );
                }
                if self.settings_.trees_in_viz_in_viz_process {
                    metadata.set_send_frame_token_to_embedder(self.send_frame_token_to_embedder_);
                }
            }
            if !self.CommitsToActiveTree() && !metadata.latency_info().is_empty() {
                let time = context.services.MonotonicTimeNow();
                self.ApplyFirstScrollTracking(
                    &metadata.latency_info()[0],
                    metadata.frame_token(),
                    context,
                );
                for latency in metadata.latency_info_mut() {
                    context
                        .services
                        .AddRendererSwapLatencyTimestamp(latency, time);
                }
            }
            if !frame.unbounded_render_passes.is_empty()
                && self.unbounded_frame_sink_handler_.is_some()
            {
                assert!(!self
                    .settings_
                    .TreesInVizInClientProcess(context.services.SettingsFeatures()));
                assert!(self.settings_.enable_unbounded_element);
                let mut unbounded = B::Frame::default();
                let mut unbounded_metadata = self.MakeCompositorFrameMetadata(frame, context);
                unbounded_metadata.set_begin_frame_ack(frame.begin_frame_ack.clone());
                unbounded.set_metadata(unbounded_metadata);
                unbounded.set_render_passes(std::mem::take(&mut frame.unbounded_render_passes));
                Self::PopulateFrameResources(
                    &mut unbounded,
                    None,
                    self.resource_provider_.as_deref_mut().unwrap(),
                    self.layer_tree_frame_sink_.unwrap(),
                    context,
                );
                let mut returns = HostUnboundedResourceReturn::<B> {
                    provider: self.resource_provider_.as_deref_mut().unwrap(),
                    pool: self.resource_pool_.as_deref_mut(),
                    release: UIResourceReleaseClient {
                        owner: self.owner_id_,
                        deleted: &mut self.deleted_ui_resources_,
                    },
                    services: context.services,
                };
                self.unbounded_frame_sink_handler_
                    .as_mut()
                    .unwrap()
                    .SubmitFrame(unbounded, &mut returns);
            }
            debug_assert!(frame.begin_frame_ack.IsSequenceValid());
            metadata.set_begin_frame_ack(frame.begin_frame_ack.clone());
            let mut compositor_frame = B::Frame::default();
            compositor_frame.set_metadata(metadata);
            Self::PopulateFrameResources(
                &mut compositor_frame,
                Some(&frame.render_passes),
                self.resource_provider_.as_deref_mut().unwrap(),
                self.layer_tree_frame_sink_.unwrap(),
                context,
            );
            compositor_frame.set_render_passes(std::mem::take(&mut frame.render_passes));
            assert!(
                !self.settings_.single_thread_proxy_scheduler
                    || self
                        .active_tree_
                        .as_deref()
                        .unwrap()
                        .local_surface_id_from_parent()
                        .is_valid()
            );
            if !self.settings_.trees_in_viz_in_viz_process {
                let id = self.GetCurrentLocalSurfaceId(context);
                context
                    .services
                    .FrameSinkSetCurrentLocalSurfaceId(self.layer_tree_frame_sink_.unwrap(), id);
            }
            self.last_draw_local_surface_id_ = self.GetCurrentLocalSurfaceId(context).clone();
            if let Some(name) = context.services.GetClientNameForMetrics() {
                let total = compositor_frame
                    .render_passes()
                    .iter()
                    .map(|pass| pass.quad_count())
                    .sum();
                context.services.RecordCompositorFrameQuadCount(name, total);
            }
            compositor_frame
        }
        // Source: cc/trees/layer_tree_host_impl.cc:3347-3361 (native local closure).
        fn PopulateFrameResources(
            frame: &mut B::Frame,
            passes: Option<&[Box<B::RenderPass>]>,
            provider: &mut B::ClientResourceProvider,
            sink: HostImplFrameSinkId,
            context: &mut HostImplContext<'_, B>,
        ) {
            let mut resources = Vec::new();
            for pass in passes.unwrap_or_else(|| frame.render_passes()) {
                for index in 0..pass.quad_count() {
                    let resource = pass.quad_resource_id(index);
                    if resource != crate::adapter::layer_tree_impl::ResourceId::default() {
                        resources.push(resource);
                    }
                }
            }
            let interface = context.services.FrameSinkSharedImageInterface(sink);
            context.services.PrepareSendResourcesToParent(
                provider,
                &resources,
                frame.resource_list_mut(),
                interface,
            );
        }
        // Source: cc/trees/layer_tree_host_impl.cc:3421-3433.
        pub fn DidDrawAllLayers(
            &mut self,
            frame: &FrameData<B>,
            context: &mut HostImplContext<'_, B>,
        ) {
            for &id in &frame.will_draw_layers {
                self.WithActiveTree(context, |tree, tree_context| {
                    B::WithPictureLayersContext(
                        tree,
                        tree_context.host,
                        |storage, picture_context| {
                            let layer = crate::adapter::layer_tree_impl::LayerTreeLayerStorage::LayerByIdMut(
                                storage, id,
                            )
                            .expect("native will-draw layer must still reside in active tree");
                            HostMetadataLayer::<B>::DidDraw(layer, picture_context);
                        },
                    )
                });
            }
            for &controller in &self.video_frame_controllers_ {
                context
                    .services
                    .VideoFrameControllerDidDrawFrame(controller);
            }
        }
        // Source: cc/trees/layer_tree_host_impl.cc:3435-3450.
        pub fn UpdateDisplayTree(
            &mut self,
            frame: &mut FrameData<B>,
            latency_info: Vec<B::LatencyInfo>,
            tracked_element_rects: <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects,
            is_flush: bool,
            context: &mut HostImplContext<'_, B>,
        ) -> TimeTicks {
            debug_assert!(self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures()));
            debug_assert!(self.layer_context_.is_some());
            let interface = context
                .services
                .FrameSinkSharedImageInterface(self.layer_tree_frame_sink_.unwrap());
            context.services.LayerContextUpdateDisplayTreeFrom(
                self.layer_context_.as_deref_mut().unwrap(),
                self.active_tree_.as_deref_mut().unwrap(),
                self.resource_provider_.as_deref_mut().unwrap(),
                interface,
                self.viewport_damage_rect_,
                !frame.has_no_damage,
                is_flush,
                latency_info,
                tracked_element_rects,
            )
        }
        // Source: cc/trees/layer_tree_host_impl.cc:3452-3491.
        pub fn FlushDisplayTree(&mut self, context: &mut HostImplContext<'_, B>) {
            assert!(self
                .settings_
                .TreesInVizInClientProcess(context.services.SettingsFeatures()));
            assert!(self.layer_context_.is_some());
            let mut frame = FrameData::<B>::default();
            frame.origin_begin_main_frame_args =
                crate::adapter::begin_frame::BeginFrameArgs::default();
            context
                .services
                .IncrementFrameToken(&mut self.next_frame_token_);
            self.UpdateDisplayTree(
                &mut frame,
                Vec::new(),
                <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects::default(),
                true,
                context,
            );
            if self
                .active_tree_
                .as_deref()
                .unwrap()
                .RootRenderSurface()
                .is_some()
            {
                self.viewport_damage_rect_ = Rect::default();
                self.root_layer_damage_rect_ = Rect::default();
            }
            self.active_tree_
                .as_deref_mut()
                .unwrap()
                .ResetAllChangeTracking();
        }
    }

    // Borrow-only view of the actual tree for source cc204–211/227–238.
    struct HostNativeTreeView<'a, 'c, B: HostImplTypes> {
        tree: &'a LayerTreeImpl<B>,
        context: &'a LayerTreeContext<'c, B>,
    }
    impl<'a, B: HostImplTypes> HostImplTreeView for HostNativeTreeView<'a, '_, B> {
        type Layer = B::Layer;
        fn LayerById(&self, id: i32) -> Option<&Self::Layer> {
            self.tree.LayerById(id)
        }
        fn min_page_scale_factor(&self) -> f32 {
            self.tree.min_page_scale_factor()
        }
        fn max_page_scale_factor(&self) -> f32 {
            self.tree.max_page_scale_factor()
        }
        fn current_page_scale_factor(&self) -> f32 {
            self.tree.current_page_scale_factor_for_tree(true)
        }
        fn ScrollableViewportSize(&self) -> SizeF {
            self.tree.ScrollableViewportSize(self.context)
        }
        fn ScrollableSize(&self) -> SizeF {
            self.tree.ScrollableSize()
        }
        fn viewport_mobile_optimized(&self) -> bool {
            self.tree.viewport_mobile_optimized(self.context)
        }
    }
    impl<B: HostImplTypes> HostImplFrameLayerList for FrameData<B> {
        fn will_draw_layers(&self) -> &[i32] {
            &self.will_draw_layers
        }
    }

    // Synchronous callback into the actual resource provider only. The native
    // SubmitFrame path cannot access or retain a tree/host/handler through this facet.
    struct HostUnboundedResourceReturn<'a, B: HostImplTypes> {
        provider: &'a mut B::ClientResourceProvider,
        pool: Option<&'a mut B::ResourcePool>,
        release: UIResourceReleaseClient<'a, B>,
        services: &'a mut dyn HostImplResourceReturnServices<B>,
    }
    impl<B: HostImplTypes> crate::adapter::unbounded::UnboundedResourceHost<B>
        for HostUnboundedResourceReturn<'_, B>
    {
        fn ReceiveReturnsFromParent(&mut self, resources: Vec<B::ReturnedResource>) {
            self.services.ReceiveReturnsFromParent(
                self.provider,
                resources,
                self.pool.as_deref_mut(),
                &mut self.release,
            );
        }
    }
}

// Source owner split for PictureLayerImpl's synchronous per-call environment.
// The slot conversion lends resident fields; Selected carries identity only.
impl<B: HostImplTypes> HostImplTreeSlot<'_, B> {
    fn PictureSlot(&mut self) -> crate::adapter::host_picture::HostPictureTreeSlot<'_, B> {
        use crate::adapter::host_picture::HostPictureTreeSlot as Slot;
        match self {
            Self::Resident(slot) => Slot::Resident(&mut **slot),
            Self::Direct(tree) => Slot::Direct(&mut **tree),
            Self::Reserved(id) => Slot::Selected(*id),
        }
    }
}

impl<B: HostImplTypes> crate::adapter::picture_layer_impl::PictureLayerContextHost<B>
    for HostImplTreeHostFacade<'_, '_, B>
{
    fn WithPictureContextParts(
        &mut self,
        selected: crate::adapter::layer_tree_impl::LayerTreeId,
        callback: &mut dyn FnMut(
            &dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerTreeHost<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
        ),
    ) where
        B: crate::adapter::picture_layer_impl::PictureLayerImplTypes,
    {
        let fields = crate::adapter::host_picture::HostPictureFields {
            owner_id: *self.core.owner_id_,
            selected_id: selected,
            active: self.active_tree_.PictureSlot(),
            pending: self.pending_tree_.PictureSlot(),
            recycle: self.recycle_tree_.PictureSlot(),
            settings: self.core.settings_,
            debug_state: self.core.debug_state_,
            raster_caps: self.core.raster_caps_,
            global_tile_state: self.core.global_tile_state_,
            resource_pool: self.core.resource_pool_,
            resource_provider: self.core.resource_provider_,
            layer_context: self.core.layer_context_,
            latency_monitors: self.core.latency_info_swap_promise_monitor_,
            events_metrics: self.core.events_metrics_manager_,
            tile_manager: self.core.tile_manager_,
            image_animation_controller: self.image_animation_controller_,
            paint_worklet_tracker: self.core.paint_worklet_tracker_,
            requires_high_res_to_draw: self.core.requires_high_res_to_draw_,
            external_viewport: self.core.external_viewport_,
            viewport_rect_for_tile_priority: self.core.viewport_rect_for_tile_priority_,
            layer_tree_frame_sink: self.core.layer_tree_frame_sink_,
            current_begin_frame_tracker: self.core.current_begin_frame_tracker_,
            delegate: *self.core.delegate_,
            input_view: self.input_delegate_.as_deref().map(|input| input.View()),
            services: self.context.services,
        };
        B::WithPictureContextParts(fields, callback)
    }
}

impl<B: HostImplTypes> crate::adapter::picture_layer_impl::PictureLayerContextHost<B>
    for HostImplDrawFacade<'_, '_, '_, '_, B>
{
    fn WithPictureContextParts(
        &mut self,
        selected: crate::adapter::layer_tree_impl::LayerTreeId,
        callback: &mut dyn FnMut(
            &dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerTreeHost<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
        ),
    ) where
        B: crate::adapter::picture_layer_impl::PictureLayerImplTypes,
    {
        let fields = crate::adapter::host_picture::HostPictureFields {
            owner_id: *self.host.core.owner_id_,
            selected_id: selected,
            active: self.host.active_tree_.PictureSlot(),
            pending: self.host.pending_tree_.PictureSlot(),
            recycle: self.host.recycle_tree_.PictureSlot(),
            settings: self.host.core.settings_,
            debug_state: self.host.core.debug_state_,
            raster_caps: self.host.core.raster_caps_,
            global_tile_state: self.host.core.global_tile_state_,
            resource_pool: self.host.core.resource_pool_,
            resource_provider: self.host.core.resource_provider_,
            layer_context: self.host.core.layer_context_,
            latency_monitors: self.host.core.latency_info_swap_promise_monitor_,
            events_metrics: self.host.core.events_metrics_manager_,
            tile_manager: self.host.core.tile_manager_,
            image_animation_controller: self.host.image_animation_controller_,
            paint_worklet_tracker: self.host.core.paint_worklet_tracker_,
            requires_high_res_to_draw: self.host.core.requires_high_res_to_draw_,
            external_viewport: self.host.core.external_viewport_,
            viewport_rect_for_tile_priority: self.host.core.viewport_rect_for_tile_priority_,
            layer_tree_frame_sink: self.host.core.layer_tree_frame_sink_,
            current_begin_frame_tracker: self.host.core.current_begin_frame_tracker_,
            delegate: *self.host.core.delegate_,
            input_view: Some(self.view.reborrow()),
            services: self.host.context.services,
        };
        B::WithPictureContextParts(fields, callback)
    }
}

use crate::adapter::layer_tree_impl::LayerTreeId;
use crate::cc::input::input_handler::InputHandlerView;
// Source host queries used by the call-scoped PictureLayerImpl environment.
// This split holds resident field references only. Selected is an existence
// identity for the actual tree already mutably lent to its native caller.
struct HostPictureTreeRead<'a, B: HostImplTypes> {
    id: Option<LayerTreeId>,
    tree: Option<&'a LayerTreeImpl<B>>,
}
impl<'a, B: HostImplTypes> HostPictureTreeRead<'a, B> {
    fn FromSlot(slot: &'a crate::adapter::host_picture::HostPictureTreeSlot<'_, B>) -> Self {
        use crate::adapter::host_picture::HostPictureTreeSlot::*;
        match slot {
            Resident(slot) => Self {
                id: slot.as_ref().map(|tree| tree.id),
                tree: slot.as_deref(),
            },
            Direct(tree) => Self {
                id: Some(tree.id),
                tree: Some(tree),
            },
            Selected(id) => Self {
                id: Some(*id),
                tree: None,
            },
        }
    }
}
struct HostPictureReadHost<'a, B: HostImplTypes> {
    active: &'a HostPictureTreeRead<'a, B>,
    pending: &'a HostPictureTreeRead<'a, B>,
    recycle: &'a HostPictureTreeRead<'a, B>,
    settings: &'a LayerTreeSettings,
    debug: &'a LayerTreeDebugState,
    caps: &'a RasterCapabilities,
    global: &'a crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority,
    high_res: &'a bool,
    viewport_priority: &'a Rect,
    external_viewport: &'a Rect,
    sink: &'a Option<HostImplFrameSinkId>,
    frame_tracker: &'a BeginFrameTracker,
    input: Option<InputHandlerView<'a>>,
    delegate: HostImplDelegateId,
    environment: &'a dyn crate::adapter::host_picture::HostPictureReadEnvironment,
}
impl<B> crate::adapter::picture_layer_impl::PictureLayerReadHost<B> for HostPictureReadHost<'_,B>
where B:HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes<PictureTargetColorParams=HostTargetColorParams<B::ColorSpace>>,
    <<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList:HostDisplayItemList,
{
    // Source: cc/trees/layer_tree_host_impl.cc:2009-2024,3492-3514.
    fn GetMSAASampleCountForRaster(&self,list:&<<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList,selected:LayerTreeId,device_scale:&f32)->i32{
        RasterMSAASampleCountForHost(list,self.caps,self.settings,||self.environment.IsLowEndDevice(),||{
            let scale_tree=if self.pending.id.is_some(){self.pending}else{self.active};
            if scale_tree.id==Some(selected){*device_scale}else{scale_tree.tree.expect("actual native scale tree").device_scale_factor()}
        })
    }
    // Source: cc/trees/layer_tree_host_impl.cc:1922-1932,1947-1968.
    fn GetTargetColorParams(&self,usage:ContentColorUsage,selected:LayerTreeId,display_colors:&B::DisplayColorSpaces)->B::PictureTargetColorParams{
        TargetColorParamsForHost::<B>(*self.sink,usage,self.environment,||{
            let color_tree=if self.pending.id.is_some(){self.pending}else{self.active};
            if color_tree.id==Some(selected){display_colors.clone()}
            else if let Some(tree)=color_tree.tree{tree.display_color_spaces().clone()}
            else{B::DisplayColorSpaces::default()}
        })
    }
    // Source: cc/trees/layer_tree_host_impl.h:482-483; cc/trees/layer_tree_impl.h:207-216.
    fn IsSyncTree(&self,id:LayerTreeId)->bool{if self.settings.commit_to_active_tree{self.IsActiveTree(id)}else{self.IsPendingTree(id)}}
    fn TreeAnimationsInVizInVizProcess(&self)->bool{self.settings.TreeAnimationsInVizInVizProcess(self.environment.SettingsFeatures())}
    fn IsActiveTree(&self,id:LayerTreeId)->bool{self.active.id==Some(id)}
    fn IsPendingTree(&self,id:LayerTreeId)->bool{self.pending.id==Some(id)}
    fn IsRecycleTree(&self,id:LayerTreeId)->bool{self.recycle.id==Some(id)}
    // Source: cc/trees/layer_tree_host_impl.cc:2026-2028.
    fn HasPendingTree(&self)->bool{self.pending.id.is_some()}
    fn settings(&self)->&LayerTreeSettings{self.settings}
    fn debug_state(&self)->&LayerTreeDebugState{self.debug}
    fn raster_caps(&self)->&RasterCapabilities{self.caps}
    fn IsCCSlimmingEnabled(&self)->bool{self.environment.IsCCSlimmingEnabled()}
    fn TreesInVizInClientProcess(&self)->bool{self.settings.TreesInVizInClientProcess(self.environment.SettingsFeatures())}
    // Source: cc/trees/layer_tree_host_impl.cc:6241-6243.
    fn IsReadyToActivate(&self)->bool{self.environment.IsReadyToActivate(self.delegate)}
    // Source: cc/trees/layer_tree_host_impl.cc:3987-3993; input_handler.cc:1616-1637.
    fn GetActivelyScrollingType(&self,selected_scroll_tree:&crate::cc::trees::property_tree::ScrollTree)->crate::cc::input::actively_scrolling_type::ActivelyScrollingType{
        let Some(input)=self.input.as_ref()else{return crate::cc::input::actively_scrolling_type::ActivelyScrollingType::kNone;};
        let active_scroll=self.active.tree.map(|tree|tree.property_trees().scroll_tree()).unwrap_or(selected_scroll_tree);
        input.GetActivelyScrollingTypeForTree(active_scroll,self.settings)
    }
    fn ViewportRectForTilePriority(&self)->Rect{*self.viewport_priority}
    // Source: cc/trees/layer_tree_impl.cc:3202-3225.
    fn SmoothnessTakesPriority(&self)->bool{self.global.tree_priority==crate::cc::tiles::tile_priority::TreePriority::SMOOTHNESS_TAKES_PRIORITY}
    // Source: cc/trees/layer_tree_host_impl.cc:5522-5526.
    fn CurrentBeginFrameArgs(&self)->&crate::adapter::begin_frame::BeginFrameArgs{self.frame_tracker.DangerousMethodCurrentOrLast()}
    fn UseGpuRasterization(&self)->bool{self.caps.use_gpu_rasterization}
    fn RequiresHighResToDraw(&self)->bool{*self.high_res}
    fn global_tile_state(&self)->&crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority{self.global}
    // Source: cc/trees/layer_tree_host_impl.cc:3980-3985.
    fn IsPinchGestureActive(&self)->bool{self.input.as_ref().is_some_and(|input|input.PinchGestureActive())}
    fn external_viewport(&self)->Rect{*self.external_viewport}
}

/// Borrow-only registration field projection. Decisions are the canonical
/// HostImpl source helpers above, not runtime/service implementations.
pub(crate) struct HostPictureRegistration<'a, B: HostImplTypes>(
    pub(crate) crate::adapter::host_picture::HostPictureRegistrationFields<'a, B>,
);
struct HostPictureRegistrationEffects<'a, B: HostImplTypes> {
    owner: HostImplWeak,
    mutator: &'a mut <B as LayerTreeTypes>::MutatorHost,
    services: &'a mut dyn crate::adapter::host_picture::HostPictureRegistrationServices<B>,
}
impl<B: HostImplTypes> crate::adapter::scrollbar::ScrollbarRegistrationHost
    for HostPictureRegistrationEffects<'_, B>
{
    fn PostDelayedAutoscrollTask(
        &mut self,
        task: crate::adapter::scrollbar::AutoscrollTask,
        delay: TimeDelta,
    ) {
        self.services
            .PostDelayedAutoscrollTask(self.owner.clone(), task, delay)
    }
    fn ScrollAnimationAbort(&mut self, id: ElementId) {
        self.services.ScrollAnimationAbort(self.mutator, id)
    }
}
impl<B: HostImplTypes> crate::adapter::layer_tree_impl::LayerTreeScrollbarRegistrationHost<B>
    for HostPictureRegistration<'_, B>
{
    // Source: cc/trees/layer_tree_host_impl.cc:5410-5416.
    fn DidRegisterScrollbarLayer(&mut self, id: ElementId, orientation: ScrollbarOrientation) {
        if let Some(input) = self.0.input_controller.as_deref_mut() {
            let mut effects = HostPictureRegistrationEffects {
                owner: self.0.owner.clone(),
                mutator: self.0.mutator,
                services: self.0.services,
            };
            input.DidRegisterScrollbar(&mut effects, id, orientation);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5418-5431.
    fn DidUnregisterScrollbarLayer(
        &mut self,
        id: ElementId,
        orientation: ScrollbarOrientation,
        client: &mut dyn crate::adapter::scrollbar::ScrollbarInitializationClient,
    ) {
        let empty = client.ScrollbarsFor(id).is_empty();
        RemoveUnregisteredScrollbarStateAfterQueryForHost(
            empty,
            self.0.controllers,
            self.0.flashed,
            self.0.previously_visible,
            self.0.accumulated,
            id,
        );
        if let Some(input) = self.0.input_controller.as_deref_mut() {
            let mut effects = HostPictureRegistrationEffects {
                owner: self.0.owner.clone(),
                mutator: self.0.mutator,
                services: self.0.services,
            };
            input.DidUnregisterScrollbar(&mut effects, id, orientation);
        }
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5389-5408.
    fn RegisterScrollbarAnimationController(
        &mut self,
        id: ElementId,
        opacity: f32,
        tree: &dyn crate::adapter::layer_tree_impl::ScrollbarControllerTree,
        client: &mut dyn crate::adapter::scrollbar::ScrollbarInitializationClient,
    ) {
        if ScrollbarControllerElementIdForTree(tree, self.0.controllers, id)
            .and_then(|key| self.0.controllers.get(&key).unwrap().as_ref())
            .is_some()
        {
            return;
        }
        RegisterScrollbarControllerAfterLookupForHost::<B>(
            self.0.settings,
            self.0.controllers,
            self.0.flashed,
            self.0.previously_visible,
            id,
            opacity,
            client,
        );
    }
}

/// Native other-tree pointer and current readonly host facts, lent only during
/// the source Picture call. All getters enter the original resident objects.
struct HostPictureSiblingTree<
    'a,
    B: HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes,
> {
    tree: &'a LayerTreeImpl<B>,
    host: &'a dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<B>,
}
impl<B: HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes>
    crate::adapter::main_layer::MainLayerImplTree<B> for HostPictureSiblingTree<'_, B>
{
    fn source_frame_number(&self) -> i32 {
        self.tree.source_frame_number()
    }
}
impl<B: HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes>
    crate::adapter::picture_layer_impl::PictureLayerReadContext for HostPictureSiblingTree<'_, B>
{
    fn IsRecycleTree(&self) -> bool {
        self.host.IsRecycleTree(self.tree.id)
    }
}
impl<B: HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes>
    crate::adapter::layer_impl::LayerImplReadTree<B> for HostPictureSiblingTree<'_, B>
{
    fn id(&self) -> LayerTreeId {
        self.tree.id
    }
    fn IsActiveTree(&self) -> bool {
        self.host.IsActiveTree(self.tree.id)
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.host.TreesInVizInClientProcess()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.host.IsCCSlimmingEnabled()
    }
    fn property_trees(&self) -> &crate::adapter::layer_tree_impl::PropertyTreesFor<B> {
        self.tree.property_trees()
    }
    fn settings(&self) -> &LayerTreeSettings {
        self.host.settings()
    }
    fn debug_state(&self) -> &LayerTreeDebugState {
        self.host.debug_state()
    }
    fn device_scale_factor(&self) -> f32 {
        self.tree.device_scale_factor()
    }
    fn max_texture_size(&self) -> i32 {
        self.host.raster_caps().max_texture_size
    }
    fn current_page_scale_factor(&self) -> f32 {
        self.tree
            .page_scale_factor()
            .Current(self.host.IsActiveTree(self.tree.id))
    }
    fn LifecycleAllowsPropertyTreeAccess(&self) -> bool {
        self.tree.LifecycleAllowsPropertyTreeAccess()
    }
}
impl<B: HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes>
    crate::adapter::picture_layer_impl::PictureLayerImplReadTree<B>
    for HostPictureSiblingTree<'_, B>
{
    fn raster_caps(&self) -> &RasterCapabilities {
        self.host.raster_caps()
    }
    fn GetMSAASampleCountForRaster(
        &self,
        list:&<<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList,
    ) -> i32 {
        self.host
            .GetMSAASampleCountForRaster(list, self.tree.id, &self.tree.device_scale_factor())
    }
    fn GetTargetColorParams(&self, usage: ContentColorUsage) -> B::PictureTargetColorParams {
        self.host
            .GetTargetColorParams(usage, self.tree.id, self.tree.display_color_spaces())
    }
    fn ViewportRectForTilePriority(&self) -> Rect {
        // Source: cc/trees/layer_tree_impl.cc:2204-2211. This read facet is
        // the Rust borrow split of the selected LayerTreeImpl, so preserve the
        // tree method's device-viewport fallback when no external constraint
        // was supplied by the embedder.
        let viewport = self.host.ViewportRectForTilePriority();
        if viewport.IsEmpty() {
            self.GetDeviceViewport()
        } else {
            viewport
        }
    }
    fn IsReadyToActivate(&self) -> bool {
        self.host.IsReadyToActivate()
    }
    fn HasPendingTree(&self) -> bool {
        self.host.HasPendingTree()
    }
    fn GetActivelyScrollingType(
        &self,
    ) -> crate::cc::input::actively_scrolling_type::ActivelyScrollingType {
        self.host
            .GetActivelyScrollingType(self.tree.property_trees().scroll_tree())
    }
    fn SmoothnessTakesPriority(&self) -> bool {
        self.host.SmoothnessTakesPriority()
    }
    fn CurrentBeginFrameArgs(&self) -> &BeginFrameArgs {
        self.host.CurrentBeginFrameArgs()
    }
    fn UseGpuRasterization(&self) -> bool {
        self.host.UseGpuRasterization()
    }
    fn RequiresHighResToDraw(&self) -> bool {
        self.host.RequiresHighResToDraw()
    }
    fn global_tile_state(
        &self,
    ) -> &crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority {
        self.host.global_tile_state()
    }
    fn IsPendingTree(&self) -> bool {
        self.host.IsPendingTree(self.tree.id)
    }
    fn IsSyncTree(&self) -> bool {
        self.host.IsSyncTree(self.tree.id)
    }
    fn PinchGestureActive(&self) -> bool {
        self.host.IsPinchGestureActive()
    }
    fn device_viewport_rect_changed(&self) -> bool {
        self.tree.device_viewport_rect_changed()
    }
    fn GetDeviceViewport(&self) -> Rect {
        LayerTreeImpl::<B>::GetDeviceViewportForRects(
            self.tree.internal_device_viewport(),
            self.host.external_viewport(),
        )
    }
    fn PageScaleTransformNode(
        &self,
    ) -> Option<
        &crate::cc::trees::transform_node::TransformNode<
            <B as LayerTreeTypes>::Transform,
        >,
    > {
        self.tree.PageScaleTransformNode()
    }
    fn external_page_scale_factor(&self) -> f32 {
        self.tree.external_page_scale_factor()
    }
}
/// Exactly the three original resident tree roles. These stack values contain
/// references only, not cloned layer lists, property trees or client metadata.
struct HostPictureSiblingQueries<
    'a,
    B: HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes,
> {
    active: Option<HostPictureSiblingTree<'a, B>>,
    pending: Option<HostPictureSiblingTree<'a, B>>,
    recycle: Option<HostPictureSiblingTree<'a, B>>,
}
impl<B> HostPictureSiblingQueries<'_, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    fn PictureByClient(
        &self,
        id: crate::adapter::picture_layer_tiling::PictureLayerTilingClientId,
    ) -> (
        &crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        &HostPictureSiblingTree<'_, B>,
    ) {
        for sibling in [&self.active, &self.pending, &self.recycle]
            .into_iter()
            .flatten()
        {
            for picture in sibling.tree.picture_layers() {
                if picture.identity().stable as u64 == id.0 {
                    return (picture, sibling);
                }
            }
        }
        panic!(
            "native client must belong to an actual other-tree resident; selected receiver resolves itself"
        )
    }
    fn PictureByIdentity(
        &self,
        id: crate::cc::layers::picture_layer_impl::PictureLayerImplId,
    ) -> (
        &crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        &HostPictureSiblingTree<'_, B>,
    ) {
        let (picture, tree) = self.PictureByClient(
            crate::adapter::picture_layer_tiling::PictureLayerTilingClientId(id.stable as u64),
        );
        assert_eq!(picture.identity(), id, "native stable Picture identity");
        (picture, tree)
    }
}
impl<B> crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>
    for HostPictureSiblingQueries<'_, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    fn WithClient(
        &self,
        id: crate::adapter::picture_layer_tiling::PictureLayerTilingClientId,
        callback: &mut dyn FnMut(
            &dyn crate::cc::tiles::picture_layer_tiling::PictureLayerTilingClient<B>,
        ),
    ) {
        let (picture, tree) = self.PictureByClient(id);
        let client =
            crate::cc::layers::picture_layer_impl::PictureTilingReadClient::new(
                picture.ReadView(tree),
            );
        callback(&client)
    }
    fn PaintWorkletRecordsForClient(
        &self,
        id: crate::adapter::picture_layer_tiling::PictureLayerTilingClientId,
    ) -> &B::PaintWorkletRecordMap {
        self.PictureByClient(id).0.GetPaintWorkletRecordMap()
    }
    fn TilingById(
        &self,
        id: crate::adapter::tile::PictureLayerTilingId,
    ) -> &crate::cc::tiles::picture_layer_tiling::PictureLayerTiling<B> {
        for sibling in [&self.active, &self.pending, &self.recycle]
            .into_iter()
            .flatten()
        {
            for picture in sibling.tree.picture_layers() {
                if let Some(tiling) = picture.picture_layer_tiling_set().TilingByIdIfPresent(id) {
                    return tiling;
                }
            }
        }
        panic!("native other-tree tiling identity must be resident")
    }
    fn IsClientOnActiveOrPendingTree(
        &self,
        id: crate::adapter::picture_layer_tiling::PictureLayerTilingClientId,
    ) -> bool {
        [&self.active, &self.pending]
            .into_iter()
            .flatten()
            .any(|sibling| {
                sibling
                    .tree
                    .picture_layers()
                    .any(|picture| picture.identity().stable as u64 == id.0)
            })
    }
    fn TilingStateForClientAndScale(
        &self,
        id: crate::adapter::picture_layer_tiling::PictureLayerTilingClientId,
        scale: f32,
    ) -> Option<&crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>>
    {
        self.PictureByClient(id)
            .0
            .picture_layer_tiling_set()
            .FindTilingWithScaleKeyRef(scale)
            .map(|tiling| &**tiling)
    }
    fn PendingInvalidationById(
        &self,
        id: crate::adapter::picture_layer_tiling::PendingInvalidationId,
    ) -> &Region<<B as LayerTreeTypes>::RegionBackend> {
        self.PictureByClient(crate::adapter::picture_layer_tiling::PictureLayerTilingClientId(id.0))
            .0
            .InvalidationForTesting()
    }
}

struct HostPictureMutableHost<'a, B: HostImplTypes> {
    image_animation: &'a mut Option<Box<ImageAnimationController>>,
    worklets: &'a mut AnimatedPaintWorkletTracker<B>,
    checker: crate::adapter::tile_manager::CheckerImageTrackerId,
    delegate: HostImplDelegateId,
    services: &'a mut dyn crate::adapter::host_picture::HostPictureMutationServices,
}
impl<B: HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes>
    crate::adapter::picture_layer_impl::PictureLayerTreeHost<B> for HostPictureMutableHost<'_, B>
{
    // Source: cc/trees/layer_tree_host_impl.cc:6245-6248.
    fn RequestImplSideInvalidationForRerasterTiling(&mut self) {
        self.services
            .RequestImplSideInvalidation(self.delegate, true)
    }
    // Source: cc/trees/layer_tree_host_impl.h:584-586,862-864.
    fn image_animation_controller(&mut self) -> &mut ImageAnimationController {
        self.image_animation
            .as_deref_mut()
            .expect("source dereferenced image animation controller")
    }
    fn paint_worklet_tracker(&mut self) -> &mut AnimatedPaintWorkletTracker<B> {
        self.worklets
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4825-4829; tile_manager.h:226-228.
    // Only the immutable native checker pointer identity is split from the
    // resident manager, whose mutable allocation/release fields are lent apart.
    fn UpdateImageDecodingHints(
        &mut self,
        modes: crate::adapter::picture_layer_impl::PictureImageDecodingModeMap,
    ) {
        self.services
            .UpdateCheckerImageDecodingHints(self.checker, modes)
    }
}
struct HostPictureTileTrace<'a>(&'a dyn crate::adapter::host_picture::HostPictureTileServices);
impl crate::adapter::tile::TileTraceContext for HostPictureTileTrace<'_> {
    fn MakeDictIntoImplicitSnapshotWithCategory<V: PropertyTrace>(
        &self,
        category: &str,
        value: &mut V,
        name: &str,
        id: crate::adapter::tile::TileTraceId,
    ) {
        self.0.MakeTileTrace(category, value, name, id)
    }
    fn TraceTileDeleted(
        &self,
        category: &str,
        event: &str,
        id: crate::adapter::tile::TileTraceId,
        flow: &str,
    ) {
        self.0.TraceTileDeleted(category, event, id, flow)
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:4274-4290. Source body shared by the
// complete owner and selected TileRelease continuation; only borrow is split.
fn RequestRedrawWithPictureServices<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    delegate: HostImplDelegateId,
    monitors: &BTreeSet<HostImplLatencyMonitorId>,
    events: &mut B::EventsMetricsManager,
    animation_only: bool,
    skip_if_inside_draw: bool,
    services: &mut (impl crate::adapter::host_picture::HostPictureRedrawServices<B> + ?Sized),
) -> Vec<Box<B::SwapPromise>> {
    if skip_if_inside_draw && services.DelegateIsInsideDraw(delegate) {
        return Vec::new();
    }
    let promises = NotifyLatencyMonitorsWithPictureServices(monitors, services);
    services.SaveActiveEventMetrics(events);
    if settings.TreesInVizInClientProcess(services.SettingsFeatures()) {
        if !animation_only
            || !settings.TreeAnimationsInVizInClientProcess(services.SettingsFeatures())
        {
            services.DelegateSetNeedsRedrawOnImplThread(delegate);
        }
    } else {
        services.DelegateSetNeedsRedrawOnImplThread(delegate);
    }
    promises
}
fn NotifyLatencyMonitorsWithPictureServices<B: HostImplTypes>(
    monitors: &BTreeSet<HostImplLatencyMonitorId>,
    services: &mut (impl crate::adapter::host_picture::HostPictureRedrawServices<B> + ?Sized),
) -> Vec<Box<B::SwapPromise>> {
    monitors
        .iter()
        .filter_map(|&monitor| services.NotifyLatencyInfoSwapPromiseMonitor(monitor))
        .collect()
}
struct HostFullPictureRedraw<'a, B: HostImplTypes>(&'a mut dyn HostImplServices<B>);
impl<B: HostImplTypes> crate::adapter::host_picture::HostPictureRedrawServices<B>
    for HostFullPictureRedraw<'_, B>
{
    fn SettingsFeatures(&self) -> &dyn crate::adapter::settings::LayerTreeSettingsFeatures {
        self.0.SettingsFeatures()
    }
    fn DelegateIsInsideDraw(&self, d: HostImplDelegateId) -> bool {
        self.0.DelegateIsInsideDraw(d)
    }
    fn NotifyLatencyInfoSwapPromiseMonitor(
        &mut self,
        id: HostImplLatencyMonitorId,
    ) -> Option<Box<B::SwapPromise>> {
        self.0.NotifyLatencyInfoSwapPromiseMonitor(id)
    }
    fn SaveActiveEventMetrics(&mut self, events: &mut B::EventsMetricsManager) {
        self.0.SaveActiveEventMetrics(events)
    }
    fn DelegateSetNeedsRedrawOnImplThread(&mut self, d: HostImplDelegateId) {
        self.0.DelegateSetNeedsRedrawOnImplThread(d)
    }
}

struct HostPictureReleaseParts<
    'a,
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes,
> {
    settings: &'a LayerTreeSettings,
    delegate: HostImplDelegateId,
    monitors: &'a BTreeSet<HostImplLatencyMonitorId>,
    events: &'a mut B::EventsMetricsManager,
    pool: &'a mut Option<Box<crate::cc::resources::resource_pool::ResourcePool<B>>>,
    provider: &'a mut Option<Box<B::ClientResourceProvider>>,
    layer_context: &'a mut Option<Box<<B as HostImplTypes>::LayerContext>>,
    sink: &'a Option<HostImplFrameSinkId>,
    latency_promises: &'a mut Vec<Box<B::SwapPromise>>,
    services: &'a mut dyn crate::adapter::host_picture::HostPictureReleaseServices<B>,
}
struct HostPictureTileRelease<
    'a,
    'b,
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes,
> {
    parts: &'a mut HostPictureReleaseParts<'b, B>,
    trace: &'a dyn crate::adapter::host_picture::HostPictureTileServices,
    tiling: &'a crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
    notification:
        Option<&'a mut dyn crate::adapter::picture_layer_impl::PictureReleasedTileClient<B>>,
}
impl<
        B: HostImplTypes
            + crate::adapter::picture_layer_impl::PictureLayerImplTypes
            + crate::adapter::tile_manager::TileManagerTypes,
    > crate::adapter::tile::TileTraceContext for HostPictureTileRelease<'_, '_, B>
{
    fn MakeDictIntoImplicitSnapshotWithCategory<V: PropertyTrace>(
        &self,
        category: &str,
        value: &mut V,
        name: &str,
        id: crate::adapter::tile::TileTraceId,
    ) {
        self.trace.MakeTileTrace(category, value, name, id)
    }
    fn TraceTileDeleted(
        &self,
        category: &str,
        event: &str,
        id: crate::adapter::tile::TileTraceId,
        flow: &str,
    ) {
        self.trace.TraceTileDeleted(category, event, id, flow)
    }
}
impl<
        B: HostImplTypes
            + crate::adapter::picture_layer_impl::PictureLayerImplTypes
            + crate::adapter::tile_manager::TileManagerTypes,
    > crate::adapter::tile_manager::TileReleaseRuntime<B> for HostPictureTileRelease<'_, '_, B>
{
    fn ResourcePoolMutable(
        &mut self,
        id: crate::adapter::resource_pool::ResourcePoolId,
    ) -> &mut crate::cc::resources::resource_pool::ResourcePool<B> {
        let pool = self
            .parts
            .pool
            .as_deref_mut()
            .expect("source initialized resource pool");
        assert_eq!(
            pool.identity(),
            id,
            "source manager references this resident pool"
        );
        pool
    }
    // Source: cc/trees/layer_tree_host_impl.cc:2068-2108. Same source phase
    // after the selected current Picture has been located by the native caller.
    fn NotifyTileStateChanged(
        &mut self,
        _client: crate::adapter::tile_manager::TileManagerClientId,
        tile: &crate::cc::tiles::tile::Tile<B>,
        damage: bool,
        redraw: bool,
    ) {
        self.parts.NotificationParts().Notify(
            self.tiling,
            self.notification.as_mut().map(|notification| {
                &mut **notification
                    as &mut dyn crate::adapter::picture_layer_impl::PictureReleasedTileClient<B>
            }),
            tile,
            damage,
            redraw,
        );
    }
}
struct HostPictureTilingEnvironment<
    'a,
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes,
> {
    queries: HostPictureSiblingQueries<'a, B>,
    manager: &'a mut crate::cc::tiles::tile_manager::TileManager<B>,
    tile_services: &'a mut dyn crate::adapter::host_picture::HostPictureTileServices,
    release: HostPictureReleaseParts<'a, B>,
}
impl<B> crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>
    for HostPictureTilingEnvironment<'_, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    fn WithClient(
        &self,
        id: crate::adapter::picture_layer_tiling::PictureLayerTilingClientId,
        callback: &mut dyn FnMut(
            &dyn crate::cc::tiles::picture_layer_tiling::PictureLayerTilingClient<B>,
        ),
    ) {
        self.queries.WithClient(id, callback)
    }
    fn PaintWorkletRecordsForClient(
        &self,
        id: crate::adapter::picture_layer_tiling::PictureLayerTilingClientId,
    ) -> &<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::PaintWorkletRecordMap
    {
        self.queries.PaintWorkletRecordsForClient(id)
    }
    fn TilingById(
        &self,
        id: crate::adapter::tile::PictureLayerTilingId,
    ) -> &crate::cc::tiles::picture_layer_tiling::PictureLayerTiling<B> {
        self.queries.TilingById(id)
    }
    fn IsClientOnActiveOrPendingTree(
        &self,
        id: crate::adapter::picture_layer_tiling::PictureLayerTilingClientId,
    ) -> bool {
        self.queries.IsClientOnActiveOrPendingTree(id)
    }
    fn TilingStateForClientAndScale(
        &self,
        id: crate::adapter::picture_layer_tiling::PictureLayerTilingClientId,
        scale: f32,
    ) -> Option<&crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>>
    {
        self.queries.TilingStateForClientAndScale(id, scale)
    }
    fn PendingInvalidationById(
        &self,
        id: crate::adapter::picture_layer_tiling::PendingInvalidationId,
    ) -> &Region<<B as LayerTreeTypes>::RegionBackend> {
        self.queries.PendingInvalidationById(id)
    }
}
impl<B> crate::adapter::picture_layer_tiling::PictureLayerTileDestroyer<B>
    for HostPictureTilingEnvironment<'_, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes,
{
    fn RebindTileWithManager(
        &mut self,
        old: crate::adapter::tile::TileId,
        new: crate::adapter::tile::TileId,
    ) {
        self.manager.RebindTile(old, new)
    }
    fn DestroyTileWithManager(
        &mut self,
        id: crate::adapter::tile::TileManagerId,
        tile: &mut crate::cc::tiles::tile::Tile<B>,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
    ) {
        assert_eq!(
            self.manager.identity(),
            id,
            "same canonical resident manager"
        );
        self.manager.DestroyTileWithRelease(
            tile,
            &mut HostPictureTileRelease {
                parts: &mut self.release,
                trace: self.tile_services,
                tiling,
                notification: None,
            },
        );
    }
}
impl<B> crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>
    for HostPictureTilingEnvironment<'_, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    fn TwinLayerRead(
        &self,
        id: crate::cc::layers::picture_layer_impl::PictureLayerImplId,
    ) -> crate::cc::layers::picture_layer_impl::PictureLayerImplReadView<'_, B> {
        let (picture, tree) = self.queries.PictureByIdentity(id);
        picture.ReadView(tree)
    }
    fn AllocateTilingId(&mut self) -> crate::adapter::tile::PictureLayerTilingId {
        self.tile_services.AllocateTilingId()
    }
    fn CreateTileWithManager(
        &mut self,
        info: &crate::cc::tiles::tile::CreateInfo<
            <B as crate::adapter::tile::TileTypes>::RasterTransform,
        >,
        layer: i32,
        source_frame: i32,
        flags: i32,
    ) -> Box<crate::cc::tiles::tile::Tile<B>> {
        self.manager.CreateTile(
            info,
            layer,
            source_frame,
            flags,
            &HostPictureTileTrace(self.tile_services),
        )
    }
    fn DestroyTileWithManagerForSelected(
        &mut self,
        id: crate::adapter::tile::TileManagerId,
        tile: &mut crate::cc::tiles::tile::Tile<B>,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        notification: &mut dyn crate::adapter::picture_layer_impl::PictureReleasedTileClient<B>,
    ) {
        assert_eq!(
            self.manager.identity(),
            id,
            "same canonical resident manager"
        );
        self.manager.DestroyTileWithRelease(
            tile,
            &mut HostPictureTileRelease {
                parts: &mut self.release,
                trace: self.tile_services,
                tiling,
                notification: Some(notification),
            },
        );
    }
}

/// Source raw owner pointers become short resident field borrows. This helper
/// implements no alternate layer graph and retains none of the callback parts.
fn WithHostPictureParts<B>(fields:crate::adapter::host_picture::HostPictureFields<'_,B>,callback:&mut dyn FnMut(
    &dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<B>,
    &mut dyn crate::adapter::picture_layer_impl::PictureLayerTreeHost<B>,
    &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<B>,
    &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
    &mut dyn crate::adapter::host_picture::HostPictureRegistrationServices<B>,
))
where B:HostImplTypes<ResourcePool=crate::cc::resources::resource_pool::ResourcePool<B>>
    +crate::adapter::picture_layer_impl::PictureLayerImplTypes<PictureTargetColorParams=HostTargetColorParams<<B as HostImplTypes>::ColorSpace>>
    +crate::adapter::tile_manager::TileManagerTypes
    +LayerTreeTypes<PictureLayer=crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,TileManager=crate::cc::tiles::tile_manager::TileManager<B>,GlobalTileState=crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority>,
    <<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList:HostDisplayItemList,
{
    let crate::adapter::host_picture::HostPictureFields {
        owner_id: _,
        selected_id: _,
        active,
        pending,
        recycle,
        settings,
        debug_state,
        raster_caps,
        global_tile_state,
        resource_pool,
        resource_provider,
        layer_context,
        latency_monitors,
        events_metrics,
        tile_manager,
        image_animation_controller,
        paint_worklet_tracker,
        requires_high_res_to_draw,
        external_viewport,
        viewport_rect_for_tile_priority,
        layer_tree_frame_sink,
        current_begin_frame_tracker,
        delegate,
        input_view,
        services,
    } = fields;
    let mut active_slot = active;
    let active = HostPictureTreeRead::FromSlot(&active_slot);
    let pending = HostPictureTreeRead::FromSlot(&pending);
    let recycle = HostPictureTreeRead::FromSlot(&recycle);
    let checker = tile_manager.checker_image_tracker_id();
    let mut latency_promises = Vec::new();
    services.WithPictureExternalParts(
        &mut |environment,
              picture_services,
              tile_services,
              mutation_services,
              registration_services,
              release_services| {
            let read_host = HostPictureReadHost {
                active: &active,
                pending: &pending,
                recycle: &recycle,
                settings,
                debug: debug_state,
                caps: raster_caps,
                global: global_tile_state,
                high_res: requires_high_res_to_draw,
                viewport_priority: viewport_rect_for_tile_priority,
                external_viewport,
                sink: layer_tree_frame_sink,
                frame_tracker: current_begin_frame_tracker,
                input: input_view.as_ref().map(|view| view.reborrow()),
                delegate,
                environment,
            };
            let queries = HostPictureSiblingQueries {
                active: active.tree.map(|tree| HostPictureSiblingTree {
                    tree,
                    host: &read_host,
                }),
                pending: pending.tree.map(|tree| HostPictureSiblingTree {
                    tree,
                    host: &read_host,
                }),
                recycle: recycle.tree.map(|tree| HostPictureSiblingTree {
                    tree,
                    host: &read_host,
                }),
            };
            let mut host = HostPictureMutableHost {
                image_animation: image_animation_controller,
                worklets: paint_worklet_tracker,
                checker,
                delegate,
                services: mutation_services,
            };
            let mut tiling = HostPictureTilingEnvironment {
                queries,
                manager: tile_manager,
                tile_services,
                release: HostPictureReleaseParts {
                    settings,
                    delegate,
                    monitors: latency_monitors,
                    events: events_metrics,
                    pool: resource_pool,
                    provider: resource_provider,
                    layer_context,
                    sink: layer_tree_frame_sink,
                    latency_promises: &mut latency_promises,
                    services: release_services,
                },
            };
            callback(
                &read_host,
                &mut host,
                picture_services,
                &mut tiling,
                registration_services,
            )
        },
    );
    use crate::adapter::host_picture::HostPictureTreeSlot;
    let active_tree = match &mut active_slot {
        HostPictureTreeSlot::Resident(slot) => slot.as_deref_mut(),
        HostPictureTreeSlot::Direct(tree) => Some(&mut **tree),
        HostPictureTreeSlot::Selected(_) => None,
    };
    if !latency_promises.is_empty() {
        QueuePinnedLatencyPromisesOnActiveTree(
            active_tree.expect("active picture tree must be resident while queuing latency"),
            latency_promises,
        );
    }
}
pub(crate) fn WithHostPictureContextParts<B>(fields:crate::adapter::host_picture::HostPictureFields<'_,B>,callback:&mut dyn FnMut(
    &dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<B>,
    &mut dyn crate::adapter::picture_layer_impl::PictureLayerTreeHost<B>,
    &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<B>,
    &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
))
where B:HostImplTypes<ResourcePool=crate::cc::resources::resource_pool::ResourcePool<B>>
    +crate::adapter::picture_layer_impl::PictureLayerImplTypes<PictureTargetColorParams=HostTargetColorParams<<B as HostImplTypes>::ColorSpace>>
    +crate::adapter::tile_manager::TileManagerTypes
    +LayerTreeTypes<PictureLayer=crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,TileManager=crate::cc::tiles::tile_manager::TileManager<B>,GlobalTileState=crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority>,
    <<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList:HostDisplayItemList,
{
    WithHostPictureParts(
        fields,
        &mut |read, host, services, tiling, _registration| callback(read, host, services, tiling),
    );
}
pub(crate) fn WithHostLayerContextParts<B>(fields:crate::adapter::host_picture::HostPictureFields<'_,B>,mut registration:crate::adapter::host_picture::HostPictureRegistrationOwnerFields<'_,B>,callback:&mut dyn FnMut(
    &dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<B>,
    &mut dyn crate::adapter::picture_layer_impl::PictureLayerTreeHost<B>,
    &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<B>,
    &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
    &mut dyn crate::adapter::layer_tree_impl::LayerTreeScrollbarRegistrationHost<B>,
))
where B:HostImplTypes<ResourcePool=crate::cc::resources::resource_pool::ResourcePool<B>>
    +crate::adapter::picture_layer_impl::PictureLayerImplTypes<PictureTargetColorParams=HostTargetColorParams<<B as HostImplTypes>::ColorSpace>>
    +crate::adapter::tile_manager::TileManagerTypes
    +LayerTreeTypes<PictureLayer=crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,TileManager=crate::cc::tiles::tile_manager::TileManager<B>,GlobalTileState=crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority>,
    <<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList:HostDisplayItemList,
{
    WithHostPictureParts(
        fields,
        &mut |read, host, services, tiling, registration_services| {
            let mut registrar = HostPictureRegistration(
                crate::adapter::host_picture::HostPictureRegistrationFields {
                    owner: registration.owner.clone(),
                    settings: registration.settings,
                    input_controller: registration.input_controller.as_deref_mut(),
                    controllers: registration.controllers,
                    flashed: registration.flashed,
                    previously_visible: registration.previously_visible,
                    accumulated: registration.accumulated,
                    mutator: registration.mutator,
                    services: registration_services,
                },
            );
            callback(read, host, services, tiling, &mut registrar)
        },
    );
}

// Native picture range adapters borrow the same resident pointers. Each item
// is a stack vtable with references only; it owns no layer/tree/chunk state.
struct HostPictureQueueLayers<
    'a,
    B: HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes,
> {
    layers: Option<
        <B::LayerList as crate::adapter::layer_tree_impl::LayerTreeLayerList<B>>::PictureIter<'a>,
    >,
    tree: Option<&'a HostPictureSiblingTree<'a, B>>,
}
impl<'a, B> HostPictureQueueLayers<'a, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    fn new(tree: Option<&'a HostPictureSiblingTree<'a, B>>) -> Self {
        Self {
            layers: tree.map(|tree| tree.tree.picture_layers()),
            tree,
        }
    }
}
impl<'a, B> Iterator for HostPictureQueueLayers<'a, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    type Item = crate::cc::layers::picture_layer_impl::PictureQueueLayerRead<'a, B>;
    fn next(&mut self) -> Option<Self::Item> {
        self.layers
            .as_mut()?
            .next()
            .map(|picture| Self::Item::new(picture, self.tree.unwrap()))
    }
}
impl<B> crate::cc::tiles::raster_tile_priority_queue::RasterQueueContext<B>
    for HostPictureSiblingQueries<'_, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    fn TilingSetById(
        &self,
        id: crate::cc::tiles::raster_tile_priority_queue::PictureLayerTilingSetId,
    ) -> &crate::cc::tiles::picture_layer_tiling_set::PictureLayerTilingSet<B> {
        for sibling in [&self.active, &self.pending, &self.recycle]
            .into_iter()
            .flatten()
        {
            for picture in sibling.tree.picture_layers() {
                let set = picture.picture_layer_tiling_set();
                if set.identity() == id {
                    return set;
                }
            }
        }
        panic!("native queue Set pointer must remain resident")
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.active
            .as_ref()
            .expect("native active tree")
            .host
            .IsCCSlimmingEnabled()
    }
}
impl<B> LayerTreeHostImpl<B>
where B:HostImplTypes+crate::adapter::picture_layer_impl::PictureLayerImplTypes<PictureTargetColorParams=HostTargetColorParams<B::ColorSpace>>
    +LayerTreeTypes<PictureLayer=crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,GlobalTileState=crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority>,
    <<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList:HostDisplayItemList,
{
    // Source: cc/trees/layer_tree_host_impl.cc:1887-1912. The source raw tree
    // pointers become these call-scoped references to the same live owners.
    fn WithRasterQueueContext<R>(&self,context:&mut HostImplContext<'_,B>,callback:impl FnOnce(&HostPictureSiblingQueries<'_,B>)->R)->R {
        let active=HostPictureTreeRead{id:self.active_tree_.as_ref().map(|tree|tree.id),tree:self.active_tree_.as_deref()};
        let pending=HostPictureTreeRead{id:self.pending_tree_.as_ref().map(|tree|tree.id),tree:self.pending_tree_.as_deref()};
        let recycle=HostPictureTreeRead{id:self.recycle_tree_.as_ref().map(|tree|tree.id),tree:self.recycle_tree_.as_deref()};
        let mut callback=Some(callback);
        let mut result=None;
        context.services.WithPictureExternalParts(&mut|environment,_picture,_tile,_mutation,_registration,_release|{
            let read_host=HostPictureReadHost{active:&active,pending:&pending,recycle:&recycle,settings:&self.settings_,debug:&self.debug_state_,caps:&self.raster_caps_,global:&self.global_tile_state_,high_res:&self.requires_high_res_to_draw_,viewport_priority:&self.viewport_rect_for_tile_priority_,external_viewport:&self.external_viewport_,sink:&self.layer_tree_frame_sink_,frame_tracker:&self.current_begin_frame_tracker_,input:self.input_delegate_.as_ref().map(|input|input.View()),delegate:self.delegate_,environment};
            let queries=HostPictureSiblingQueries{active:active.tree.map(|tree|HostPictureSiblingTree{tree,host:&read_host}),pending:pending.tree.map(|tree|HostPictureSiblingTree{tree,host:&read_host}),recycle:recycle.tree.map(|tree|HostPictureSiblingTree{tree,host:&read_host})};
            result=Some(callback.take().expect("single synchronous external capability split")(&queries));
        });
        result.expect("native external capability split must run once")
    }
}

// The client commit path lends the pending invalidation set independently of
// the Picture-only factory; this carrier has no pending-set access at all.
// Source: cc/trees/client_layer_tree_host_impl.cc:379-380.
struct HostPictureFieldsFactory<'a, B: HostImplTypes>(
    crate::adapter::host_picture::HostPictureFields<'a, B>,
);
impl<B: HostImplTypes> crate::adapter::host_picture::HostPictureTreeSlot<'_, B> {
    fn Reborrow(&mut self) -> crate::adapter::host_picture::HostPictureTreeSlot<'_, B> {
        use crate::adapter::host_picture::HostPictureTreeSlot::*;
        match self {
            Resident(slot) => Resident(&mut **slot),
            Direct(tree) => Direct(&mut **tree),
            Selected(id) => Selected(*id),
        }
    }
}
impl<B: HostImplTypes> crate::adapter::picture_layer_impl::PictureLayerContextHost<B>
    for HostPictureFieldsFactory<'_, B>
{
    fn WithPictureContextParts(
        &mut self,
        selected: LayerTreeId,
        callback: &mut dyn FnMut(
            &dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerTreeHost<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<B>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
        ),
    ) where
        B: crate::adapter::picture_layer_impl::PictureLayerImplTypes,
    {
        assert_eq!(selected, self.0.selected_id, "same actual selected tree");
        B::WithPictureContextParts(
            crate::adapter::host_picture::HostPictureFields {
                owner_id: self.0.owner_id,
                selected_id: selected,
                active: self.0.active.Reborrow(),
                pending: self.0.pending.Reborrow(),
                recycle: self.0.recycle.Reborrow(),
                settings: self.0.settings,
                debug_state: self.0.debug_state,
                raster_caps: self.0.raster_caps,
                global_tile_state: self.0.global_tile_state,
                resource_pool: self.0.resource_pool,
                resource_provider: self.0.resource_provider,
                layer_context: self.0.layer_context,
                latency_monitors: self.0.latency_monitors,
                events_metrics: self.0.events_metrics,
                tile_manager: self.0.tile_manager,
                image_animation_controller: self.0.image_animation_controller,
                paint_worklet_tracker: self.0.paint_worklet_tracker,
                requires_high_res_to_draw: self.0.requires_high_res_to_draw,
                external_viewport: self.0.external_viewport,
                viewport_rect_for_tile_priority: self.0.viewport_rect_for_tile_priority,
                layer_tree_frame_sink: self.0.layer_tree_frame_sink,
                current_begin_frame_tracker: self.0.current_begin_frame_tracker,
                delegate: self.0.delegate,
                input_view: self.0.input_view.as_ref().map(|view| view.reborrow()),
                services: self.0.services,
            },
            callback,
        )
    }
}
impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/client_layer_tree_host_impl.cc:379-380.
    pub(super) fn InvalidateSyncTreeRasterInducingScrolls(
        &mut self,
        context: &mut HostImplContext<'_, B>,
    ) {
        use crate::adapter::host_picture::HostPictureTreeSlot;
        let Self {
            owner_id_,
            active_tree_,
            pending_tree_,
            recycle_tree_,
            settings_,
            debug_state_,
            raster_caps_,
            global_tile_state_,
            resource_pool_,
            resource_provider_,
            layer_context_,
            latency_info_swap_promise_monitor_,
            events_metrics_manager_,
            tile_manager_,
            image_animation_controller_,
            paint_worklet_tracker_,
            requires_high_res_to_draw_,
            external_viewport_,
            viewport_rect_for_tile_priority_,
            layer_tree_frame_sink_,
            current_begin_frame_tracker_,
            delegate_,
            input_delegate_,
            pending_invalidation_raster_inducing_scrolls_,
            ..
        } = self;
        let sync_identity = if settings_.commit_to_active_tree {
            active_tree_.as_ref().map(|tree| tree.id)
        } else {
            pending_tree_.as_ref().map(|tree| tree.id)
        };
        macro_rules! invalidate {
            ($tree:expr,$active:expr,$pending:expr) => {{
                let tree = $tree;
                let selected = tree.id;
                let mut factory =
                    HostPictureFieldsFactory(crate::adapter::host_picture::HostPictureFields {
                        owner_id: *owner_id_,
                        selected_id: selected,
                        active: $active,
                        pending: $pending,
                        recycle: HostPictureTreeSlot::Resident(recycle_tree_),
                        settings: settings_,
                        debug_state: debug_state_,
                        raster_caps: raster_caps_,
                        global_tile_state: global_tile_state_,
                        resource_pool: resource_pool_,
                        resource_provider: resource_provider_,
                        layer_context: layer_context_,
                        latency_monitors: latency_info_swap_promise_monitor_,
                        events_metrics: events_metrics_manager_,
                        tile_manager: tile_manager_,
                        image_animation_controller: image_animation_controller_,
                        paint_worklet_tracker: paint_worklet_tracker_,
                        requires_high_res_to_draw: requires_high_res_to_draw_,
                        external_viewport: external_viewport_,
                        viewport_rect_for_tile_priority: viewport_rect_for_tile_priority_,
                        layer_tree_frame_sink: layer_tree_frame_sink_,
                        current_begin_frame_tracker: current_begin_frame_tracker_,
                        delegate: *delegate_,
                        input_view: input_delegate_.as_deref().map(|input| input.View()),
                        services: context.services,
                    });
                tree.InvalidateRasterInducingScrollsWithPictureHost(
                    pending_invalidation_raster_inducing_scrolls_,
                    sync_identity == Some(selected),
                    &mut factory,
                );
            }};
        }
        if settings_.commit_to_active_tree {
            let tree = active_tree_
                .as_deref_mut()
                .expect("source active sync tree");
            let selected = tree.id;
            invalidate!(
                tree,
                HostPictureTreeSlot::Selected(selected),
                HostPictureTreeSlot::Resident(pending_tree_)
            );
        } else {
            let tree = pending_tree_
                .as_deref_mut()
                .expect("source pending sync tree");
            let selected = tree.id;
            invalidate!(
                tree,
                HostPictureTreeSlot::Resident(active_tree_),
                HostPictureTreeSlot::Selected(selected)
            );
        }
        pending_invalidation_raster_inducing_scrolls_.clear();
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    /// Source weak-bound callbacks at cc/trees/layer_tree_host_impl.cc:624-628,
    /// 709-711,2226-2230. The command carries no callable owner; Engine lends
    /// this actual owner when the originating compositor sequence delivers it.
    pub(crate) fn RunTask(&mut self, task: HostImplTask, context: &mut HostImplContext<'_, B>) {
        if !task.target.IsValid() {
            return;
        }
        assert_eq!(
            task.target.owner, self.owner_id_,
            "HostImpl callback owner mismatch"
        );
        assert!(
            task.target
                .alive
                .ptr_eq(&std::sync::Arc::downgrade(&self.weak_factory_)),
            "HostImpl callback generation mismatch"
        );
        match task.kind {
            HostImplTaskKind::ResetHasInputForFrameInterval => self.ResetHasInputForFrameInterval(),
            HostImplTaskKind::MaybeFlushPendingWork => self.MaybeFlushPendingWork(context),
        }
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:6159-6192. Rust borrow split:
// both the owner entry and the TileManager completion callback operate on the
// same visibility handles while the resident manager is independently borrowed.
fn SetContextVisibilityForLiveParts<B: HostImplTypes>(
    sink: Option<HostImplFrameSinkId>,
    compositor_visibility: &mut Option<Box<B::ContextVisibility>>,
    worker_visibility: &mut Option<Box<B::ContextVisibility>>,
    is_visible: bool,
    services: &mut dyn HostImplServices<B>,
) {
    let Some(sink) = sink else {
        return;
    };
    if let Some(provider) = services.FrameSinkContextProvider(sink) {
        if is_visible != compositor_visibility.is_some() {
            if is_visible {
                *compositor_visibility = Some(services.ClientBecameVisible(provider));
            } else {
                services.ClientBecameNotVisible(provider, compositor_visibility.take());
            }
        }
    }
    if let Some(provider) = services.FrameSinkWorkerContextProvider(sink) {
        if is_visible != worker_visibility.is_some() {
            let mut hold = services.LockRasterContext(provider);
            if is_visible {
                *worker_visibility = Some(services.WorkerClientBecameVisible(&mut hold));
            } else {
                services.WorkerClientBecameNotVisible(&mut hold, worker_visibility.take());
            }
        }
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:2030-2040. The callback borrows
// the actual Host state instead of trying to recover the owner from a service.
fn NotifyReadyToActivateForLiveParts<B: HostImplTypes>(
    pending_tree_fully_painted: bool,
    delegate: HostImplDelegateId,
    services: &mut dyn HostImplServices<B>,
) {
    if !pending_tree_fully_painted {
        return;
    }
    services.DelegateNotifyReadyToActivate(delegate);
}

// Source: cc/trees/layer_tree_host_impl.cc:2042-2049.
fn NotifyReadyToDrawForLiveParts<B: HostImplTypes>(
    is_likely_to_require_a_draw: &mut bool,
    delegate: HostImplDelegateId,
    services: &mut dyn HostImplServices<B>,
) {
    *is_likely_to_require_a_draw = false;
    services.DelegateNotifyReadyToDraw(delegate);
}

// Source: cc/trees/layer_tree_host_impl.cc:2051-2066. No manager or tree
// reference participates in this source callback; each mutable field is lent
// directly from its current Host allocation and never copied into a service.
fn NotifyAllTileTasksCompletedForLiveParts<B: HostImplTypes>(
    settings: &LayerTreeSettings,
    global_tile_state: &B::GlobalTileState,
    image_decode_cache_holder: &mut Option<Box<ImageDecodeCacheHolder<B::ImageDecodeFactory>>>,
    sink: Option<HostImplFrameSinkId>,
    compositor_visibility: &mut Option<Box<B::ContextVisibility>>,
    worker_visibility: &mut Option<Box<B::ContextVisibility>>,
    services: &mut dyn HostImplServices<B>,
) {
    debug_assert!(!settings.trees_in_viz_in_viz_process);
    if global_tile_state.hard_memory_limit_in_bytes() == 0 {
        if let Some(holder) = image_decode_cache_holder.as_mut() {
            holder.SetShouldAggressivelyFreeResources(true);
        }
        SetContextVisibilityForLiveParts::<B>(
            sink,
            compositor_visibility,
            worker_visibility,
            false,
            services,
        );
    }
}

trait HostPictureNotification<B: crate::adapter::picture_layer_impl::PictureLayerImplTypes> {
    fn NotifyTileStateChanged(
        &mut self,
        tile: &crate::cc::tiles::tile::Tile<B>,
        damage: bool,
    );
    fn layer(&self) -> &dyn PictureLayerDisplayRead;
    fn should_batch_updated_tiles(&self) -> bool;
}
struct ReleasedPictureNotification<'a, B: crate::adapter::picture_layer_impl::PictureLayerImplTypes>(
    &'a mut dyn crate::adapter::picture_layer_impl::PictureReleasedTileClient<B>,
);
impl<B: crate::adapter::picture_layer_impl::PictureLayerImplTypes> HostPictureNotification<B>
    for ReleasedPictureNotification<'_, B>
{
    fn NotifyTileStateChanged(
        &mut self,
        tile: &crate::cc::tiles::tile::Tile<B>,
        damage: bool,
    ) {
        assert!(
            !damage,
            "source resource Release passes update_damage=false"
        );
        self.0.NotifyReleasedTile(tile)
    }
    fn layer(&self) -> &dyn PictureLayerDisplayRead {
        self.0.layer()
    }
    fn should_batch_updated_tiles(&self) -> bool {
        self.0.should_batch_updated_tiles()
    }
}

/// Borrow-only source NotifyTileStateChanged fields. ResourcePool is excluded:
/// the source manager may synchronously free the current lease while notifying
/// its Picture's disjoint flags. None of these references is retained.
struct HostPictureNotificationParts<
    'a,
    B: HostImplTypes + crate::adapter::picture_layer_impl::PictureLayerImplTypes,
> {
    settings: &'a LayerTreeSettings,
    delegate: HostImplDelegateId,
    monitors: &'a BTreeSet<HostImplLatencyMonitorId>,
    events: &'a mut B::EventsMetricsManager,
    provider: &'a mut Option<Box<B::ClientResourceProvider>>,
    layer_context: &'a mut Option<Box<<B as HostImplTypes>::LayerContext>>,
    sink: &'a Option<HostImplFrameSinkId>,
    latency_promises: &'a mut Vec<Box<B::SwapPromise>>,
    services: &'a mut dyn crate::adapter::host_picture::HostPictureReleaseServices<B>,
}
impl<B> HostPictureReleaseParts<'_, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes,
{
    fn NotificationParts(&mut self) -> HostPictureNotificationParts<'_, B> {
        HostPictureNotificationParts {
            settings: self.settings,
            delegate: self.delegate,
            monitors: self.monitors,
            events: self.events,
            provider: self.provider,
            layer_context: self.layer_context,
            sink: self.sink,
            latency_promises: self.latency_promises,
            services: self.services,
        }
    }
}
impl<B> HostPictureNotificationParts<'_, B>
where
    B: HostImplTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::tile_manager::TileManagerTypes,
{
    // Source: cc/trees/layer_tree_host_impl.cc:2068-2108. One body shared by
    // Tile destruction and resident-resource traversal; lookup was performed by
    // the native caller, and the actual current Picture is lent synchronously.
    fn Notify(
        &mut self,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        notification: Option<
            &mut dyn crate::adapter::picture_layer_impl::PictureReleasedTileClient<B>,
        >,
        tile: &crate::cc::tiles::tile::Tile<B>,
        damage: bool,
        redraw: bool,
    ) {
        if let Some(notification) = notification {
            let mut notification = ReleasedPictureNotification(notification);
            self.NotifyCurrent(tiling, Some(&mut notification), tile, damage, redraw);
        } else {
            self.NotifyCurrent(tiling, None, tile, damage, redraw);
        }
    }
    fn NotifyCurrent(
        &mut self,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        notification: Option<&mut dyn HostPictureNotification<B>>,
        tile: &crate::cc::tiles::tile::Tile<B>,
        damage: bool,
        redraw: bool,
    ) {
        debug_assert!(!self.settings.trees_in_viz_in_viz_process);
        self.services.TraceTileStateNotification();
        let Some(notification) = notification else {
            // This path is the already-detached Picture destructor only.
            // Recreate and all operational Set/Tiling releases explicitly pass
            // the current-source continuation below. Native LayerById fails
            // after that lifecycle removal, at cc2078-2080.
            return;
        };
        notification.NotifyTileStateChanged(tile, damage);
        if self
            .settings
            .TreesInVizInClientProcess(self.services.SettingsFeatures())
            && !notification.should_batch_updated_tiles()
        {
            self.services.UpdateDisplayTile(
                self.layer_context
                    .as_deref_mut()
                    .expect("source Viz layer context"),
                notification.layer(),
                tiling,
                tile,
                self.provider
                    .as_deref_mut()
                    .expect("source resource provider"),
                self.sink.expect("source frame sink"),
                damage,
            );
        }
        if redraw && !self.services.DelegateIsInsideDraw(self.delegate) && tile.required_for_draw()
        {
            self.latency_promises
                .extend(RequestRedrawWithPictureServices::<B>(
                    self.settings,
                    self.delegate,
                    self.monitors,
                    self.events,
                    false,
                    false,
                    self.services,
                ));
        }
    }
}
// Source: cc/trees/layer_tree_host_impl.cc:1997-2007. The manager client
// and owner entry borrow the same animation controller, with one source body.
fn GetFrameIndexForImageForLiveParts<P: HostPaintImage>(
    image: &P,
    tree: crate::adapter::image_animation::WhichTree,
    settings: &LayerTreeSettings,
    image_animation: &Option<Box<ImageAnimationController>>,
) -> usize {
    if !image.ShouldAnimate() {
        return crate::adapter::image_animation::kDefaultFrameIndex;
    }
    assert!(!settings.trees_in_viz_in_viz_process);
    assert!(image_animation.is_some());
    image_animation
        .as_ref()
        .unwrap()
        .GetFrameIndexForImage(image.stable_id(), tree)
}

// Source: cc/trees/layer_tree_host_impl.cc:1914-1920,2068-2108;
// cc/tiles/tiles_with_resource_iterator.cc:13-103. Rust borrow-only native
// field assembly: the actual cursor, selected Picture notification and sibling
// queries coexist on the stack; no current owner is removed or relooked up.
pub(crate) mod source_tile_manager_runtime {
    use super::*;
    use crate::adapter::tile::TileManagerId;
    use crate::cc::tiles::tile_manager::TileManagerCommand;
    use crate::{
        adapter::{
            host_picture::{HostPictureReadEnvironment, HostPictureTileServices},
            layer_impl::LayerImplPushContext,
            picture_layer_impl::{
                PictureLayerImplTypes, PictureLayerReadHost, PictureReleasedTileClient,
            },
            picture_layer_tiling::PictureLayerTilingQueries,
            tile_manager::{ResourceTileReleaseTraversal, TileStateNotification},
        },
        runtime::types::{CpuCompositorTypes, CpuSkiaBackend},
        cc::{
            layers::{
                layer_collections::LayerTracking,
                picture_layer_impl::PictureResourceTileNotificationParts,
            },
            tiles::{
                picture_layer_tiling::PictureLayerTilingState, tile::Tile,
                tile_priority::GlobalStateThatImpactsTilePriority,
                tiles_with_resource_iterator::ForEachResourceTileInPicture,
            },
        },
    };
    use std::{collections::HashSet, marker::PhantomData, ops::ControlFlow};
    type Cpu<R> = CpuCompositorTypes<R>;

    /// Only actual resident Host fields used by the source readonly query
    /// facade. The tree roles themselves are lent afresh for each source walk.
    pub(super) struct TileReadFields<'a, R: CpuSkiaBackend> {
        pub settings: &'a LayerTreeSettings,
        pub debug: &'a LayerTreeDebugState,
        pub caps: &'a RasterCapabilities,
        pub global: &'a GlobalStateThatImpactsTilePriority,
        pub high_res: &'a bool,
        pub viewport_priority: &'a Rect,
        pub external_viewport: &'a Rect,
        pub sink: &'a Option<HostImplFrameSinkId>,
        pub frame_tracker: &'a BeginFrameTracker,
        pub input: Option<InputHandlerView<'a>>,
        pub delegate: HostImplDelegateId,
        pub environment: &'a dyn HostPictureReadEnvironment,
        pub family: PhantomData<fn() -> R>,
    }
    impl<R: CpuSkiaBackend> TileReadFields<'_, R> {
        fn WithReadHost<Output>(
            &self,
            active: HostPictureTreeRead<'_, Cpu<R>>,
            pending: HostPictureTreeRead<'_, Cpu<R>>,
            recycle: HostPictureTreeRead<'_, Cpu<R>>,
            body: impl FnOnce(&dyn PictureLayerReadHost<Cpu<R>>) -> Output,
        ) -> Output {
            let read = HostPictureReadHost {
                active: &active,
                pending: &pending,
                recycle: &recycle,
                settings: self.settings,
                debug: self.debug,
                caps: self.caps,
                global: self.global,
                high_res: self.high_res,
                viewport_priority: self.viewport_priority,
                external_viewport: self.external_viewport,
                sink: self.sink,
                frame_tracker: self.frame_tracker,
                input: self.input.as_ref().map(InputHandlerView::reborrow),
                delegate: self.delegate,
                environment: self.environment,
            };
            body(&read)
        }
    }
    /// Resource traversal excludes the manager and pool, which are executing
    /// the source call. All mutable references point to disjoint Host fields.
    pub(super) struct ResourceTraversal<'a, R: CpuSkiaBackend> {
        pub active: &'a mut LayerTreeImpl<Cpu<R>>,
        pub pending: Option<&'a mut LayerTreeImpl<Cpu<R>>>,
        pub recycle: Option<&'a LayerTreeImpl<Cpu<R>>>,
        pub read: TileReadFields<'a, R>,
        pub notification: HostPictureNotificationParts<'a, Cpu<R>>,
    }
    struct ResourcePush<'a, R: CpuSkiaBackend> {
        id: LayerTreeId,
        active: LayerTreeId,
        settings: &'a LayerTreeSettings,
        environment: &'a dyn HostPictureReadEnvironment,
        should_push: &'a mut BTreeSet<i32>,
        needs_update: &'a mut bool,
        family: PhantomData<fn() -> R>,
    }
    impl<R: CpuSkiaBackend> LayerImplPushContext for ResourcePush<'_, R> {
        type Types = Cpu<R>;
        fn id(&self) -> LayerTreeId {
            self.id
        }
        fn IsActiveTree(&self) -> bool {
            self.id == self.active
        }
        fn TreesInVizInClientProcess(&self) -> bool {
            self.settings
                .TreesInVizInClientProcess(self.environment.SettingsFeatures())
        }
        fn IsCCSlimmingEnabled(&self) -> bool {
            self.environment.IsCCSlimmingEnabled()
        }
        fn AddLayerShouldPushProperties(&mut self, id: i32) {
            self.should_push.insert(id);
        }
        fn set_needs_update_draw_properties(&mut self) {
            *self.needs_update = true;
        }
    }
    struct CurrentPictureNotification<'a, 'p, R: CpuSkiaBackend> {
        picture: &'a mut PictureResourceTileNotificationParts<'p>,
        push: &'a mut dyn LayerImplPushContext<Types = Cpu<R>>,
    }
    impl<R: CpuSkiaBackend> PictureReleasedTileClient<Cpu<R>>
        for CurrentPictureNotification<'_, '_, R>
    {
        fn NotifyReleasedTile(&mut self, tile: &Tile<Cpu<R>>) {
            self.picture.NotifyReleasedTile::<Cpu<R>>(tile, self.push)
        }
        fn layer(&self) -> &dyn crate::adapter::layer_tree_host_impl::PictureLayerDisplayRead {
            self.picture.layer()
        }
        fn should_batch_updated_tiles(&self) -> bool {
            self.picture.should_batch_updated_tiles()
        }
    }
    struct CurrentResourceNotification<'a, 'p, 'h, R: CpuSkiaBackend> {
        host: &'a mut HostPictureNotificationParts<'h, Cpu<R>>,
        tiling: &'a PictureLayerTilingState<Cpu<R>>,
        picture: CurrentPictureNotification<'a, 'p, R>,
    }
    impl<R: CpuSkiaBackend> TileStateNotification<Cpu<R>>
        for CurrentResourceNotification<'_, '_, '_, R>
    {
        fn NotifyTileStateChanged(&mut self, tile: &Tile<Cpu<R>>, damage: bool, redraw: bool) {
            self.host
                .Notify(self.tiling, Some(&mut self.picture), tile, damage, redraw)
        }
    }
    fn VisitResourceTree<R: CpuSkiaBackend>(
        tree: &mut LayerTreeImpl<Cpu<R>>,
        active_id: LayerTreeId,
        read: &TileReadFields<'_, R>,
        queries: &dyn PictureLayerTilingQueries<Cpu<R>>,
        notification: &mut HostPictureNotificationParts<'_, Cpu<R>>,
        visited: &mut HashSet<crate::adapter::tile::TileId>,
        callback: &mut dyn FnMut(
            &mut Tile<Cpu<R>>,
            bool,
            &mut dyn TileStateNotification<Cpu<R>>,
        ) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        let (layers, fields) = tree.SplitLayerTreeFields();
        layers.WithStorageAndTracking(|storage, tracking| {
            let mut push = ResourcePush::<R> {
                id: fields.id,
                active: active_id,
                settings: read.settings,
                environment: read.environment,
                should_push: tracking.should_push_properties,
                needs_update: fields.needs_update_draw_properties,
                family: PhantomData,
            };
            storage.ForEachPictureLayer(&mut |picture| {
                ForEachResourceTileInPicture(
                    picture,
                    visited,
                    &mut |tile, state, current_picture| {
                        let occluded =
                            current_picture.IsTileOccluded::<Cpu<R>>(tile, state, queries);
                        let mut current = CurrentResourceNotification {
                            host: notification,
                            tiling: state,
                            picture: CurrentPictureNotification {
                                picture: current_picture,
                                push: &mut push,
                            },
                        };
                        callback(tile, occluded, &mut current)
                    },
                )
            })
        })
    }
    impl<R: CpuSkiaBackend> ResourceTileReleaseTraversal<Cpu<R>> for ResourceTraversal<'_, R> {
        fn ForEachResourceTile(
            &mut self,
            callback: &mut dyn FnMut(
                &mut Tile<Cpu<R>>,
                bool,
                &mut dyn TileStateNotification<Cpu<R>>,
            ) -> ControlFlow<()>,
        ) -> ControlFlow<()> {
            let mut visited = HashSet::new();
            let active_id = self.active.id;
            let pending_id = self.pending.as_ref().map(|tree| tree.id);
            let recycle_id = self.recycle.map(|tree| tree.id);
            // During this source phase only the selected active receiver is
            // reserved. Twin queries borrow the actual pending tree directly.
            self.read.WithReadHost(
                HostPictureTreeRead {
                    id: Some(active_id),
                    tree: None,
                },
                HostPictureTreeRead {
                    id: pending_id,
                    tree: self.pending.as_deref(),
                },
                HostPictureTreeRead {
                    id: recycle_id,
                    tree: self.recycle,
                },
                |host| {
                    let queries = HostPictureSiblingQueries {
                        active: None,
                        pending: self
                            .pending
                            .as_deref()
                            .map(|tree| HostPictureSiblingTree { tree, host }),
                        recycle: self
                            .recycle
                            .map(|tree| HostPictureSiblingTree { tree, host }),
                    };
                    VisitResourceTree(
                        self.active,
                        active_id,
                        &self.read,
                        &queries,
                        &mut self.notification,
                        &mut visited,
                        callback,
                    )
                },
            )?;
            if let Some(pending) = self.pending.as_deref_mut() {
                self.read.WithReadHost(
                    HostPictureTreeRead {
                        id: Some(active_id),
                        tree: Some(self.active),
                    },
                    HostPictureTreeRead {
                        id: pending_id,
                        tree: None,
                    },
                    HostPictureTreeRead {
                        id: recycle_id,
                        tree: self.recycle,
                    },
                    |host| {
                        let queries = HostPictureSiblingQueries {
                            active: Some(HostPictureSiblingTree {
                                tree: self.active,
                                host,
                            }),
                            pending: None,
                            recycle: self
                                .recycle
                                .map(|tree| HostPictureSiblingTree { tree, host }),
                        };
                        VisitResourceTree(
                            pending,
                            active_id,
                            &self.read,
                            &queries,
                            &mut self.notification,
                            &mut visited,
                            callback,
                        )
                    },
                )?;
            }
            ControlFlow::Continue(())
        }
    }
    use crate::adapter::{
        picture_layer_tiling::{PendingInvalidationId, PictureLayerTilingClientId},
        tile::{PictureLayerTilingId, TileId},
        tile_manager::TileManagerImageFrameClient,
    };
    use crate::cc::tiles::{
        picture_layer_tiling::PictureLayerTiling,
        picture_layer_tiling_set::PictureLayerTilingSet,
        raster_tile_priority_queue::{PictureLayerTilingSetId, RasterQueueContext},
    };

    /// Native per-call manager context. In particular this object cannot borrow
    /// `tile_manager_` itself: that canonical owner is the source receiver.
    pub(crate) struct Runtime<'a, R: CpuSkiaBackend> {
        pub owner: LayerTreeHostId,
        pub active: &'a mut Option<Box<LayerTreeImpl<Cpu<R>>>>,
        pub pending: &'a mut Option<Box<LayerTreeImpl<Cpu<R>>>>,
        pub recycle: &'a mut Option<Box<LayerTreeImpl<Cpu<R>>>>,
        pub settings: &'a LayerTreeSettings,
        pub debug: &'a LayerTreeDebugState,
        pub caps: &'a RasterCapabilities,
        pub global: &'a GlobalStateThatImpactsTilePriority,
        pub high_res: &'a bool,
        pub viewport_priority: &'a Rect,
        pub external_viewport: &'a Rect,
        pub sink: &'a Option<HostImplFrameSinkId>,
        pub frame_tracker: &'a BeginFrameTracker,
        pub input: Option<InputHandlerView<'a>>,
        pub delegate: HostImplDelegateId,
        pub pool: &'a mut Option<
            Box<crate::cc::resources::resource_pool::ResourcePool<Cpu<R>>>,
        >,
        pub provider: &'a mut Option<Box<<Cpu<R> as LayerTreeTypes>::ClientResourceProvider>>,
        pub layer_context: &'a mut Option<Box<<Cpu<R> as HostImplTypes>::LayerContext>>,
        pub monitors: &'a BTreeSet<HostImplLatencyMonitorId>,
        pub events: &'a mut <Cpu<R> as HostImplTypes>::EventsMetricsManager,
        pub pending_fully_painted: &'a bool,
        pub likely_draw: &'a mut bool,
        pub image_cache: &'a mut Option<
            Box<ImageDecodeCacheHolder<<Cpu<R> as HostImplTypes>::ImageDecodeFactory>>,
        >,
        pub compositor_visibility:
            &'a mut Option<Box<<Cpu<R> as HostImplTypes>::ContextVisibility>>,
        pub worker_visibility: &'a mut Option<Box<<Cpu<R> as HostImplTypes>::ContextVisibility>>,
        pub image_frame: ImageFrameFields<'a>,
        pub execution: &'a mut crate::runtime::tile_execution::CpuTileExecution<R>,
        pub raster_provider: &'a mut Option<Box<<Cpu<R> as HostImplTypes>::RasterBufferProvider>>,
        pub raster_queries: &'a mut Option<Box<<Cpu<R> as HostImplTypes>::RasterQueryQueue>>,
        pub services: &'a mut dyn HostImplServices<Cpu<R>>,
    }
    impl<R: CpuSkiaBackend> Runtime<'_, R> {
        fn ReadFields(&self) -> TileReadFields<'_, R> {
            TileReadFields {
                settings: self.settings,
                debug: self.debug,
                caps: self.caps,
                global: self.global,
                high_res: self.high_res,
                viewport_priority: self.viewport_priority,
                external_viewport: self.external_viewport,
                sink: self.sink,
                frame_tracker: self.frame_tracker,
                input: self.input.as_ref().map(InputHandlerView::reborrow),
                delegate: self.delegate,
                environment: self.services.PictureReadEnvironment(),
                family: PhantomData,
            }
        }
        fn WithQueries<Output>(
            &self,
            body: impl FnOnce(&HostPictureSiblingQueries<'_, Cpu<R>>) -> Output,
        ) -> Output {
            let read = self.ReadFields();
            read.WithReadHost(
                HostPictureTreeRead {
                    id: self.active.as_ref().map(|t| t.id),
                    tree: self.active.as_deref(),
                },
                HostPictureTreeRead {
                    id: self.pending.as_ref().map(|t| t.id),
                    tree: self.pending.as_deref(),
                },
                HostPictureTreeRead {
                    id: self.recycle.as_ref().map(|t| t.id),
                    tree: self.recycle.as_deref(),
                },
                |host| {
                    let queries = HostPictureSiblingQueries {
                        active: self
                            .active
                            .as_deref()
                            .map(|tree| HostPictureSiblingTree { tree, host }),
                        pending: self
                            .pending
                            .as_deref()
                            .map(|tree| HostPictureSiblingTree { tree, host }),
                        recycle: self
                            .recycle
                            .as_deref()
                            .map(|tree| HostPictureSiblingTree { tree, host }),
                    };
                    body(&queries)
                },
            )
        }
        fn CheckClient(&self, client: crate::adapter::tile_manager::TileManagerClientId) {
            assert_eq!(
                client.0, self.owner.0,
                "source TileManager client is this actual Host"
            );
        }
        fn FindPictureIn<'a>(
            trees: [Option<&'a LayerTreeImpl<Cpu<R>>>; 3],
            client: PictureLayerTilingClientId,
        ) -> (
            &'a crate::cc::layers::picture_layer_impl::PictureLayerImpl<Cpu<R>>,
            &'a LayerTreeImpl<Cpu<R>>,
        ) {
            for tree in trees.into_iter().flatten() {
                for picture in tree.picture_layers() {
                    if picture.identity().stable as u64 == client.0 {
                        return (picture, tree);
                    }
                }
            }
            panic!("native Picture client pointer must belong to a current resident tree")
        }
        fn FindTilingIn<'a>(
            trees: [Option<&'a LayerTreeImpl<Cpu<R>>>; 3],
            id: PictureLayerTilingId,
        ) -> &'a PictureLayerTiling<Cpu<R>> {
            for tree in trees.into_iter().flatten() {
                for picture in tree.picture_layers() {
                    if let Some(tiling) = picture.picture_layer_tiling_set().TilingByIdIfPresent(id)
                    {
                        return tiling;
                    }
                }
            }
            panic!("native tiling pointer must belong to a current resident tree")
        }
        fn TilingMutableIn<'a>(
            trees: [Option<&'a mut LayerTreeImpl<Cpu<R>>>; 3],
            id: PictureLayerTilingId,
        ) -> &'a mut PictureLayerTiling<Cpu<R>> {
            use crate::adapter::layer_tree_impl::LayerTreeLayer;
            for tree in trees.into_iter().flatten() {
                let layer_id = tree
                    .picture_layers()
                    .find(|picture| {
                        picture
                            .picture_layer_tiling_set()
                            .TilingByIdIfPresent(id)
                            .is_some()
                    })
                    .map(|picture| picture.base().id());
                if let Some(layer_id) = layer_id {
                    return tree
                        .LayerByIdMutable(layer_id)
                        .expect("actual source layer")
                        .AsPictureLayerMut()
                        .expect("source Picture subtype")
                        .picture_layer_tiling_set_mut()
                        .TilingByIdMut(id);
                }
            }
            panic!("native mutable tiling pointer must belong to a current resident tree")
        }
    }
    impl<R: CpuSkiaBackend> PictureLayerTilingQueries<Cpu<R>> for Runtime<'_, R> {
        fn WithClient(
            &self,
            id: PictureLayerTilingClientId,
            callback: &mut dyn FnMut(
                &dyn crate::cc::tiles::picture_layer_tiling::PictureLayerTilingClient<
                    Cpu<R>,
                >,
            ),
        ) {
            self.WithQueries(|queries| queries.WithClient(id, callback))
        }
        fn PaintWorkletRecordsForClient(&self,id:PictureLayerTilingClientId)->&<Cpu<R> as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::PaintWorkletRecordMap{
            Self::FindPictureIn(
                [
                    self.active.as_deref(),
                    self.pending.as_deref(),
                    self.recycle.as_deref(),
                ],
                id,
            )
            .0
            .GetPaintWorkletRecordMap()
        }
        fn TilingById(&self, id: PictureLayerTilingId) -> &PictureLayerTiling<Cpu<R>> {
            Self::FindTilingIn(
                [
                    self.active.as_deref(),
                    self.pending.as_deref(),
                    self.recycle.as_deref(),
                ],
                id,
            )
        }
        fn IsClientOnActiveOrPendingTree(&self, id: PictureLayerTilingClientId) -> bool {
            // Source PictureLayerImpl::GetPendingOrActiveTwinLayer is a
            // nullable residency test. A twin from the tree just recycled by
            // activation is expected to miss; it must not enter the strict
            // dereference helper used by methods whose precondition is a live
            // resident owner.
            [self.active.as_deref(), self.pending.as_deref()]
                .into_iter()
                .flatten()
                .any(|tree| {
                    tree.picture_layers()
                        .any(|picture| picture.identity().stable as u64 == id.0)
                })
        }
        fn TilingStateForClientAndScale(
            &self,
            id: PictureLayerTilingClientId,
            scale: f32,
        ) -> Option<&PictureLayerTilingState<Cpu<R>>> {
            Self::FindPictureIn(
                [
                    self.active.as_deref(),
                    self.pending.as_deref(),
                    self.recycle.as_deref(),
                ],
                id,
            )
            .0
            .picture_layer_tiling_set()
            .FindTilingWithScaleKeyRef(scale)
            .map(|tiling| &**tiling)
        }
        fn PendingInvalidationById(&self, id: PendingInvalidationId) -> &Region<R> {
            Self::FindPictureIn(
                [
                    self.active.as_deref(),
                    self.pending.as_deref(),
                    self.recycle.as_deref(),
                ],
                PictureLayerTilingClientId(id.0),
            )
            .0
            .InvalidationForTesting()
        }
    }
    impl<R: CpuSkiaBackend> RasterQueueContext<Cpu<R>> for Runtime<'_, R> {
        fn TilingSetById(&self, id: PictureLayerTilingSetId) -> &PictureLayerTilingSet<Cpu<R>> {
            for tree in [
                self.active.as_deref(),
                self.pending.as_deref(),
                self.recycle.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                for picture in tree.picture_layers() {
                    let set = picture.picture_layer_tiling_set();
                    if set.identity() == id {
                        return set;
                    }
                }
            }
            panic!("source resident tiling set identity")
        }
        fn IsCCSlimmingEnabled(&self) -> bool {
            self.services.PictureReadEnvironment().IsCCSlimmingEnabled()
        }
    }

    pub(super) struct ImageFrameFields<'a> {
        pub settings: &'a LayerTreeSettings,
        pub animation: &'a Option<Box<ImageAnimationController>>,
    }
    impl<R: CpuSkiaBackend> TileManagerImageFrameClient<Cpu<R>> for ImageFrameFields<'_> {
        fn GetFrameIndexForImage(
            &self,
            image: &crate::runtime::paint::CpuPaintImage,
            tree: crate::adapter::image_animation::WhichTree,
        ) -> usize {
            GetFrameIndexForImageForLiveParts(image, tree, self.settings, self.animation)
        }
    }

    use crate::adapter::{
        resource_pool::{PoolClockId, PoolSharedImageInterface, ResourcePoolId},
        task_runner::{OnceClosure, RepeatingClosure, SequencedTaskRunner},
        tile::{TileTraceContext, TileTraceId},
        tile_manager::*,
        time::{TimeDelta, TimeTicks},
    };
    use crate::runtime::{
        execution::EngineTask,
        paint::{CpuPlaybackImageProvider, CpuWorkletProvider},
        resources::CpuColorSpace,
    };
    use crate::cc::resources::resource_pool::{InUsePoolResource, ResourcePool};
    use std::sync::{Arc, Weak};
    impl<R: CpuSkiaBackend> TileManagerInitialization<Cpu<R>> for Runtime<'_, R> {
        fn CreateImageController(
            &mut self,
            origin: Arc<dyn SequencedTaskRunner>,
            worker: Arc<dyn SequencedTaskRunner>,
            owner: crate::adapter::tile::TileManagerId,
        ) -> ImageControllerId {
            self.execution.CreateImageController(origin, worker, owner)
        }
        fn CreateDecodedImageTracker(
            &mut self,
            controller: ImageControllerId,
            origin: Arc<dyn SequencedTaskRunner>,
        ) -> DecodedImageTrackerId {
            self.execution.CreateDecodedImageTracker(controller, origin)
        }
        fn CreateCheckerImageTracker(
            &mut self,
            controller: ImageControllerId,
            owner: crate::adapter::tile::TileManagerId,
            enabled: bool,
            min: usize,
        ) -> CheckerImageTrackerId {
            self.execution
                .CreateCheckerImageTracker(controller, owner, enabled, min)
        }
        fn HasCurrentDefaultTaskRunner(&self) -> bool {
            self.execution.HasCurrentDefaultTaskRunner()
        }
        fn RegisterDumpProvider(&mut self, owner: crate::adapter::tile::TileManagerId, name: &str) {
            self.execution.RegisterDumpProvider(owner, name)
        }
        fn RepeatingCommand(
            &self,
            owner: crate::adapter::tile::TileManagerId,
            alive: Weak<()>,
            command: crate::cc::tiles::tile_manager::TileManagerCommand,
        ) -> RepeatingClosure {
            self.execution.RepeatingCommand(owner, alive, command)
        }
    }
    impl<R: CpuSkiaBackend> TileTraceContext for Runtime<'_, R> {
        fn MakeDictIntoImplicitSnapshotWithCategory<V: PropertyTrace>(
            &self,
            category: &str,
            value: &mut V,
            name: &str,
            id: TileTraceId,
        ) {
            self.execution
                .MakeDictIntoImplicitSnapshotWithCategory(category, value, name, id)
        }
        fn TraceTileDeleted(&self, category: &str, event: &str, id: TileTraceId, flow: &str) {
            self.execution.TraceTileDeleted(category, event, id, flow)
        }
    }
    // Source: cc/raster/zero_copy_raster_buffer_provider.cc:190. There is no
    // deferred GPU command stream in the native CPU provider.
    struct CpuRasterFlush;
    impl crate::cc::tiles::tile_manager::TileManagerRasterFlush for CpuRasterFlush {
        fn FlushTileRasterGraphiteCommands(&self) {}
    }
    impl<R: CpuSkiaBackend> Runtime<'_, R> {
        fn CheckRasterProvider(&self, id: RasterBufferProviderId) {
            let provider = self
                .raster_provider
                .as_deref()
                .expect("source raster provider");
            assert_eq!(
                id.0, provider as *const _ as usize as u64,
                "same native source raster provider"
            );
        }
        fn ResourcePool(&self, id: ResourcePoolId) -> &ResourcePool<Cpu<R>> {
            let pool = self.pool.as_deref().expect("source resource pool");
            assert_eq!(pool.identity(), id, "same native resident resource pool");
            pool
        }
        fn ResourcePoolMutable(&mut self, id: ResourcePoolId) -> &mut ResourcePool<Cpu<R>> {
            let pool = self.pool.as_deref_mut().expect("source resource pool");
            assert_eq!(pool.identity(), id, "same native resident resource pool");
            pool
        }
        fn RasterBufferProvider(
            &mut self,
            id: RasterBufferProviderId,
        ) -> &mut dyn RasterBufferProvider<Cpu<R>> {
            self.CheckRasterProvider(id);
            self.raster_provider.as_deref_mut().unwrap().0.as_mut()
        }
        fn ImageController(&mut self, id: ImageControllerId) -> &mut dyn ImageController<Cpu<R>> {
            self.execution.images.ImageController(id)
        }
        fn DecodedImageTracker(
            &mut self,
            id: DecodedImageTrackerId,
        ) -> &mut dyn DecodedImageTracker<Cpu<R>> {
            self.execution.images.DecodedImageTracker(id)
        }
        fn CheckerImageTracker(
            &mut self,
            id: CheckerImageTrackerId,
        ) -> &mut dyn CheckerImageTracker<Cpu<R>> {
            self.execution.images.CheckerImageTracker(id)
        }
        fn DestroyImageController(&mut self, id: ImageControllerId) {
            self.execution.images.DestroyImageController(id)
        }
        fn DestroyDecodedImageTracker(&mut self, id: DecodedImageTrackerId) {
            self.execution.images.DestroyDecodedImageTracker(id)
        }
        fn DestroyCheckerImageTracker(&mut self, id: CheckerImageTrackerId) {
            self.execution.images.DestroyCheckerImageTracker(id)
        }
        fn CreatePlaybackImageProvider(
            &mut self,
            cache: Option<ImageDecodeCacheId>,
            target: &crate::runtime::paint::TargetColorParams,
            settings: PlaybackImageProviderSettings<Cpu<R>>,
        ) -> CpuPlaybackImageProvider<R> {
            self.execution
                .images
                .CreatePlaybackImageProvider(cache, target, settings)
        }
        fn CreatePaintWorkletImageProvider(
            &mut self,
            records:<Cpu<R> as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::PaintWorkletRecordMap,
        ) -> CpuWorkletProvider<R> {
            self.execution
                .images
                .CreatePaintWorkletImageProvider(records)
        }
        fn AcquireRasterBufferForResource(
            &mut self,
            provider: RasterBufferProviderId,
            pool_id: ResourcePoolId,
            resource: &InUsePoolResource<Cpu<R>>,
            content: u64,
            previous: u64,
        ) -> Option<crate::runtime::paint::CpuTileRasterBuffer<R>> {
            self.CheckRasterProvider(provider);
            let pool = self.pool.as_deref_mut().expect("source resource pool");
            assert_eq!(pool.identity(), pool_id, "same source pool");
            self.raster_provider
                .as_deref_mut()
                .unwrap()
                .0
                .AcquireBufferForRaster(resource, pool, content, previous)
        }
        fn RasterQueryCapability(
            &self,
            id: RasterQueryQueueId,
        ) -> Arc<dyn crate::cc::tiles::tile_manager::TileManagerRasterQueryQueue>
        {
            let query = self
                .raster_queries
                .as_deref()
                .expect("source raster query queue exists");
            assert_eq!(
                id.0, query as *const _ as usize as u64,
                "same native query queue"
            );
            query.0.clone()
        }
        fn CheckRasterFinishedQueries(&mut self, id: RasterQueryQueueId) -> bool {
            self.RasterQueryCapability(id).CheckRasterFinishedQueries()
        }
        fn RasterFlushCapability(
            &self,
            id: RasterBufferProviderId,
        ) -> Arc<dyn crate::cc::tiles::tile_manager::TileManagerRasterFlush> {
            self.CheckRasterProvider(id);
            Arc::new(CpuRasterFlush)
        }
        fn TestSharedImageInterface(&self) -> &dyn PoolSharedImageInterface<Cpu<R>> {
            &self.execution.shared_images
        }
        fn TaskManagerServices(
            &self,
        ) -> Arc<dyn crate::adapter::raster_task::TileTaskManagerServices> {
            self.execution.diagnostics.clone()
        }
        fn WorkerDiagnostics(
            &self,
        ) -> Arc<dyn crate::cc::tiles::tile_manager::TileManagerWorkerDiagnostics>
        {
            self.execution.diagnostics.clone()
        }
        fn GraphTrace(
            &self,
        ) -> &dyn crate::cc::tiles::tile_manager::TileManagerGraphTrace {
            self.execution.diagnostics.as_ref()
        }
        fn OnceCommand(
            &self,
            owner: crate::adapter::tile::TileManagerId,
            alive: Weak<()>,
            command: crate::cc::tiles::tile_manager::TileManagerCommand,
        ) -> OnceClosure {
            let commands = self.execution.commands.clone();
            Box::new(move || {
                if alive.strong_count() != 0 {
                    commands.Push(EngineTask::Tile {
                        owner,
                        alive,
                        command,
                    });
                }
            })
        }
        fn AllTilesDoneCommand(
            &self,
            owner: crate::adapter::tile::TileManagerId,
            alive: Weak<()>,
            start: TimeTicks,
        ) -> Box<dyn FnOnce(bool) + Send> {
            let commands = self.execution.commands.clone();
            Box::new(move |all_tiles_done| {
                if alive.strong_count() != 0 {
                    commands.Push(EngineTask::TileTasksFinished {
                        owner,
                        alive,
                        start,
                        all_tiles_done,
                    });
                }
            })
        }
        fn FeatureEnabled(&self, feature: TileManagerFeature) -> bool {
            let config = &self.execution.config;
            match feature {
                TileManagerFeature::ReclaimPrepaintTilesWhenIdle => {
                    config.reclaim_prepaint_when_idle
                }
                TileManagerFeature::ReclaimOldPrepaintTiles => config.reclaim_old_prepaint,
                TileManagerFeature::TileOOMFreezeMitigation => config.tile_oom_freeze_mitigation,
            }
        }
        fn ReclaimDelayInSeconds(&self) -> i64 {
            self.execution.config.reclaim_delay_seconds
        }
        fn Now(&self) -> TimeTicks {
            self.execution.origin.Now()
        }
        fn NowWithClock(&self, clock: PoolClockId) -> TimeTicks {
            assert_eq!(
                clock,
                PoolClockId(1),
                "CPU pool uses its injected default monotonic clock"
            );
            self.Now()
        }
        fn ShouldRecordSubsampledMetric(&self, rate: f32) -> bool {
            self.services.ShouldRecordSubsampledMetric(rate as f64)
        }
        fn UnregisterDumpProvider(&mut self, owner: crate::adapter::tile::TileManagerId) {
            self.execution.diagnostics.events.Emit(
                "compositor.tile_manager",
                format!("unregister_dump owner={}", owner.0),
            );
        }
        fn TraceScope(&self, event: &'static str) -> Box<dyn TileManagerTraceScope> {
            Box::new(self.execution.diagnostics.Scope(event))
        }
        fn TraceScopeWithArgs(
            &self,
            event: &'static str,
            arguments: impl FnOnce() -> Vec<TileManagerTraceArgument>,
        ) -> Box<dyn TileManagerTraceScope> {
            if self.execution.config.trace_enabled {
                for argument in arguments() {
                    let message = match argument {
                        TileManagerTraceArgument::Unsigned(name, value) => {
                            format!("{event} {name}={value}")
                        }
                        TileManagerTraceArgument::String(name, value) => {
                            format!("{event} {name}={value}")
                        }
                    };
                    self.execution
                        .diagnostics
                        .events
                        .Emit("compositor.tile_manager", message);
                }
            }
            self.TraceScope(event)
        }
        fn TraceBeginDefault(&self, event: &'static str) {
            if self.execution.config.trace_enabled {
                self.execution
                    .diagnostics
                    .events
                    .Emit("compositor.tile_manager", format!("{event} begin"));
            }
        }
        fn TraceEndWithState(
            &self,
            state: impl FnOnce() -> crate::adapter::property_types::TracedValueJSON,
        ) {
            if self.execution.config.trace_enabled {
                self.execution
                    .diagnostics
                    .events
                    .Emit("compositor.tile_manager", state().ToFormattedJSON());
            }
        }
        fn TraceBeginOnTrack(&self, event: &'static str, track: TileManagerTraceTrack) {
            if self.execution.config.trace_enabled {
                self.execution.diagnostics.events.Emit(
                    "compositor.tile_manager",
                    format!("{event} begin {}:{}", track.name, track.owner.0),
                );
            }
        }
        fn TraceEndOnTrack(&self, track: TileManagerTraceTrack) {
            if self.execution.config.trace_enabled {
                self.execution.diagnostics.events.Emit(
                    "compositor.tile_manager",
                    format!("end {}:{}", track.name, track.owner.0),
                );
            }
        }
        fn TraceInstantOnTrack(
            &self,
            event: &'static str,
            track: TileManagerTraceTrack,
            state: impl FnOnce() -> Option<crate::adapter::property_types::TracedValueJSON>,
        ) {
            if self.execution.config.trace_enabled {
                self.execution.diagnostics.events.Emit(
                    "compositor.tile_manager",
                    format!(
                        "{event} {}:{} {}",
                        track.name,
                        track.owner.0,
                        state().map(|v| v.ToFormattedJSON()).unwrap_or_default()
                    ),
                );
            }
        }
        fn TraceInstant(
            &self,
            event: &'static str,
            state: impl FnOnce() -> Option<crate::adapter::property_types::TracedValueJSON>,
        ) {
            if self.execution.config.trace_enabled {
                self.execution.diagnostics.events.Emit(
                    "compositor.tile_manager",
                    format!(
                        "{event} {}",
                        state().map(|v| v.ToFormattedJSON()).unwrap_or_default()
                    ),
                );
            }
        }
        fn LogError(&self, message: &'static str) {
            self.execution
                .diagnostics
                .events
                .Emit("compositor.error", message.to_owned());
        }
        fn HistogramBoolean(&self, name: &'static str, value: bool) {
            self.execution
                .diagnostics
                .events
                .HistogramCount(name, if value { 1.0 } else { 0.0 });
        }
        fn HistogramMemoryMB(&self, name: &'static str, value: i64) {
            self.execution
                .diagnostics
                .events
                .HistogramCount(name, value as f32);
        }
        fn HistogramDuration(
            &self,
            name: &'static str,
            value: TimeDelta,
            _min: TimeDelta,
            _max: TimeDelta,
            _buckets: usize,
        ) {
            self.execution.diagnostics.events.HistogramTime(name, value);
        }
    }

    use crate::adapter::layer_tree_impl::LayerTreeLayer;
    use crate::cc::tiles::eviction_tile_priority_queue::EvictionTilePriorityQueue;
    use crate::cc::tiles::raster_tile_priority_queue::{RasterTilePriorityQueue, Type};
    use crate::cc::tiles::tile_priority::{TreePriority, WhichTree};
    struct CurrentFullPictureNotification<'a, 'p, R: CpuSkiaBackend> {
        parts:
            &'a mut crate::cc::layers::picture_layer_impl::PictureTileNotificationParts<
                'p,
                Cpu<R>,
            >,
        push: &'a mut dyn LayerImplPushContext<Types = Cpu<R>>,
    }
    impl<R: CpuSkiaBackend> HostPictureNotification<Cpu<R>>
        for CurrentFullPictureNotification<'_, '_, R>
    {
        fn NotifyTileStateChanged(&mut self, tile: &Tile<Cpu<R>>, damage: bool) {
            self.parts.Notify(tile, damage, self.push)
        }
        fn layer(&self) -> &dyn PictureLayerDisplayRead {
            self.parts.layer()
        }
        fn should_batch_updated_tiles(&self) -> bool {
            self.parts.should_batch_updated_tiles()
        }
    }
    impl<R: CpuSkiaBackend> Runtime<'_, R> {
        fn TileByIdMutable(&mut self, id: TileId) -> &mut Tile<Cpu<R>> {
            Self::TilingMutableIn(
                [
                    self.active.as_deref_mut(),
                    self.pending.as_deref_mut(),
                    self.recycle.as_deref_mut(),
                ],
                id.tiling,
            )
            .TileByIdMutable(id)
        }
        fn TileAndPool(
            &mut self,
            id: TileId,
            pool: ResourcePoolId,
        ) -> (&mut Tile<Cpu<R>>, &mut ResourcePool<Cpu<R>>) {
            let pool_ref = self.pool.as_deref_mut().expect("source pool");
            assert_eq!(pool_ref.identity(), pool);
            let tiling = Self::TilingMutableIn(
                [
                    self.active.as_deref_mut(),
                    self.pending.as_deref_mut(),
                    self.recycle.as_deref_mut(),
                ],
                id.tiling,
            );
            (tiling.TileByIdMutable(id), pool_ref)
        }
        fn TileRasterSourceAndPool(&mut self,id:TileId,source:PictureLayerTilingId,pool:ResourcePoolId)->(&Tile<Cpu<R>>,&<Cpu<R> as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource,&mut ResourcePool<Cpu<R>>){
            let pool_ref = self.pool.as_deref_mut().expect("source pool");
            assert_eq!(pool_ref.identity(), pool);
            let trees = [
                self.active.as_deref(),
                self.pending.as_deref(),
                self.recycle.as_deref(),
            ];
            let tile = Self::FindTilingIn(trees, id.tiling).TileById(id);
            let source = Self::FindTilingIn(trees, source)
                .raster_source()
                .as_deref()
                .expect("source raster source");
            (tile, source, pool_ref)
        }
        fn ImagePartitionParts(
            &mut self,
            source: PictureLayerTilingId,
            id: TileId,
            rect: Rect,
            client: TileManagerClientId,
            checker: CheckerImageTrackerId,
        ) -> ImagePartitionParts<'_, Cpu<R>> {
            self.CheckClient(client);
            let tiling = Self::FindTilingIn(
                [
                    self.active.as_deref(),
                    self.pending.as_deref(),
                    self.recycle.as_deref(),
                ],
                source,
            );
            let (picture, _) = Self::FindPictureIn(
                [
                    self.active.as_deref(),
                    self.pending.as_deref(),
                    self.recycle.as_deref(),
                ],
                tiling.client(),
            );
            ImagePartitionParts {
                tile: Self::FindTilingIn(
                    [
                        self.active.as_deref(),
                        self.pending.as_deref(),
                        self.recycle.as_deref(),
                    ],
                    id.tiling,
                )
                .TileById(id),
                tile_tiling: tiling,
                images: picture.GetDiscardableImagesInRect(&rect),
                client: &self.image_frame,
                checker: self.execution.images.CheckerImageTracker(checker),
            }
        }
        fn GetTileFormat(
            &self,
            client: TileManagerClientId,
        ) -> crate::adapter::resource_types::SharedImageFormat {
            self.CheckClient(client);
            self.caps.tile_format
        }
        fn GetTargetColorParams(
            &self,
            client: TileManagerClientId,
            usage: ContentColorUsage,
        ) -> HostTargetColorParams<CpuColorSpace> {
            self.CheckClient(client);
            TargetColorParamsForHost::<Cpu<R>>(*self.sink, usage, self.services, || {
                self.pending
                    .as_deref()
                    .or(self.active.as_deref())
                    .map(|tree| tree.display_color_spaces().clone())
                    .unwrap_or_default()
            })
        }
        fn GetMSAASampleCountForRaster(
            &self,
            client: TileManagerClientId,
            list: &crate::runtime::paint::ContentDisplayItemList<R>,
        ) -> i32 {
            self.CheckClient(client);
            RasterMSAASampleCountForHost(
                list,
                self.caps,
                self.settings,
                || self.services.PictureReadEnvironment().IsLowEndDevice(),
                || {
                    self.pending
                        .as_deref()
                        .or(self.active.as_deref())
                        .expect("source scale tree")
                        .device_scale_factor()
                },
            )
        }
        fn IsResourceReadyToDrawForTile(
            &mut self,
            provider: RasterBufferProviderId,
            id: TileId,
        ) -> bool {
            self.CheckRasterProvider(provider);
            let tile = Self::FindTilingIn(
                [
                    self.active.as_deref(),
                    self.pending.as_deref(),
                    self.recycle.as_deref(),
                ],
                id.tiling,
            )
            .TileById(id);
            self.raster_provider
                .as_deref()
                .unwrap()
                .0
                .IsResourceReadyToDraw(tile.draw_info().GetResource())
        }
        fn SetReadyToDrawCallbackForTiles(
            &mut self,
            provider: RasterBufferProviderId,
            _resources: &[TileId],
            _callback: OnceClosure,
            _previous: u64,
        ) -> u64 {
            self.CheckRasterProvider(provider);
            // Source: cc/raster/zero_copy_raster_buffer_provider.cc:203-209.
            // CPU playback publishes before task completion, so no delayed
            // GPU readiness callback exists. This is the native CPU branch.
            0
        }
        fn BuildRasterQueue(
            &mut self,
            client: TileManagerClientId,
            priority: TreePriority,
            kind: Type,
        ) -> Box<dyn RasterTilePriorityQueue<Cpu<R>>> {
            self.CheckClient(client);
            self.WithQueries(|queries| {
                <dyn RasterTilePriorityQueue<Cpu<R>>>::Create(
                    HostPictureQueueLayers::new(queries.active.as_ref()),
                    HostPictureQueueLayers::new(queries.pending.as_ref()),
                    priority,
                    kind,
                    queries,
                )
            })
        }
        fn BuildEvictionQueue(
            &mut self,
            client: TileManagerClientId,
        ) -> Box<EvictionTilePriorityQueue<Cpu<R>>> {
            self.CheckClient(client);
            self.WithQueries(|queries| {
                let mut queue = Box::new(EvictionTilePriorityQueue::new());
                queue.Build(
                    HostPictureQueueLayers::new(queries.active.as_ref()),
                    HostPictureQueueLayers::new(queries.pending.as_ref()),
                    queries,
                );
                queue
            })
        }
        fn NotifyReadyToActivate(&mut self, client: TileManagerClientId) {
            self.CheckClient(client);
            NotifyReadyToActivateForLiveParts::<Cpu<R>>(
                *self.pending_fully_painted,
                self.delegate,
                self.services,
            )
        }
        fn NotifyReadyToDraw(&mut self, client: TileManagerClientId) {
            self.CheckClient(client);
            NotifyReadyToDrawForLiveParts::<Cpu<R>>(self.likely_draw, self.delegate, self.services)
        }
        fn NotifyAllTileTasksCompleted(&mut self, client: TileManagerClientId) {
            self.CheckClient(client);
            NotifyAllTileTasksCompletedForLiveParts::<Cpu<R>>(
                self.settings,
                self.global,
                self.image_cache,
                *self.sink,
                self.compositor_visibility,
                self.worker_visibility,
                self.services,
            )
        }
        fn HasPendingTree(&mut self, client: TileManagerClientId) -> bool {
            self.CheckClient(client);
            self.pending.is_some()
        }
        fn SetNeedsRedraw(
            &mut self,
            client: TileManagerClientId,
            animation_only: bool,
            skip: bool,
        ) {
            self.CheckClient(client);
            let promises = RequestRedrawForHost::<Cpu<R>>(
                self.settings,
                self.delegate,
                self.monitors,
                self.events,
                animation_only,
                skip,
                &mut HostImplContext {
                    services: self.services,
                },
            );
            QueuePinnedLatencyPromisesOnActiveTree(
                self.active.as_deref_mut().expect("source active tree"),
                promises,
            );
        }
        fn SetIsLikelyToRequireADraw(&mut self, client: TileManagerClientId, value: bool) {
            self.CheckClient(client);
            *self.likely_draw = value;
        }
        fn RequestImplSideInvalidationForCheckerImagedTiles(
            &mut self,
            client: TileManagerClientId,
        ) {
            self.CheckClient(client);
            self.services
                .SetNeedsImplSideInvalidation(self.delegate, false);
        }
        fn WithTilesWithResourceIterator<Output>(
            &mut self,
            client: TileManagerClientId,
            pool_id: ResourcePoolId,
            body: impl for<'s> FnOnce(ResourceIterationParts<'s, Cpu<R>>) -> Output,
        ) -> Output {
            self.CheckClient(client);
            let mut body = Some(body);
            let mut result = None;
            let pool = self.pool.as_deref_mut().expect("source initialized pool");
            assert_eq!(pool.identity(), pool_id);
            let active = self.active.as_deref_mut().expect("source active tree");
            let mut pending = self.pending.as_deref_mut();
            let recycle = self.recycle.as_deref();
            let input = self.input.as_ref().map(InputHandlerView::reborrow);
            let mut latency_promises = Vec::new();
            self.services.WithPictureExternalParts(
                &mut |environment, _picture, _tile, _mutation, _registration, release| {
                    let read = TileReadFields {
                        settings: self.settings,
                        debug: self.debug,
                        caps: self.caps,
                        global: self.global,
                        high_res: self.high_res,
                        viewport_priority: self.viewport_priority,
                        external_viewport: self.external_viewport,
                        sink: self.sink,
                        frame_tracker: self.frame_tracker,
                        input: input.as_ref().map(InputHandlerView::reborrow),
                        delegate: self.delegate,
                        environment,
                        family: PhantomData,
                    };
                    let traversal = ResourceTraversal {
                        active,
                        pending: pending.as_deref_mut(),
                        recycle,
                        read,
                        notification: HostPictureNotificationParts {
                            settings: self.settings,
                            delegate: self.delegate,
                            monitors: self.monitors,
                            events: self.events,
                            provider: self.provider,
                            layer_context: self.layer_context,
                            sink: self.sink,
                            latency_promises: &mut latency_promises,
                            services: release,
                        },
                    };
                    result = Some(body
                        .take()
                        .expect("source external field split executes once")(
                        ResourceIterationParts {
                            traversal: Box::new(traversal),
                            pool,
                        },
                    ));
                },
            );
            QueuePinnedLatencyPromisesOnActiveTree(
                self.active.as_deref_mut().expect("source active tree"),
                latency_promises,
            );
            result.expect("source external field split executes synchronously")
        }
    }

    fn NotifyOnTree<R: CpuSkiaBackend>(
        tree: &mut LayerTreeImpl<Cpu<R>>,
        active_id: LayerTreeId,
        id: TileId,
        layer_id: i32,
        external: Option<&Tile<Cpu<R>>>,
        damage: bool,
        redraw: bool,
        environment: &dyn HostPictureReadEnvironment,
        host: &mut HostPictureNotificationParts<'_, Cpu<R>>,
    ) {
        use crate::adapter::layer_tree_impl::LayerTreeLayerStorage;
        let (layers, fields) = tree.SplitLayerTreeFields();
        layers.WithStorageAndTracking(|storage, tracking| {
            let Some(layer) = storage.LayerByIdMut(layer_id) else {
                return;
            };
            let Some(picture) = layer.AsPictureLayerMut() else {
                return;
            };
            let mut push = ResourcePush::<R> {
                id: fields.id,
                active: active_id,
                settings: host.settings,
                environment,
                should_push: tracking.should_push_properties,
                needs_update: fields.needs_update_draw_properties,
                family: PhantomData,
            };
            let mut parts = picture.TileNotificationParts();
            if let Some(tile) = external {
                parts.WithTileTilingState(id.tiling, |state, parts| {
                    let mut notification = CurrentFullPictureNotification {
                        parts,
                        push: &mut push,
                    };
                    host.NotifyCurrent(state, Some(&mut notification), tile, damage, redraw)
                });
            } else {
                parts.WithResidentTile(id, |tile, state, parts| {
                    let mut notification = CurrentFullPictureNotification {
                        parts,
                        push: &mut push,
                    };
                    host.NotifyCurrent(state, Some(&mut notification), tile, damage, redraw)
                });
            }
        });
    }
    impl<R: CpuSkiaBackend> Runtime<'_, R> {
        fn NotifyTileStateChanged(
            &mut self,
            client: TileManagerClientId,
            tile: &Tile<Cpu<R>>,
            damage: bool,
            redraw: bool,
        ) {
            self.CheckClient(client);
            self.NotifyTile(tile.identity(), tile.layer_id(), Some(tile), damage, redraw)
        }
        fn NotifyTileStateChangedById(
            &mut self,
            client: TileManagerClientId,
            id: TileId,
            damage: bool,
            redraw: bool,
        ) {
            self.CheckClient(client);
            let tile = Self::FindTilingIn(
                [
                    self.active.as_deref(),
                    self.pending.as_deref(),
                    self.recycle.as_deref(),
                ],
                id.tiling,
            )
            .TileById(id);
            self.NotifyTile(id, tile.layer_id(), None, damage, redraw)
        }
        fn NotifyTile(
            &mut self,
            id: TileId,
            layer_id: i32,
            external: Option<&Tile<Cpu<R>>>,
            damage: bool,
            redraw: bool,
        ) {
            // Source: cc/trees/layer_tree_host_impl.cc:2068-2080. The source's
            // non-owning tiling pointer is resolved through its actual pending
            // or active Picture, and detached tilings retain source early exit.
            let tree_for_id = |tree: Option<&LayerTreeImpl<Cpu<R>>>| {
                tree.and_then(|tree| tree.LayerById(layer_id))
                    .and_then(|layer| layer.AsPictureLayer())
                    .and_then(|picture| {
                        picture
                            .picture_layer_tiling_set()
                            .TilingByIdIfPresent(id.tiling)
                    })
                    .map(|tiling| tiling.tree())
            };
            let Some(role) = tree_for_id(self.pending.as_deref())
                .or_else(|| tree_for_id(self.active.as_deref()))
            else {
                return;
            };
            let active_id = self.active.as_ref().expect("source active tree").id;
            let tree = if role == WhichTree::PENDING_TREE {
                self.pending.as_deref_mut().unwrap()
            } else {
                self.active.as_deref_mut().unwrap()
            };
            let mut latency_promises = Vec::new();
            self.services.WithPictureExternalParts(
                &mut |environment, _picture, _tiles, _mutation, _registration, release| {
                    let mut host = HostPictureNotificationParts {
                        settings: self.settings,
                        delegate: self.delegate,
                        monitors: self.monitors,
                        events: self.events,
                        provider: self.provider,
                        layer_context: self.layer_context,
                        sink: self.sink,
                        latency_promises: &mut latency_promises,
                        services: release,
                    };
                    NotifyOnTree(
                        tree,
                        active_id,
                        id,
                        layer_id,
                        external,
                        damage,
                        redraw,
                        environment,
                        &mut host,
                    );
                },
            );
            QueuePinnedLatencyPromisesOnActiveTree(
                self.active.as_deref_mut().expect("source active tree"),
                latency_promises,
            );
        }
    }

    impl<R: CpuSkiaBackend> TileManagerRuntime<Cpu<R>> for Runtime<'_, R> {
        fn UnregisterDumpProvider(&mut self, owner: TileManagerId) {
            Runtime::<R>::UnregisterDumpProvider(self, owner)
        }
        fn OnceCommand(
            &self,
            owner: TileManagerId,
            lifetime: Weak<()>,
            command: TileManagerCommand,
        ) -> crate::adapter::task_runner::OnceClosure {
            Runtime::<R>::OnceCommand(self, owner, lifetime, command)
        }
        fn CheckRasterFinishedQueries(&mut self, id: RasterQueryQueueId) -> bool {
            Runtime::<R>::CheckRasterFinishedQueries(self, id)
        }
        fn IsResourceReadyToDrawForTile(
            &mut self,
            provider: RasterBufferProviderId,
            tile: TileId,
        ) -> bool {
            Runtime::<R>::IsResourceReadyToDrawForTile(self, provider, tile)
        }
        fn SetReadyToDrawCallbackForTiles(
            &mut self,
            provider: RasterBufferProviderId,
            resources: &[TileId],
            callback: crate::adapter::task_runner::OnceClosure,
            previous_callback_id: u64,
        ) -> u64 {
            Runtime::<R>::SetReadyToDrawCallbackForTiles(
                self,
                provider,
                resources,
                callback,
                previous_callback_id,
            )
        }
        fn TaskManagerServices(
            &self,
        ) -> Arc<dyn crate::adapter::raster_task::TileTaskManagerServices> {
            Runtime::<R>::TaskManagerServices(self)
        }
        fn DestroyCheckerImageTracker(&mut self, id: CheckerImageTrackerId) {
            Runtime::<R>::DestroyCheckerImageTracker(self, id)
        }
        fn DestroyDecodedImageTracker(&mut self, id: DecodedImageTrackerId) {
            Runtime::<R>::DestroyDecodedImageTracker(self, id)
        }
        fn DestroyImageController(&mut self, id: ImageControllerId) {
            Runtime::<R>::DestroyImageController(self, id)
        }
        fn ImagePartitionParts(
            &mut self,
            source_tiling: crate::adapter::tile::PictureLayerTilingId,
            tile: TileId,
            enclosing_rect: Rect,
            client: TileManagerClientId,
            checker: CheckerImageTrackerId,
        ) -> ImagePartitionParts<'_, Cpu<R>> {
            Runtime::<R>::ImagePartitionParts(
                self,
                source_tiling,
                tile,
                enclosing_rect,
                client,
                checker,
            )
        }
        fn WithTilesWithResourceIterator<Output>(
            &mut self,
            client: TileManagerClientId,
            pool: ResourcePoolId,
            body: impl for<'s> FnOnce(ResourceIterationParts<'s, Cpu<R>>) -> Output,
        ) -> Output {
            Runtime::<R>::WithTilesWithResourceIterator(self, client, pool, body)
        }
        fn GetTargetColorParams(
            &self,
            client: TileManagerClientId,
            usage: <Cpu<R> as TileManagerTypes>::ContentColorUsage,
        ) -> <Cpu<R> as TileManagerTypes>::TargetColorParams {
            Runtime::<R>::GetTargetColorParams(self, client, usage)
        }
        fn GetTileFormat(
            &self,
            client: TileManagerClientId,
        ) -> crate::adapter::resource_types::SharedImageFormat {
            Runtime::<R>::GetTileFormat(self, client)
        }
        fn GraphTrace(
            &self,
        ) -> &dyn crate::cc::tiles::tile_manager::TileManagerGraphTrace {
            Runtime::<R>::GraphTrace(self)
        }
        fn ShouldRecordSubsampledMetric(&self, sampling_rate: f32) -> bool {
            Runtime::<R>::ShouldRecordSubsampledMetric(self, sampling_rate)
        }
        fn TraceBeginDefault(&self, event: &'static str) {
            Runtime::<R>::TraceBeginDefault(self, event)
        }
        fn TraceEndWithState(
            &self,
            state: impl FnOnce() -> crate::adapter::property_types::TracedValueJSON,
        ) {
            Runtime::<R>::TraceEndWithState(self, state)
        }
        fn LogError(&self, message: &'static str) {
            Runtime::<R>::LogError(self, message)
        }
        fn HistogramBoolean(&self, name: &'static str, value: bool) {
            Runtime::<R>::HistogramBoolean(self, name, value)
        }
        fn HistogramMemoryMB(&self, name: &'static str, value: i64) {
            Runtime::<R>::HistogramMemoryMB(self, name, value)
        }
        fn TraceBeginOnTrack(&self, event: &'static str, track: TileManagerTraceTrack) {
            Runtime::<R>::TraceBeginOnTrack(self, event, track)
        }
        fn TraceEndOnTrack(&self, track: TileManagerTraceTrack) {
            Runtime::<R>::TraceEndOnTrack(self, track)
        }
        fn TraceInstantOnTrack(
            &self,
            event: &'static str,
            track: TileManagerTraceTrack,
            state: impl FnOnce() -> Option<crate::adapter::property_types::TracedValueJSON>,
        ) {
            Runtime::<R>::TraceInstantOnTrack(self, event, track, state)
        }
        fn TestSharedImageInterface(
            &self,
        ) -> &dyn crate::adapter::resource_pool::PoolSharedImageInterface<Cpu<R>> {
            Runtime::<R>::TestSharedImageInterface(self)
        }
        fn TraceScopeWithArgs(
            &self,
            event: &'static str,
            arguments: impl FnOnce() -> Vec<TileManagerTraceArgument>,
        ) -> Box<dyn TileManagerTraceScope> {
            Runtime::<R>::TraceScopeWithArgs(self, event, arguments)
        }
        fn AllTilesDoneCommand(
            &self,
            owner: TileManagerId,
            lifetime: Weak<()>,
            start_time: TimeTicks,
        ) -> Box<dyn FnOnce(bool) + Send> {
            Runtime::<R>::AllTilesDoneCommand(self, owner, lifetime, start_time)
        }
        fn RasterQueryCapability(
            &self,
            id: RasterQueryQueueId,
        ) -> Arc<dyn crate::cc::tiles::tile_manager::TileManagerRasterQueryQueue>
        {
            Runtime::<R>::RasterQueryCapability(self, id)
        }
        fn RasterFlushCapability(
            &self,
            id: RasterBufferProviderId,
        ) -> Arc<dyn crate::cc::tiles::tile_manager::TileManagerRasterFlush> {
            Runtime::<R>::RasterFlushCapability(self, id)
        }
        fn FeatureEnabled(&self, feature: TileManagerFeature) -> bool {
            Runtime::<R>::FeatureEnabled(self, feature)
        }
        fn ReclaimDelayInSeconds(&self) -> i64 {
            Runtime::<R>::ReclaimDelayInSeconds(self)
        }
        fn Now(&self) -> TimeTicks {
            Runtime::<R>::Now(self)
        }
        fn NowWithClock(&self, clock: PoolClockId) -> TimeTicks {
            Runtime::<R>::NowWithClock(self, clock)
        }
        fn RasterBufferProvider(
            &mut self,
            id: RasterBufferProviderId,
        ) -> &mut dyn RasterBufferProvider<Cpu<R>> {
            Runtime::<R>::RasterBufferProvider(self, id)
        }
        fn AcquireRasterBufferForResource(
            &mut self,
            provider: RasterBufferProviderId,
            pool: ResourcePoolId,
            resource: &InUsePoolResource<Cpu<R>>,
            resource_content_id: u64,
            previous_content_id: u64,
        ) -> Option<<Cpu<R> as TileManagerTypes>::RasterBuffer> {
            Runtime::<R>::AcquireRasterBufferForResource(
                self,
                provider,
                pool,
                resource,
                resource_content_id,
                previous_content_id,
            )
        }
        fn TileRasterSourceAndPool(
        &mut self,
        tile: TileId,
        source_tiling: crate::adapter::tile::PictureLayerTilingId,
        pool: ResourcePoolId,
        ) -> (&Tile<Cpu<R>>, &<Cpu<R> as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource, &mut ResourcePool<Cpu<R>>){
            Runtime::<R>::TileRasterSourceAndPool(self, tile, source_tiling, pool)
        }
        fn GetMSAASampleCountForRaster(
            &self,
            client: TileManagerClientId,
            display_list: &<<Cpu<R> as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as TileManagerRasterSource<Cpu<R>>>::DisplayItemList,
        ) -> i32 {
            Runtime::<R>::GetMSAASampleCountForRaster(self, client, display_list)
        }
        fn WorkerDiagnostics(
            &self,
        ) -> Arc<dyn crate::cc::tiles::tile_manager::TileManagerWorkerDiagnostics>
        {
            Runtime::<R>::WorkerDiagnostics(self)
        }
        fn CreatePlaybackImageProvider(
            &mut self,
            cache: Option<ImageDecodeCacheId>,
            target: &<Cpu<R> as TileManagerTypes>::TargetColorParams,
            settings: PlaybackImageProviderSettings<Cpu<R>>,
        ) -> <Cpu<R> as TileManagerTypes>::PlaybackImageProvider {
            Runtime::<R>::CreatePlaybackImageProvider(self, cache, target, settings)
        }
        fn CreatePaintWorkletImageProvider(
            &mut self,
            records: <Cpu<R> as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::PaintWorkletRecordMap,
        ) -> <Cpu<R> as TileManagerTypes>::PaintWorkletImageProvider {
            Runtime::<R>::CreatePaintWorkletImageProvider(self, records)
        }
        fn TileAndPool(
            &mut self,
            tile: TileId,
            pool: ResourcePoolId,
        ) -> (&mut Tile<Cpu<R>>, &mut ResourcePool<Cpu<R>>) {
            Runtime::<R>::TileAndPool(self, tile, pool)
        }
        fn NotifyTileStateChanged(
            &mut self,
            client: TileManagerClientId,
            tile: &Tile<Cpu<R>>,
            update_damage: bool,
            set_needs_redraw: bool,
        ) {
            Runtime::<R>::NotifyTileStateChanged(
                self,
                client,
                tile,
                update_damage,
                set_needs_redraw,
            )
        }
        fn NotifyTileStateChangedById(
            &mut self,
            client: TileManagerClientId,
            tile: TileId,
            update_damage: bool,
            set_needs_redraw: bool,
        ) {
            Runtime::<R>::NotifyTileStateChangedById(
                self,
                client,
                tile,
                update_damage,
                set_needs_redraw,
            )
        }
        fn NotifyReadyToActivate(&mut self, client: TileManagerClientId) {
            Runtime::<R>::NotifyReadyToActivate(self, client)
        }
        fn NotifyReadyToDraw(&mut self, client: TileManagerClientId) {
            Runtime::<R>::NotifyReadyToDraw(self, client)
        }
        fn NotifyAllTileTasksCompleted(&mut self, client: TileManagerClientId) {
            Runtime::<R>::NotifyAllTileTasksCompleted(self, client)
        }
        fn HasPendingTree(&mut self, client: TileManagerClientId) -> bool {
            Runtime::<R>::HasPendingTree(self, client)
        }
        fn SetNeedsRedraw(
            &mut self,
            client: TileManagerClientId,
            animation_only: bool,
            skip_if_inside_draw: bool,
        ) {
            Runtime::<R>::SetNeedsRedraw(self, client, animation_only, skip_if_inside_draw)
        }
        fn SetIsLikelyToRequireADraw(&mut self, client: TileManagerClientId, value: bool) {
            Runtime::<R>::SetIsLikelyToRequireADraw(self, client, value)
        }
        fn RequestImplSideInvalidationForCheckerImagedTiles(
            &mut self,
            client: TileManagerClientId,
        ) {
            Runtime::<R>::RequestImplSideInvalidationForCheckerImagedTiles(self, client)
        }
        fn ResourcePool(&self, id: ResourcePoolId) -> &ResourcePool<Cpu<R>> {
            Runtime::<R>::ResourcePool(self, id)
        }
        fn ResourcePoolMutable(&mut self, id: ResourcePoolId) -> &mut ResourcePool<Cpu<R>> {
            Runtime::<R>::ResourcePoolMutable(self, id)
        }
        fn TileByIdMutable(&mut self, id: TileId) -> &mut Tile<Cpu<R>> {
            Runtime::<R>::TileByIdMutable(self, id)
        }
        fn BuildRasterQueue(
            &mut self,
            client: TileManagerClientId,
            tree_priority: TreePriority,
            kind: Type,
        ) -> Box<dyn RasterTilePriorityQueue<Cpu<R>>> {
            Runtime::<R>::BuildRasterQueue(self, client, tree_priority, kind)
        }
        fn BuildEvictionQueue(
            &mut self,
            client: TileManagerClientId,
        ) -> Box<EvictionTilePriorityQueue<Cpu<R>>> {
            Runtime::<R>::BuildEvictionQueue(self, client)
        }
        fn TraceScope(&self, event: &'static str) -> Box<dyn TileManagerTraceScope> {
            Runtime::<R>::TraceScope(self, event)
        }
        fn TraceInstant(
            &self,
            event: &'static str,
            state: impl FnOnce() -> Option<crate::adapter::property_types::TracedValueJSON>,
        ) {
            Runtime::<R>::TraceInstant(self, event, state)
        }
        fn HistogramDuration(
            &self,
            name: &'static str,
            value: TimeDelta,
            min: TimeDelta,
            max: TimeDelta,
            buckets: usize,
        ) {
            Runtime::<R>::HistogramDuration(self, name, value, min, max, buckets)
        }
    }

    impl<R: CpuSkiaBackend> TileManagerImages<Cpu<R>> for Runtime<'_, R> {
        fn ImageController(&mut self, id: ImageControllerId) -> &mut dyn ImageController<Cpu<R>> {
            Runtime::<R>::ImageController(self, id)
        }
        fn DecodedImageTracker(
            &mut self,
            id: DecodedImageTrackerId,
        ) -> &mut dyn DecodedImageTracker<Cpu<R>> {
            Runtime::<R>::DecodedImageTracker(self, id)
        }
        fn CheckerImageTracker(
            &mut self,
            id: CheckerImageTrackerId,
        ) -> &mut dyn CheckerImageTracker<Cpu<R>> {
            Runtime::<R>::CheckerImageTracker(self, id)
        }
    }
}

/// Source: cc/trees/layer_tree_host_impl.cc:877-903,1618-1647,4534-4565.
/// Only native fields are split here; TileManager retains every scheduling,
/// completion, priority and resource-lifetime decision in its source body.
pub(crate) fn WithCpuTileManagerContext<R: crate::runtime::types::CpuSkiaBackend, Output>(
    host: &mut LayerTreeHostImpl<crate::runtime::types::CpuCompositorTypes<R>>,
    context: &mut HostImplContext<'_, crate::runtime::types::CpuCompositorTypes<R>>,
    body: impl FnOnce(
        &mut crate::cc::tiles::tile_manager::TileManager<
            crate::runtime::types::CpuCompositorTypes<R>,
        >,
        &mut source_tile_manager_runtime::Runtime<'_, R>,
    ) -> Output,
) -> Output {
    let external = context.services.TileExecution();
    let output = {
        let mut execution = external
            .lock()
            .expect("independent native tile execution capabilities");
        let mut runtime = source_tile_manager_runtime::Runtime {
            owner: host.owner_id_,
            active: &mut host.active_tree_,
            pending: &mut host.pending_tree_,
            recycle: &mut host.recycle_tree_,
            settings: &host.settings_,
            debug: &host.debug_state_,
            caps: &host.raster_caps_,
            global: &host.global_tile_state_,
            high_res: &host.requires_high_res_to_draw_,
            viewport_priority: &host.viewport_rect_for_tile_priority_,
            external_viewport: &host.external_viewport_,
            sink: &host.layer_tree_frame_sink_,
            frame_tracker: &host.current_begin_frame_tracker_,
            input: host.input_delegate_.as_ref().map(|input| input.View()),
            delegate: host.delegate_,
            pool: &mut host.resource_pool_,
            provider: &mut host.resource_provider_,
            layer_context: &mut host.layer_context_,
            monitors: &host.latency_info_swap_promise_monitor_,
            events: &mut host.events_metrics_manager_,
            pending_fully_painted: &host.pending_tree_fully_painted_,
            likely_draw: &mut host.is_likely_to_require_a_draw_,
            image_cache: &mut host.image_decode_cache_holder_,
            compositor_visibility: &mut host.compositor_context_visibility_,
            worker_visibility: &mut host.worker_context_visibility_,
            image_frame: source_tile_manager_runtime::ImageFrameFields {
                settings: &host.settings_,
                animation: &host.image_animation_controller_,
            },
            execution: &mut execution,
            raster_provider: &mut host.raster_buffer_provider_,
            raster_queries: &mut host.pending_raster_queries_,
            services: context.services,
        };
        body(&mut host.tile_manager_, &mut runtime)
    };
    // Even a directly executing injected worker starts only after all native
    // manager/tree/pool borrows above ended. Worker results are typed commands.
    let graph = external
        .lock()
        .expect("independent native tile execution capabilities")
        .task_graph
        .clone();
    graph.Kick();
    output
}

/// Source: cc/trees/layer_tree_host_impl.cc:4351-4382. Safe raw-pointer
/// mapping: all identities name the actual source-owned pool/provider/cache.
/// The task runner clones only its single allocation's native lifetime handle.
pub(crate) fn ConfigureCpuTileManagerResources<R: crate::runtime::types::CpuSkiaBackend>(
    host: &mut LayerTreeHostImpl<crate::runtime::types::CpuCompositorTypes<R>>,
    context: &mut HostImplContext<'_, crate::runtime::types::CpuCompositorTypes<R>>,
) {
    use crate::adapter::tile_manager::{
        ImageDecodeCacheId, RasterBufferProviderId, RasterQueryQueueId,
    };
    use crate::cc::{
        raster::task_graph_runner::TaskGraphRunner, tiles::tile_manager::RasterCompletion,
    };
    type Cpu<R> = crate::runtime::types::CpuCompositorTypes<R>;
    let pool = host.resource_pool_.as_deref().map(|pool| pool.identity());
    let cache = host
        .image_decode_cache_holder_
        .as_ref()
        .map(|holder| holder.image_decode_cache().ImageDecodeCacheIdentity());
    let provider = host
        .raster_buffer_provider_
        .as_deref()
        .map(|provider| RasterBufferProviderId(provider as *const _ as usize as u64));
    let queries = host
        .pending_raster_queries_
        .as_deref()
        .map(|queries| RasterQueryQueueId(queries as *const _ as usize as u64));
    let use_gpu = host.raster_caps_.use_gpu_rasterization;
    let runner: std::sync::Arc<dyn TaskGraphRunner<RasterCompletion<Cpu<R>>>> =
        if host.is_synchronous_single_threaded_ {
            host.single_thread_synchronous_task_graph_runner_
                .as_deref()
                .expect("source synchronous runner")
                .clone()
        } else {
            let external = context.services.TileExecution();
            let execution = external.lock().expect("independent native tile execution");
            assert_eq!(
                host.task_graph_runner_.0,
                std::sync::Arc::as_ptr(&execution.task_graph) as usize as u64,
                "same injected source TaskGraphRunner"
            );
            execution.task_graph.clone()
        };
    WithCpuTileManagerContext(host, context, |manager, runtime| {
        manager.SetResources(pool, cache, runner, provider, use_gpu, queries, runtime)
    });
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    /// Source: cc/input/scrollbar_controller.cc:151-156. Queued weak callback
    /// resolution borrows the same InputHandler/controller; no owner is captured.
    pub(crate) fn RunAutoscrollTask(
        &mut self,
        owner: HostImplWeak,
        task: crate::adapter::scrollbar::AutoscrollTask,
        context: &mut HostImplContext<'_, B>,
    ) {
        if !owner.IsValid() {
            return;
        }
        assert_eq!(owner.owner, self.owner_id_);
        assert!(owner
            .alive
            .ptr_eq(&std::sync::Arc::downgrade(&self.weak_factory_)));
        self.WithInputHandler(context, |input, host| {
            if let Some(input) = input {
                input.RunAutoscrollTask(task, host);
            }
        });
    }
    /// Source: cc/trees/image_animation_controller.cc:612-616. Borrow only
    /// the controller and the independent fields used by its synchronous client.
    pub(crate) fn RunImageAnimationTask(
        &mut self,
        task: crate::adapter::image_animation::ImageAnimationTask,
        context: &mut HostImplContext<'_, B>,
    ) {
        self.WithImageAnimationController(context, |controller, client| {
            if let Some(controller) = controller {
                controller.RunDelayedTask(&task, client);
            }
        });
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:5979-6012,6090-6101,6127-6131,6140-6153.
// Rust call-scoped delegate: this never leaves the current animation tick. Each
// operation uses the unique resident Host and its original canonical method.
// In particular Finish may synchronously enter InputHandler and create/abort an
// animation, so neither a ScrollTree borrow nor an animation map lock survives
// a target callback.
struct HostImplAnimationTarget<'a, 'c, B: HostImplTypes> {
    host: &'a mut LayerTreeHostImpl<B>,
    context: HostImplContext<'c, B>,
}
impl<B: HostImplTypes> crate::adapter::animation_host::AnimationHostQueries
    for HostImplAnimationTarget<'_, '_, B>
{
    fn AnimationHostOwner(&self) -> LayerTreeHostId {
        self.host.owner_id_
    }
    fn IsElementInPropertyTrees(&self, id: ElementId, list: ElementListType) -> bool {
        self.host.IsElementInPropertyTrees(id, list)
    }
    fn CurrentScrollOffset(&self, id: ElementId, list: ElementListType) -> PointF {
        let tree = if list == ElementListType::ACTIVE {
            self.host.active_tree_.as_deref()
        } else {
            self.host.pending_tree_.as_deref()
        }
        .expect("animation queries an existing selected tree");
        let (scroll, _, state) = tree.property_trees().scroll_tree_read_context();
        scroll.current_scroll_offset(id, state)
    }
    fn WithScrollTree(
        &self,
        list: ElementListType,
        callback: &mut dyn FnMut(&crate::cc::trees::property_tree::ScrollTree),
    ) {
        let tree = if list == ElementListType::ACTIVE {
            self.host.active_tree_.as_deref()
        } else {
            self.host.pending_tree_.as_deref()
        }
        .expect("animation queries an existing selected tree");
        callback(tree.property_trees().scroll_tree());
    }
}
impl<B: HostImplTypes> crate::adapter::animation_host::AnimationMutationTarget
    for HostImplAnimationTarget<'_, '_, B>
{
    fn SetMutatorsNeedCommit(&mut self) {
        self.host.SetMutatorsNeedCommit();
    }
    fn SetMutatorsNeedRebuildPropertyTrees(&mut self) {
        self.host.SetMutatorsNeedRebuildPropertyTrees();
    }
    fn SetElementScrollOffsetMutated(
        &mut self,
        id: ElementId,
        list: ElementListType,
        value: &PointF,
    ) {
        self.host
            .SetElementScrollOffsetMutated(id, list, value, &mut self.context);
    }
    fn ScrollOffsetAnimationFinished(
        &mut self,
        id: ElementId,
        selected: Option<&mut dyn crate::adapter::animation_host::ScrollAnimationReentry>,
    ) {
        self.host
            .ScrollOffsetAnimationFinishedWithReentry(id, selected, &mut self.context);
    }
    fn NotifyAnimationWorkletStateChange(
        &mut self,
        state: AnimationWorkletMutationState,
        list: ElementListType,
    ) {
        self.host
            .NotifyAnimationWorkletStateChange(state, list, &mut self.context);
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/input/scrollbar_animation_controller.cc:129-136.
    // Safe weak callback routing. The token's same allocation/generation check
    // remains in the actual resident controller; no controller owner is copied.
    pub(crate) fn RunScrollbarAnimationTask(
        &mut self,
        delegate: HostImplDelegateId,
        task: crate::adapter::scrollbar_animation::ScrollbarAnimationTask,
        context: &mut HostImplContext<'_, B>,
    ) {
        assert_eq!(self.delegate_, delegate);
        if !task.IsAlive() {
            return;
        }
        let element = ElementId::new(task.owner_id);
        self.WithScrollbarAnimationController(element, context, |controller, client| {
            if let Some(controller) = controller {
                controller.RunDelayedAnimationTask(client, task);
            }
        });
    }
}

// Source: cc/trees/layer_tree_host_impl.cc:5557-5564;
// cc/tiles/prioritized_tile.cc:30-42. Raw pointer identities are resolved only
// against the current resident tree/Set/Tiling allocations for this one call.
pub(crate) fn TraceCpuPrioritizedTile<R: crate::runtime::types::CpuSkiaBackend>(
    tile: &crate::cc::tiles::prioritized_tile::PrioritizedTile,
    trees: [Option<&LayerTreeImpl<crate::runtime::types::CpuCompositorTypes<R>>>; 3],
    value: &mut <crate::runtime::types::CpuCompositorTypes<R> as HostImplTypes>::HostTraceValue,
    trace: &dyn HostImplHServices<crate::runtime::types::CpuCompositorTypes<R>>,
) {
    type Cpu<R> = crate::runtime::types::CpuCompositorTypes<R>;
    struct Trace<'a, B: HostImplTypes>(&'a dyn HostImplHServices<B>);
    impl<B: HostImplTypes> crate::adapter::tile::TileTraceContext for Trace<'_, B> {
        fn MakeDictIntoImplicitSnapshotWithCategory<
            V: crate::adapter::property_types::PropertyTrace,
        >(
            &self,
            category: &str,
            value: &mut V,
            name: &str,
            id: crate::adapter::tile::TileTraceId,
        ) {
            self.0.TraceTileSnapshot(category, value, name, id);
        }
        fn TraceTileDeleted(
            &self,
            category: &str,
            event: &str,
            id: crate::adapter::tile::TileTraceId,
            flow: &str,
        ) {
            self.0.TraceTileDeletion(category, event, id, flow)
        }
    }
    impl<B: HostImplTypes> crate::adapter::prioritized_tile::PrioritizedTileTraceContext
        for Trace<'_, B>
    {
        fn SetIDRef<V: crate::adapter::property_types::PropertyTrace>(
            &self,
            id: crate::adapter::prioritized_tile::RasterSourceTraceId,
            value: &mut V,
            name: &str,
        ) {
            self.0.TraceRasterSourceIdRef(id, value, name);
        }
    }
    let tiling_id = tile
        .source_tiling_id()
        .expect("source nonnull prioritized tiling");
    let native_id = tile.tile_id().expect("source nonnull prioritized tile");
    for tree in trees.into_iter().flatten() {
        for picture in tree.picture_layers() {
            if let Some(tiling) = picture
                .picture_layer_tiling_set()
                .TilingByIdIfPresent(tiling_id)
            {
                tile.AsValueIntoWithResident(
                    tiling.TileById(native_id),
                    tiling.raster_source(),
                    value,
                    &Trace::<Cpu<R>>(trace),
                );
                return;
            }
        }
    }
    panic!("prioritized tile's source tiling must remain resident during source trace");
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/layer_tree_host_impl.cc:5315-5325,5361-5387.
    // Rust borrow split only: lend the original H allocation and temporarily
    // move the same nullable event Box. Every delegate callback synchronously
    // borrows the actual Host. Restore event ownership before propagating panic.
    fn WithLiveAnimation<R>(
        &mut self,
        context: &mut HostImplContext<'_, B>,
        callback: impl FnOnce(
            &mut dyn HostImplHServices<B>,
            LayerTreeHostId,
            Option<&mut B::HostMutatorEvents>,
            &mut dyn crate::adapter::animation_host::AnimationMutationTarget,
        ) -> R,
    ) -> R {
        let mut events = self.mutator_events_.take();
        let owner = self.owner_id_;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut callback = Some(callback);
            let mut result = None;
            context
                .services
                .WithAnimationFacet(&mut |animation, remaining| {
                    let mut target = HostImplAnimationTarget {
                        host: self,
                        context: HostImplContext {
                            services: remaining,
                        },
                    };
                    result = Some(
                        callback.take().expect("source invokes the live facet once")(
                            animation,
                            owner,
                            events.as_deref_mut(),
                            &mut target,
                        ),
                    );
                });
            result.expect("source invokes the live facet once")
        }));
        let replaced = std::mem::replace(&mut self.mutator_events_, events);
        assert!(
            replaced.is_none(),
            "animation replaced the lent event owner"
        );
        match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
}

impl<B: HostImplTypes> LayerTreeHostImpl<B> {
    // Source: cc/trees/client_layer_tree_host_impl.cc:135-136;
    // cc/animation/scroll_offset_animations_impl.cc:331-352.
    // The source delegate performs live membership queries and may synchronously
    // finish an animation. Lend the same client facet, retaining the actual Host
    // and all remaining services as the lexical mutation target.
    pub(crate) fn HandleRemovedScrollAnimatingElements(
        &mut self,
        active: bool,
        context: &mut HostImplContext<'_, B>,
    ) {
        let owner = self.owner_id_;
        context.services.WithClientFacet(&mut |client, remaining| {
            let mut target = HostImplAnimationTarget {
                host: self,
                context: HostImplContext {
                    services: remaining,
                },
            };
            client.ClientHandleRemovedScrollAnimatingElements(owner, active, &mut target);
        });
    }
}

// Source: cc/input/input_handler.cc:101-107; cc/trees/layer_tree_host_impl.cc:4982-4986.
// The actual canonical InputHandler is constructed against the live field
// facade, then moved once into the native delegate slot after that borrow ends.
impl<B> LayerTreeHostImpl<B>
where
    B: HostImplTypes
        + crate::adapter::input_handler::InputHandlerTypes
        + crate::adapter::input_services::InputHostTypes<
            InputHandler = crate::cc::input::input_handler::InputHandler<B>,
        >,
{
    pub(crate) fn CreateInputHandler(
        &mut self,
        c: &mut HostImplContext<'_, B>,
    ) -> crate::adapter::input_handler::InputHandlerWeak {
        assert!(
            self.input_delegate_.is_none(),
            "one canonical input owner per Host"
        );
        let (input, weak) = self.WithInputRuntime(c, |host| {
            crate::cc::input::input_handler::InputHandler::<B>::CreateUnbound(host)
        });
        self.BindToInputHandler(input);
        weak
    }
}

// Source: cc/trees/layer_tree_impl.cc:182-187;
// cc/layers/picture_layer_impl.cc:105-109 and implicit member destructors.
// Rust borrow split for destruction only: detached unique layer owners remain
// on this stack while their actual sibling twins and resident manager are lent.
pub(crate) fn ShutdownCpuResidentTree<R: crate::runtime::types::CpuSkiaBackend>(
    host: &mut LayerTreeHostImpl<crate::runtime::types::CpuCompositorTypes<R>>,
    id: crate::adapter::layer_tree_impl::LayerTreeId,
    context: &mut HostImplContext<'_, crate::runtime::types::CpuCompositorTypes<R>>,
) {
    use crate::runtime::{layer::CompositorLayer, types::CpuCompositorTypes as Cpu};
    use crate::cc::layers::picture_layer_impl::{
        PictureLayerImpl, PictureLayerTwinAccess,
    };
    type Tree<R> = crate::cc::trees::layer_tree_impl::LayerTreeImpl<Cpu<R>>;
    struct Twins<'a, R: crate::runtime::types::CpuSkiaBackend> {
        trees: [Option<&'a mut Tree<R>>; 2],
    }
    impl<R: crate::runtime::types::CpuSkiaBackend> PictureLayerTwinAccess<Cpu<R>> for Twins<'_, R> {
        fn TwinMutable(
            &mut self,
            id: crate::cc::layers::picture_layer_impl::PictureLayerImplId,
        ) -> &mut PictureLayerImpl<Cpu<R>> {
            let tree = self
                .trees
                .iter_mut()
                .filter_map(Option::as_deref_mut)
                .find(|tree| tree.id == id.tree)
                .expect("native twin belongs to a remaining resident tree");
            let layer = tree
                .LayerByIdMutable(id.layer)
                .expect("native resident twin layer");
            let CompositorLayer::Picture(picture) = layer else {
                panic!("native twin is a PictureLayerImpl")
            };
            assert_eq!(picture.identity(), id);
            picture
        }
    }
    struct Tiles<'a, 'b, R: crate::runtime::types::CpuSkiaBackend> {
        manager: &'a mut crate::cc::tiles::tile_manager::TileManager<Cpu<R>>,
        parts: HostPictureReleaseParts<'b, Cpu<R>>,
        trace: &'a dyn crate::adapter::host_picture::HostPictureTileServices,
    }
    impl<R: crate::runtime::types::CpuSkiaBackend>
        crate::adapter::picture_layer_tiling::PictureLayerTileDestroyer<Cpu<R>>
        for Tiles<'_, '_, R>
    {
        fn RebindTileWithManager(
            &mut self,
            _old: crate::adapter::tile::TileId,
            _new: crate::adapter::tile::TileId,
        ) {
            unreachable!("tree shutdown never transfers a resident tile")
        }
        fn DestroyTileWithManager(
            &mut self,
            manager: crate::adapter::tile::TileManagerId,
            tile: &mut crate::cc::tiles::tile::Tile<Cpu<R>>,
            tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<
                Cpu<R>,
            >,
        ) {
            assert_eq!(self.manager.identity(), manager);
            self.manager.DestroyTileWithRelease(
                tile,
                &mut HostPictureTileRelease {
                    parts: &mut self.parts,
                    trace: self.trace,
                    tiling,
                    notification: None,
                },
            );
        }
    }
    let owner = host.owner_id_;
    let active_id = host.active_tree_.as_ref().map(|tree| tree.id);
    let (tree, mut twins) = if active_id == Some(id) {
        (
            host.active_tree_.as_deref_mut().unwrap(),
            Twins {
                trees: [
                    host.pending_tree_.as_deref_mut(),
                    host.recycle_tree_.as_deref_mut(),
                ],
            },
        )
    } else if host.pending_tree_.as_ref().map(|tree| tree.id) == Some(id) {
        (
            host.pending_tree_.as_deref_mut().unwrap(),
            Twins {
                trees: [
                    host.active_tree_.as_deref_mut(),
                    host.recycle_tree_.as_deref_mut(),
                ],
            },
        )
    } else {
        let tree = host
            .recycle_tree_
            .as_deref_mut()
            .expect("source resident recycle tree");
        assert_eq!(tree.id, id);
        (
            tree,
            Twins {
                trees: [
                    host.active_tree_.as_deref_mut(),
                    host.pending_tree_.as_deref_mut(),
                ],
            },
        )
    };
    let mut layers = tree.DetachLayers();
    let mut latency_promises = Vec::new();
    context
        .services
        .WithPictureExternalParts(&mut |_, _, trace, _, _, release| {
            let mut tiles = Tiles {
                manager: &mut host.tile_manager_,
                parts: HostPictureReleaseParts {
                    settings: &host.settings_,
                    delegate: host.delegate_,
                    monitors: &host.latency_info_swap_promise_monitor_,
                    events: &mut host.events_metrics_manager_,
                    pool: &mut host.resource_pool_,
                    provider: &mut host.resource_provider_,
                    layer_context: &mut host.layer_context_,
                    sink: &host.layer_tree_frame_sink_,
                    latency_promises: &mut latency_promises,
                    services: release,
                },
                trace,
            };
            for layer in layers.iter_mut() {
                if let CompositorLayer::Picture(picture) = layer {
                    picture.Destroy(&mut twins, &mut tiles);
                }
            }
        });
    // The source destructor releases every most-derived owner, including its
    // ordinary base destructor, before breaking this tree's swap promises.
    drop(layers);
    assert!(
        latency_promises.is_empty(),
        "resource teardown cannot schedule a new frame"
    );
    tree.FinishShutdownWithHost(
        owner,
        |tree| Some(tree) == active_id,
        || context.services.MonotonicTimeNow(),
    );
}
