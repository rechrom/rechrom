//! The unique private production construction site. Only independent executor,
//! paint value, diagnostics and native resource capabilities enter this file;
//! every resident compositor owner remains internal to the Engine assembly.
use super::{
    client_resources::RasterClientResources,
    engine_core::NativeEngineCore,
    engine_queries::{CallScopedSchedulerQueries, NativeFactoryPictureState},
    execution::{EngineTask, EngineTaskQueue},
    host_services::{CpuHostServices, HostInit},
    main_endpoint::EngineMainEndpoint,
    main_host::CpuMainLayerHost,
    native_activation::NativeActivationServices,
    native_animation::NativeAnimationOwner,
    native_animation_services::{NativeAnimationServices, NativeScrollbarRegistrationServices},
    native_begin_frame::NativeBeginFrameServices,
    native_client::{NativeClientServices, NativeClientWorkletServices},
    native_core_services::{
        CpuNoGpuCapabilities, NativeCoreConfig, NativeCoreServices, NativeRenderingStats,
    },
    native_frame_build::NativeFrameBuildServices,
    native_frame_sink::NativeFrameSinkEndpoint,
    native_host_action::NativeHostActionServices,
    native_host_core::{NativeHostCoreOwners, NativeImageAnimationDispatch, OsSurfaceTokenSource},
    native_host_frame_sink::{
        NativeHostFrameSinkCallbacks, NativeHostFrameSinkConfig, NativeHostFrameSinkServices,
    },
    native_host_lifecycle::{HostLifecycleEffects, NativeHostLifecycle},
    native_host_metadata::{NativeHostMetadataConfig, NativeHostMetadataServices},
    native_host_picture::{
        NativeHostPictureEnvironment, NativeHostPictureFeatures, NativeHostPictureMutationServices,
        NativeHostPictureState, NativeHostPictureTileServices, NativePictureLayerServices,
    },
    native_images::{CpuNativeImageServices, NativeCheckerImageClient, NativeImageDecodeCaches},
    native_latency::NativeLatencyMonitorRegistry,
    native_metrics::{NativeMetricsConfig, NativeMetricsServices},
    native_render::{NativeRenderConfig, NativeRenderServices},
    native_resource_events::NativeResourceEffects,
    native_resources::{NativePictureReleaseServices, NativeResourceServices},
    native_scheduler::{
        NativeSchedulerConfig, NativeSchedulerEffects, NativeSchedulerHost, NativeTimingHistory,
    },
    native_scrollbar::NativeScrollbarLayerServices,
    native_services::{
        NativeBrowserControlsServices, NativeDiagnosticConfig, NativeDiagnosticServices,
        NativeEventSink, NativeInputConfig, NativeInputEnvironmentServices,
        NativeTaskRunnerServices,
    },
    native_tree_environment::{NativeTreeEnvironmentConfig, NativeTreeEnvironmentServices},
    native_ui::{NativeUiRasterServices, NativeUiServices},
    native_worklet::NativeWorkletServices,
    paint_services::CpuPaintServices,
    skia::SkRegion,
    tile_execution::TileExecution,
    types::{LocalSurfaceId, MainMutatorHostIdentity},
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
    NativeCompositorTypes,
};
use std::sync::{Arc, Mutex};

/// Explicit native feature/configuration values. These are ownerless process
/// settings, not synthesized Layer/Host state or a Page callback registry.
pub(crate) struct NativeFactoryOptions {
    pub settings: LayerTreeSettings,
    pub core: NativeCoreConfig,
    pub picture: Arc<NativeHostPictureFeatures>,
    pub frame_sink: NativeHostFrameSinkConfig,
    pub metadata: NativeHostMetadataConfig,
    pub render: NativeRenderConfig,
    pub scheduler: NativeSchedulerConfig,
    pub metrics: NativeMetricsConfig,
    pub diagnostics: NativeDiagnosticConfig,
    pub input: NativeInputConfig,
    pub events: Arc<dyn NativeEventSink>,
    pub frame_trace_categories: std::collections::BTreeSet<String>,
    pub tree_environment: NativeTreeEnvironmentConfig,
}

