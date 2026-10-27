//! Draw-property visibility, update, and pixel-alignment decisions.

use compositor_tree::{
    effect_node::RenderSurfaceReason,
    geometry::{Rect, Transform, Vector2dF},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawPropertyStats {
    pub input_layers: usize,
    pub visible_layers: usize,
    pub render_surfaces: usize,
    pub transform_nodes: usize,
    pub effect_nodes: usize,
    pub clip_nodes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerSkipReason {
    HiddenSubtree,
    NonInvertibleTransform,
    EffectNotDrawn,
    BackFaceVisible,
}

/// Classifies whether an impl-side layer can be omitted from draw-property
/// computation. Backface evaluation remains lazy because it may walk a 3D
/// rendering context.
pub fn ClassifyLayerSkip(
    has_surface_copy_request: bool,
    subtree_hidden: bool,
    cache_render_surface: bool,
    has_subtree_capture: bool,
    transform_is_drawable: bool,
    effect_is_drawn: bool,
    backface_interop_enabled: bool,
    should_check_backface: bool,
    hidden_by_backface: bool,
    backface_visible: impl FnOnce() -> bool,
) -> Option<LayerSkipReason> {
    if has_surface_copy_request {
        return None;
    }
    if subtree_hidden && !cache_render_surface && !has_subtree_capture {
        return Some(LayerSkipReason::HiddenSubtree);
    }
    if !transform_is_drawable {
        return Some(LayerSkipReason::NonInvertibleTransform);
    }
    if !effect_is_drawn {
        return Some(LayerSkipReason::EffectNotDrawn);
    }
    let is_backface_hidden = if backface_interop_enabled {
        should_check_backface && backface_visible()
    } else {
        hidden_by_backface
    };
    is_backface_hidden.then_some(LayerSkipReason::BackFaceVisible)
}

/// Main-thread update-list eligibility with ordered, lazy backface testing.
pub fn LayerNeedsUpdate(
    layer_is_drawn: bool,
    draws_content: impl FnOnce() -> bool,
    bounds_empty: impl FnOnce() -> bool,
    has_reference_filter: impl FnOnce() -> bool,
    should_check_backface: impl FnOnce() -> bool,
    backface_visible: impl FnOnce() -> bool,
) -> bool {
    if !layer_is_drawn || !draws_content() {
        return false;
    }
    if bounds_empty() && !has_reference_filter() {
        return false;
    }
    !(should_check_backface() && backface_visible())
}

pub fn MainLayerShouldBeSkipped(
    has_surface_copy_request: bool,
    transform_is_drawable: bool,
    effect_is_drawn: bool,
) -> bool {
    !has_surface_copy_request && (!transform_is_drawable || !effect_is_drawn)
}

pub fn LayerDrawableContentRect(is_clipped: bool, bounds: Rect, clip: Rect) -> Rect {
    if !is_clipped {
        return bounds;
    }
    let mut rect = bounds;
    rect.Intersect(clip);
    rect
}

pub fn ShouldRemoveRenderSurface(
    is_root: bool,
    content_is_empty: bool,
    has_reference_filter: impl FnOnce() -> bool,
    target_is_surface_list_member: bool,
    copy_output_required: bool,
) -> bool {
    if is_root {
        return false;
    }
    (content_is_empty && !has_reference_filter())
        || (!target_is_surface_list_member && !copy_output_required)
}

pub fn MayContainBackdropBlurFilter(reason: RenderSurfaceReason) -> bool {
    matches!(
        reason,
        RenderSurfaceReason::kMask
            | RenderSurfaceReason::kTrilinearFiltering
            | RenderSurfaceReason::kFilter
            | RenderSurfaceReason::kBackdropFilter
    )
}

pub fn RasterScalesApproximatelyEqual(a: Vector2dF, b: Vector2dF) -> bool {
    const THRESHOLD: f32 = 0.001 / 10000.0;
    let difference = a - b;
    difference.x().abs() <= THRESHOLD && difference.y().abs() <= THRESHOLD
}

pub fn PixelAlignmentOffset<T: Transform>(screen: &T, target: &T) -> Option<Vector2dF> {
    if !screen.IsScaleOrTranslation() || !target.IsScaleOrTranslation() {
        return None;
    }
    if !RasterScalesApproximatelyEqual(screen.To2dScale(), target.To2dScale()) {
        return None;
    }
    let subpixel = |value: f32| value - value.floor();
    let screen_translation = screen.To2dTranslation();
    let screen_x = subpixel(screen_translation.x());
    let screen_y = subpixel(screen_translation.y());
    let draw_translation = target.To2dTranslation();
    let draw_x = subpixel(draw_translation.x());
    let draw_y = subpixel(draw_translation.y());
    const THRESHOLD: f32 = 0.001;
    if (screen_x - draw_x).abs() > THRESHOLD || (screen_y - draw_y).abs() > THRESHOLD {
        return None;
    }
    Some(Vector2dF::new(screen_x, screen_y))
}

#[cfg(test)]
mod tests {
    use super::{ClassifyLayerSkip, LayerNeedsUpdate, LayerSkipReason, ShouldRemoveRenderSurface};
    use std::cell::Cell;

    #[test]
    fn hidden_subtree_short_circuits_backface_walk() {
        let backface_called = Cell::new(false);
        let result = ClassifyLayerSkip(
            false,
            true,
            false,
            false,
            true,
            true,
            true,
            true,
            false,
            || {
                backface_called.set(true);
                true
            },
        );
        assert_eq!(result, Some(LayerSkipReason::HiddenSubtree));
        assert!(!backface_called.get());
    }

    #[test]
    fn copy_request_keeps_hidden_surface() {
        let result = ClassifyLayerSkip(
            true,
            true,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            || true,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn empty_unfiltered_layer_does_not_evaluate_backface() {
        let backface_called = Cell::new(false);
        assert!(!LayerNeedsUpdate(
            false,
            || true,
            || true,
            || false,
            || true,
            || {
                backface_called.set(true);
                true
            }
        ));
        assert!(!backface_called.get());
    }

    #[test]
    fn empty_non_root_surface_is_removed_unless_a_reference_filter_needs_it() {
        assert!(ShouldRemoveRenderSurface(
            false,
            true,
            || false,
            true,
            false
        ));
        assert!(!ShouldRemoveRenderSurface(
            false,
            true,
            || true,
            true,
            false
        ));
        assert!(!ShouldRemoveRenderSurface(
            true,
            true,
            || unreachable!(),
            false,
            false
        ));
    }
}
