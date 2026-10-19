// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/layer_tree_settings.h:1-253; cc/trees/layer_tree_settings.cc:1-14.
use super::managed_memory_policy::ManagedMemoryPolicy;
use crate::adapter::{
    geometry::Size, resource_types::PriorityCutoff, settings::LayerTreeSettingsFeatures,
    time::TimeDelta,
};
use crate::cc::{
    debug::layer_tree_debug_state::LayerTreeDebugState,
    scheduler::scheduler_settings::SchedulerSettings,
    tiles::tile_manager_settings::TileManagerSettings,
};
// Source: cc/trees/layer_tree_settings.h:68-73.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarAnimator {
    NO_ANIMATOR,
    ANDROID_OVERLAY,
    AURA_OVERLAY,
}
// Source: cc/trees/layer_tree_settings.h:24-249.
// Source: cc/trees/layer_tree_settings.cc:27-28 (defaulted copy/destruction).
#[derive(Clone, Debug)]
pub struct LayerTreeSettings {
    // Source: cc/trees/layer_tree_settings.h:50.
    pub trees_in_viz_in_viz_process: bool,
    // Source: cc/trees/layer_tree_settings.h:53.
    pub display_tree_draw_mode_is_gpu: bool,
    // Source: cc/trees/layer_tree_settings.h:55.
    pub single_thread_proxy_scheduler: bool,
    // Source: cc/trees/layer_tree_settings.h:56.
    pub main_frame_before_activation_enabled: bool,
    // Source: cc/trees/layer_tree_settings.h:57.
    pub using_synchronous_renderer_compositor: bool,
    // Source: cc/trees/layer_tree_settings.h:58.
    pub enable_early_damage_check: bool,
    // Source: cc/trees/layer_tree_settings.h:61.
    pub damaged_frame_limit: i32,
    // Source: cc/trees/layer_tree_settings.h:62.
    pub can_use_lcd_text: bool,
    // Source: cc/trees/layer_tree_settings.h:63.
    pub gpu_rasterization_disabled: bool,
    // Source: cc/trees/layer_tree_settings.h:64.
    pub gpu_rasterization_msaa_sample_count: i32,
    // Source: cc/trees/layer_tree_settings.h:65.
    pub gpu_rasterization_skewport_target_time_in_seconds: f32,
    // Source: cc/trees/layer_tree_settings.h:66.
    pub use_gpu_memory_buffer_resources: bool,
    // Source: cc/trees/layer_tree_settings.h:73.
    pub scrollbar_animator: ScrollbarAnimator,
    // Source: cc/trees/layer_tree_settings.h:74.
    pub scrollbar_fade_delay: TimeDelta,
    // Source: cc/trees/layer_tree_settings.h:75.
    pub scrollbar_fade_duration: TimeDelta,
    // Source: cc/trees/layer_tree_settings.h:76.
    pub scrollbar_thinning_duration: TimeDelta,
    // Source: cc/trees/layer_tree_settings.h:77.
    pub idle_thickness_scale: f32,
    // Source: cc/trees/layer_tree_settings.h:78.
    pub scrollbar_flash_after_any_scroll_update: bool,
    // Source: cc/trees/layer_tree_settings.h:79.
    pub scrollbar_flash_once_after_scroll_update: bool,
    // Source: cc/trees/layer_tree_settings.h:80.
    pub scrollbar_flash_once_visible_on_viewport: bool,
    // Source: cc/trees/layer_tree_settings.h:81.
    pub scrollbar_flash_when_mouse_enter: bool,
    // Source: cc/trees/layer_tree_settings.h:82.
    pub scroll_animation_duration_for_testing: TimeDelta,
    // Source: cc/trees/layer_tree_settings.h:83.
    pub layers_always_allowed_lcd_text: bool,
    // Source: cc/trees/layer_tree_settings.h:84.
    pub top_controls_show_threshold: f32,
    // Source: cc/trees/layer_tree_settings.h:85.
    pub top_controls_hide_threshold: f32,
    // Source: cc/trees/layer_tree_settings.h:86.
    pub default_tile_size: Size,
    // Source: cc/trees/layer_tree_settings.h:87.
    pub max_untiled_layer_size: Size,
    // Source: cc/trees/layer_tree_settings.h:90.
    pub max_gpu_raster_tile_size: Size,
    // Source: cc/trees/layer_tree_settings.h:94.
    pub min_height_for_gpu_raster_tile: i32,
    // Source: cc/trees/layer_tree_settings.h:95.
    pub minimum_occlusion_tracking_size: Size,
    // Source: cc/trees/layer_tree_settings.h:100.
    pub tiling_interest_area_padding: i32,
    // Source: cc/trees/layer_tree_settings.h:103.
    pub skewport_target_time_in_seconds: f32,
    // Source: cc/trees/layer_tree_settings.h:104.
    pub skewport_extrapolation_limit_in_screen_pixels: i32,
    // Source: cc/trees/layer_tree_settings.h:110.
    pub max_memory_for_prepaint_percentage: usize,
    // Source: cc/trees/layer_tree_settings.h:111.
    pub use_zero_copy: bool,
    // Source: cc/trees/layer_tree_settings.h:112.
    pub use_partial_raster: bool,
    // Source: cc/trees/layer_tree_settings.h:115.
    pub enable_elastic_overscroll_on_root: bool,
    // Source: cc/trees/layer_tree_settings.h:118.
    pub enable_elastic_overscroll_for_subscroll: bool,
    // Source: cc/trees/layer_tree_settings.h:120.
    pub scheduled_raster_task_limit: usize,
    // Source: cc/trees/layer_tree_settings.h:121.
    pub use_occlusion_for_tile_prioritization: bool,
    // Source: cc/trees/layer_tree_settings.h:122.
    pub use_layer_lists: bool,
    // Source: cc/trees/layer_tree_settings.h:123.
    pub max_staging_buffer_usage_in_bytes: i32,
    // Source: cc/trees/layer_tree_settings.h:124.
    pub memory_policy: ManagedMemoryPolicy,
    // Source: cc/trees/layer_tree_settings.h:125.
    pub decoded_image_working_set_budget_bytes: usize,
    // Source: cc/trees/layer_tree_settings.h:128.
    pub max_preraster_distance_in_screen_pixels: i32,
    // Source: cc/trees/layer_tree_settings.h:129.
    pub prefer_rgba_4444: bool,
    // Source: cc/trees/layer_tree_settings.h:134.
    pub enable_checker_imaging: bool,
    // Source: cc/trees/layer_tree_settings.h:138.
    pub min_image_bytes_to_checker: usize,
    // Source: cc/trees/layer_tree_settings.h:141.
    pub only_checker_images_with_gpu_raster: bool,
    // Source: cc/trees/layer_tree_settings.h:143.
    pub initial_debug_state: LayerTreeDebugState,
    // Source: cc/trees/layer_tree_settings.h:147.
    pub is_for_embedded_frame: bool,
    // Source: cc/trees/layer_tree_settings.h:151.
    pub is_for_scalable_page: bool,
    // Source: cc/trees/layer_tree_settings.h:156.
    pub disallow_non_exact_resource_reuse: bool,
    // Source: cc/trees/layer_tree_settings.h:162.
    pub wait_for_all_pipeline_stages_before_draw: bool,
    // Source: cc/trees/layer_tree_settings.h:169.
    pub enable_smooth_scroll: bool,
    // Source: cc/trees/layer_tree_settings.h:176.
    pub commit_to_active_tree: bool,
    // Source: cc/trees/layer_tree_settings.h:180.
    pub enable_image_animation_resync: bool,
    // Source: cc/trees/layer_tree_settings.h:183.
    pub enable_edge_anti_aliasing: bool,
    // Source: cc/trees/layer_tree_settings.h:187.
    pub use_painted_device_scale_factor: bool,
    // Source: cc/trees/layer_tree_settings.h:193.
    pub commit_fractional_scroll_deltas: bool,
    #[cfg(debug_assertions)]
    // Source: cc/trees/layer_tree_settings.h:197.
    pub log_on_ui_double_background_blur: bool,
    // Source: cc/trees/layer_tree_settings.h:202.
    pub enable_backface_visibility_interop: bool,
    // Source: cc/trees/layer_tree_settings.h:208.
    pub enable_compositing_based_throttling: bool,
    // Source: cc/trees/layer_tree_settings.h:211.
    pub is_layer_tree_for_ui: bool,
    // Source: cc/trees/layer_tree_settings.h:217.
    pub release_tile_resources_for_hidden_layers: bool,
    // Source: cc/trees/layer_tree_settings.h:221.
    pub enable_fluent_scrollbar: bool,
    // Source: cc/trees/layer_tree_settings.h:226.
    pub enable_fluent_overlay_scrollbar: bool,
    // Source: cc/trees/layer_tree_settings.h:229.
    pub disable_frame_rate_limit: bool,
    // Source: cc/trees/layer_tree_settings.h:234.
    pub max_render_buffer_bounds_for_sw: i32,
    // Source: cc/trees/layer_tree_settings.h:238.
    pub enable_hit_test_opaqueness: bool,
    // Source: cc/trees/layer_tree_settings.h:241.
    pub enable_variable_refresh_rate: bool,
    // Source: cc/trees/layer_tree_settings.h:244.
    pub enable_unbounded_element: bool,
    // Source: cc/trees/layer_tree_settings.h:248.
    pub enable_scroll_performance_timing: bool,
}
impl LayerTreeSettings {
    // Source: cc/trees/layer_tree_settings.h:99.
    pub const kDefaultSkewportExtrapolationLimitInScrenPixels: i32 = 2000;
    // Source: cc/trees/layer_tree_settings.cc:16-24; cc/trees/layer_tree_settings.h:50-248.
    pub fn new(features: &(impl LayerTreeSettingsFeatures + ?Sized)) -> Self {
        Self {
            trees_in_viz_in_viz_process: false,
            display_tree_draw_mode_is_gpu: false,
            single_thread_proxy_scheduler: true,
            main_frame_before_activation_enabled: false,
            using_synchronous_renderer_compositor: false,
            enable_early_damage_check: false,
            damaged_frame_limit: 3,
            can_use_lcd_text: true,
            gpu_rasterization_disabled: false,
            gpu_rasterization_msaa_sample_count: -1,
            gpu_rasterization_skewport_target_time_in_seconds: 0.2,
            use_gpu_memory_buffer_resources: false,
            scrollbar_animator: ScrollbarAnimator::NO_ANIMATOR,
            scrollbar_fade_delay: TimeDelta::default(),
            scrollbar_fade_duration: TimeDelta::default(),
            scrollbar_thinning_duration: TimeDelta::default(),
            idle_thickness_scale: 0.4,
            scrollbar_flash_after_any_scroll_update: false,
            scrollbar_flash_once_after_scroll_update: false,
            scrollbar_flash_once_visible_on_viewport: false,
            scrollbar_flash_when_mouse_enter: false,
            scroll_animation_duration_for_testing: TimeDelta::default(),
            layers_always_allowed_lcd_text: false,
            top_controls_show_threshold: 0.5,
            top_controls_hide_threshold: 0.5,
            default_tile_size: Size::new(256, 256),
            max_untiled_layer_size: Size::new(512, 512),
            max_gpu_raster_tile_size: Size::default(),
            min_height_for_gpu_raster_tile: 256,
            minimum_occlusion_tracking_size: Size::new(160, 160),
            tiling_interest_area_padding: 3000,
            skewport_target_time_in_seconds: 1.0,
            skewport_extrapolation_limit_in_screen_pixels:
                Self::kDefaultSkewportExtrapolationLimitInScrenPixels,
            max_memory_for_prepaint_percentage: 100,
            use_zero_copy: false,
            use_partial_raster: false,
            enable_elastic_overscroll_on_root: false,
            enable_elastic_overscroll_for_subscroll: false,
            scheduled_raster_task_limit: 32,
            use_occlusion_for_tile_prioritization: false,
            use_layer_lists: features.UseLayerListsByDefault(),
            max_staging_buffer_usage_in_bytes: 32 * 1024 * 1024,
            memory_policy: ManagedMemoryPolicy::WithLimits(
                64 * 1024 * 1024,
                PriorityCutoff::CUTOFF_ALLOW_EVERYTHING,
                ManagedMemoryPolicy::kDefaultNumResourcesLimit,
            ),
            decoded_image_working_set_budget_bytes: features.ImageDecodeWorkingSetBytes(false),
            max_preraster_distance_in_screen_pixels: 1000,
            prefer_rgba_4444: false,
            enable_checker_imaging: false,
            min_image_bytes_to_checker: 1 * 1024 * 1024,
            only_checker_images_with_gpu_raster: false,
            initial_debug_state: LayerTreeDebugState::default(),
            is_for_embedded_frame: false,
            is_for_scalable_page: true,
            disallow_non_exact_resource_reuse: false,
            wait_for_all_pipeline_stages_before_draw: false,
            enable_smooth_scroll: false,
            commit_to_active_tree: true,
            enable_image_animation_resync: true,
            enable_edge_anti_aliasing: true,
            use_painted_device_scale_factor: false,
            commit_fractional_scroll_deltas: false,
            #[cfg(debug_assertions)]
            log_on_ui_double_background_blur: false,
            enable_backface_visibility_interop: false,
            enable_compositing_based_throttling: false,
            is_layer_tree_for_ui: false,
            release_tile_resources_for_hidden_layers: false,
            enable_fluent_scrollbar: false,
            enable_fluent_overlay_scrollbar: false,
            disable_frame_rate_limit: false,
            max_render_buffer_bounds_for_sw: 16 * 1024,
            enable_hit_test_opaqueness: false,
            enable_variable_refresh_rate: false,
            enable_unbounded_element: false,
            enable_scroll_performance_timing: false,
        }
    }
    // Source: cc/trees/layer_tree_settings.cc:29-32.
    pub fn TreesInVizInClientProcess(
        &self,
        features: &(impl LayerTreeSettingsFeatures + ?Sized),
    ) -> bool {
        !self.is_layer_tree_for_ui && !self.trees_in_viz_in_viz_process && features.TreesInViz()
    }
    // Source: cc/trees/layer_tree_settings.cc:34-37.
    pub fn TreeAnimationsInVizInClientProcess(
        &self,
        features: &(impl LayerTreeSettingsFeatures + ?Sized),
    ) -> bool {
        self.TreesInVizInClientProcess(features) && features.TreeAnimationsInViz()
    }
    // Source: cc/trees/layer_tree_settings.cc:39-42.
    pub fn TreeAnimationsInVizInVizProcess(
        &self,
        features: &(impl LayerTreeSettingsFeatures + ?Sized),
    ) -> bool {
        self.trees_in_viz_in_viz_process && features.TreeAnimationsInViz()
    }
    // Source: cc/trees/layer_tree_settings.cc:44-80.
    pub fn ToSchedulerSettings(
        &self,
        features: &(impl LayerTreeSettingsFeatures + ?Sized),
    ) -> SchedulerSettings {
        let mut settings = SchedulerSettings::default();
        settings.main_frame_before_activation_enabled = self.main_frame_before_activation_enabled;
        settings.using_synchronous_renderer_compositor = self.using_synchronous_renderer_compositor;
        settings.wait_for_all_pipeline_stages_before_draw =
            self.wait_for_all_pipeline_stages_before_draw;
        settings.disable_frame_rate_limit = self.disable_frame_rate_limit;
        if features.DeferImplInvalidation() {
            settings.delay_impl_invalidation_frames = features.DeferImplInvalidationFrames();
        }
        if !self.single_thread_proxy_scheduler {
            let mode = features.ScrollEventDispatchMode();
            settings.scroll_deadline_mode_enabled = features.WaitForLateScrollEvents()
                && (mode == "DispatchScrollEventsImmediately"
                    || mode == "UseScrollPredictorForDeadline"
                    || mode == "DispatchScrollEventsUntilDeadline");
            if settings.scroll_deadline_mode_enabled {
                settings.scroll_deadline_ratio = features.WaitForLateScrollEventsDeadlineRatio();
            }
        }
        settings.single_threaded_proxy = self.single_thread_proxy_scheduler;
        settings
    }
    // Source: cc/trees/layer_tree_settings.cc:82-90.
    pub fn ToTileManagerSettings(&self) -> TileManagerSettings {
        let mut settings = TileManagerSettings::default();
        settings.use_partial_raster = self.use_partial_raster;
        settings.enable_checker_imaging = self.enable_checker_imaging;
        settings.min_image_bytes_to_checker = self.min_image_bytes_to_checker;
        settings.needs_notify_ready_to_draw =
            self.commit_to_active_tree | self.wait_for_all_pipeline_stages_before_draw;
        settings
    }
}
// Source: cc/trees/layer_tree_settings.h:106-109.
const _: () = assert!(LayerTreeSettings::kDefaultSkewportExtrapolationLimitInScrenPixels <= 3000);
