use layoutng_style::style::computed_style::ComputedStyle;

use super::outline_info::LayoutOutlineInfo;

// cpp: layoutng/internal/outline_geometry.cc:11-15
fn adjust_border_width_by_zoom(border_width: f32, zoom_level: f32) -> f32 {
    let scaled = (border_width * zoom_level).floor();
    if scaled < 1.0 {
        1.0
    } else {
        scaled
    }
}

// cpp: layoutng/internal/outline_geometry.cc:17-25
fn focus_ring_stroke_width(style: &ComputedStyle) -> f32 {
    debug_assert!(style.OutlineStyleIsAuto());
    const WIDTH: f32 = 3.0;
    if style.EffectiveZoom() >= 1.0 {
        adjust_border_width_by_zoom(WIDTH, style.EffectiveZoom())
    } else {
        WIDTH
    }
}

// cpp: layoutng/internal/outline_geometry.cc:27-42
fn focus_ring_offset(style: &ComputedStyle, info: &LayoutOutlineInfo) -> i32 {
    debug_assert!(style.OutlineStyleIsAuto());
    let max_inside_border_width = adjust_border_width_by_zoom(1.0, style.EffectiveZoom());
    let mut offset = info.offset;
    // std::min with the first argument retained on unordered comparisons.
    let widths = [
        style.BorderTopWidth(),
        style.BorderBottomWidth(),
        style.BorderLeftWidth(),
        style.BorderRightWidth(),
    ];
    let mut min_border_width = widths[0];
    for width in widths.into_iter().skip(1) {
        if width < min_border_width {
            min_border_width = width;
        }
    }
    if (min_border_width as f32) >= max_inside_border_width {
        offset = (offset as f32 - max_inside_border_width) as i32;
    }
    offset
}

// cpp: layoutng/internal/outline_geometry.h:6-9
// cpp: layoutng/internal/outline_geometry.cc:45-56
#[allow(non_snake_case)]
pub fn OutlineOutsetExtent(style: &ComputedStyle, info: &LayoutOutlineInfo) -> i32 {
    if !style.HasOutline() {
        return 0;
    }
    if style.OutlineStyleIsAuto() {
        return (focus_ring_offset(style, info) as f32
            + (focus_ring_stroke_width(style) / 3.0).ceil() * 2.0) as i32;
    }
    info.width.saturating_add(info.offset).max(0)
}
