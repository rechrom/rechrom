//! Layout-box paint invalidation during PrePaint.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/box_paint_invalidator.{h,cc}`.

use layout::fragment::ResidentBoxGeometry as _;
use foundation::{
    AppearanceValue, BorderLineStyle, IsFullPaintInvalidationReason,
    IsLayoutFullPaintInvalidationReason, PaintInvalidationReason,
};
use layout::fragment::PreviousBoxGeometry;
use layout::paint::{FragmentPaintData, GapRulePaintData};
use paint_input_types::FragmentPaintRect;
use paint_input_types::layout_input::{Offset, Size};
use layout::paint::PaintFragmentCursor;
use paint_input_types::paint_input::{
    BackgroundBox, BackgroundImageLayer, BackgroundRepeat, BackgroundRepeatRule, BackgroundSizeMode,
};

use crate::paint_invalidator::PaintInvalidatorContext;

pub struct BoxPaintInvalidator;

/// LayoutView::PreviousBackgroundRect() and BackgroundRect() of a fragment,
/// read live from its source object (None for anything but a LayoutView
/// source; `current` falls back to a detached fragment's stored rect).
/// Source: `box_paint_invalidator.cc:257-259`. Both stay fixed during a
/// PrePaint walk (the previous rect is saved after the artifact commits), so
/// the walk reads them once and shares them between the fixed-background
/// check and the LayoutView's own background invalidation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewBackgroundRects {
    pub previous: Option<FragmentPaintRect>,
    pub current: Option<FragmentPaintRect>,
}

