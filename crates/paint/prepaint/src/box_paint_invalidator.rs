//! Layout-box paint invalidation during PrePaint.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/box_paint_invalidator.{h,cc}`.

use foundation::{
    AppearanceValue, BorderLineStyle, IsFullPaintInvalidationReason,
    IsLayoutFullPaintInvalidationReason, PaintInvalidationReason,
};
use layoutng_assembly::fragment_tree::{FragmentPaintData, FragmentPaintRect, PreviousBoxGeometry};
use layoutng_assembly::internal::layout_input::{Offset, Size};
use layoutng_assembly::internal::paint_input::{
    BackgroundBox, BackgroundImageLayer, BackgroundRepeat, BackgroundRepeatRule, BackgroundSizeMode,
};

use crate::paint_invalidator::PaintInvalidatorContext;

pub struct BoxPaintInvalidator;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackgroundInvalidationType {
    #[default]
    None,
    Incremental,
    Full,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackgroundInvalidation {
    pub kind: BackgroundInvalidationType,
    pub invalidate_both_spaces: bool,
}

impl BoxPaintInvalidator {
    /// Source: `box_paint_invalidator.cc:321-374`, plus the directly
    /// representable LayoutView branches at lines 253-319.
    pub fn ComputeBackgroundInvalidation(
        fragment: &FragmentPaintData,
        previous: Option<&FragmentPaintData>,
        context: PaintInvalidatorContext,
    ) -> BackgroundInvalidation {
        let mut result = BackgroundInvalidation::default();
        let previous_geometry = PreviousGeometry(fragment, previous);
        let invalidation = fragment.paint.pre_paint_invalidation;
        if invalidation.background_needs_full_paint_invalidation
            || context.HasSubtreeFullInvalidation()
            || ShouldInvalidateGapDecorations(fragment, previous)
        {
            result.kind = BackgroundInvalidationType::Full;
            result.invalidate_both_spaces = true;
        } else if HasEffectiveBackground(fragment) {
            let old_size = previous_geometry.map(|old| old.size).unwrap_or_default();
            let new_size = StitchedSize(fragment);
            if fragment.paint.background_any_default_attachment_image
                && ShouldFullyInvalidateFillLayersOnSizeChange(fragment, old_size, new_size)
            {
                result.kind = BackgroundInvalidationType::Full;
            } else if previous_geometry.is_some_and(|old| {
                BackgroundUsesContentBox(fragment)
                    && (old.content_box.offset, old.content_box.size)
                        != PhysicalContentBoxSignature(fragment)
            }) {
                result.kind = BackgroundInvalidationType::Full;
            } else if BackgroundGeometryDependsOnScrollableOverflowRect(fragment)
                || BackgroundPaintsInContentsSpace(fragment)
            {
                if let (Some(old), Some(new)) = (
                    previous_geometry.map(|old| old.scrollable_overflow),
                    fragment.paint.scrollable_overflow,
                ) {
                    if ShouldFullyInvalidateBackgroundOnScrollableOverflowChange(fragment, old, new)
                    {
                        result.kind = BackgroundInvalidationType::Full;
                    } else if old != new {
                        result.kind = if old.offset == new.offset {
                            BackgroundInvalidationType::Incremental
                        } else {
                            BackgroundInvalidationType::Full
                        };
                    }
                }
            }
        }

        if fragment.paint.IsLayoutView() {
            let view_kind = ComputeViewBackgroundInvalidation(fragment, previous, context);
            result.kind = result.kind.max(view_kind);
        }
        result
    }

    pub fn ViewBackgroundGeometryChanged(
        fragment: &FragmentPaintData,
        previous: Option<&FragmentPaintData>,
    ) -> bool {
        match (
            fragment
                .paint
                .previous_view_background_rect
                .or_else(|| previous.and_then(|old| old.paint.view_background_rect)),
            fragment.paint.view_background_rect,
        ) {
            (Some(old), Some(new)) => old != new,
            _ => false,
        }
    }

