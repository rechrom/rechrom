//! Paint-backed inline-fragment admission policy.
//!
//! The policy determines fragment materialization from typed or source style
//! and therefore belongs to `layout_fragment`, even when called during
//! object construction or invalidation preparation.

#![allow(non_snake_case)]

use super::*;

fn ProjectedStyleRequiresPaintBackedInlineFragment(style: &ComputedStyle) -> bool {
    let paint = &style.paint;
    paint.background_color.alpha > 0.0
        || !paint.background_images.is_empty()
        || paint.border_image.is_some()
        || paint.mask_box_image.is_some()
        || !paint.box_shadows.is_empty()
        || (paint.outline_width > 0.0 && paint.outline_style != BorderLineStyle::kNone)
        || paint.opacity != 1.0
        || !paint.visible
        || paint.transform.is_some()
        || paint.clip_path.is_some()
        || !paint.filters.is_empty()
        || !paint.backdrop_filters.is_empty()
        || paint.backdrop_filter_has_reference
        || !paint.mask_images.is_empty()
        || paint.blend_mode != PaintBlendMode::kNormal
        || paint.isolate_blending
}

// cpp: core/layout/layout_inline.cc:183-230
#[inline(never)]
pub(crate) fn ComputedStyleRequiresPaintBackedInlineFragment(style: &StyleComputedStyle) -> bool {
    style.HasBoxDecorationBackground()
        || style.MayHavePadding()
        || style.MayHaveMargin()
        || style.HasOutline()
        || style.HasMask()
}

#[inline(never)]
pub(crate) fn RequiresPaintBackedInlineFragment(source: &NodeConstructionData) -> bool {
    let standard = unsafe { source.computed_style.as_ref() }.map_or_else(
        || ProjectedStyleRequiresPaintBackedInlineFragment(&source.style),
        ComputedStyleRequiresPaintBackedInlineFragment,
    );
    let first_line = source.first_line_style.as_ref().is_some_and(|style| {
        unsafe { source.first_line_computed_style.as_ref() }.map_or_else(
            || ProjectedStyleRequiresPaintBackedInlineFragment(style),
            ComputedStyleRequiresPaintBackedInlineFragment,
        )
    });
    standard || first_line
}
