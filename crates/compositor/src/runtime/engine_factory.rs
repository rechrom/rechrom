//! The unique private production construction site. Only independent executor,
//! paint value, diagnostics and capabilities resource capabilities enter this file;
//! every resident compositor owner remains internal to the Engine assembly.
use super::{
    activation::ActivationServices,
    animation_host::AnimationHostOwner,
    animation_services::{AnimationServices, ScrollbarRegistrationServices},
    begin_frame::BeginFrameServices,
    client::{ClientServices, ClientWorkletServices},
    client_resources::RasterClientResources,
    core_services::{CoreConfig, CoreServices, CpuNoGpuCapabilities, RenderingStats},
    engine_core::EngineCore,
    engine_queries::{CallScopedSchedulerQueries, FactoryPictureState},
    execution::{EngineTask, EngineTaskQueue},
    frame_build::FrameBuildServices,
    host_action::HostActionServices,
    host_core::{HostCoreOwners, ImageAnimationDispatch, OsSurfaceTokenSource},
    host_frame_sink::{HostFrameSinkCallbacks, HostFrameSinkConfig, HostFrameSinkServices},
    host_lifecycle::{HostLifecycle, HostLifecycleEffects},
    host_metadata::{HostMetadataConfig, HostMetadataServices},
    host_picture_services::{
        CpuHostPictureMutationServices, CpuHostPictureTileServices, CpuPictureLayerServices,
        HostPictureEnvironment, HostPictureFeatures, HostPictureState,
    },
    host_services::{CpuHostServices, HostInit},
    images::{CheckerImageClient, CpuImageServices, ImageDecodeCacheRegistry},
    latency_info_swap_promise_monitor::LatencyMonitorRegistry,
    layer_tree_frame_sink_endpoint::FrameSinkEndpoint,
    main_endpoint::EngineMainEndpoint,
    main_host::CpuMainLayerHost,
    metrics::{MetricsConfig, MetricsServices},
    paint_services::CpuPaintServices,
    render::{RenderConfig, RenderServices},
    resource_events::ResourceEffects,
    resource_services::{PictureReleaseServices, ResourceServices},
    scheduler_services::{EngineSchedulerHost, SchedulerConfig, SchedulerEffects, TimingHistory},
    scrollbar::ScrollbarLayerServices,
    services::{
        BrowserControlsServices, DiagnosticConfig, DiagnosticServices, EventSink, InputConfig,
        InputEnvironmentServices, TaskRunnerServices,
    },
    skia::SkRegion,
    tile_execution::TileExecution,
    tree_environment::{TreeEnvironmentConfig, TreeEnvironmentServices},
    types::{LocalSurfaceId, MainMutatorHostIdentity},
    ui::{UiRasterServices, UiResourceServices},
    worklet::WorkletServices,
};
use crate::{
    adapter::{
        layer_tree_host_impl::*,
        layer_tree_impl::{Color4f, LayerTreeHostId},
        main_layer::MainLayerHostId,
        property_tree::PropertyTreesOwnerId,
    },
    cc::{
        metrics::compositor_timing_history::UMACategory,
        trees::layer_tree_settings::LayerTreeSettings,
    },
    engine::{CompositorConfig, CompositorError, CompositorExecutors},
    CompositorTypes,
};
use std::sync::{Arc, Mutex};

/// Explicit capabilities feature/configuration values. These are ownerless process
/// settings, not synthesized Layer/Host state or a Page callback registry.
pub(crate) struct FactoryOptions {
    pub settings: LayerTreeSettings,
    pub core: CoreConfig,
    pub picture: Arc<HostPictureFeatures>,
    pub frame_sink: HostFrameSinkConfig,
    pub metadata: HostMetadataConfig,
    pub render: RenderConfig,
    pub scheduler: SchedulerConfig,
    pub metrics: MetricsConfig,
    pub diagnostics: DiagnosticConfig,
    pub input: InputConfig,
    pub events: Arc<dyn EventSink>,
    pub frame_trace_categories: std::collections::BTreeSet<String>,
    pub tree_environment: TreeEnvironmentConfig,
}

