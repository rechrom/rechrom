#![allow(non_snake_case)]

use std::cell::RefCell;

use crate::box_decoration_data::BoxDecorationData;
use crate::box_fragment_painter::{BoxFragmentPainter, DecorationVisualRect};
use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::paint_context::PaintContext;
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;

// Chromium entity: core/paint/inline_box_fragment_painter.h:154-211.
// The retained FragmentItem identity and stitched decoration geometry cross
// the Layout/Paint boundary; this painter keeps foreground decoration phase
// ownership in paint_record.
pub struct InlineBoxFragmentPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> InlineBoxFragmentPainter<'n, 'f, 'c, 'o> {
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // Chromium: core/paint/inline_box_fragment_painter.cc:61-94,152-214.
    pub fn Paint(
        &self,
        paint_info: &PaintInfo<'c, 'o>,
        box_painter: &BoxFragmentPainter<'n, 'f, 'c, 'o>,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("inline box paint requires a fragment");
        if !fragment.paint.style.visible() || fragment.paint.hidden {
            return;
        }

        let decoration_data = BoxDecorationData::new(self.node, paint_info);
        if !decoration_data.ShouldPaint() {
            return;
        }
        debug_assert!(!decoration_data.HasAppearance());

        if DrawingRecorder::UseCachedDrawingIfPossible(
            self.context,
            self.node,
            DisplayItemIdType::kBoxDecorationBackground,
        ) {
            return;
        }
        let _recorder = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::kBoxDecorationBackground,
            DecorationVisualRect(self.node),
        );
        box_painter.PaintCssBoxDecoration(paint_info, &decoration_data);
    }
}

// Chromium entity: core/paint/inline_box_fragment_painter.h:213-267.
pub struct LineBoxFragmentPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> LineBoxFragmentPainter<'n, 'f, 'c, 'o> {
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // Chromium: core/paint/inline_box_fragment_painter.cc:223-275.
    pub fn Paint(
        &self,
        paint_info: &PaintInfo<'c, 'o>,
        box_painter: &BoxFragmentPainter<'n, 'f, 'c, 'o>,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("line box paint requires a fragment");
        if fragment.paint.first_line_decoration_rect.is_none()
            || fragment.paint.hidden
            || !fragment.paint.style.visible()
        {
            return;
        }
        // Borders are not part of `::first-line` styling
        // (inline_box_fragment_painter.h:233-236).
        let decoration_data = BoxDecorationData::new(self.node, paint_info).WithoutBorder();
        if !decoration_data.ShouldPaint() {
            return;
        }
        if DrawingRecorder::UseCachedDrawingIfPossible(
            self.context,
            self.node,
            DisplayItemIdType::kBoxDecorationBackground,
        ) {
            return;
        }
        let _recorder = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::kBoxDecorationBackground,
            DecorationVisualRect(self.node),
        );
        box_painter.PaintCssBoxDecoration(paint_info, &decoration_data);
    }
}
