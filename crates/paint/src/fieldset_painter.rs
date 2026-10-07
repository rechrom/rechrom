#![allow(non_snake_case)]

use std::cell::RefCell;

use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/fieldset_painter.h:10-22
pub struct FieldsetPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> FieldsetPainter<'n, 'f, 'c, 'o> {
    // cpp: paint/fieldset_painter.h:12-13
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // cpp: paint/fieldset_painter.h:15
    // cpp: paint/fieldset_painter.cc:8-30
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

    // cpp: paint/fieldset_painter.h:16
    // cpp: paint/fieldset_painter.cc:32-64
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

    // cpp: paint/fieldset_painter.h:17
    // cpp: paint/fieldset_painter.cc:66-71
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
