#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::fragment_tree::MathMLPaintKind;
use layoutng_assembly::internal::layout_input::{Offset, TextDirection};

use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::geometry_mapper::MapRectToRoot;
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/mathml_painter.cc:12-15
fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

// cpp: paint/mathml_painter.cc:17-22
fn IsVisible(node: &PaintTreeNode<'_>, rect: &PaintRect) -> bool {
    let Some(cull_rect) = node.cull_rect else {
        return true;
    };
    let mapped = MapRectToRoot(*rect, &node.transforms);
    mapped.is_none_or(|mapped| Intersects(&mapped, &cull_rect))
}

// cpp: paint/mathml_painter.h:9-27
pub struct MathMLPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> MathMLPainter<'n, 'f, 'c, 'o> {
    // cpp: paint/mathml_painter.h:11-12
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // cpp: paint/mathml_painter.h:14
    // cpp: paint/mathml_painter.cc:26-41
    pub fn Paint(&self, info: &PaintInfo<'c, 'o>) {
        let paint = &self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment")
            .paint;
        let Some(math) = paint.mathml.as_ref() else {
            return;
        };
        // MathMLPainter::Paint (.cc:170) records the whole math decoration,
        // including all radical/operator glyph pieces and the fraction bar.
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let visual_rect = PaintRect {
            x: self.node.paint_offset.x,
            y: self.node.paint_offset.y,
            width: fragment.size.width,
            height: fragment.size.height,
        };
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::PaintPhaseToDrawingType(info.phase),
            visual_rect,
        );
        match math.kind {
            MathMLPaintKind::kFraction => self.PaintFractionBar(info),
            MathMLPaintKind::kOperator => self.PaintOperator(info),
            MathMLPaintKind::kRadical => self.PaintRadical(info),
        }
    }

    // cpp: paint/mathml_painter.h:17
    // cpp: paint/mathml_painter.cc:43-76
    fn PaintBar(&self, info: &PaintInfo<'c, 'o>, mut rect: PaintRect) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if !rect.x.is_finite()
            || !rect.y.is_finite()
            || !rect.width.is_finite()
            || !rect.height.is_finite()
            || rect.width <= 0.0
            || rect.height <= 0.0
        {
            return;
        }
        let rounded = |value: f64| (value + 0.5).floor();
        let snapped_size = |size: f64, location: f64| {
            let fraction = location - location.floor();
            let mut result = rounded(fraction + size) - rounded(fraction);
            if result == 0.0 && size.abs() > 4.0 / 64.0 {
                result = if size > 0.0 { 1.0 } else { -1.0 };
            }
            result
        };
        let snapped_width = snapped_size(rect.width, rect.x);
        let snapped_height = snapped_size(rect.height, rect.y);
        rect = PaintRect {
            x: rounded(rect.x),
            y: rounded(rect.y) - (snapped_height / 2.0).floor(),
            width: snapped_width,
            height: snapped_height,
        };
        if rect.width <= 0.0 || rect.height <= 0.0 || !IsVisible(self.node, &rect) {
            return;
        }
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                phase: info.phase,
                node_id: fragment.node_id,
                rect,
                color: fragment.paint.style.color,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: paint/mathml_painter.h:18
    // cpp: paint/mathml_painter.cc:78-92
    fn PaintFractionBar(&self, info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let paint = &fragment.paint;
        let math = paint
            .mathml
            .as_ref()
            .expect("fraction requires MathML paint data");
        let Some(first_baseline) = paint.first_baseline else {
            return;
        };
        if math.rule_thickness <= 0.0 {
            return;
        }
        let width = fragment.size.width
            - paint.border.left
            - paint.border.right
            - paint.padding.left
            - paint.padding.right;
        self.PaintBar(
            info,
            PaintRect {
                x: self.node.paint_offset.x + paint.border.left + paint.padding.left,
                y: self.node.paint_offset.y + first_baseline - math.axis_height,
                width,
                height: math.rule_thickness,
            },
        );
    }

    // cpp: paint/mathml_painter.h:22-23
    // cpp: paint/mathml_painter.cc:94-100
    fn LogicalToPhysicalX(&self, logical_offset: f64, inner_width: f64) -> f64 {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.direction == TextDirection::kLtr {
            logical_offset
        } else {
            fragment.size.width - logical_offset - inner_width
        }
    }

    // cpp: paint/mathml_painter.h:21
    // cpp: paint/mathml_painter.cc:102-132
    fn PaintOperatorGlyphs(&self, info: &PaintInfo<'c, 'o>, baseline_origin: Offset) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let math = fragment
            .paint
            .mathml
            .as_ref()
            .expect("operator requires MathML paint data");
        for input_run in &math.operator_glyph_runs {
            let mut item = DisplayItem {
                r#type: DisplayItemType::kDrawGlyphRun,
                phase: info.phase,
                node_id: fragment.node_id,
                rect: PaintRect {
                    x: self.node.paint_offset.x,
                    y: self.node.paint_offset.y,
                    width: fragment.size.width,
                    height: fragment.size.height,
                },
                color: fragment.paint.style.color,
                font_face_index: input_run.font_face_index,
                font_variations: input_run.font_variations.clone(),
                font_size: input_run.font_size,
                baseline: baseline_origin.y,
                horizontal: input_run.horizontal,
                rtl: input_run.rtl,
                synthetic_bold: input_run.synthetic_bold,
                synthetic_italic: input_run.synthetic_italic,
                font_smoothing: input_run.font_smoothing,
                writing_mode: input_run.writing_mode,
                glyphs: input_run.glyphs.clone(),
                ..Default::default()
            };
            for glyph in &mut item.glyphs {
                glyph.offset.x += self.node.paint_offset.x + baseline_origin.x;
                glyph.offset.y += self.node.paint_offset.y + baseline_origin.y;
            }
            if IsVisible(self.node, &item.rect) {
                self.context.borrow_mut().Append(item, self.node);
            }
        }
    }

    // cpp: paint/mathml_painter.h:19
    // cpp: paint/mathml_painter.cc:134-147
    fn PaintOperator(&self, info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let paint = &fragment.paint;
        let math = paint
            .mathml
            .as_ref()
            .expect("operator requires MathML paint data");
        let content_width = fragment.size.width
            - paint.border.left
            - paint.border.right
            - paint.padding.left
            - paint.padding.right;
        let ltr = paint.direction == TextDirection::kLtr;
        let mut x = self.LogicalToPhysicalX(0.0, math.operator_inline_size)
            + paint.border.left
            + paint.padding.left;
        x += (content_width - math.operator_inline_size) / 2.0 * if ltr { 1.0 } else { -1.0 };
        self.PaintOperatorGlyphs(
            info,
            Offset {
                x,
                y: paint.border.top + paint.padding.top + math.operator_ascent,
            },
        );
    }

    // cpp: paint/mathml_painter.h:20
    // cpp: paint/mathml_painter.cc:149-178
    fn PaintRadical(&self, info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let paint = &fragment.paint;
        let math = paint
            .mathml
            .as_ref()
            .expect("radical requires MathML paint data");
        let mut base_width = 0.0;
        let mut base_ascent = 0.0;
        if let Some(base_node) = self.node.children.first() {
            let base = base_node
                .fragment
                .as_deref()
                .expect("radical base has a fragment");
            base_width = base.size.width;
            base_ascent = base.paint.first_baseline.unwrap_or(base.size.height);
        }
        let baseline = paint.first_baseline.unwrap_or(fragment.size.height);
        let block_offset =
            baseline - math.vertical_gap - (base_ascent + math.radical_margin_inline_start);
        let inline_offset =
            paint.border.left + paint.padding.left + math.radical_operator_inline_offset;
        let symbol_x = self.LogicalToPhysicalX(inline_offset, math.operator_inline_size);
        self.PaintOperatorGlyphs(
            info,
            Offset {
                x: symbol_x,
                y: block_offset + math.operator_ascent,
            },
        );
        let bar_width =
            base_width + math.radical_margin_inline_start + math.radical_margin_inline_end;
        let bar_inline = inline_offset + math.operator_inline_size;
        let bar_x = self.LogicalToPhysicalX(bar_inline, bar_width);
        self.PaintBar(
            info,
            PaintRect {
                x: self.node.paint_offset.x + bar_x,
                y: self.node.paint_offset.y + block_offset,
                width: bar_width,
                height: math.rule_thickness,
            },
        );
    }
}
