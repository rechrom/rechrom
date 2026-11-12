#![allow(non_snake_case)]

use layoutng_assembly::internal::background_bleed_avoidance::BackgroundBleedAvoidance;
use layoutng_assembly::internal::layout_input::BorderLineStyle;
use layoutng_style::style::appearance::AppearanceValue;

use crate::box_fragment_painter::{BackgroundLayerOccludesFollowingLayers, BackgroundRadii};
use crate::contoured_border_geometry::IncludedBorderEdges;
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use layoutng_assembly::internal::paint_input::BackgroundBox;

// Chromium: core/paint/box_decoration_data.h:21-173 and
// core/paint/box_decoration_data.cc:11-70.
//
// Keep the decision object in paint_record. Layout exports the immutable facts
// it needs, while the painter consumes only these resolved decisions.
#[derive(Clone, Copy, Debug)]
pub struct BoxDecorationData {
    has_appearance: bool,
    should_paint_background: bool,
    should_paint_border: bool,
    should_paint_shadow: bool,
    should_paint_gap_decorations: bool,
    bleed_avoidance: BackgroundBleedAvoidance,
}

impl BoxDecorationData {
    pub fn new(node: &PaintTreeNode<'_>, paint_info: &PaintInfo<'_, '_>) -> Self {
        let fragment = node
            .fragment
            .as_deref()
            .expect("box decoration requires a fragment");
        let style = &fragment.paint.style;
        let has_appearance = fragment.paint.form_control.as_ref().is_some_and(|control| {
            !matches!(
                control.appearance,
                AppearanceValue::kNone | AppearanceValue::kAuto
            )
        });
        let has_background = fragment.paint.has_background
            || fragment.paint.is_page_border_box
            || style.background_color().alpha > 0.0
            || style.has_background_images()
            || (paint_info.ShouldOmitCompositingInfo() && style.has_backdrop_filters());
        let should_paint_background = has_background
            && !fragment.paint.background_transfers_to_view
            && !paint_info.ShouldSkipBackground();
        let border = IncludedBorderEdges(&fragment.paint);
        let widths = [border.top, border.right, border.bottom, border.left];
        let has_non_collapsed_border_decoration = !fragment.paint.has_collapsed_borders
            && (style.border_image.is_some()
                || widths.iter().enumerate().any(|(edge, width)| {
                    *width > 0.0
                        && !matches!(
                            fragment.paint.border_styles[edge],
                            BorderLineStyle::kNone | BorderLineStyle::kHidden
                        )
                }));
        let should_paint_border = !paint_info.IsPaintingBackgroundInContentsSpace()
            && has_non_collapsed_border_decoration;
        let should_paint_shadow =
            !paint_info.IsPaintingBackgroundInContentsSpace() && style.box_shadow_count() != 0;

        let should_paint_gap_decorations =
            fragment.paint.gap_rules.is_some() && !paint_info.ShouldSkipGapDecorations();

        let mut result = Self {
            has_appearance,
            should_paint_background,
            should_paint_border,
            should_paint_shadow,
            should_paint_gap_decorations,
            bleed_avoidance: BackgroundBleedAvoidance::kBackgroundBleedNone,
        };
        result.bleed_avoidance = result.ComputeBleedAvoidance(node, paint_info);
        result
    }

    pub fn BackgroundOnly(self) -> Self {
        debug_assert!(self.should_paint_background);
        Self {
            has_appearance: false,
            should_paint_background: true,
            should_paint_border: false,
            should_paint_shadow: false,
            should_paint_gap_decorations: false,
            bleed_avoidance: self.bleed_avoidance,
        }
    }

    pub fn BorderOnly(self) -> Self {
        debug_assert!(self.should_paint_border);
        Self {
            has_appearance: false,
            should_paint_background: false,
            should_paint_border: true,
            should_paint_shadow: false,
            should_paint_gap_decorations: false,
            bleed_avoidance: self.bleed_avoidance,
        }
    }

    pub fn WithoutBackground(self) -> Self {
        Self {
            should_paint_background: false,
            bleed_avoidance: BackgroundBleedAvoidance::kBackgroundBleedNone,
            ..self
        }
    }

    pub fn WithoutBorder(self) -> Self {
        Self {
            should_paint_border: false,
            ..self
        }
    }

    pub fn HasAppearance(self) -> bool {
        self.has_appearance
    }

    pub fn ShouldPaintBackground(self) -> bool {
        self.should_paint_background
    }

    pub fn ShouldPaintBorder(self) -> bool {
        self.should_paint_border
    }

    pub fn ShouldPaintShadow(self) -> bool {
        self.should_paint_shadow
    }

    pub fn ShouldPaintGapDecorations(self) -> bool {
        self.should_paint_gap_decorations
    }

    pub fn ShouldPaint(self) -> bool {
        self.HasAppearance()
            || self.ShouldPaintBackground()
            || self.ShouldPaintBorder()
            || self.ShouldPaintShadow()
            || self.ShouldPaintGapDecorations()
    }

    pub fn GetBackgroundBleedAvoidance(self) -> BackgroundBleedAvoidance {
        self.bleed_avoidance
    }

    fn BorderObscuresBackgroundEdge(self, node: &PaintTreeNode<'_>) -> bool {
        let fragment = node
            .fragment
            .as_deref()
            .expect("box decoration requires a fragment");
        let style = &fragment.paint.style;
        let border = IncludedBorderEdges(&fragment.paint);
        let widths = [border.top, border.right, border.bottom, border.left];
        widths.iter().enumerate().all(|(edge, width)| {
            *width > 0.0
                && style.border_colors()[edge].alpha >= 1.0
                && !matches!(
                    fragment.paint.border_styles[edge],
                    BorderLineStyle::kNone
                        | BorderLineStyle::kHidden
                        | BorderLineStyle::kDotted
                        | BorderLineStyle::kDashed
                )
        })
    }

    fn ComputeBleedAvoidance(
        self,
        node: &PaintTreeNode<'_>,
        paint_info: &PaintInfo<'_, '_>,
    ) -> BackgroundBleedAvoidance {
        use BackgroundBleedAvoidance::*;

        if !self.should_paint_background
            || paint_info.IsPaintingBackgroundInContentsSpace()
            || node.is_root
        {
            return kBackgroundBleedNone;
        }

        let fragment = node
            .fragment
            .as_deref()
            .expect("box decoration requires a fragment");
        let style = &fragment.paint.style;
        let has_border_image = style.border_image.is_some();
        let has_border_radius = BackgroundRadii(node, BackgroundBox::kBorderBox).HasRadius();

        if !self.should_paint_border || !has_border_radius || has_border_image {
            if let Some(border_image) = style.border_image.as_ref() {
                if has_border_radius
                    || border_image.outsets.top != 0.0
                    || border_image.outsets.right != 0.0
                    || border_image.outsets.bottom != 0.0
                    || border_image.outsets.left != 0.0
                {
                    return kBackgroundBleedNone;
                }
            }
            if fragment.paint.background_should_always_be_clipped {
                return kBackgroundBleedClipOnly;
            }
            if has_border_radius && style.has_background_images() {
                let background_images = style.background_images();
                let top_layer = &background_images[0];
                if (style.background_color().alpha > 0.0 || background_images.len() > 1)
                    && !BackgroundLayerOccludesFollowingLayers(top_layer)
                {
                    return kBackgroundBleedClipLayer;
                }
            }
            return kBackgroundBleedNone;
        }

        if self.BorderObscuresBackgroundEdge(node) {
            kBackgroundBleedShrinkBackground
        } else {
            kBackgroundBleedClipLayer
        }
    }
}