/// All shared objects here are independent external capabilities. In
/// particular this allocation cannot contain a PAC/MainHost/Host/Tree/Tile or a
/// callable owner endpoint. Thin capabilities facades clone the same capability Arc;
/// none creates a second resource table, image cache or animation state.
pub(crate) struct FactoryCapabilities {
    pub tree_environment: TreeEnvironmentConfig,
    pub rendering_stats: Arc<RenderingStats>,
    pub ui_raster: Arc<dyn UiRasterServices>,
    pub diagnostics: DiagnosticConfig,
    pub frame_trace_categories: std::collections::BTreeSet<String>,
    pub events: Arc<dyn EventSink>,
    pub lifecycle: Arc<HostLifecycle>,
    pub resources: RasterClientResources,
    pub frame_sink: Arc<Mutex<FrameSinkEndpoint>>,
    pub tile_execution: Arc<Mutex<TileExecution<SkRegion>>>,
    pub main_animations: AnimationHostOwner,
    pub animations: AnimationHostOwner,
    pub latency_monitors: LatencyMonitorRegistry,
    pub runners: Arc<TaskRunnerServices>,
    pub commands: EngineTaskQueue,
    pub render: RenderServices,
    pub picture: Arc<HostPictureFeatures>,
    pub timing: Arc<TimingHistory>,
    pub resource_effects: ResourceEffects,
    pub scheduler_queries: Arc<CallScopedSchedulerQueries>,
}
struct ImageAnimationDelivery(EngineTaskQueue);
impl ImageAnimationDispatch for ImageAnimationDelivery {
    fn Dispatch(&self, task: crate::adapter::image_animation::ImageAnimationTask) {
        self.0.Push(EngineTask::ImageAnimation(task));
    }
}
struct CheckerDelivery {
    commands: EngineTaskQueue,
    // Only source routing identity. No manager/host can be recovered here.
    owner: HostImplDelegateId,
}
impl CheckerImageClient for CheckerDelivery {
    fn NeedsInvalidationForCheckerImagedTiles(&self) {
        self.commands.Push(EngineTask::CheckerNeedsInvalidation {
            delegate: self.owner,
        });
    }
}

