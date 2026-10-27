//! Constraint-space equivalence used to admit retained layout state.
//!
//! Geometry, provider identity, font catalog and image geometry comparisons
//! are invalidation decisions and keep an out-of-line package sampling frame.

#![allow(non_snake_case)]

use super::*;

pub(super) fn SameOptionalArc<T: ?Sized>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn SameImageGeometry(
    a: &crate::internal::layout_input::PaintImage,
    b: &crate::internal::layout_input::PaintImage,
) -> bool {
    a.id == b.id
        && a.paint_image_id == b.paint_image_id
        && a.width == b.width
        && a.height == b.height
        && a.resolution_scale == b.resolution_scale
        && a.orientation == b.orientation
        && a.document_container_key == b.document_container_key
}

pub(super) fn SameImageCatalogGeometry(
    a: &[crate::internal::layout_input::PaintImage],
    b: &[crate::internal::layout_input::PaintImage],
) -> bool {
    a.len() == b.len()
        && a.iter().all(|left| {
            b.iter()
                .find(|right| right.id == left.id)
                .is_some_and(|right| SameImageGeometry(left, right))
        })
}

pub(super) fn SameShapeImageCatalog(
    a: &[crate::internal::layout_input::PaintImage],
    b: &[crate::internal::layout_input::PaintImage],
) -> bool {
    a.len() == b.len()
        && a.iter().all(|left| {
            b.iter()
                .find(|right| right.id == left.id)
                .is_some_and(|right| right.revision == left.revision)
        })
}

/// ConstraintSpace carries both geometry and shared host services. DOM emits
/// the resolved value before every layout pass, so compare it with the
/// resident value before invalidating a retained fragment snapshot. Provider
/// identity is part of the comparison: replacing a service is a real input
/// change even when its current answer happens to match.
pub(super) fn SameConstraintScalars(a: &ConstraintSpace, b: &ConstraintSpace) -> bool {
    a.available_size == b.available_size
        && a.fragmentainer_block_size == b.fragmentainer_block_size
        && a.compatibility_mode == b.compatibility_mode
        && a.printing == b.printing
        && a.vertical_scroll_enforced == b.vertical_scroll_enforced
        && a.scrollbar_theme == b.scrollbar_theme
        && a.minimum_font_physical_size == b.minimum_font_physical_size
        && a.device_pixel_ratio == b.device_pixel_ratio
        && a.preferred_color_scheme == b.preferred_color_scheme
        && a.document_image_container == b.document_image_container
        && a.layout_zoom_factor == b.layout_zoom_factor
        && a.canvas_draw_element_enabled == b.canvas_draw_element_enabled
        && a.prefer_non_composited_scrolling_enabled == b.prefer_non_composited_scrolling_enabled
        && a.viewport == b.viewport
        && a.control_theme == b.control_theme
}

#[inline(never)]
pub(super) fn SameConstraints(a: &ConstraintSpace, b: &ConstraintSpace) -> bool {
    SameConstraintScalars(a, b)
        && SameOptionalArc(&a.hyphenation, &b.hyphenation)
        && SameOptionalArc(&a.phrase_break, &b.phrase_break)
        && SameOptionalArc(&a.text_transform, &b.text_transform)
        && SameOptionalArc(&a.font_backend_factory, &b.font_backend_factory)
        && a.font_queries == b.font_queries
        && a.fonts == b.fonts
        // Decoded pixels, animation metadata and revision are Paint resource
        // inputs. Chromium invalidates layout only when intrinsic image
        // geometry changes; Page publishes content-only mutations separately.
        && SameImageCatalogGeometry(&a.images, &b.images)
}
