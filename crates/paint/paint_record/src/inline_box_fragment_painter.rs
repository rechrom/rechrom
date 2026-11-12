#![allow(non_snake_case)]

use std::cell::RefCell;

use crate::box_decoration_data::BoxDecorationData;
use crate::box_fragment_painter::{BoxFragmentPainter, DecorationVisualRect};
use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::paint_context::PaintContext;
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

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

        // Source: inline_box_fragment_painter.cc:160-171. Tracking is
        // published even when the inline has no authored decoration; the DOM
        // input forces a real inline box fragment instead of a culled inline.
        if !fragment.paint.tracked_element_sub_rects.is_empty() {
            let paint_rect = self.node.EnclosingRect(PaintRect {
                x: self.node.paint_offset.x,
                y: self.node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            });
            self.context.borrow_mut().RecordTrackedElementData(
                self.node,
                paint_rect,
                &fragment.paint.tracked_element_sub_rects,
            );
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
        // Chromium BoxFragmentPainter::PaintLineBox publishes element
        // capture metadata before checking whether this is the decorated
        // ::first-line box. Ordinary line boxes therefore retain their own
        // exact border-box contribution as well.
        box_painter.RecordRegionCaptureAndTrackedElementData();
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use layoutng_assembly::fragment_tree::{FragmentKind, FragmentPaintData};
    use layoutng_assembly::internal::layout_input::{Offset, Size, TrackedElementSubRect};

    use crate::box_fragment_painter::BoxFragmentPainter;
    use crate::paint_context::PaintContext;
    use crate::paint_info::PaintInfo;
    use crate::pre_paint_tree_walk::PaintTreeNode;
    use crate::{PaintPhase, PaintRecording, PaintRect};

    use super::LineBoxFragmentPainter;

    #[test]
    fn ordinary_line_box_publishes_region_capture_before_decoration_early_return() {
        let crop_id = [17, 29];
        let mut fragment = FragmentPaintData::default();
        fragment.kind = FragmentKind::kLine;
        fragment.size = Size {
            width: 31.0,
            height: 13.0,
        };
        fragment.paint.display_item_client_id = 73;
        fragment.paint.region_capture_crop_id = Some(crop_id);
        fragment
            .paint
            .tracked_element_sub_rects
            .push(TrackedElementSubRect {
                feature: 7,
                id: [41, 43],
                should_add_to_compositor_frame_metadata: true,
                should_exclude_fixed_and_sticky_occlusions: false,
                sub_rect: None,
                frame_token: None,
                parent_frame_token: None,
            });
        assert!(fragment.paint.first_line_decoration_rect.is_none());

        let mut node = PaintTreeNode::default();
        node.paint_offset = Offset { x: 5.0, y: 7.0 };
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
            let box_painter = BoxFragmentPainter::new(&node, &context);
            LineBoxFragmentPainter::new(&node, &context).Paint(&paint_info, &box_painter);
            context.borrow_mut().Finish();
        }

        let metadata = recording
            .chunks
            .iter()
            .find_map(|chunk| chunk.compositor.as_deref())
            .and_then(|metadata| metadata.region_capture_data.as_deref())
            .expect("line-box region capture metadata");
        assert_eq!(
            metadata.as_slice(),
            &[(
                crop_id,
                PaintRect {
                    x: 5.0,
                    y: 7.0,
                    width: 31.0,
                    height: 13.0,
                },
            )]
        );
        let tracked = recording
            .chunks
            .iter()
            .find_map(|chunk| {
                chunk
                    .compositor
                    .as_deref()
                    .and_then(|metadata| metadata.tracked_element_rects.as_deref())
            })
            .expect("line-box tracked-element metadata");
        assert_eq!(tracked.len(), 1);
        assert_eq!(tracked[0].id, [41, 43]);
        assert_eq!(
            tracked[0].rect,
            PaintRect {
                x: 5.0,
                y: 7.0,
                width: 31.0,
                height: 13.0,
            }
        );
        assert!(recording.items.is_empty());
    }
}