/// The embedding's capabilities implementation supplies Paint's pure value/recording
/// primitives inside this crate; these are never arguments to public Commit.
pub(crate) fn Create(
    config: CompositorConfig,
    executors: CompositorExecutors,
    options: FactoryOptions,
    paint: Arc<CpuPaintServices<SkRegion>>,
) -> Result<super::engine_assembly::EngineAssembly, CompositorError> {
    if config.thread {
        return Err(CompositorError::UnsupportedExecutionConfiguration {
            reason: "separate main/impl Proxy transport is not yet constructed",
        });
    }
    // The factory owns identity generation; Page never passes internal IDs.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let owner = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    assert!(
        owner <= i32::MAX as u64,
        "capabilities engine identity exhausted"
    );
    let commands = EngineTaskQueue::default();
    let lifecycle_effects = HostLifecycleEffects::default();
    let lifecycle = Arc::new(HostLifecycle::new(lifecycle_effects.clone()));
    let queries = Arc::new(CallScopedSchedulerQueries::default());
    let resources = RasterClientResources::new();
    let sink_id = HostImplFrameSinkId(owner);
    let sink = Arc::new(Mutex::new(FrameSinkEndpoint::new(
        sink_id,
        resources.clone(),
    )));
    let runners = Arc::new(TaskRunnerServices::new_threaded_on_single_executor(
        executors.compositor.clone(),
    ));
    let main_animations = AnimationHostOwner::default();
    let impl_animations = AnimationHostOwner::default();
    let latency_monitors = LatencyMonitorRegistry::default();
    let scheduler_effects = SchedulerEffects::default();
    let scheduler_host = Arc::new(EngineSchedulerHost::new(
        owner,
        executors.compositor.clone(),
        commands.clone(),
        scheduler_effects.clone(),
        options.events.clone(),
        options.scheduler,
    ));
    let timing = Arc::new(TimingHistory::new(
        UMACategory::RENDERER_UMA,
        executors.compositor.clone(),
        options.events.clone(),
    ));
    let checker = Arc::new(CheckerDelivery {
        commands: commands.clone(),
        owner: HostImplDelegateId(owner),
    });
    let caches = ImageDecodeCacheRegistry::default();
    let image_decode = super::host_owners::ImageDecodeFactory(Box::new(
        super::software_image_decode_cache::CpuSoftwareImageDecodeFactory::new(
            caches.clone(),
            Arc::new(super::raster_resources::EngineRasterFailure(
                commands.clone(),
            )),
            paint.PaintValues(),
        ),
    ));
    let images = CpuImageServices::new(caches, checker);
    let tile_execution = Arc::new(Mutex::new(TileExecution::new(
        images,
        executors.raster_provider.clone(),
        executors.compositor.clone(),
        executors.raster,
        commands.clone(),
        options.events.clone(),
        config.raster_worker_limit,
    )));
    let stats = Arc::new(RenderingStats::new(HostImplRenderingStatsId(owner)));
    let capabilities = Arc::new(FactoryCapabilities {
        main_animations: main_animations.clone(),
        tree_environment: options.tree_environment,
        rendering_stats: stats.clone(),
        ui_raster: executors.ui_raster,
        diagnostics: options.diagnostics.clone(),
        frame_trace_categories: options.frame_trace_categories.clone(),
        events: options.events.clone(),
        lifecycle: lifecycle.clone(),
        resources: resources.clone(),
        frame_sink: sink.clone(),
        tile_execution: tile_execution.clone(),
        animations: impl_animations.clone(),
        latency_monitors,
        runners: runners.clone(),
        commands: commands.clone(),
        render: RenderServices::new(options.render, options.events.clone()),
        picture: options.picture.clone(),
        timing: timing.clone(),
        resource_effects: ResourceEffects::default(),
        scheduler_queries: queries.clone(),
    });

    let core_owners = HostCoreOwners::new(
        MetricsServices::new(options.metrics, options.events.clone()),
        impl_animations.clone(),
        TaskRunnerServices::new_threaded_on_single_executor(executors.compositor.clone()),
        commands.clone(),
        Arc::new(ImageAnimationDelivery(commands.clone())),
        Arc::new(OsSurfaceTokenSource),
    );
    let core = CoreServices::new(
        core_owners,
        lifecycle.clone(),
        resources.clone(),
        options.picture.clone(),
        options.core,
        executors.compositor.clone(),
        options.events.clone(),
        stats,
        Box::new(CpuNoGpuCapabilities { sink: sink_id }),
        HostImplTaskRunnerProviderId(owner),
    );
    let tile_execution_for_g = tile_execution.clone();
    let g = || {
        HostFrameSinkServices::new(
            sink.clone(),
            lifecycle.clone(),
            options.events.clone(),
            resources.clone(),
            tile_execution_for_g.clone(),
            runners.clone(),
            commands.clone(),
            options.frame_sink,
            timing.clone(),
            impl_animations.clone(),
        )
    };
    let services = CpuHostServices::<CompositorTypes> {
        tile_execution,
        client: Some(Box::new(ClientServices::new(capabilities.clone()))),
        client_worklet: Box::new(ClientWorkletServices::new(capabilities.clone())),
        picture_environment: Box::new(HostPictureEnvironment::new(
            options.picture.clone(),
            Arc::new(FactoryPictureState::new(
                sink.clone(),
                queries.clone(),
                HostImplDelegateId(owner),
            )),
        )),
        picture_services: Box::new(CpuPictureLayerServices::new(
            options.picture.clone(),
            options.events.clone(),
            resources.clone(),
        )),
        picture_tile_services: Box::new(CpuHostPictureTileServices::new(options.events.clone())),
        picture_mutation_services: Box::new(CpuHostPictureMutationServices::new(
            lifecycle.clone(),
            capabilities.tile_execution.clone(),
        )),
        picture_registration_services: Box::new(ScrollbarRegistrationServices {
            animations: impl_animations.clone(),
            executor: executors.compositor.clone(),
            commands: commands.clone(),
            events: options.events.clone(),
        }),
        picture_release_services: Box::new(PictureReleaseServices::new(capabilities.clone())),
        core: Box::new(core),
        debug: Box::new(DiagnosticServices::new(
            options.diagnostics,
            options.events.clone(),
        )),
        ui: Box::new(UiResourceServices::new(capabilities.clone())),
        animation: Some(Box::new(AnimationServices::new(
            impl_animations.clone(),
            capabilities.resource_effects.clone(),
            options.frame_trace_categories.clone(),
            options.events.clone(),
        ))),
        scrollbar_layer: Box::new(ScrollbarLayerServices::new(
            capabilities.clone(),
            options.settings.clone(),
        )),
        worklet: Box::new(WorkletServices::new(capabilities.clone())),
        begin_frame: Box::new(BeginFrameServices::new(capabilities.clone())),
        activation: Box::new(ActivationServices::new(capabilities.clone(), image_decode)),
        lifecycle: Box::new(HostActionServices::new(capabilities.clone())),
        input_environment: Box::new(InputEnvironmentServices::new(
            options.input,
            options.events.clone(),
        )),
        scrollbar_registration: Box::new(ScrollbarRegistrationServices {
            animations: impl_animations.clone(),
            executor: executors.compositor.clone(),
            commands: commands.clone(),
            events: options.events.clone(),
        }),
        browser_controls: Box::new(BrowserControlsServices::new(options.events.clone())),
        frame_build: Box::new(FrameBuildServices::new(capabilities.clone())),
        resources: Box::new(ResourceServices::new(capabilities.clone())),
        frame_sink: Box::new(g()),
        tree_environment: Box::new(TreeEnvironmentServices::new(
            capabilities.clone(),
            LayerTreeHostId(owner),
            HostImplTaskRunnerProviderId(owner),
        )),
        metadata: Box::new(HostMetadataServices::new(
            sink.clone(),
            lifecycle.clone(),
            options.events.clone(),
            options.metadata,
        )),
        task_runners: Box::new(TaskRunnerServices::new_threaded_on_single_executor(
            executors.compositor.clone(),
        )),
        resource_returns: Box::new(g()),
        frame_sink_callbacks: Box::new(g()),
        draw: Box::new(capabilities.render.clone()),
        surface: Box::new(capabilities.render.clone()),
    };
    let endpoint = Arc::new(EngineMainEndpoint::new(
        executors.compositor.clone(),
        commands.clone(),
        main_animations.clone(),
        options.events,
    ));
    let settings = options.settings;
    let scheduler_settings = settings.ToSchedulerSettings(options.picture.as_ref());
    let mut main = CpuMainLayerHost::new(
        MainLayerHostId(owner),
        PropertyTreesOwnerId::FromOwnerValue(owner),
        main_animations.identity(),
        settings.clone(),
        config.thread,
        endpoint.clone(),
    );
    if config.transparent_background {
        // Chromium's WebLayerTreeView forwards the embedder background color
        // to LayerTreeHost. An alpha-zero root keeps popup Pages compositable
        // over the document instead of producing an opaque white fill quad.
        main.pending_commit_state_mut().background_color = Color4f {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        };
    }
    let image_worker_task_runner: <CompositorTypes as HostImplTypes>::ImageWorkerTaskRunner =
        Arc::new(super::execution::EngineSequencedTaskRunner::new(
            executors.compositor.clone(),
            commands.clone(),
        ));
    let init = HostInit {
        owner: LayerTreeHostId(owner),
        settings,
        delegate: HostImplDelegateId(owner),
        task_runner_provider: HostImplTaskRunnerProviderId(owner),
        rendering_stats: HostImplRenderingStatsId(owner),
        task_graph_runner: HostImplTaskGraphRunnerId(Arc::as_ptr(
            &capabilities
                .tile_execution
                .lock()
                .expect("tile execution")
                .task_graph,
        ) as usize as u64),
        mutator_host: Box::new(impl_animations.CreateMutatorHost()),
        dark_mode_filter: None,
        id: owner as i32,
        image_worker_task_runner,
        scheduling_delegate: HostImplSchedulingDelegateId(owner),
    };
    let local_surface_id = LocalSurfaceId {
        parent_sequence_number: 1,
        child_sequence_number: 1,
        embed_token: super::host_core::SurfaceTokenSource::NextEmbedToken(&OsSurfaceTokenSource),
    };
    let input_client = Arc::new(super::input_client::QueuedInputHandlerClient::default());
    let core = EngineCore::new(
        services,
        init,
        main,
        paint,
        endpoint,
        lifecycle,
        queries,
        sink,
        local_surface_id,
        owner + 1,
        input_client.clone(),
    )?;
    super::engine_assembly::EngineAssembly::new(
        core,
        scheduler_settings,
        scheduler_host,
        timing,
        commands,
        lifecycle_effects,
        scheduler_effects,
        capabilities.render.clone(),
        capabilities.resource_effects.clone(),
        capabilities.latency_monitors.clone(),
        input_client,
    )
}