impl ViewBackgroundRects {
    pub fn Read(fragment: &FragmentPaintData) -> Self {
        Self {
            previous: fragment.paint.PreviousViewBackgroundRect(),
            current: fragment.paint.ViewBackgroundRect(),
        }
    }
}

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
    /// `source` is the fragment's resident cursor; the LayoutView reads its
    /// canvas background selection live through it. Hand-built fixtures pass
    /// None and keep their stored selection.
    pub fn ComputeBackgroundInvalidation(
        fragment: &FragmentPaintData,
        source: Option<PaintFragmentCursor<'_>>,
        previous: Option<&FragmentPaintData>,
        context: PaintInvalidatorContext,
    ) -> BackgroundInvalidation {
        Self::ComputeBackgroundInvalidationWithPreviousGapRules(
            fragment,
            source,
            previous,
            previous.and_then(|old| old.paint.gap_rules.as_deref()),
            context,
        )
    }

    /// ComputeBackgroundInvalidation with the gap geometry this box last
    /// painted with kept beside the previous fragment, as LayoutBox keeps its
    /// previous GapGeometry (box_paint_invalidator.cc:467-510), instead of
    /// written into a copy of the shared paint payload.
    pub fn ComputeBackgroundInvalidationWithPreviousGapRules(
        fragment: &FragmentPaintData,
        source: Option<PaintFragmentCursor<'_>>,
        previous: Option<&FragmentPaintData>,
        previous_gap_rules: Option<&GapRulePaintData>,
        context: PaintInvalidatorContext,
    ) -> BackgroundInvalidation {
        Self::ComputeBackgroundInvalidationWithViewRects(
            fragment,
            source,
            previous,
            previous_gap_rules,
            context,
            None,
        )
    }

    /// ComputeBackgroundInvalidationWithPreviousGapRules with the
    /// LayoutView's background rects already read for this walk
    /// (`view_background_rects`, None to read them from `fragment`).
    pub fn ComputeBackgroundInvalidationWithViewRects(
        fragment: &FragmentPaintData,
        source: Option<PaintFragmentCursor<'_>>,
        previous: Option<&FragmentPaintData>,
        previous_gap_rules: Option<&GapRulePaintData>,
        context: PaintInvalidatorContext,
        view_background_rects: Option<ViewBackgroundRects>,
    ) -> BackgroundInvalidation {
        let mut result = BackgroundInvalidation::default();
        let previous_geometry = PreviousGeometry(fragment, previous);
        let invalidation = fragment.pre_paint_invalidation;
        if invalidation.background_needs_full_paint_invalidation
            || context.HasSubtreeFullInvalidation()
            || ShouldInvalidateGapDecorations(fragment, previous_gap_rules)
        {
            result.kind = BackgroundInvalidationType::Full;
            result.invalidate_both_spaces = true;
        } else if HasEffectiveBackground(fragment) {
            let old_size = previous_geometry.map(|old| old.size).unwrap_or_default();
            let new_size = StitchedSize(fragment);
            if fragment.paint.BackgroundAnyDefaultAttachmentImage()
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
                    fragment.paint.ScrollableOverflow(),
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
            let rects =
                view_background_rects.unwrap_or_else(|| ViewBackgroundRects::Read(fragment));
            let view_kind =
                ComputeViewBackgroundInvalidation(fragment, source, previous, context, rects);
            result.kind = result.kind.max(view_kind);
        }
        result
    }

    pub fn ViewBackgroundGeometryChanged(
        fragment: &FragmentPaintData,
        previous: Option<&FragmentPaintData>,
    ) -> bool {
        Self::ViewBackgroundGeometryChangedWithRects(ViewBackgroundRects::Read(fragment), previous)
    }

    /// ViewBackgroundGeometryChanged over rects already read from the
    /// fragment (ViewBackgroundRects::Read).
    pub fn ViewBackgroundGeometryChangedWithRects(
        rects: ViewBackgroundRects,
        previous: Option<&FragmentPaintData>,
    ) -> bool {
        match (
            rects
                .previous
                .or_else(|| previous.and_then(|old| old.paint.view_background_rect)),
            rects.current,
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
        let scrolling_background_client = fragment
            .ViewScrollingBackgroundClient()
            .map(|(id, _)| id);
        if let Some(client_id) = scrolling_background_client {
            if invalidation.invalidate_both_spaces
                || (paints_contents && invalidation.kind != BackgroundInvalidationType::None)
            {
                let reason = if invalidation.kind == BackgroundInvalidationType::Full {
                    PaintInvalidationReason::kBackground
                } else {
                    PaintInvalidationReason::kIncremental
                };
                // DisplayItemClient::Invalidate on the live scrolling-background
                // client; a detached payload keeps the monotonic rule locally.
                // box_paint_invalidator.cc marks the painting layer first.
                fragment.paint.SetPaintingLayerNeedsRepaint();
                if !fragment.paint.InvalidateLiveScrollClient(client_id, reason) {
                    let current = fragment
                        .ViewScrollContainer()
                        .and_then(|scroll| scroll.scrolling_background_client_invalidation_reason)
                        .unwrap_or(PaintInvalidationReason::kNone);
                    if reason > current {
                        if let Some(scroll) = fragment.paint.scroll_container.as_mut() {
                            scroll.scrolling_background_client_invalidation_reason = Some(reason);
                        }
                    }
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
                
                .pre_paint_invalidation
                .should_check_layout_for_paint_invalidation
        {
            return reason;
        }

        let Some(previous) = PreviousGeometry(fragment, previous) else {
            return reason;
        };

        // box_paint_invalidator.cc:130-211 reads the style's MaskLayers() in
        // place (FillLayer::AnyLayerUsesContentBox).
        // MaskUsesContentBox projects every mask layer from the style, so
        // test the content-box geometry first: an unchanged content box (the
        // usual incremental case) needs no layer list.
        if (previous.content_box.offset, previous.content_box.size)
            != PhysicalContentBoxSignature(fragment)
            && MaskUsesContentBox(fragment)
        {
            return PaintInvalidationReason::kLayout;
        }

        if fragment
            .ViewReplacedContent()
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

        if fragment.Borders().right != 0.0 || fragment.Borders().bottom != 0.0 {
            return PaintInvalidationReason::kLayout;
        }

        let style = &fragment.paint.style;
        let has_visual_overflowing_effect = style.box_shadow_count() != 0
            || (style.outline_style() != BorderLineStyle::kNone && style.outline_width() != 0.0);
        let has_effective_appearance = fragment
            .paint
            .FormControl()
            .is_some_and(|control| control.appearance != AppearanceValue::kNone);
        if has_visual_overflowing_effect
            || has_effective_appearance
            || style.has_filters()
            || style.has_mask_images()
            || style.has_mask_box_image()
            || style.clip_path.is_some()
        {
            return PaintInvalidationReason::kLayout;
        }

        if style
            .border_radii()
            .as_ref()
            .is_some_and(|radii| radii.HasRadius())
            || style.border_radius != 0.0
            // The resource id and generated shader do not depend on the
            // document image catalog (only the resolved slices do).
            || fragment
                .BorderImage(None)
                .is_some_and(|image| image.resource_id != 0 || image.shader.is_some())
        {
            return PaintInvalidationReason::kLayout;
        }

        if fragment.ViewFrameSet().is_some() {
            return PaintInvalidationReason::kLayout;
        }

        reason
    }
}

pub(crate) fn PreviousGeometry(
    fragment: &FragmentPaintData,
    previous: Option<&FragmentPaintData>,
) -> Option<PreviousBoxGeometry> {
    fragment.paint.PreviousBoxGeometry().or_else(|| {
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
        .ViewBoxSelfVisualOverflow()
        .unwrap_or(FragmentPaintRect {
            offset: Offset::default(),
            size: StitchedSize(fragment),
        })
}

fn ComputeViewBackgroundInvalidation(
    fragment: &FragmentPaintData,
    source: Option<PaintFragmentCursor<'_>>,
    previous: Option<&FragmentPaintData>,
    context: PaintInvalidatorContext,
    rects: ViewBackgroundRects,
) -> BackgroundInvalidationType {
    let (old_rect, new_rect) = match (
        rects
            .previous
            .or_else(|| previous.and_then(|old| old.paint.view_background_rect)),
        rects.current,
    ) {
        (Some(old), Some(new)) => (old, new),
        _ => return BackgroundInvalidationType::None,
    };
    let location_changed = old_rect.offset != new_rect.offset;
    let size_changed = old_rect.size != new_rect.size;
    if location_changed
        || fragment
            
            .pre_paint_invalidation
            .background_needs_full_paint_invalidation
        || context.HasSubtreeFullInvalidation()
    {
        return BackgroundInvalidationType::Full;
    }

    // The canvas background selection and root element geometry, read live
    // from the LayoutView (box_paint_invalidator.cc ComputeViewBackground-
    // Invalidation reads the document element's LayoutBox).
    // Chromium reads the root box geometry first and a background layer
    // only for a geometry that changed (box_paint_invalidator.cc:280-311).
    // When the root geometry is unchanged and untransformed, every branch
    // below falls through to the size-change result, so the canvas
    // background source is not selected or read.
    let previous_view = previous.and_then(|old| old.paint.view_background_invalidation.as_ref());
    let view_background = match source {
        Some(cursor) => cursor
            .ViewBackgroundRootGeometry()
            .filter(|root| ViewRootGeometryMayInvalidate(root, previous_view))
            .and_then(|_| cursor.ViewBackground()),
        None => fragment
            .paint
            .view_background_invalidation
            .as_ref()
            .map(|view| view.AsRef()),
    };
    if let Some(new) = view_background.as_ref() {
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
        // ShouldFullyInvalidateFillLayersOnSizeChange reads the FillLayers in
        // place and only for a changed size (box_paint_invalidator.cc:
        // 253-319). background_images() projects every layer, so build the
        // list only when a size it is compared for actually changed.
        let layers_need_full_invalidation = |old_size: Size, new_size: Size| {
            old_size != new_size
                && ShouldFullyInvalidateBackgroundLayersOnSizeChange(
                    new.background_fill_layer_count,
                    &new.style.background_images(),
                    old_size,
                    new_size,
                )
        };
        if layers_need_full_invalidation(old_geometry.size, new.document_element_size) {
            return BackgroundInvalidationType::Full;
        }
        if let (Some(old_overflow), Some(new_overflow)) = (
            Some(old_geometry.scrollable_overflow),
            new.document_element_scrollable_overflow,
        ) {
            if old_overflow != new_overflow && new.background_any_local_attachment_image {
                if old_overflow.offset != new_overflow.offset
                    || layers_need_full_invalidation(old_overflow.size, new_overflow.size)
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

/// Whether ComputeViewBackgroundInvalidation's root-box branch can return
/// anything but the size-change result for `root` (ViewBackgroundRoot-
/// Geometry): it reads the same old geometry as that branch and reports
/// whether the document element's size, scrollable overflow or content box
/// changed, or whether it has a transform. Each Full return of the branch
/// requires one of these (box_paint_invalidator.cc:283-311).
fn ViewRootGeometryMayInvalidate(
    root: &layout::paint::CanvasBackgroundPaintView<'_>,
    previous_view: Option<&layout::paint::CanvasBackgroundPaintData>,
) -> bool {
    let old_geometry = root.previous_document_element_geometry.or_else(|| {
        previous_view.map(|old| PreviousBoxGeometry {
            size: old.document_element_size,
            content_box: old.document_element_content_box,
            scrollable_overflow: old.document_element_scrollable_overflow.unwrap_or_default(),
            ..PreviousBoxGeometry::default()
        })
    });
    let Some(old) = old_geometry else {
        return false;
    };
    root.document_element_has_transform
        || old.size != root.document_element_size
        || root
            .document_element_scrollable_overflow
            .is_some_and(|overflow| overflow != old.scrollable_overflow)
        || old.content_box != root.document_element_content_box
}

fn HasEffectiveBackground(fragment: &FragmentPaintData) -> bool {
    fragment.paint.IsLayoutView()
        || (fragment.paint.HasBackground() && !fragment.paint.BackgroundTransfersToView())
}

fn BackgroundUsesContentBox(fragment: &FragmentPaintData) -> bool {
    fragment.paint.BackgroundAnyLayerUsesContentBox()
}

fn BackgroundPaintsInContentsSpace(fragment: &FragmentPaintData) -> bool {
    HasEffectiveBackground(fragment) && fragment.paint.BackgroundPaintLocation().PaintsContents()
}

fn BackgroundPaintsInBorderBoxSpace(fragment: &FragmentPaintData) -> bool {
    HasEffectiveBackground(fragment) && fragment.paint.BackgroundPaintLocation().PaintsBorderBox()
}

fn BackgroundGeometryDependsOnScrollableOverflowRect(fragment: &FragmentPaintData) -> bool {
    HasEffectiveBackground(fragment) && fragment.paint.BackgroundAnyLocalAttachmentImage()
}

/// Source: `box_paint_invalidator.cc:467-510,530-534`. The immutable Paint
/// boundary retains the resolved rule segments with the previous fragment, so
/// comparing those segments is equivalent to comparing LayoutBox's saved
/// per-fragment GapGeometry for the purpose of deciding whether to re-record.
fn ShouldInvalidateGapDecorations(
    fragment: &FragmentPaintData,
    previous_rules: Option<&GapRulePaintData>,
) -> bool {
    let Some(previous_rules) = previous_rules else {
        return false;
    };
    fragment.ViewGapRules().as_deref() != Some(previous_rules)
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
    // An unchanged size invalidates no layer; skip projecting the layers.
    if old_size == new_size {
        return false;
    }
    let layers = fragment.paint.style.background_images();
    ShouldFullyInvalidateBackgroundLayersOnSizeChange(
        fragment.paint.BackgroundFillLayerCount(),
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
    old: paint_input_types::FragmentPaintRect,
    new: paint_input_types::FragmentPaintRect,
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
            x: fragment.Borders().left + fragment.Padding().left,
            y: fragment.Borders().top + fragment.Padding().top,
        },
        fragment.ViewContentSize(),
    )
}

fn StitchedSize(fragment: &FragmentPaintData) -> Size {
    fragment
        .ViewStitchedDecoration()
        .as_ref()
        .map_or(fragment.ViewSize(), |decoration| decoration.size)
}

fn SelfVisualOverflowSize(fragment: &FragmentPaintData) -> Size {
    fragment
        .ViewBoxSelfVisualOverflow()
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
    use layout::paint::{
        BackgroundPaintLocation, CanvasBackgroundPaintData, GapRule, GapRulePaintData,
        ScrollContainerPaintData,
    };
    use paint_input_types::FragmentPaintRect;
    use paint_input_types::layout_input::Edges;
    use paint_input_types::paint_input::BackgroundAttachment;
    use std::sync::Arc;

    fn incremental_fragment() -> FragmentPaintData {
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 100.0,
            height: 50.0,
        };
        fragment.content_size = fragment.ViewSize();
        fragment
            
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
                None,
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
                None,
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
                None,
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
            scroll_position: fragment.paint.ScrollOffset(),
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
                .ViewScrollContainer()
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
            document_element_scrollable_overflow: fragment.paint.ScrollableOverflow(),
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
                None,
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