/// All shared objects here are independent external capabilities. In
/// particular this allocation cannot contain a PAC/MainHost/Host/Tree/Tile or a
/// callable owner endpoint. Thin native facades clone the same capability Arc;
/// none creates a second resource table, image cache or animation state.
pub(crate) struct NativeFactoryCapabilities {
    pub tree_environment: NativeTreeEnvironmentConfig,
    pub rendering_stats: Arc<NativeRenderingStats>,
    pub ui_raster: Arc<dyn NativeUiRasterServices>,
    pub diagnostics: NativeDiagnosticConfig,
    pub frame_trace_categories: std::collections::BTreeSet<String>,
    pub events: Arc<dyn NativeEventSink>,
    pub lifecycle: Arc<NativeHostLifecycle>,
    pub resources: RasterClientResources,
    pub frame_sink: Arc<Mutex<NativeFrameSinkEndpoint>>,
    pub tile_execution: Arc<Mutex<TileExecution<SkRegion>>>,
    pub main_animations: NativeAnimationOwner,
    pub animations: NativeAnimationOwner,
    pub latency_monitors: NativeLatencyMonitorRegistry,
    pub runners: Arc<NativeTaskRunnerServices>,
    pub commands: EngineTaskQueue,
    pub render: NativeRenderServices,
    pub picture: Arc<NativeHostPictureFeatures>,
    pub timing: Arc<NativeTimingHistory>,
    pub resource_effects: NativeResourceEffects,
    pub scheduler_queries: Arc<CallScopedSchedulerQueries>,
}
struct ImageAnimationDelivery(EngineTaskQueue);
impl NativeImageAnimationDispatch for ImageAnimationDelivery {
    fn Dispatch(&self, task: crate::adapter::image_animation::ImageAnimationTask) {
        self.0.Push(EngineTask::ImageAnimation(task));
    }
}
struct CheckerDelivery {
    commands: EngineTaskQueue,
    // Only source routing identity. No manager/host can be recovered here.
    owner: HostImplDelegateId,
}
impl NativeCheckerImageClient for CheckerDelivery {
    fn NeedsInvalidationForCheckerImagedTiles(&self) {
        self.commands.Push(EngineTask::CheckerNeedsInvalidation {
            delegate: self.owner,
        });
    }
}

