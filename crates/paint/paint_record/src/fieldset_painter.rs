#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::background_bleed_avoidance::{
    BackgroundBleedAvoidance, BleedAvoidanceIsClipping,
};
use layoutng_assembly::internal::paint_input::{BackgroundBox, PaintBlendMode};

use crate::box_decoration_data::BoxDecorationData;
use crate::box_fragment_painter::{BackgroundRadii, BoxFragmentPainter};
use crate::contoured_border_geometry::UniformCornerRadius;
use crate::display_item_id::DisplayItemIdType;
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// Chromium: core/paint/fieldset_painter.h:19-35.
pub struct FieldsetPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> FieldsetPainter<'n, 'f, 'c, 'o> {
    // Chromium: core/paint/fieldset_painter.h:23-24.
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // Chromium: core/paint/fieldset_painter.cc:25-71,81-84.
    pub fn DecorationRect(node: &PaintTreeNode<'_>) -> PaintRect {
        let fragment = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let mut rect = PaintRect {
            x: node.paint_offset.x,
            y: node.paint_offset.y,
            width: fragment.size.width,
            height: fragment.size.height,
        };
        let Some(fieldset) = fragment.paint.fieldset.as_ref() else {
            return rect;
        };
        let outsets = &fieldset.border_outsets;
        rect.x += outsets.left;
        rect.y += outsets.top;
        rect.width = (rect.width - outsets.left - outsets.right).max(0.0);
        rect.height = (rect.height - outsets.top - outsets.bottom).max(0.0);
        let right = (rect.x + rect.width).round();
        let bottom = (rect.y + rect.height).round();
        rect.x = rect.x.round();
        rect.y = rect.y.round();
        rect.width = (right - rect.x).max(0.0);
        rect.height = (bottom - rect.y).max(0.0);
        rect
    }

    // Chromium: core/paint/fieldset_painter.h:25-27 and
    // core/paint/fieldset_painter.cc:75-151.
    //
    // Fieldset owns the complete decoration sequence because its legend
    // changes both the decoration box and the border clip. Keeping only the
    // clip here would leave the shadow/background/bleed ordering owned by the
    // generic box path, unlike Chromium.
    pub fn PaintBoxDecorationBackground(
        &self,
        fragment_painter: &BoxFragmentPainter<'n, 'f, 'c, 'o>,
        paint_info: &PaintInfo<'c, 'o>,
        decoration_data: &BoxDecorationData,
    ) {
        debug_assert!(decoration_data.ShouldPaint());
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let phase = paint_info.phase;
        let bleed_avoidance = decoration_data.GetBackgroundBleedAvoidance();

        if decoration_data.ShouldPaintShadow() {
            fragment_painter.PaintBoxShadows(phase, false);
        }

        let clips_for_bleed = BleedAvoidanceIsClipping(bleed_avoidance);
        if clips_for_bleed {
            let border_radii = BackgroundRadii(self.node, BackgroundBox::kBorderBox);
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipRoundedRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: self.node.PixelSnappedRect(&Self::DecorationRect(self.node)),
                    corner_radius: UniformCornerRadius(&border_radii),
                    corner_radii: border_radii,
                    ..Default::default()
                },
                self.node,
            );
            if bleed_avoidance == BackgroundBleedAvoidance::kBackgroundBleedClipLayer {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSaveLayerBlend,
                        phase,
                        node_id: fragment.node_id,
                        blend_mode: PaintBlendMode::kNormal,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }

        if decoration_data.ShouldPaintBackground() {
            fragment_painter.PaintBackground(paint_info, bleed_avoidance);
        }
        if decoration_data.ShouldPaintShadow() {
            fragment_painter.PaintBoxShadows(phase, true);
        }
        if decoration_data.ShouldPaintBorder() {
            let clipped = self.BeginBorderClip(paint_info);
            fragment_painter.PaintBorder(phase, bleed_avoidance);
            if clipped {
                self.EndBorderClip(paint_info);
            }
        }

        if clips_for_bleed {
            if bleed_avoidance == BackgroundBleedAvoidance::kBackgroundBleedClipLayer {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    // Chromium: core/paint/fieldset_painter.h:29 and
    // core/paint/fieldset_painter.cc:153-166. PrePaint has already resolved
    // mask layers against this contracted border box; pass the same reference
    // rect to mask-border painting so both mask sources share its geometry.
    pub fn PaintMask(
        &self,
        fragment_painter: &BoxFragmentPainter<'n, 'f, 'c, 'o>,
        item_type: DisplayItemIdType,
    ) {
        fragment_painter.PaintMaskImages(item_type, Some(Self::DecorationRect(self.node)));
    }

    // Chromium: core/paint/fieldset_painter.cc:129-147.
    pub fn BeginBorderClip(&self, info: &PaintInfo<'c, 'o>) -> bool {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(data) = fragment.paint.fieldset.as_ref() else {
            return false;
        };
        if !data.has_legend {
            return false;
        }
        if data.legend_cutout_size.width <= 0.0 || data.legend_cutout_size.height <= 0.0 {
            return false;
        }
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kSave,
                phase: info.phase,
                node_id: fragment.node_id,
                ..Default::default()
            },
            self.node,
        );
        let mut legend_cutout = PaintRect {
            x: self.node.paint_offset.x + data.legend_cutout_offset.x,
            y: self.node.paint_offset.y + data.legend_cutout_offset.y,
            width: data.legend_cutout_size.width,
            height: data.legend_cutout_size.height,
        };
        let snapped_right = (legend_cutout.x + legend_cutout.width + 0.5).floor();
        let snapped_bottom = (legend_cutout.y + legend_cutout.height + 0.5).floor();
        legend_cutout.x = (legend_cutout.x + 0.5).floor();
        legend_cutout.y = (legend_cutout.y + 0.5).floor();
        legend_cutout.width = (snapped_right - legend_cutout.x).max(0.0);
        legend_cutout.height = (snapped_bottom - legend_cutout.y).max(0.0);
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kClipOutRect,
                phase: info.phase,
                node_id: fragment.node_id,
                rect: legend_cutout,
                ..Default::default()
            },
            self.node,
        );
        true
    }

    // Explicit recording counterpart of the scoped clip/layer restoration in
    // core/paint/fieldset_painter.cc:95-111,149-150.
    pub fn EndBorderClip(&self, info: &PaintInfo<'c, 'o>) {
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase: info.phase,
                node_id: self
                    .node
                    .fragment
                    .as_deref()
                    .expect("paint tree node has a fragment")
                    .node_id,
                ..Default::default()
            },
            self.node,
        );
    }
}
