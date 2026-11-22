//! Style-based scrollbar thickness facts that StyleCascade needs for viewport
//! units before it can borrow a retained LayoutView. Layout re-exports these
//! items from their source module paths.

#![allow(non_snake_case)]

use foundation::{gfx, EOverflow, EScrollbarWidth};
use layout_style::style::computed_style_constants::ScrollbarGutter;
use paint_input_types::layout_input_types::ScrollbarThemeMetrics;

use self::mojom::blink::ScrollbarMode;

// cpp: layoutng/internal/scrollbar_mode.h:7-17
pub mod mojom {
    pub mod blink {
        #[repr(i32)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        pub enum ScrollbarMode {
            kAuto = 0,
            kAlwaysOff = 1,
            kAlwaysOn = 2,
        }
    }
}

// cpp: layoutng/internal/scrollbar_theme_metrics.cc:19-26
pub fn ScrollbarThemeThickness(metrics: &ScrollbarThemeMetrics, width: EScrollbarWidth) -> i32 {
    match width {
        EScrollbarWidth::kAuto => metrics.auto_thickness,
        EScrollbarWidth::kThin => metrics.thin_thickness,
        EScrollbarWidth::kNone => 0,
    }
}

// Source: core/paint/paint_layer_scrollable_area.h:708-714.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleBasedScrollbarData {
    pub overflow_x: EOverflow,
    pub overflow_y: EOverflow,
    pub gutter: ScrollbarGutter,
    pub width: EScrollbarWidth,
    pub writing_mode: foundation::WritingMode,
}

// The source method reads these values through LayoutView, Settings and Page.
// StyleCascade runs before it can borrow the retained LayoutView, so its
// package boundary calls this same algorithm with the immutable host facts.
// Source: core/paint/paint_layer_scrollable_area.cc:1698-1738.
pub fn ComputeScrollbarWidthsForViewportUnitsFromInputs(
    scrollbar_properties: StyleBasedScrollbarData,
    visual_viewport_supplies_scrollbars: bool,
    hide_scrollbars: bool,
    theme: &ScrollbarThemeMetrics,
) -> gfx::Size {
    if visual_viewport_supplies_scrollbars
        || hide_scrollbars
        || theme.uses_overlay_scrollbars
        || scrollbar_properties.width == EScrollbarWidth::kNone
    {
        return gfx::Size::default();
    }

    let mode = |overflow: EOverflow| match overflow {
        EOverflow::kScroll => ScrollbarMode::kAlwaysOn,
        EOverflow::kHidden | EOverflow::kClip => ScrollbarMode::kAlwaysOff,
        _ => ScrollbarMode::kAuto,
    };
    let horizontal_mode = mode(scrollbar_properties.overflow_x);
    let vertical_mode = mode(scrollbar_properties.overflow_y);
    let scrollbar_thickness = ScrollbarThemeThickness(theme, scrollbar_properties.width);
    let mut scrollbar_thicknesses = gfx::Size::default();
    if horizontal_mode == ScrollbarMode::kAlwaysOn {
        scrollbar_thicknesses.set_height(scrollbar_thickness);
    }
    if vertical_mode == ScrollbarMode::kAlwaysOn {
        scrollbar_thicknesses.set_width(scrollbar_thickness);
    }
    if scrollbar_properties.gutter.value() & ScrollbarGutter::kScrollbarGutterBothEdges.value() != 0
    {
        if foundation::IsVerticalWritingMode(scrollbar_properties.writing_mode) {
            scrollbar_thicknesses.set_height(scrollbar_thickness * 2);
        } else {
            scrollbar_thicknesses.set_width(scrollbar_thickness * 2);
        }
    } else if scrollbar_properties.gutter.value() & ScrollbarGutter::kScrollbarGutterStable.value()
        != 0
    {
        if foundation::IsVerticalWritingMode(scrollbar_properties.writing_mode) {
            scrollbar_thicknesses.set_height(scrollbar_thickness);
        } else {
            scrollbar_thicknesses.set_width(scrollbar_thickness);
        }
    }
    scrollbar_thicknesses
}