/// The embedding's native implementation supplies Paint's pure value/recording
/// primitives inside this crate; these are never arguments to public Commit.
pub(crate) fn Create(
    config: CompositorConfig,
    executors: CompositorExecutors,
    options: NativeFactoryOptions,
    paint: Arc<CpuPaintServices<SkRegion>>,
) -> Result<super::engine_assembly::NativeEngineAssembly, CompositorError> {
    if config.thread {
        return Err(CompositorError::UnsupportedExecutionConfiguration {
            reason: "separate main/impl Proxy transport is not yet constructed",
        });
    }
    // The factory owns identity generation; Page never passes internal IDs.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let owner = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    assert!(owner <= i32::MAX as u64, "native engine identity exhausted");
    let commands = EngineTaskQueue::default();
    let lifecycle_effects = HostLifecycleEffects::default();
    let lifecycle = Arc::new(NativeHostLifecycle::new(lifecycle_effects.clone()));
    let queries = Arc::new(CallScopedSchedulerQueries::default());
    let resources = RasterClientResources::new();
    let sink_id = HostImplFrameSinkId(owner);
    let sink = Arc::new(Mutex::new(NativeFrameSinkEndpoint::new(
        sink_id,
        resources.clone(),
    )));
    let runners = Arc::new(NativeTaskRunnerServices::new_threaded_on_single_executor(
        executors.compositor.clone(),
    ));
    let main_animations = NativeAnimationOwner::default();
    let impl_animations = NativeAnimationOwner::default();
    let latency_monitors = NativeLatencyMonitorRegistry::default();
    let scheduler_effects = NativeSchedulerEffects::default();
    let scheduler_host = Arc::new(NativeSchedulerHost::new(
        owner,
        executors.compositor.clone(),
        commands.clone(),
        scheduler_effects.clone(),
        options.events.clone(),
        options.scheduler,
    ));
    let timing = Arc::new(NativeTimingHistory::new(
        UMACategory::RENDERER_UMA,
        executors.compositor.clone(),
        options.events.clone(),
    ));
    let checker = Arc::new(CheckerDelivery {
        commands: commands.clone(),
        owner: HostImplDelegateId(owner),
    });
    let caches = NativeImageDecodeCaches::default();
    let image_decode = super::host_owners::ImageDecodeFactory(Box::new(
        super::native_software_images::NativeSoftwareImageDecodeFactory::new(
            caches.clone(),
            Arc::new(super::raster_resources::EngineRasterFailure(
                commands.clone(),
            )),
            paint.PaintValues(),
        ),
    ));
    let images = CpuNativeImageServices::new(caches, checker);
    let tile_execution = Arc::new(Mutex::new(TileExecution::new(
        images,
        executors.raster_provider.clone(),
        executors.compositor.clone(),
        executors.raster,
        commands.clone(),
        options.events.clone(),
        config.raster_worker_limit,
    )));
    let stats = Arc::new(NativeRenderingStats::new(HostImplRenderingStatsId(owner)));
    let native = Arc::new(NativeFactoryCapabilities {
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
        render: NativeRenderServices::new(options.render, options.events.clone()),
        picture: options.picture.clone(),
        timing: timing.clone(),
        resource_effects: NativeResourceEffects::default(),
        scheduler_queries: queries.clone(),
    });

    let core_owners = NativeHostCoreOwners::new(
        NativeMetricsServices::new(options.metrics, options.events.clone()),
        impl_animations.clone(),
        NativeTaskRunnerServices::new_threaded_on_single_executor(executors.compositor.clone()),
        commands.clone(),
        Arc::new(ImageAnimationDelivery(commands.clone())),
        Arc::new(OsSurfaceTokenSource),
    );
    let core = NativeCoreServices::new(
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
        NativeHostFrameSinkServices::new(
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
    let services = CpuHostServices::<NativeCompositorTypes> {
        tile_execution,
        client: Some(Box::new(NativeClientServices::new(native.clone()))),
        client_worklet: Box::new(NativeClientWorkletServices::new(native.clone())),
        picture_environment: Box::new(NativeHostPictureEnvironment::new(
            options.picture.clone(),
            Arc::new(NativeFactoryPictureState::new(
                sink.clone(),
                queries.clone(),
                HostImplDelegateId(owner),
            )),
        )),
        picture_services: Box::new(NativePictureLayerServices::new(
            options.picture.clone(),
            options.events.clone(),
            resources.clone(),
        )),
        picture_tile_services: Box::new(NativeHostPictureTileServices::new(options.events.clone())),
        picture_mutation_services: Box::new(NativeHostPictureMutationServices::new(
            lifecycle.clone(),
            native.tile_execution.clone(),
        )),
        picture_registration_services: Box::new(NativeScrollbarRegistrationServices {
            animations: impl_animations.clone(),
            executor: executors.compositor.clone(),
            commands: commands.clone(),
            events: options.events.clone(),
        }),
        picture_release_services: Box::new(NativePictureReleaseServices::new(native.clone())),
        core: Box::new(core),
        debug: Box::new(NativeDiagnosticServices::new(
            options.diagnostics,
            options.events.clone(),
        )),
        ui: Box::new(NativeUiServices::new(native.clone())),
        animation: Some(Box::new(NativeAnimationServices::new(
            impl_animations.clone(),
            native.resource_effects.clone(),
            options.frame_trace_categories.clone(),
            options.events.clone(),
        ))),
        scrollbar_layer: Box::new(NativeScrollbarLayerServices::new(
            native.clone(),
            options.settings.clone(),
        )),
        worklet: Box::new(NativeWorkletServices::new(native.clone())),
        begin_frame: Box::new(NativeBeginFrameServices::new(native.clone())),
        activation: Box::new(NativeActivationServices::new(native.clone(), image_decode)),
        lifecycle: Box::new(NativeHostActionServices::new(native.clone())),
        input_environment: Box::new(NativeInputEnvironmentServices::new(
            options.input,
            options.events.clone(),
        )),
        scrollbar_registration: Box::new(NativeScrollbarRegistrationServices {
            animations: impl_animations.clone(),
            executor: executors.compositor.clone(),
            commands: commands.clone(),
            events: options.events.clone(),
        }),
        browser_controls: Box::new(NativeBrowserControlsServices::new(options.events.clone())),
        frame_build: Box::new(NativeFrameBuildServices::new(native.clone())),
        resources: Box::new(NativeResourceServices::new(native.clone())),
        frame_sink: Box::new(g()),
        tree_environment: Box::new(NativeTreeEnvironmentServices::new(
            native.clone(),
            LayerTreeHostId(owner),
            HostImplTaskRunnerProviderId(owner),
        )),
        metadata: Box::new(NativeHostMetadataServices::new(
            sink.clone(),
            lifecycle.clone(),
            options.events.clone(),
            options.metadata,
        )),
        task_runners: Box::new(NativeTaskRunnerServices::new_threaded_on_single_executor(
            executors.compositor.clone(),
        )),
        resource_returns: Box::new(g()),
        frame_sink_callbacks: Box::new(g()),
        draw: Box::new(native.render.clone()),
        surface: Box::new(native.render.clone()),
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
    let image_worker_task_runner: <NativeCompositorTypes as HostImplTypes>::ImageWorkerTaskRunner =
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
            &native
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
        embed_token: super::native_host_core::NativeSurfaceTokenSource::NextEmbedToken(
            &OsSurfaceTokenSource,
        ),
    };
    let input_client = Arc::new(super::native_input_client::NativeInputHandlerClient::default());
    let core = NativeEngineCore::new(
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
    super::engine_assembly::NativeEngineAssembly::new(
        core,
        scheduler_settings,
        scheduler_host,
        timing,
        commands,
        lifecycle_effects,
        scheduler_effects,
        native.render.clone(),
        native.resource_effects.clone(),
        native.latency_monitors.clone(),
        input_client,
    )
}

/// Default capability construction is admitted only after the native Paint
/// value/tracking/recording constructors and every Host facet are closed.

impl NativeFactoryOptions {
    /// Explicit desktop CPU embedding configuration. Optional GPU/Viz/trace
    /// capabilities are disabled here; canonical settings retain their defaults.
    pub(crate) fn cpu_default(config: &CompositorConfig) -> Self {
        let picture = Arc::new(NativeHostPictureFeatures {
            use_layer_lists_by_default: false,
            trees_in_viz: false,
            tree_animations_in_viz: false,
            defer_impl_invalidation: true,
            defer_impl_invalidation_frames: 4,
            scroll_event_dispatch_mode: "DispatchScrollEventsUntilDeadline".into(),
            // NativeInputHandlerClient is the embedding's CTEQ boundary. It
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
        // Source: third_party/blink/renderer/platform/widget/compositing/layer_tree_settings.cc:364.
        settings.use_layer_lists = true;
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc:374-379. Production renderer widgets
        // use ProxyImpl scheduling, including the late-scroll deadline mode.
        settings.single_thread_proxy_scheduler = false;
        // Chromium enables fractional commit deltas only when the experimental
        // FractionalScrollOffsets runtime feature is enabled. That feature is
        // disabled by default; keep cc's source default so Blink receives
        // integer committed offsets and the residual remains compositor-owned.
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc:364-370.
        // Source: content/public/common/content_features.cc:521.
        settings.commit_fractional_scroll_deltas = false;
        // Source: third_party/blink/renderer/platform/widget/compositing/
        //         layer_tree_settings.cc:65-85,472-493 and
        //         ui/native_theme/overlay_scrollbar_constants.cc:12-20.
        // The browser constraint theme uses desktop overlay scrollbars, so cc
        // owns their visibility and fade lifecycle rather than leaving the
        // solid-color thumb permanently visible.
        settings.scrollbar_animator =
            crate::cc::trees::layer_tree_settings::ScrollbarAnimator::AURA_OVERLAY;
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
            core: NativeCoreConfig {
                report_android_scroll_jank: false,
                scroll_snap_animation: false,
                height_change_cancel_animations: true,
                smooth_scroll: false,
                new_content_for_checkerboarded_scrolls: true,
                new_content_per_scroll: false,
                image_animation_trace_enabled: false,
            },
            frame_sink: NativeHostFrameSinkConfig {
                send_explicit_decode_requests_immediately: true,
                new_content_per_frame: true,
                pool: super::pool_services::RasterPoolConfig {
                    prefer_exact_size_reuse: false,
                    invalidate_resources_on_size_change: false,
                },
            },
            metadata: NativeHostMetadataConfig {
                checkerboard_mode: HostCheckerboardMode::kPerFrame,
                is_new_trace: false,
                trees_in_viz: false,
            },
            render: NativeRenderConfig {
                size_oopif_effect_surfaces_at_external_scale: true,
                allow_undamaged_nonroot_render_pass_to_skip: true,
                view_transitions_new_rounding_change: true,
                trace_enabled: false,
            },
            scheduler: NativeSchedulerConfig {
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
            metrics: NativeMetricsConfig {
                reporting_enabled: false,
                frame_sequence_metrics_enabled: true,
                lag_tracking_enabled: true,
                event_metrics_enabled: true,
            },
            diagnostics: NativeDiagnosticConfig {
                maximum_vlog_level: None,
                client_name_for_metrics: Some("Renderer".into()),
                dump_compositor_frame_switch: None,
                allow_debug_file_output: false,
            },
            input: NativeInputConfig {
                // Keep this capability in lockstep with the picture feature
                // and the queue-aware InputHandlerClient above.
                wait_for_late_scroll_events: true,
                overscroll_effect_on_non_root_scrollers: true,
                snap_fling_near_extremes: true,
                limit_scroll_delta_to_scroller_size: true,
                scroll_event_dispatch_mode: "DispatchScrollEventsUntilDeadline".into(),
                late_scroll_deadline_ratio: 0.333,
            },
            events: Arc::new(super::native_services::NoopNativeEventSink),
            frame_trace_categories: Default::default(),
            tree_environment: NativeTreeEnvironmentConfig {
                selection_edge_visibility_uses_full_edge: true,
                view_transition_capture_and_display: false,
            },
        }
    }
}
pub(crate) fn CreateDefault(
    config: CompositorConfig,
    executors: CompositorExecutors,
) -> Result<super::engine_assembly::NativeEngineAssembly, CompositorError> {
    if config.thread {
        return Err(CompositorError::UnsupportedExecutionConfiguration {
            reason: "separate main/impl Proxy transport is not yet constructed",
        });
    }
    let options = NativeFactoryOptions::cpu_default(&config);
    let paint = CpuPaintServices::<SkRegion>::new_native(
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
        super::paint::CpuNativePaintDiagnostics {
            trace: std::env::var_os("RECHROM_DIAG_STACKING").is_some(),
            verbose_level: 0,
            histogram: false,
        },
    )
    .map_err(|error| CompositorError::Backend(std::io::Error::other(error.to_string())))?;
    Create(config, executors, options, Arc::new(paint))
}
