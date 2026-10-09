// Copyright 2014 The Chromium Authors. BSD-style license; see Chromium LICENSE.
use super::color_space_gamut::ColorSpaceGamut;
use super::container_state::*;
use super::device_posture_provider::DevicePostureType;
use super::display_mode::DisplayMode;
use super::forced_colors::ForcedColors;
use super::navigation_controls::NavigationControls;
use super::preferred_color_scheme::PreferredColorScheme;
use super::preferred_contrast::PreferredContrast;
use super::scripting::Scripting;
use super::web_preferences::{HoverType, OutputDeviceUpdateAbilityType, PointerType};
use super::window_show_state::WindowShowState;
use foundation::{CSSValueID, IsHorizontalWritingMode, String, WritingMode};
// cpp: third_party/blink/renderer/core/css/media_values.cc:36-45
pub fn CSSValueIDToForcedColors(id: CSSValueID) -> ForcedColors {
    match id {
        CSSValueID::kActive => ForcedColors::kActive,
        CSSValueID::kNone => ForcedColors::kNone,
        _ => unreachable!("invalid forced-colors identifier"),
    }
}
// cpp: third_party/blink/renderer/core/css/media_values.cc:47-57
pub fn CSSValueIDToPreferredColorScheme(id: CSSValueID) -> PreferredColorScheme {
    match id {
        CSSValueID::kLight => PreferredColorScheme::kLight,
        CSSValueID::kDark => PreferredColorScheme::kDark,
        _ => unreachable!("invalid preferred color scheme identifier"),
    }
}
// cpp: third_party/blink/renderer/core/css/media_values.cc:59-72
pub fn CSSValueIDToPreferredContrast(id: CSSValueID) -> PreferredContrast {
    match id {
        CSSValueID::kMore => PreferredContrast::kMore,
        CSSValueID::kLess => PreferredContrast::kLess,
        CSSValueID::kNoPreference => PreferredContrast::kNoPreference,
        CSSValueID::kCustom => PreferredContrast::kCustom,
        _ => unreachable!("invalid preferred contrast identifier"),
    }
}
// cpp: third_party/blink/renderer/core/css/media_values.h:64-82,84-97,204
// Pure virtual reads stay required Rust trait methods. Frame factories,
// Document/Element APIs and full CSSLengthResolver dispatch remain unported.
pub trait MediaValues {
    // cpp: media_values.h:87. Cached values have no owning document;
    // dynamic/container values override this with their actual owner.
    fn GetDocument(&self) -> Option<&dom::Document>;
    fn DeviceWidth(&self) -> i32;
    fn DeviceHeight(&self) -> i32;
    fn DevicePixelRatio(&self) -> f32;
    fn DeviceSupportsHDR(&self) -> bool;
    fn ColorBitsPerComponent(&self) -> i32;
    fn MonochromeBitsPerComponent(&self) -> i32;
    fn InvertedColors(&self) -> bool;
    fn PrimaryPointerType(&self) -> PointerType;
    fn AvailablePointerTypes(&self) -> i32;
    fn PrimaryHoverType(&self) -> HoverType;
    fn OutputDeviceUpdateAbilityType(&self) -> OutputDeviceUpdateAbilityType;
    fn AvailableHoverTypes(&self) -> i32;
    fn ThreeDEnabled(&self) -> bool;
    fn MediaType(&self) -> String;
    fn DisplayMode(&self) -> DisplayMode;
    fn WindowShowState(&self) -> WindowShowState;
    fn Resizable(&self) -> bool;
    fn StrictMode(&self) -> bool;
    fn HasValues(&self) -> bool;
    fn ColorGamut(&self) -> ColorSpaceGamut;
    fn GetPreferredColorScheme(&self) -> PreferredColorScheme;
    fn GetPreferredContrast(&self) -> PreferredContrast;
    fn PrefersReducedMotion(&self) -> bool;
    fn PrefersReducedData(&self) -> bool;
    fn PrefersReducedTransparency(&self) -> bool;
    fn GetForcedColors(&self) -> ForcedColors;
    fn GetNavigationControls(&self) -> NavigationControls;
    fn GetHorizontalViewportSegments(&self) -> i32;
    fn GetVerticalViewportSegments(&self) -> i32;
    fn GetDevicePosture(&self) -> DevicePostureType;
    fn GetScripting(&self) -> Scripting;
    fn ViewportWidth(&self) -> f64;
    fn ViewportHeight(&self) -> f64;
    fn GetWritingMode(&self) -> WritingMode;
    // Typed CSSLengthResolver boundary; implementations resolve media lengths.
    fn ComputeLength(&self, value: f64, unit: crate::css_primitive_value::UnitType) -> f64;

