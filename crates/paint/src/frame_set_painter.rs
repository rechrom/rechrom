#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input_types::Color;

use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::geometry_mapper::MapRectToRoot;
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, PaintPhase};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/frame_set_painter.cc:8-12
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

// cpp: paint/frame_set_painter.cc:14-17
fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

// cpp: paint/frame_set_painter.cc:19-24
fn IsVisible(node: &PaintTreeNode<'_>, rect: &PaintRect) -> bool {
    let Some(cull_rect) = node.cull_rect else {
        return true;
    };
    let mapped = MapRectToRoot(*rect, &node.transforms);
    mapped.is_none_or(|mapped| Intersects(&mapped, &cull_rect))
}

// cpp: paint/frame_set_painter.cc:26-30
fn ShouldPaintBorderAfter(allow_border: &[bool], index: usize) -> bool {
    allow_border.len() >= 2 && index + 1 < allow_border.len() - 1 && allow_border[index + 1]
}

// cpp: paint/frame_set_painter.h:9-23
pub struct FrameSetPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> FrameSetPainter<'n, 'f, 'c, 'o> {
    // cpp: paint/frame_set_painter.h:11-12
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // cpp: paint/frame_set_painter.h:19
    // cpp: paint/frame_set_painter.cc:34-45
    fn AppendRect(&self, info: &PaintInfo<'c, 'o>, rect: PaintRect, color: Color) {
        if rect.width <= 0.0 || rect.height <= 0.0 || !IsVisible(self.node, &rect) {
            return;
        }
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                phase: info.phase,
                node_id: self
                    .node
                    .fragment
                    .as_deref()
                    .expect("paint tree node has a fragment")
                    .node_id,
                rect,
                color,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: paint/frame_set_painter.h:17
    // cpp: paint/frame_set_painter.cc:47-56
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

    // cpp: paint/frame_set_painter.h:18
    // cpp: paint/frame_set_painter.cc:58-67
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

    // cpp: paint/frame_set_painter.h:14
    // cpp: paint/frame_set_painter.cc:69-110
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
            fragment.paint.style.border_colors[3]
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
                    self.PaintColumnBorder(
                        info,
                        PaintRect {
                            x,
                            y,
                            width: data.border_thickness,
                            height: fragment.size.height - (y - self.node.paint_offset.y),
                        },
                        fill,
                    );
                    x += data.border_thickness;
                }
                children_remaining -= 1;
                if children_remaining == 0 {
                    return;
                }
            }
            y += *row_size;
            if ShouldPaintBorderAfter(&data.row_allows_border, row) {
                self.PaintRowBorder(
                    info,
                    PaintRect {
                        x: self.node.paint_offset.x,
                        y,
                        width: fragment.size.width,
                        height: data.border_thickness,
                    },
                    fill,
                );
                y += data.border_thickness;
            }
        }
    }
}
