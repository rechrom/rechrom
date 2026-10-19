// Source: components/viz/common/display/renderer_settings.h:1-93
// Source: components/viz/common/display/renderer_settings.cc:1-15
// Chromium revision: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// C++ default/copy constructors and destructors map to Default/Clone/Drop.
#![allow(dead_code)]

// Source: components/viz/common/display/renderer_settings.h:21-63
#[derive(Clone, Debug)]
pub struct RendererSettings {
    pub allow_antialiasing: bool,
    pub force_antialiasing: bool,
    pub force_blending_with_shaders: bool,
    pub partial_swap_enabled: bool,
    pub should_clear_root_render_pass: bool,
    pub release_overlay_resources_after_gpu_query: bool,
    pub dont_round_texture_sizes_for_pixel_tests: bool,
    pub highp_threshold_min: i32,
    pub auto_resize_output_surface: bool,
    pub requires_alpha_channel: bool,
    pub disable_render_pass_bypassing: bool,
    pub force_non_scanout_backing_for_pixel_tests: bool,
    pub slow_down_compositing_scale_factor: i32,
    pub occlusion_culler_settings: OcclusionCullerSettings,
}

impl Default for RendererSettings {
    fn default() -> Self {
        Self {
            allow_antialiasing: true,
            force_antialiasing: false,
            force_blending_with_shaders: false,
            partial_swap_enabled: false,
            should_clear_root_render_pass: true,
            release_overlay_resources_after_gpu_query: false,
            dont_round_texture_sizes_for_pixel_tests: false,
            highp_threshold_min: 0,
            auto_resize_output_surface: true,
            requires_alpha_channel: false,
            disable_render_pass_bypassing: false,
            force_non_scanout_backing_for_pixel_tests: false,
            slow_down_compositing_scale_factor: 1,
            occlusion_culler_settings: OcclusionCullerSettings::default(),
        }
    }
}

// Source: components/viz/common/display/renderer_settings.h:44-61
#[derive(Clone, Debug)]
pub struct OcclusionCullerSettings {
    pub maximum_occluder_complexity: i32,
    pub quad_split_limit: i32,
    pub minimum_fragments_reduced: i32,
    pub occluder_minium_visible_quad_size: i32,
    pub generate_complex_occluder_for_rounded_corners: bool,
    pub minumum_quad_size_with_rounded_corners: i32,
}
impl Default for OcclusionCullerSettings {
    fn default() -> Self {
        Self {
            maximum_occluder_complexity: 10,
            quad_split_limit: 5,
            minimum_fragments_reduced: 128 * 128,
            occluder_minium_visible_quad_size: 32 * 32,
            generate_complex_occluder_for_rounded_corners: false,
            minumum_quad_size_with_rounded_corners: 128 * 128,
        }
    }
}

// SOURCE_EXCLUDED: components/viz/common/display/renderer_settings.h:65-74. Ozone overlay strategies and
// macOS display IDs belong to the host output device, outside this CPU Engine.

// Source: components/viz/common/display/renderer_settings.h:77-89
#[derive(Clone, Debug, Default)]
pub struct DebugRendererSettings {
    pub tint_composited_content: bool,
    pub tint_composited_content_modulate: bool,
    pub show_overdraw_feedback: bool,
    pub show_dc_layer_debug_borders: bool,
    pub show_aggregated_damage: bool,
}
