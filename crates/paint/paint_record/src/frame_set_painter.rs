#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input_types::Color;

use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::paint_auto_dark_mode::{ElementRole, InvertColorIfNeeded};
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, PaintPhase};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// Chromium: core/paint/frame_set_painter.cc:17-19.
const BORDER_START_EDGE_COLOR: Color = Color {
    red: 170.0 / 255.0,
    green: 170.0 / 255.0,
    blue: 170.0 / 255.0,
    alpha: 1.0,
};
const BORDER_END_EDGE_COLOR: Color = Color {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 1.0,
};
const BORDER_FILL_COLOR: Color = Color {
    red: 208.0 / 255.0,
    green: 208.0 / 255.0,
    blue: 208.0 / 255.0,
    alpha: 1.0,
};

// GraphicsContext applies the same cull admission to emitted drawing ops.  Keep
// the explicit predicate here because FrameSetPainter::PaintColumnBorder has an
// early cull return in Chromium (core/paint/frame_set_painter.cc:136-137).
fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

// Chromium: core/paint/frame_set_painter.cc:21-25.
fn IsVisible(node: &PaintTreeNode<'_>, rect: &PaintRect) -> bool {
    let Some(cull_rect) = node.cull_rect else {
        return true;
    };
    Intersects(rect, &cull_rect)
}

// cpp: paint/frame_set_painter.cc:26-30
fn ShouldPaintBorderAfter(allow_border: &[bool], index: usize) -> bool {
    allow_border.len() >= 2 && index + 1 < allow_border.len() - 1 && allow_border[index + 1]
}

// Chromium entity: core/paint/frame_set_painter.h:23-48.
pub struct FrameSetPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> FrameSetPainter<'n, 'f, 'c, 'o> {
    // Chromium: core/paint/frame_set_painter.h:27-30.
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // Chromium: core/paint/frame_set_painter.cc:117-129,140-152.
    fn AppendRect(&self, info: &PaintInfo<'c, 'o>, rect: PaintRect, color: Color) {
        if rect.width <= 0.0 || rect.height <= 0.0 || !IsVisible(self.node, &rect) {
            return;
        }
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let color = InvertColorIfNeeded(&fragment.paint.style, color, ElementRole::kBackground);
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                phase: info.phase,
                node_id: fragment.node_id,
                rect,
                color,
                ..Default::default()
            },
            self.node,
        );
    }

    // Chromium: core/paint/frame_set_painter.cc:112-130.
    fn PaintRowBorder(&self, info: &PaintInfo<'c, 'o>, rect: PaintRect, fill: Color) {
        self.AppendRect(info, rect, fill);
        if rect.height < 3.0 {
            return;
        }
        self.AppendRect(
            info,
            PaintRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: 1.0,
            },
            BORDER_START_EDGE_COLOR,
        );
        self.AppendRect(
            info,
            PaintRect {
                x: rect.x,
                y: rect.y + rect.height - 1.0,
                width: rect.width,
                height: 1.0,
            },
            BORDER_END_EDGE_COLOR,
        );
    }

    // Chromium: core/paint/frame_set_painter.cc:132-153.
    fn PaintColumnBorder(&self, info: &PaintInfo<'c, 'o>, rect: PaintRect, fill: Color) {
        self.AppendRect(info, rect, fill);
        if rect.width < 3.0 {
            return;
        }
        self.AppendRect(
            info,
            PaintRect {
                x: rect.x,
                y: rect.y,
                width: 1.0,
                height: rect.height,
            },
            BORDER_START_EDGE_COLOR,
        );
        self.AppendRect(
            info,
            PaintRect {
                x: rect.x + rect.width - 1.0,
                y: rect.y,
                width: 1.0,
                height: rect.height,
            },
            BORDER_END_EDGE_COLOR,
        );
    }

    // Chromium: core/paint/frame_set_painter.cc:60-110. PaintObject and
    // PaintChildren (lines 29-58) are owned by the shared BoxFragmentPainter /
    // Paint walk; this method is called after those descendants.
    pub fn PaintBorders(&self, info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if info.phase != PaintPhase::kForeground
            || fragment.paint.frame_set.is_none()
            || self.node.children.is_empty()
        {
            return;
        }
        // FrameSetPainter::PaintBorders (.cc:66) owns one phase drawing for
        // all row/column borders, including their fill and beveled edge ops.
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::PaintPhaseToDrawingType(info.phase),
            PaintRect {
                x: self.node.paint_offset.x,
                y: self.node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            },
        );
        let data = fragment.paint.frame_set.as_ref().unwrap();
        if data.border_thickness <= 0.0 {
            return;
        }
        let fill = if data.has_border_color {
            fragment.paint.style.border_colors()[3]
        } else {
            BORDER_FILL_COLOR
        };
        let mut children_remaining = self.node.children.len();
        if children_remaining == 0 {
            return;
        }
        let mut y = self.node.paint_offset.y;
        for (row, row_size) in data.row_sizes.iter().enumerate() {
            let mut x = self.node.paint_offset.x;
            for (column, column_size) in data.column_sizes.iter().enumerate() {
                x += *column_size;
                if ShouldPaintBorderAfter(&data.column_allows_border, column) {
                    let rect = self.node.PixelSnappedRect(&PaintRect {
                        x,
                        y,
                        width: data.border_thickness,
                        height: fragment.size.height - (y - self.node.paint_offset.y),
                    });
                    self.PaintColumnBorder(info, rect, fill);
                    x += data.border_thickness;
                }
                children_remaining -= 1;
                if children_remaining == 0 {
                    return;
                }
            }
            y += *row_size;
            if ShouldPaintBorderAfter(&data.row_allows_border, row) {
                let rect = self.node.PixelSnappedRect(&PaintRect {
                    x: self.node.paint_offset.x,
                    y,
                    width: fragment.size.width,
                    height: data.border_thickness,
                });
                self.PaintRowBorder(info, rect, fill);
                y += data.border_thickness;
            }
        }
    }
}