    /// Source: `box_paint_invalidator.cc:376-414`. The scrolling background
    /// uses PaintLayerScrollableArea's independent DisplayItemClient.
    pub fn InvalidateBackground(
        fragment: &mut FragmentPaintData,
        invalidation: BackgroundInvalidation,
        mut object_reason: PaintInvalidationReason,
    ) -> PaintInvalidationReason {
        let paints_contents = BackgroundPaintsInContentsSpace(fragment);
        let paints_border_box = BackgroundPaintsInBorderBoxSpace(fragment);
        if let Some(scroll) = &mut fragment.paint.scroll_container {
            if invalidation.invalidate_both_spaces
                || (paints_contents && invalidation.kind != BackgroundInvalidationType::None)
            {
                let reason = if invalidation.kind == BackgroundInvalidationType::Full {
                    PaintInvalidationReason::kBackground
                } else {
                    PaintInvalidationReason::kIncremental
                };
                let current = scroll
                    .scrolling_background_client_invalidation_reason
                    .unwrap_or(PaintInvalidationReason::kNone);
                if reason > current {
                    scroll.scrolling_background_client_invalidation_reason = Some(reason);
                }
            }
        }
        if invalidation.invalidate_both_spaces
            || (paints_border_box && invalidation.kind == BackgroundInvalidationType::Full)
        {
            object_reason = object_reason.max(PaintInvalidationReason::kBackground);
        }
        object_reason
    }