/// Default capability construction is admitted only after the capabilities Paint
/// value/tracking/recording constructors and every Host facet are closed.

impl FactoryOptions {
    /// Explicit desktop CPU embedding configuration. Optional GPU/Viz/trace
    /// capabilities are disabled here; canonical settings retain their defaults.
    pub(crate) fn cpu_default(config: &CompositorConfig) -> Self {
        let picture = Arc::new(HostPictureFeatures {
            use_layer_lists_by_default: false,
            trees_in_viz: false,
            tree_animations_in_viz: false,
            defer_impl_invalidation: true,
            defer_impl_invalidation_frames: 4,
            scroll_event_dispatch_mode: "DispatchScrollEventsUntilDeadline".into(),
            // QueuedInputHandlerClient is the embedding's CTEQ boundary. It
            // reports queued precision-wheel updates and InputHandler drains
            // them from WillBeginImplFrame before the deadline decision.
            wait_for_late_scroll_events: true,
            wait_for_late_scroll_events_deadline_ratio: 0.333,
            renderer_image_decode_working_set_bytes: 128 * 1024 * 1024,
            browser_image_decode_working_set_bytes: 128 * 1024 * 1024,
            low_end_device: false,
            cc_slimming: true,
            compute_raster_translate_for_external_scale: true,
            raw_draw: false,
            raw_draw_tile_size_factor: 1.,
        });
        let mut settings = LayerTreeSettings::new(picture.as_ref());
        // Source: ui/base/ui_base_switches_util.cc and Blink's
        // layer_tree_settings.cc. Apple platforms enable root elastic
        // overscroll; the compositor keeps the live helper/controller state.
        settings.enable_elastic_overscroll_on_root = cfg!(target_vendor = "apple");
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc:245-274. macOS uses 512 px tiles on
        // high-DPI displays. This is selected from the initial DSF, independent
        // of whether the tile buffer is GPU- or CPU-backed.
        #[cfg(target_vendor = "apple")]
        if config.initial_device_scale_factor >= 2.0 {
            settings.default_tile_size = crate::adapter::geometry::Size::new(512, 512);
        }
        // Chromium's default 3000/2000 px interest-area/skewport pair is
        // tuned for its production GPU raster latency. Keep the identical
        // priority-rect algorithm for this CPU embedding, but give it the
        // larger source-supported runway so required tiles can finish before
        // a high-velocity precision-wheel scroll reaches them. The 3:2 ratio
        // is the invariant used by Blink's GetTilingInterestAreaSizes().
        settings.tiling_interest_area_padding = 18000;
        settings.skewport_extrapolation_limit_in_screen_pixels = 12000;
        // PictureLayerTiling otherwise turns content farther than 1000
        // screen pixels into image-decode-only work. At 2x scale that is only
        // 500 CSS px, shorter than one fast precision-wheel frame. Let the CPU
        // backend raster content across the same 12000 px skewport runway.
        settings.max_preraster_distance_in_screen_pixels = 12000;
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc:135-204,559-561. Desktop renderers
        // admit nice-to-have prepaint tiles and retain at least 512 MiB. The
        // old generic 64 MiB default repeatedly evicted the skewport during
        // fast back-and-forth scrolling.
        let width = config.initial_viewport_size.width().max(0) as f64;
        let height = config.initial_viewport_size.height().max(0) as f64;
        let scale = f64::from(config.initial_device_scale_factor.max(1.0));
        let display_pixels = width * scale * height * scale;
        let large_resolution_pixels = 2056.0 * 1329.0 * 2.0 * 2.0;
        let memory_mebibytes = (1152.0 * display_pixels / large_resolution_pixels)
            .round()
            .max(512.0) as usize;
        settings.memory_policy =
            crate::cc::trees::managed_memory_policy::ManagedMemoryPolicy::WithLimits(
                memory_mebibytes.saturating_mul(1024 * 1024),
                crate::adapter::resource_types::PriorityCutoff::CUTOFF_ALLOW_NICE_TO_HAVE,
                crate::cc::trees::managed_memory_policy::ManagedMemoryPolicy::kDefaultNumResourcesLimit,
            );
        // Source: third_party/blink/renderer/platform/widget/compositing/layer_tree_settings.cc:364.
        settings.use_layer_lists = true;
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc. Renderer widgets enable hit-test
        // opaqueness together with recorded-bounds tiling. RecordingSource
        // then intersects a sparse layer with its display-list bounds instead
        // of allocating tiles across the layer's full layout bounds.
        settings.enable_hit_test_opaqueness = true;
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc:374-379. Production renderer widgets
        // use ProxyImpl scheduling, including the late-scroll deadline mode.
        settings.single_thread_proxy_scheduler = false;
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc. Renderer widgets enable partial
        // raster whenever the selected platform raster backend supports it.
        // The injected host-memory backend supports preserved-pixel replay;
        // leaving the generic cc default disabled changes rounded clip pixels
        // after asynchronous image invalidations and defeats tile reuse.
        settings.use_partial_raster = true;
        // Chromium enables fractional commit deltas only when the experimental
        // FractionalScrollOffsets runtime feature is enabled. That feature is
        // disabled by default; keep cc's source default so Blink receives
        // integer committed offsets and the residual remains compositor-owned.
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc:364-370.
        // Source: content/public/common/content_features.cc:521.
        settings.commit_fractional_scroll_deltas = false;
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc:472-499 and
        //         ui/theme/overlay_scrollbar_constants.cc:12-20.
        // Aura's thinning controller is selected only by Chromium's desktop
        // overlay-scrollbar branch. macOS uses Blink-painted scrollbars and
        // must not apply Aura's idle_thickness_scale to their native bitmap.
        #[cfg(target_vendor = "apple")]
        {
            settings.scrollbar_animator =
                crate::cc::trees::layer_tree_settings::ScrollbarAnimator::ANDROID_OVERLAY;
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            settings.scrollbar_animator =
                crate::cc::trees::layer_tree_settings::ScrollbarAnimator::AURA_OVERLAY;
        }
        settings.scrollbar_fade_delay = crate::adapter::time::TimeDelta::Milliseconds(500);
        settings.scrollbar_fade_duration = crate::adapter::time::TimeDelta::Milliseconds(200);
        settings.scrollbar_thinning_duration = crate::adapter::time::TimeDelta::Milliseconds(200);
        settings.idle_thickness_scale = 0.4;
        // This assembly implements the renderer's pending-tree ProxyImpl
        // protocol even when both logical sequences use one injected executor.
        // Source settings defaults target SingleThreadProxy and commit active;
        // that must not disagree with this canonical Scheduler's pending mode.
        settings.commit_to_active_tree = false;
        settings.scheduled_raster_task_limit = config.raster_task_limit;
        Self {
            settings,
            picture,
            core: CoreConfig {
                report_android_scroll_jank: false,
                scroll_snap_animation: false,
                height_change_cancel_animations: true,
                smooth_scroll: false,
                new_content_for_checkerboarded_scrolls: true,
                new_content_per_scroll: false,
                image_animation_trace_enabled: false,
            },
            frame_sink: HostFrameSinkConfig {
                send_explicit_decode_requests_immediately: true,
                new_content_per_frame: true,
                pool: super::pool_services::RasterPoolConfig {
                    prefer_exact_size_reuse: false,
                    invalidate_resources_on_size_change: false,
                },
            },
            metadata: HostMetadataConfig {
                checkerboard_mode: HostCheckerboardMode::kPerFrame,
                is_new_trace: false,
                trees_in_viz: false,
            },
            render: RenderConfig {
                size_oopif_effect_surfaces_at_external_scale: true,
                allow_undamaged_nonroot_render_pass_to_skip: true,
                view_transitions_new_rounding_change: true,
                trace_enabled: false,
            },
            scheduler: SchedulerConfig {
                features: crate::adapter::scheduler::SchedulerFeatures {
                    is_android: cfg!(target_os = "android"),
                    is_desktop_android: false,
                    is_eligible_for_throttle_main_frame_to_60hz: false,
                    kThrottleMainFrameTo60HzDesktopAndroid: false,
                    kThrottleMainFrameTo60HzWebView: false,
                    kThrottleMainFrameTo60Hz: cfg!(target_os = "android"),
                    kThrottleRepeatedNoDamageFrames: false,
                    kThrottleRepeatedNoDamageFramesThreshold1: 90,
                    kThrottleRepeatedNoDamageFramesThreshold2: 90,
                    kThrottleRepeatedNoDamageFramesIntervalFactor1: 2,
                    kThrottleRepeatedNoDamageFramesIntervalFactor2: 2,
                    kHighFramerateRequestFromClient: false,
                    kNoCompositorFrameAcks: false,
                },
                slim_scheduler: false,
                // This Page uses Chromium's threaded renderer compositor
                // protocol with an external BeginFrame source. WebView's
                // synchronous-compositor state machine requires
                // using_synchronous_renderer_compositor and changes deadline,
                // invalidation, and activation policy; selecting it here made
                // an ordinary desktop commit miss the current frame's tile
                // preparation window.
                webview_scheduler: false,
                registered_trace_categories: Default::default(),
                enabled_trace_categories: Default::default(),
            },
            metrics: MetricsConfig {
                reporting_enabled: false,
                frame_sequence_metrics_enabled: true,
                lag_tracking_enabled: true,
                event_metrics_enabled: true,
            },
            diagnostics: DiagnosticConfig {
                maximum_vlog_level: None,
                client_name_for_metrics: Some("Renderer".into()),
                dump_compositor_frame_switch: None,
                allow_debug_file_output: false,
            },
            input: InputConfig {
                // Keep this capability in lockstep with the picture feature
                // and the queue-aware InputHandlerClient above.
                wait_for_late_scroll_events: true,
                overscroll_effect_on_non_root_scrollers: true,
                snap_fling_near_extremes: true,
                limit_scroll_delta_to_scroller_size: true,
                scroll_event_dispatch_mode: "DispatchScrollEventsUntilDeadline".into(),
                late_scroll_deadline_ratio: 0.333,
            },
            events: Arc::new(super::services::NoopEventSink),
            frame_trace_categories: Default::default(),
            tree_environment: TreeEnvironmentConfig {
                selection_edge_visibility_uses_full_edge: true,
                view_transition_capture_and_display: false,
            },
        }
    }
}
pub(crate) fn CreateDefault(
    config: CompositorConfig,
    executors: CompositorExecutors,
) -> Result<super::engine_assembly::EngineAssembly, CompositorError> {
    if config.thread {
        return Err(CompositorError::UnsupportedExecutionConfiguration {
            reason: "separate main/impl Proxy transport is not yet constructed",
        });
    }
    let options = FactoryOptions::cpu_default(&config);
    let paint = CpuPaintServices::<SkRegion>::new_cpu(
        super::paint_services::CpuPaintFeatures {
            cc_slimming: true,
            singular_transform_fix: true,
            merge_fixed_layers: true,
            merge_sticky_layers: true,
            raster_inducing_scroll: cfg!(any(target_os = "linux", target_os = "windows")),
            scrolling_contents_cull_rect: true,
            remove_scroll_node_workaround: true,
            selection_edge_visibility_uses_full_edge: true,
            canvas_draw_element: false,
            expensive_dchecks: cfg!(debug_assertions),
            paint_under_invalidation_checking: false,
            layer_merge_distance_limit: 0x10000000,
        },
        super::paint::SkiaPaintDiagnostics {
            trace: std::env::var_os("RECHROM_DIAG_STACKING").is_some(),
            verbose_level: 0,
            histogram: false,
        },
    )
    .map_err(|error| CompositorError::Backend(std::io::Error::other(error.to_string())))?;
    Create(config, executors, options, Arc::new(paint))
}