    // Availability at the container owner boundary. Non-container values keep
    // Chromium's defined empty-state reads; partial container snapshots must
    // override these before a typed evaluator reads a state feature.
    fn HasScrollState(&self) -> bool {
        true
    }
    fn HasAnchoredState(&self) -> bool {
        true
    }

    // cpp: third_party/blink/renderer/core/css/media_values.h:65-68
    fn Width(&self) -> Option<f64> {
        Some(self.ViewportWidth())
    }
    fn Height(&self) -> Option<f64> {
        Some(self.ViewportHeight())
    }
    // cpp: third_party/blink/renderer/core/css/media_values.cc:74-86
    fn InlineSize(&self) -> Option<f64> {
        if IsHorizontalWritingMode(self.GetWritingMode()) {
            self.Width()
        } else {
            self.Height()
        }
    }
    fn BlockSize(&self) -> Option<f64> {
        if IsHorizontalWritingMode(self.GetWritingMode()) {
            self.Height()
        } else {
            self.Width()
        }
    }
    // cpp: third_party/blink/renderer/core/css/media_values.h:98-188
    fn StuckHorizontal(&self) -> ContainerStuckPhysical {
        ContainerStuckPhysical::kNo
    }
    fn StuckVertical(&self) -> ContainerStuckPhysical {
        ContainerStuckPhysical::kNo
    }
    fn StuckInline(&self) -> ContainerStuckLogical {
        ContainerStuckLogical::kNo
    }
    fn StuckBlock(&self) -> ContainerStuckLogical {
        ContainerStuckLogical::kNo
    }
    fn Stuck(&self) -> bool {
        self.StuckHorizontal() != ContainerStuckPhysical::kNo
            || self.StuckVertical() != ContainerStuckPhysical::kNo
    }
    fn SnappedFlags(&self) -> ContainerSnappedFlags {
        ContainerSnapped::kNone as u32
    }
    fn SnappedX(&self) -> bool {
        self.SnappedFlags() & ContainerSnapped::kX as u32 != 0
    }
    fn SnappedY(&self) -> bool {
        self.SnappedFlags() & ContainerSnapped::kY as u32 != 0
    }
    fn Snapped(&self) -> bool {
        self.SnappedFlags() != ContainerSnapped::kNone as u32
    }
    // cpp: third_party/blink/renderer/core/css/media_values.cc:88-100
    fn SnappedBlock(&self) -> bool {
        if IsHorizontalWritingMode(self.GetWritingMode()) {
            self.SnappedY()
        } else {
            self.SnappedX()
        }
    }
    fn SnappedInline(&self) -> bool {
        if IsHorizontalWritingMode(self.GetWritingMode()) {
            self.SnappedX()
        } else {
            self.SnappedY()
        }
    }
    fn ScrollableHorizontal(&self) -> ContainerScrollableFlags {
        ContainerScrollable::kNone as u32
    }
    fn ScrollableVertical(&self) -> ContainerScrollableFlags {
        ContainerScrollable::kNone as u32
    }
    fn ScrollableInline(&self) -> ContainerScrollableFlags {
        ContainerScrollable::kNone as u32
    }
    fn ScrollableBlock(&self) -> ContainerScrollableFlags {
        ContainerScrollable::kNone as u32
    }
    fn Scrollable(&self) -> bool {
        self.ScrollableHorizontal() != ContainerScrollable::kNone as u32
            || self.ScrollableVertical() != ContainerScrollable::kNone as u32
    }
    fn ScrolledHorizontal(&self) -> ContainerScrolled {
        ContainerScrolled::kNone
    }
    fn ScrolledVertical(&self) -> ContainerScrolled {
        ContainerScrolled::kNone
    }
    fn ScrolledInline(&self) -> ContainerScrolled {
        ContainerScrolled::kNone
    }
    fn ScrolledBlock(&self) -> ContainerScrolled {
        ContainerScrolled::kNone
    }
    fn Scrolled(&self) -> bool {
        self.ScrolledHorizontal() != ContainerScrolled::kNone
            || self.ScrolledVertical() != ContainerScrolled::kNone
    }
}