    /// Source: `box_paint_invalidator.cc:130-211`. The comparison consumes the
    /// retained fragment as Chromium consumes LayoutBox's saved geometry.
    pub fn ComputePaintInvalidationReason(
        fragment: &FragmentPaintData,
        previous: Option<&FragmentPaintData>,
        old_paint_offset: Option<Offset>,
        new_paint_offset: Offset,
        _context: PaintInvalidatorContext,
        reason: PaintInvalidationReason,
    ) -> PaintInvalidationReason {
        if reason == PaintInvalidationReason::kNone {
            return reason;
        }
        if IsLayoutFullPaintInvalidationReason(reason) {
            return reason;
        }
        if IsFullPaintInvalidationReason(reason)
            && !fragment
                .paint
                .pre_paint_invalidation
                .should_check_layout_for_paint_invalidation
        {
            return reason;
        }

        let Some(previous) = PreviousGeometry(fragment, previous) else {
            return reason;
        };

        if MaskUsesContentBox(fragment)
            && (previous.content_box.offset, previous.content_box.size)
                != PhysicalContentBoxSignature(fragment)
        {
            return PaintInvalidationReason::kLayout;
        }

        if fragment
            .paint
            .replaced_content
            .as_ref()
            .is_some_and(|current| {
                previous.replaced_content
                    != Some(FragmentPaintRect {
                        offset: current.offset,
                        size: current.size,
                    })
            })
        {
            return PaintInvalidationReason::kLayout;
        }

        let previous_size = previous.size;
        let stitched_size = StitchedSize(fragment);
        if previous_size == stitched_size
            && previous.self_visual_overflow == SelfVisualOverflow(fragment)
        {
            return if IsFullPaintInvalidationReason(reason) {
                reason
            } else {
                PaintInvalidationReason::kNone
            };
        }

        if previous.self_visual_overflow.size != previous_size
            || SelfVisualOverflowSize(fragment) != stitched_size
        {
            return PaintInvalidationReason::kLayout;
        }

        if old_paint_offset.is_some_and(OffsetHasFraction)
            || OffsetHasFraction(new_paint_offset)
            || SizeHasFraction(previous_size)
            || SizeHasFraction(stitched_size)
        {
            return PaintInvalidationReason::kLayout;
        }

        if fragment.paint.border.right != 0.0 || fragment.paint.border.bottom != 0.0 {
            return PaintInvalidationReason::kLayout;
        }

        let style = &fragment.paint.style;
        let has_visual_overflowing_effect = style.box_shadow_count() != 0
            || (style.outline_style() != BorderLineStyle::kNone && style.outline_width() != 0.0);
        let has_effective_appearance = fragment
            .paint
            .form_control
            .as_ref()
            .is_some_and(|control| control.appearance != AppearanceValue::kNone);
        if has_visual_overflowing_effect
            || has_effective_appearance
            || style.has_filters()
            || style.has_mask_images()
            || style.mask_box_image.is_some()
            || style.clip_path.is_some()
        {
            return PaintInvalidationReason::kLayout;
        }

        if style
            .border_radii
            .as_ref()
            .is_some_and(|radii| radii.HasRadius())
            || style.border_radius != 0.0
            || style
                .border_image
                .as_ref()
                .is_some_and(|image| image.resource_id != 0 || image.shader.is_some())
        {
            return PaintInvalidationReason::kLayout;
        }

        if fragment.paint.frame_set.is_some() {
            return PaintInvalidationReason::kLayout;
        }

        reason
    }
}

pub(crate) fn PreviousGeometry(
    fragment: &FragmentPaintData,
    previous: Option<&FragmentPaintData>,
) -> Option<PreviousBoxGeometry> {
    fragment.paint.previous_box_geometry.or_else(|| {
        // Standalone immutable fixtures have no native LayoutBox owner.
        previous.map(|old| {
            let (offset, size) = PhysicalContentBoxSignature(old);
            PreviousBoxGeometry {
                size: StitchedSize(old),
                content_box: FragmentPaintRect { offset, size },
                self_visual_overflow: SelfVisualOverflow(old),
                scrollable_overflow: old.paint.scrollable_overflow.unwrap_or(FragmentPaintRect {
                    offset: Offset::default(),
                    size: StitchedSize(old),
                }),
                replaced_content: old.paint.replaced_content.as_ref().map(|content| {
                    FragmentPaintRect {
                        offset: content.offset,
                        size: content.size,
                    }
                }),
            }
        })
    })
}

fn SelfVisualOverflow(fragment: &FragmentPaintData) -> FragmentPaintRect {
    fragment
        .paint
        .box_self_visual_overflow
        .unwrap_or(FragmentPaintRect {
            offset: Offset::default(),
            size: StitchedSize(fragment),
        })
}

fn ComputeViewBackgroundInvalidation(
    fragment: &FragmentPaintData,
    previous: Option<&FragmentPaintData>,
    context: PaintInvalidatorContext,
) -> BackgroundInvalidationType {
    let (old_rect, new_rect) = match (
        fragment
            .paint
            .previous_view_background_rect
            .or_else(|| previous.and_then(|old| old.paint.view_background_rect)),
        fragment.paint.view_background_rect,
    ) {
        (Some(old), Some(new)) => (old, new),
        _ => return BackgroundInvalidationType::None,
    };
    let location_changed = old_rect.offset != new_rect.offset;
    let size_changed = old_rect.size != new_rect.size;
    if location_changed
        || fragment
            .paint
            .pre_paint_invalidation
            .background_needs_full_paint_invalidation
        || context.HasSubtreeFullInvalidation()
    {
        return BackgroundInvalidationType::Full;
    }

    if let Some(new) = fragment.paint.view_background_invalidation.as_ref() {
        let previous_view =
            previous.and_then(|old| old.paint.view_background_invalidation.as_ref());
        let old_geometry = new.previous_document_element_geometry.or_else(|| {
            previous_view.map(|old| PreviousBoxGeometry {
                size: old.document_element_size,
                content_box: old.document_element_content_box,
                scrollable_overflow: old.document_element_scrollable_overflow.unwrap_or_default(),
                ..PreviousBoxGeometry::default()
            })
        });
        let Some(old_geometry) = old_geometry else {
            return if size_changed {
                BackgroundInvalidationType::Incremental
            } else {
                BackgroundInvalidationType::None
            };
        };
        let layers = new.style.background_images();
        if ShouldFullyInvalidateBackgroundLayersOnSizeChange(
            new.background_fill_layer_count,
            &layers,
            old_geometry.size,
            new.document_element_size,
        ) {
            return BackgroundInvalidationType::Full;
        }
        if let (Some(old_overflow), Some(new_overflow)) = (
            Some(old_geometry.scrollable_overflow),
            new.document_element_scrollable_overflow,
        ) {
            if old_overflow != new_overflow && new.background_any_local_attachment_image {
                if old_overflow.offset != new_overflow.offset
                    || ShouldFullyInvalidateBackgroundLayersOnSizeChange(
                        new.background_fill_layer_count,
                        &layers,
                        old_overflow.size,
                        new_overflow.size,
                    )
                {
                    return BackgroundInvalidationType::Full;
                }
            }
        }
        if new.background_any_layer_uses_content_box
            && old_geometry.content_box != new.document_element_content_box
        {
            return BackgroundInvalidationType::Full;
        }
        if new.document_element_has_transform {
            return BackgroundInvalidationType::Full;
        }
    }

    if size_changed {
        BackgroundInvalidationType::Incremental
    } else {
        BackgroundInvalidationType::None
    }
}

fn HasEffectiveBackground(fragment: &FragmentPaintData) -> bool {
    fragment.paint.IsLayoutView()
        || (fragment.paint.has_background && !fragment.paint.background_transfers_to_view)
}

fn BackgroundUsesContentBox(fragment: &FragmentPaintData) -> bool {
    fragment.paint.background_any_layer_uses_content_box
}

fn BackgroundPaintsInContentsSpace(fragment: &FragmentPaintData) -> bool {
    HasEffectiveBackground(fragment) && fragment.paint.background_paint_location.PaintsContents()
}

fn BackgroundPaintsInBorderBoxSpace(fragment: &FragmentPaintData) -> bool {
    HasEffectiveBackground(fragment) && fragment.paint.background_paint_location.PaintsBorderBox()
}

fn BackgroundGeometryDependsOnScrollableOverflowRect(fragment: &FragmentPaintData) -> bool {
    HasEffectiveBackground(fragment) && fragment.paint.background_any_local_attachment_image
}

/// Source: `box_paint_invalidator.cc:467-510,530-534`. The immutable Paint
/// boundary retains the resolved rule segments with the previous fragment, so
/// comparing those segments is equivalent to comparing LayoutBox's saved
/// per-fragment GapGeometry for the purpose of deciding whether to re-record.
fn ShouldInvalidateGapDecorations(
    fragment: &FragmentPaintData,
    previous: Option<&FragmentPaintData>,
) -> bool {
    let Some(previous_rules) = previous.and_then(|old| old.paint.gap_rules.as_ref()) else {
        return false;
    };
    fragment.paint.gap_rules.as_ref() != Some(previous_rules)
}

fn RepeatRule(layer: &BackgroundImageLayer, horizontal: bool) -> BackgroundRepeatRule {
    let explicit = if horizontal {
        layer.repeat_rule_x
    } else {
        layer.repeat_rule_y
    };
    explicit.unwrap_or_else(|| match (layer.repeat, horizontal) {
        (BackgroundRepeat::kRepeat, _) => BackgroundRepeatRule::kRepeat,
        (BackgroundRepeat::kRepeatX, true) | (BackgroundRepeat::kRepeatY, false) => {
            BackgroundRepeatRule::kRepeat
        }
        _ => BackgroundRepeatRule::kNoRepeat,
    })
}

fn ShouldFullyInvalidateFillLayersOnWidthChange(
    layer_count: usize,
    layer: Option<&BackgroundImageLayer>,
) -> bool {
    if layer_count > 1 {
        return true;
    }
    let Some(layer) = layer.filter(|layer| layer.can_render) else {
        return false;
    };
    if !matches!(
        RepeatRule(layer, true),
        BackgroundRepeatRule::kRepeat | BackgroundRepeatRule::kNoRepeat
    ) || layer.position_x_has_nonzero_percent
        || layer.origin_x_from_right
        || matches!(
            layer.size_mode,
            BackgroundSizeMode::kContain | BackgroundSizeMode::kCover
        )
        || layer.width_percentage.is_some_and(|value| value != 0.0)
    {
        return true;
    }
    let width_is_auto = layer.size_mode == BackgroundSizeMode::kAuto
        || (layer.size_mode == BackgroundSizeMode::kExplicit
            && layer.width.is_none()
            && layer.width_percentage.is_none()
            && layer.width_calculation.is_none());
    width_is_auto && !layer.has_intrinsic_size
}

fn ShouldFullyInvalidateFillLayersOnHeightChange(
    layer_count: usize,
    layer: Option<&BackgroundImageLayer>,
) -> bool {
    if layer_count > 1 {
        return true;
    }
    let Some(layer) = layer.filter(|layer| layer.can_render) else {
        return false;
    };
    if !matches!(
        RepeatRule(layer, false),
        BackgroundRepeatRule::kRepeat | BackgroundRepeatRule::kNoRepeat
    ) || layer.position_y_has_nonzero_percent
        || layer.origin_y_from_bottom
        || matches!(
            layer.size_mode,
            BackgroundSizeMode::kContain | BackgroundSizeMode::kCover
        )
        || layer.height_percentage.is_some_and(|value| value != 0.0)
    {
        return true;
    }
    let height_is_auto = layer.size_mode == BackgroundSizeMode::kAuto
        || (layer.size_mode == BackgroundSizeMode::kExplicit
            && layer.height.is_none()
            && layer.height_percentage.is_none()
            && layer.height_calculation.is_none());
    height_is_auto && !layer.has_intrinsic_size
}

fn ShouldFullyInvalidateFillLayersOnSizeChange(
    fragment: &FragmentPaintData,
    old_size: Size,
    new_size: Size,
) -> bool {
    let layers = fragment.paint.style.background_images();
    ShouldFullyInvalidateBackgroundLayersOnSizeChange(
        fragment.paint.background_fill_layer_count,
        &layers,
        old_size,
        new_size,
    )
}

fn ShouldFullyInvalidateBackgroundLayersOnSizeChange(
    layer_count: usize,
    layers: &[BackgroundImageLayer],
    old_size: Size,
    new_size: Size,
) -> bool {
    let layer = layers.first();
    (old_size.width != new_size.width
        && ShouldFullyInvalidateFillLayersOnWidthChange(layer_count, layer))
        || (old_size.height != new_size.height
            && ShouldFullyInvalidateFillLayersOnHeightChange(layer_count, layer))
}

fn ShouldFullyInvalidateBackgroundOnScrollableOverflowChange(
    fragment: &FragmentPaintData,
    old: layoutng_assembly::fragment_tree::FragmentPaintRect,
    new: layoutng_assembly::fragment_tree::FragmentPaintRect,
) -> bool {
    if old == new || !BackgroundGeometryDependsOnScrollableOverflowRect(fragment) {
        return false;
    }
    old.offset != new.offset
        || ShouldFullyInvalidateFillLayersOnSizeChange(fragment, old.size, new.size)
}

fn MaskUsesContentBox(fragment: &FragmentPaintData) -> bool {
    fragment.paint.style.mask_images().iter().any(|layer| {
        layer.image.origin == BackgroundBox::kContentBox
            || layer.image.clip == BackgroundBox::kContentBox
    })
}

fn PhysicalContentBoxSignature(fragment: &FragmentPaintData) -> (Offset, Size) {
    (
        Offset {
            x: fragment.paint.border.left + fragment.paint.padding.left,
            y: fragment.paint.border.top + fragment.paint.padding.top,
        },
        fragment.content_size,
    )
}

fn StitchedSize(fragment: &FragmentPaintData) -> Size {
    fragment
        .paint
        .stitched_decoration
        .as_ref()
        .map_or(fragment.size, |decoration| decoration.size)
}

fn SelfVisualOverflowSize(fragment: &FragmentPaintData) -> Size {
    fragment
        .paint
        .box_self_visual_overflow
        .map_or_else(|| StitchedSize(fragment), |rect| rect.size)
}

fn HasFraction(value: f64) -> bool {
    value.fract() != 0.0
}

fn OffsetHasFraction(offset: Offset) -> bool {
    HasFraction(offset.x) || HasFraction(offset.y)
}

fn SizeHasFraction(size: Size) -> bool {
    HasFraction(size.width) || HasFraction(size.height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::fragment_tree::{
        BackgroundPaintLocation, CanvasBackgroundPaintData, FragmentPaintRect, GapRule,
        GapRulePaintData, ScrollContainerPaintData,
    };
    use layoutng_assembly::internal::layout_input::Edges;
    use layoutng_assembly::internal::paint_input::BackgroundAttachment;
    use std::sync::Arc;

    fn incremental_fragment() -> FragmentPaintData {
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 100.0,
            height: 50.0,
        };
        fragment.content_size = fragment.size;
        fragment
            .paint
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        fragment
    }

    #[test]
    fn unchanged_box_cancels_incremental_invalidation() {
        let previous = incremental_fragment();
        let current = previous.clone();
        assert_eq!(
            BoxPaintInvalidator::ComputePaintInvalidationReason(
                &current,
                Some(&previous),
                Some(Offset::default()),
                Offset::default(),
                PaintInvalidatorContext::default(),
                PaintInvalidationReason::kIncremental,
            ),
            PaintInvalidationReason::kNone
        );
    }

    #[test]
    fn right_border_upgrades_size_change_to_layout_invalidation() {
        let previous = incremental_fragment();
        let mut current = previous.clone();
        current.size.width = 110.0;
        current.content_size.width = 110.0;
        current.paint.border.right = 1.0;
        current.paint.previous_box_geometry = PreviousGeometry(&previous, Some(&previous));
        assert_eq!(
            BoxPaintInvalidator::ComputePaintInvalidationReason(
                &current,
                // Production retains this same sidecar through Layout updates.
                // Only the native previous-box values still describe old size.
                Some(&current),
                Some(Offset::default()),
                Offset::default(),
                PaintInvalidatorContext::default(),
                PaintInvalidationReason::kIncremental,
            ),
            PaintInvalidationReason::kLayout
        );
    }

    fn local_background_fragment(width: f64) -> FragmentPaintData {
        let mut fragment = incremental_fragment();
        fragment.paint.has_background = true;
        fragment.paint.background_fill_layer_count = 1;
        fragment.paint.background_any_local_attachment_image = true;
        fragment.paint.background_paint_location = BackgroundPaintLocation::Contents;
        fragment.paint.scrollable_overflow = Some(FragmentPaintRect {
            offset: Offset::default(),
            size: Size {
                width,
                height: 50.0,
            },
        });
        let mut layer = BackgroundImageLayer::default();
        layer.can_render = true;
        layer.has_intrinsic_size = true;
        layer.attachment = BackgroundAttachment::kLocal;
        fragment.paint.style.background_images.push(layer);
        fragment
    }

    #[test]
    fn local_background_overflow_growth_is_incremental_when_tile_geometry_is_stable() {
        let previous = local_background_fragment(100.0);
        let current = local_background_fragment(120.0);
        assert_eq!(
            BoxPaintInvalidator::ComputeBackgroundInvalidation(
                &current,
                Some(&previous),
                PaintInvalidatorContext::default(),
            ),
            BackgroundInvalidation {
                kind: BackgroundInvalidationType::Incremental,
                invalidate_both_spaces: false,
            }
        );
    }

    #[test]
    fn percentage_position_makes_local_background_overflow_growth_full() {
        let previous = local_background_fragment(100.0);
        let mut current = local_background_fragment(120.0);
        current.paint.style.background_images[0].position_x_has_nonzero_percent = true;
        assert_eq!(
            BoxPaintInvalidator::ComputeBackgroundInvalidation(
                &current,
                Some(&previous),
                PaintInvalidatorContext::default(),
            )
            .kind,
            BackgroundInvalidationType::Full
        );
    }

    #[test]
    fn changed_gap_rule_geometry_fully_invalidates_all_background_spaces() {
        let mut previous = incremental_fragment();
        previous.paint.gap_rules = Some(Arc::new(GapRulePaintData {
            rules: vec![GapRule::default()],
        }));
        let mut current = previous.clone();
        let mut moved_rule = GapRule::default();
        moved_rule.offset.x = 3.0;
        current.paint.gap_rules = Some(Arc::new(GapRulePaintData {
            rules: vec![moved_rule],
        }));

        assert_eq!(
            BoxPaintInvalidator::ComputeBackgroundInvalidation(
                &current,
                Some(&previous),
                PaintInvalidatorContext::default(),
            ),
            BackgroundInvalidation {
                kind: BackgroundInvalidationType::Full,
                invalidate_both_spaces: true,
            }
        );
    }

    #[test]
    fn contents_background_invalidates_its_scrollable_area_client() {
        let mut fragment = local_background_fragment(120.0);
        fragment.paint.scroll_container = Some(ScrollContainerPaintData {
            compositor_element_id: Default::default(),
            scrollable_axes: [true, true],
            scroll_origin: Offset::default(),
            scroll_position: fragment.paint.scroll_offset,
            background_needs_repaint_on_scroll: Some(false),
            prefers_non_composited_scrolling: Some(false),
            scroll_snap_type: Some(Default::default()),
            container_rect: FragmentPaintRect::default(),
            scrolling_background_visual_rect: FragmentPaintRect::default(),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset::default(),
            user_scrollable_horizontal: true,
            user_scrollable_vertical: true,
            scrolling_background_client_id: 42,
            scrolling_background_client_invalidation_reason: Some(PaintInvalidationReason::kNone),
            scrolling_background_client_is_cacheable: true,
            scrolling_background_client_is_just_created: std::cell::Cell::new(false),
        });
        let reason = BoxPaintInvalidator::InvalidateBackground(
            &mut fragment,
            BackgroundInvalidation {
                kind: BackgroundInvalidationType::Full,
                invalidate_both_spaces: false,
            },
            PaintInvalidationReason::kNone,
        );
        assert_eq!(reason, PaintInvalidationReason::kNone);
        assert_eq!(
            fragment
                .paint
                .scroll_container
                .as_ref()
                .unwrap()
                .scrolling_background_client_invalidation_reason,
            Some(PaintInvalidationReason::kBackground)
        );
    }

    fn view_fragment(background_rect_width: f64, root_width: f64) -> FragmentPaintData {
        let mut fragment = local_background_fragment(root_width);
        fragment.paint.is_layout_view = true;
        fragment.paint.view_background_rect = Some(FragmentPaintRect {
            offset: Offset::default(),
            size: Size {
                width: background_rect_width,
                height: 100.0,
            },
        });
        let mut style = fragment.paint.style.clone();
        style.background_images[0].position_x_has_nonzero_percent = true;
        fragment.paint.view_background_invalidation = Some(CanvasBackgroundPaintData {
            style,
            effective_zoom: 1.0,
            document_element_node_id: 0,
            previous_document_element_geometry: None,
            document_element_paint_offset: Offset::default(),
            document_element_size: Size {
                width: root_width,
                height: 50.0,
            },
            document_element_border: Edges::default(),
            document_element_padding: Edges::default(),
            document_element_content_box: FragmentPaintRect::default(),
            document_element_scrollable_overflow: fragment.paint.scrollable_overflow,
            document_element_has_transform: false,
            background_fill_layer_count: 1,
            background_any_local_attachment_image: true,
            background_any_layer_uses_content_box: false,
        });
        fragment
    }

    #[test]
    fn view_root_size_change_uses_root_background_layer_geometry() {
        let old = view_fragment(100.0, 100.0);
        let current = view_fragment(100.0, 120.0);
        assert_eq!(
            BoxPaintInvalidator::ComputeBackgroundInvalidation(
                &current,
                Some(&old),
                PaintInvalidatorContext::default(),
            )
            .kind,
            BackgroundInvalidationType::Full
        );
    }

    #[test]
    fn view_background_rect_change_is_exported_for_fixed_background_propagation() {
        let old = view_fragment(100.0, 100.0);
        let current = view_fragment(120.0, 100.0);
        assert!(BoxPaintInvalidator::ViewBackgroundGeometryChanged(
            &current,
            Some(&old)
        ));
    }
}
