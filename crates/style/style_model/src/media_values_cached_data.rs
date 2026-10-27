// Copyright 2014 The Chromium Authors. BSD-style license; see Chromium LICENSE.
use super::color_space_gamut::ColorSpaceGamut;
use super::device_posture_provider::DevicePostureType;
use super::display_mode::DisplayMode;
use super::forced_colors::ForcedColors;
use super::navigation_controls::NavigationControls;
use super::preferred_color_scheme::PreferredColorScheme;
use super::preferred_contrast::PreferredContrast;
use super::scripting::Scripting;
use super::web_preferences::{HoverType, OutputDeviceUpdateAbilityType, PointerType};
use super::window_show_state::WindowShowState;
use foundation::{String, WritingMode};

/// Glyph measurements carried only by a property-conversion scope. Media
/// queries retain their initial-font measurements; the cascade installs these
/// values from the element's selected font, like Blink's
/// CSSToLengthConversionData::FontSizes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontRelativeLengthMetrics {
    pub ex: f32,
    pub ch: f32,
    pub ic: f32,
    pub cap: f32,
}
// cpp: third_party/blink/renderer/core/css/media_values_cached.h:28-88
#[derive(Clone, Debug, PartialEq)]
pub struct MediaValuesCachedData {
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub small_viewport_width: f64,
    pub small_viewport_height: f64,
    pub large_viewport_width: f64,
    pub large_viewport_height: f64,
    pub dynamic_viewport_width: f64,
    pub dynamic_viewport_height: f64,
    pub device_width: i32,
    pub device_height: i32,
    /// LocalFrame::DevicePixelRatio: host/inspector device scale multiplied by
    /// LayoutZoomFactor. Consumers must not multiply zoom a second time.
    pub device_pixel_ratio: f32,
    /// LocalFrame::LayoutZoomFactor retained independently for CSS coordinate
    /// conversion; it is already represented in `device_pixel_ratio` above.
    pub layout_zoom_factor: f32,
    pub device_supports_hdr: bool,
    pub color_bits_per_component: i32,
    pub monochrome_bits_per_component: i32,
    pub inverted_colors: bool,
    pub primary_pointer_type: PointerType,
    pub available_pointer_types: i32,
    pub primary_hover_type: HoverType,
    pub output_device_update_ability_type: OutputDeviceUpdateAbilityType,
    pub available_hover_types: i32,
    pub em_size: f32,
    pub ex_size: f32,
    pub ch_size: f32,
    pub ic_size: f32,
    pub cap_size: f32,
    pub line_height: f32,
    /// Style property conversion's retained unzoomed document-element
    /// line-height.
    /// MediaValuesCached::LineHeight/RootLineHeight continue to use
    /// `line_height`; this optional owner is installed only for a translated
    /// style-application scope, matching CSSToLengthConversionData's separate
    /// LineHeightSize root member.
    pub style_root_line_height: Option<f32>,
    pub style_font_metrics: Option<FontRelativeLengthMetrics>,
    pub style_root_font_metrics: Option<FontRelativeLengthMetrics>,
    pub three_d_enabled: bool,
    pub strict_mode: bool,
    pub media_type: String,
    pub display_mode: DisplayMode,
    pub window_show_state: WindowShowState,
    pub resizable: bool,
    pub color_gamut: ColorSpaceGamut,
    pub preferred_color_scheme: PreferredColorScheme,
    pub preferred_contrast: PreferredContrast,
    pub prefers_reduced_motion: bool,
    pub prefers_reduced_data: bool,
    pub prefers_reduced_transparency: bool,
    pub forced_colors: ForcedColors,
    pub navigation_controls: NavigationControls,
    pub horizontal_viewport_segments: i32,
    pub vertical_viewport_segments: i32,
    pub device_posture: DevicePostureType,
    pub scripting: Scripting,
}
impl Default for MediaValuesCachedData {
    // cpp: third_party/blink/renderer/core/css/media_values_cached.h:32-86
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:13
    fn default() -> Self {
        Self {
            viewport_width: 0.0,
            viewport_height: 0.0,
            small_viewport_width: 0.0,
            small_viewport_height: 0.0,
            large_viewport_width: 0.0,
            large_viewport_height: 0.0,
            dynamic_viewport_width: 0.0,
            dynamic_viewport_height: 0.0,
            device_width: 0,
            device_height: 0,
            device_pixel_ratio: 1.0,
            layout_zoom_factor: 1.0,
            device_supports_hdr: false,
            color_bits_per_component: 24,
            monochrome_bits_per_component: 0,
            inverted_colors: false,
            primary_pointer_type: PointerType::kPointerNone,
            available_pointer_types: 1,
            primary_hover_type: HoverType::kHoverNone,
            output_device_update_ability_type: OutputDeviceUpdateAbilityType::kFastType,
            available_hover_types: 1,
            em_size: 16.0,
            ex_size: 8.0,
            ch_size: 8.0,
            ic_size: 16.0,
            cap_size: 16.0,
            line_height: 0.0,
            style_root_line_height: None,
            style_font_metrics: None,
            style_root_font_metrics: None,
            three_d_enabled: false,
            strict_mode: true,
            media_type: String::default(),
            display_mode: DisplayMode::kBrowser,
            window_show_state: WindowShowState::kDefault,
            resizable: true,
            color_gamut: ColorSpaceGamut::SRGB,
            preferred_color_scheme: PreferredColorScheme::kLight,
            preferred_contrast: PreferredContrast::kNoPreference,
            prefers_reduced_motion: false,
            prefers_reduced_data: false,
            prefers_reduced_transparency: false,
            forced_colors: ForcedColors::kNone,
            navigation_controls: NavigationControls::kNone,
            horizontal_viewport_segments: 0,
            vertical_viewport_segments: 0,
            device_posture: DevicePostureType::kContinuous,
            scripting: Scripting::kNone,
        }
    }
}
// cpp: third_party/blink/renderer/core/css/media_values_cached.h:26,164
