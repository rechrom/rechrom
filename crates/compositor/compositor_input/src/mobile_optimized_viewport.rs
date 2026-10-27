//! Mobile viewport classification used by compositor input policy.

use compositor_tree::input_geometry::SizeF;

const kMobileViewportWidthEpsilon: f32 = 0.15;

pub fn IsMobileOptimized(
    min_page_scale_factor: f32,
    max_page_scale_factor: f32,
    current_page_scale_factor: f32,
    scrollable_viewport_size: SizeF,
    scrollable_size: SizeF,
    viewport_meta_mobile_optimized: bool,
) -> bool {
    let has_fixed_page_scale = min_page_scale_factor == max_page_scale_factor;
    let window_width_dip = current_page_scale_factor * scrollable_viewport_size.width();
    let content_width_css = scrollable_size.width();
    let has_mobile_viewport = content_width_css <= window_width_dip + kMobileViewportWidthEpsilon;

    has_mobile_viewport || has_fixed_page_scale || viewport_meta_mobile_optimized
}
