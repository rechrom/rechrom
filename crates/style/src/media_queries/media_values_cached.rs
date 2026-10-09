// Copyright 2014 The Chromium Authors. BSD-style license; see Chromium LICENSE.
use super::color_space_gamut::ColorSpaceGamut;
use super::device_posture_provider::DevicePostureType;
use super::display_mode::DisplayMode;
use super::forced_colors::ForcedColors;
use super::media_values::MediaValues;
use super::navigation_controls::NavigationControls;
use super::preferred_color_scheme::PreferredColorScheme;
use super::preferred_contrast::PreferredContrast;
use super::scripting::Scripting;
use super::web_preferences::{HoverType, OutputDeviceUpdateAbilityType, PointerType};
use super::window_show_state::WindowShowState;
use foundation::{String, WritingMode};
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
    pub device_pixel_ratio: f32,
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
#[derive(Clone, Debug, Default)]
pub struct MediaValuesCached {
    data_: MediaValuesCachedData,
}
impl MediaValuesCached {
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:94-97
    pub fn new(data: &MediaValuesCachedData) -> Self {
        Self {
            data_: data.clone(),
        }
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:101-103
    pub fn Copy(&self) -> Self {
        self.clone()
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:105-108
    pub fn EmFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.em_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:110-114
    pub fn RemFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.em_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:116-119
    pub fn ExFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.ex_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:121-125
    pub fn RexFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.ex_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:127-130
    pub fn ChFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.ch_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:132-136
    pub fn RchFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.ch_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:138-141
    pub fn IcFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.ic_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:143-147
    pub fn RicFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.ic_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:149-152
    pub fn LineHeight(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.line_height
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:154-158
    pub fn RootLineHeight(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.line_height
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:160-164
    pub fn CapFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.cap_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:166-170
    pub fn RcapFontSize(&self, zoom: f32) -> f32 {
        debug_assert_eq!(1.0_f32, zoom);
        self.data_.cap_size
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:172-174
    pub fn ViewportWidth(&self) -> f64 {
        self.data_.viewport_width
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:176-178
    pub fn ViewportHeight(&self) -> f64 {
        self.data_.viewport_height
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:180-182
    pub fn SmallViewportWidth(&self) -> f64 {
        self.data_.small_viewport_width
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:184-186
    pub fn SmallViewportHeight(&self) -> f64 {
        self.data_.small_viewport_height
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:188-190
    pub fn LargeViewportWidth(&self) -> f64 {
        self.data_.large_viewport_width
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:192-194
    pub fn LargeViewportHeight(&self) -> f64 {
        self.data_.large_viewport_height
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:196-198
    pub fn DynamicViewportWidth(&self) -> f64 {
        self.data_.dynamic_viewport_width
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:200-202
    pub fn DynamicViewportHeight(&self) -> f64 {
        self.data_.dynamic_viewport_height
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:204-206
    pub fn ContainerWidth(&self) -> f64 {
        self.SmallViewportWidth()
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:208-210
    pub fn ContainerHeight(&self) -> f64 {
        self.SmallViewportHeight()
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:212-214
    pub fn ContainerWidthForName(&self, _name: &foundation::ScopedCSSName) -> f64 {
        self.SmallViewportWidth()
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:216-218
    pub fn ContainerHeightForName(&self, _name: &foundation::ScopedCSSName) -> f64 {
        self.SmallViewportHeight()
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:220-222
    pub fn DeviceWidth(&self) -> i32 {
        self.data_.device_width
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:224-226
    pub fn DeviceHeight(&self) -> i32 {
        self.data_.device_height
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:228-230
    pub fn DevicePixelRatio(&self) -> f32 {
        self.data_.device_pixel_ratio
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:232-234
    pub fn DeviceSupportsHDR(&self) -> bool {
        self.data_.device_supports_hdr
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:236-238
    pub fn ColorBitsPerComponent(&self) -> i32 {
        self.data_.color_bits_per_component
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:240-242
    pub fn MonochromeBitsPerComponent(&self) -> i32 {
        self.data_.monochrome_bits_per_component
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:244-246
    pub fn InvertedColors(&self) -> bool {
        self.data_.inverted_colors
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:248-250
    pub fn PrimaryPointerType(&self) -> PointerType {
        self.data_.primary_pointer_type
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:252-254
    pub fn AvailablePointerTypes(&self) -> i32 {
        self.data_.available_pointer_types
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:256-258
    pub fn PrimaryHoverType(&self) -> HoverType {
        self.data_.primary_hover_type
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:260-263
    pub fn OutputDeviceUpdateAbilityType(&self) -> OutputDeviceUpdateAbilityType {
        self.data_.output_device_update_ability_type
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:265-267
    pub fn AvailableHoverTypes(&self) -> i32 {
        self.data_.available_hover_types
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:269-271
    pub fn ThreeDEnabled(&self) -> bool {
        self.data_.three_d_enabled
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:273-275
    pub fn StrictMode(&self) -> bool {
        self.data_.strict_mode
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:277-279
    pub fn MediaType(&self) -> String {
        self.data_.media_type.clone()
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:281-283
    pub fn DisplayMode(&self) -> DisplayMode {
        self.data_.display_mode
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:285-287
    pub fn WindowShowState(&self) -> WindowShowState {
        self.data_.window_show_state
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:289-291
    pub fn Resizable(&self) -> bool {
        self.data_.resizable
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:297-299
    pub fn HasValues(&self) -> bool {
        true
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:307-309
    pub fn ColorGamut(&self) -> ColorSpaceGamut {
        self.data_.color_gamut
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:311-314
    pub fn GetPreferredColorScheme(&self) -> PreferredColorScheme {
        self.data_.preferred_color_scheme
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:316-319
    pub fn GetPreferredContrast(&self) -> PreferredContrast {
        self.data_.preferred_contrast
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:321-323
    pub fn PrefersReducedMotion(&self) -> bool {
        self.data_.prefers_reduced_motion
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:325-327
    pub fn PrefersReducedData(&self) -> bool {
        self.data_.prefers_reduced_data
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:329-331
    pub fn PrefersReducedTransparency(&self) -> bool {
        self.data_.prefers_reduced_transparency
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:333-335
    pub fn GetForcedColors(&self) -> ForcedColors {
        self.data_.forced_colors
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:337-339
    pub fn GetNavigationControls(&self) -> NavigationControls {
        self.data_.navigation_controls
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:341-343
    pub fn GetHorizontalViewportSegments(&self) -> i32 {
        self.data_.horizontal_viewport_segments
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:345-347
    pub fn GetVerticalViewportSegments(&self) -> i32 {
        self.data_.vertical_viewport_segments
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:349-351
    pub fn GetDevicePosture(&self) -> DevicePostureType {
        self.data_.device_posture
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:353-355
    pub fn GetScripting(&self) -> Scripting {
        self.data_.scripting
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:293-295
    // The source always returns nullptr; a generic borrowed Document keeps the
    // null contract without depending on DOM's concrete Document type.
    pub fn GetDocument<Document>(&self) -> Option<&Document> {
        None
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.cc:301-305
    pub fn OverrideViewportDimensions(&mut self, width: f64, height: f64) {
        self.data_.viewport_width = width;
        self.data_.viewport_height = height;
    }
    // cpp: third_party/blink/renderer/core/css/media_values_cached.h:160-162
    pub fn GetWritingMode(&self) -> WritingMode {
        WritingMode::kHorizontalTb
    }
}
impl MediaValues for MediaValuesCached {
    // cpp: media_values_cached.cc:293-295.
    fn GetDocument(&self) -> Option<&dom::Document> {
        None
    }
    // cpp: css_length_resolver.cc:24-238; media_values_cached.cc:105-218.
    // Cached values have horizontal writing mode; container units use the small viewport.
    fn ComputeLength(&self, value: f64, unit: crate::css_primitive_value::UnitType) -> f64 {
        use crate::css_primitive_value::UnitType::*;
        let d = &self.data_;
        let factor = match unit {
            kPixels => 1.0,
            kCentimeters => 96.0 / 2.54,
            kMillimeters => 96.0 / 25.4,
            kQuarterMillimeters => 96.0 / 101.6,
            kInches => 96.0,
            kPoints => 96.0 / 72.0,
            kPicas => 16.0,
            kEms | kRems => d.em_size as f64,
            kExs | kRexs => d.ex_size as f64,
            kChs | kRchs => d.ch_size as f64,
            kIcs | kRics => d.ic_size as f64,
            kCaps | kRcaps => d.cap_size as f64,
            kLhs | kRlhs => d.line_height as f64,
            kViewportWidth | kViewportInlineSize => d.viewport_width / 100.0,
            kViewportHeight | kViewportBlockSize => d.viewport_height / 100.0,
            kViewportMin => d.viewport_width.min(d.viewport_height) / 100.0,
            kViewportMax => d.viewport_width.max(d.viewport_height) / 100.0,
            kSmallViewportWidth
            | kSmallViewportInlineSize
            | kContainerWidth
            | kContainerInlineSize => d.small_viewport_width / 100.0,
            kSmallViewportHeight
            | kSmallViewportBlockSize
            | kContainerHeight
            | kContainerBlockSize => d.small_viewport_height / 100.0,
            kSmallViewportMin | kContainerMin => {
                d.small_viewport_width.min(d.small_viewport_height) / 100.0
            }
            kSmallViewportMax | kContainerMax => {
                d.small_viewport_width.max(d.small_viewport_height) / 100.0
            }
            kLargeViewportWidth | kLargeViewportInlineSize => d.large_viewport_width / 100.0,
            kLargeViewportHeight | kLargeViewportBlockSize => d.large_viewport_height / 100.0,
            kLargeViewportMin => d.large_viewport_width.min(d.large_viewport_height) / 100.0,
            kLargeViewportMax => d.large_viewport_width.max(d.large_viewport_height) / 100.0,
            kDynamicViewportWidth | kDynamicViewportInlineSize => d.dynamic_viewport_width / 100.0,
            kDynamicViewportHeight | kDynamicViewportBlockSize => d.dynamic_viewport_height / 100.0,
            kDynamicViewportMin => d.dynamic_viewport_width.min(d.dynamic_viewport_height) / 100.0,
            kDynamicViewportMax => d.dynamic_viewport_width.max(d.dynamic_viewport_height) / 100.0,
            _ => unreachable!("media length required"),
        };
        value * factor
    }

    fn DeviceWidth(&self) -> i32 {
        Self::DeviceWidth(self)
    }
    fn DeviceHeight(&self) -> i32 {
        Self::DeviceHeight(self)
    }
    fn DevicePixelRatio(&self) -> f32 {
        Self::DevicePixelRatio(self)
    }
    fn DeviceSupportsHDR(&self) -> bool {
        Self::DeviceSupportsHDR(self)
    }
    fn ColorBitsPerComponent(&self) -> i32 {
        Self::ColorBitsPerComponent(self)
    }
    fn MonochromeBitsPerComponent(&self) -> i32 {
        Self::MonochromeBitsPerComponent(self)
    }
    fn InvertedColors(&self) -> bool {
        Self::InvertedColors(self)
    }
    fn PrimaryPointerType(&self) -> PointerType {
        Self::PrimaryPointerType(self)
    }
    fn AvailablePointerTypes(&self) -> i32 {
        Self::AvailablePointerTypes(self)
    }
    fn PrimaryHoverType(&self) -> HoverType {
        Self::PrimaryHoverType(self)
    }
    fn OutputDeviceUpdateAbilityType(&self) -> OutputDeviceUpdateAbilityType {
        Self::OutputDeviceUpdateAbilityType(self)
    }
    fn AvailableHoverTypes(&self) -> i32 {
        Self::AvailableHoverTypes(self)
    }
    fn ThreeDEnabled(&self) -> bool {
        Self::ThreeDEnabled(self)
    }
    fn MediaType(&self) -> String {
        Self::MediaType(self)
    }
    fn DisplayMode(&self) -> DisplayMode {
        Self::DisplayMode(self)
    }
    fn WindowShowState(&self) -> WindowShowState {
        Self::WindowShowState(self)
    }
    fn Resizable(&self) -> bool {
        Self::Resizable(self)
    }
    fn StrictMode(&self) -> bool {
        Self::StrictMode(self)
    }
    fn HasValues(&self) -> bool {
        Self::HasValues(self)
    }
    fn ColorGamut(&self) -> ColorSpaceGamut {
        Self::ColorGamut(self)
    }
    fn GetPreferredColorScheme(&self) -> PreferredColorScheme {
        Self::GetPreferredColorScheme(self)
    }
    fn GetPreferredContrast(&self) -> PreferredContrast {
        Self::GetPreferredContrast(self)
    }
    fn PrefersReducedMotion(&self) -> bool {
        Self::PrefersReducedMotion(self)
    }
    fn PrefersReducedData(&self) -> bool {
        Self::PrefersReducedData(self)
    }
    fn PrefersReducedTransparency(&self) -> bool {
        Self::PrefersReducedTransparency(self)
    }
    fn GetForcedColors(&self) -> ForcedColors {
        Self::GetForcedColors(self)
    }
    fn GetNavigationControls(&self) -> NavigationControls {
        Self::GetNavigationControls(self)
    }
    fn GetHorizontalViewportSegments(&self) -> i32 {
        Self::GetHorizontalViewportSegments(self)
    }
    fn GetVerticalViewportSegments(&self) -> i32 {
        Self::GetVerticalViewportSegments(self)
    }
    fn GetDevicePosture(&self) -> DevicePostureType {
        Self::GetDevicePosture(self)
    }
    fn GetScripting(&self) -> Scripting {
        Self::GetScripting(self)
    }
    fn ViewportWidth(&self) -> f64 {
        Self::ViewportWidth(self)
    }
    fn ViewportHeight(&self) -> f64 {
        Self::ViewportHeight(self)
    }
    fn GetWritingMode(&self) -> WritingMode {
        Self::GetWritingMode(self)
    }
}
// Unported: MediaValuesCachedData(Document&) and MediaValuesCached(Document&),
// The no-frame default and copied values are the actual source behavior.

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_defaults_and_viewport_override_are_preserved() {
        let data = MediaValuesCachedData::default();
        assert_eq!(data.color_bits_per_component, 24);
        assert_eq!(data.available_pointer_types, 1);
        assert_eq!(data.available_hover_types, 1);
        assert!(data.media_type.IsNull());
        assert_eq!(data.preferred_color_scheme, PreferredColorScheme::kLight);
        assert_eq!(data.preferred_contrast, PreferredContrast::kNoPreference);
        let mut values = MediaValuesCached::new(&data);
        values.OverrideViewportDimensions(320.5, 180.25);
        assert_eq!(values.Width(), Some(320.5));
        assert_eq!(values.Height(), Some(180.25));
        assert_eq!(values.InlineSize(), Some(320.5));
        assert_eq!(values.BlockSize(), Some(180.25));
        assert_eq!(values.SmallViewportWidth(), 0.0);
        assert_eq!(values.ContainerWidth(), 0.0);
        assert_eq!(values.EmFontSize(1.0), 16.0);
        assert_eq!(values.RemFontSize(1.0), 16.0);
        assert_eq!(values.ExFontSize(1.0), 8.0);
        assert_eq!(values.RexFontSize(1.0), 8.0);
        assert_eq!(
            values.GetPreferredColorScheme(),
            PreferredColorScheme::kLight
        );
        assert!(values.HasValues());
        assert!(values.GetDocument::<()>().is_none());
        let copy = values.Copy();
        values.OverrideViewportDimensions(800.0, 600.0);
        assert_eq!(copy.ViewportWidth(), 320.5);
        assert_eq!(copy.ViewportHeight(), 180.25);
    }
}
