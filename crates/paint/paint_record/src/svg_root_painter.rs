#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::NodeKind;
use paint_model::PaintRect;
use prepaint::pre_paint_tree_walk::PaintTreeNode;

use crate::paint_info::PaintInfo;

/// SVG-root-specific admission around the shared retained child walk.
///
/// Source: core/paint/svg_root_painter.cc:69-86. Chromium owns child traversal
/// inside `PaintReplaced`; Rechrom's paint_walk owns that traversal, so the
/// source early returns must be exposed as one subtree predicate.
pub struct SVGRootPainter;

impl SVGRootPainter {
    pub fn IsRoot(node: &PaintTreeNode<'_>) -> bool {
        node.fragment
            .as_deref()
            .is_some_and(|fragment| fragment.paint.source_kind == NodeKind::kSvgRoot)
    }

    pub fn ShouldPaintSubtree(node: &PaintTreeNode<'_>, paint_info: &PaintInfo<'_, '_>) -> bool {
        debug_assert!(Self::IsRoot(node));
        let fragment = node
            .fragment
            .as_deref()
            .expect("SVG root must own a fragment");
        let snapped_viewport = node.PixelSnappedRect(&PaintRect {
            x: node.paint_offset.x,
            y: node.paint_offset.y,
            width: fragment.size.width,
            height: fragment.size.height,
        });
        snapped_viewport.width > 0.0
            && snapped_viewport.height > 0.0
            && !fragment
                .paint
                .svg_viewport
                .is_some_and(|viewport| viewport.disabled)
            && !fragment.paint.svg_view_box_is_empty
            && !paint_info.DescendantPaintingBlocked()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use layoutng_assembly::fragment_tree::{FragmentPaintData, SvgViewportGeometry};
    use layoutng_assembly::internal::layout_input::{NodeKind, Offset, Size};
    use prepaint::pre_paint_tree_walk::PaintTreeNode;

    use crate::paint_context::PaintContext;
    use crate::paint_info::PaintInfo;
    use crate::{PaintPhase, PaintRecording};

    use super::SVGRootPainter;

    fn root(fragment: &FragmentPaintData) -> PaintTreeNode<'static> {
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(fragment);
        node
    }

    #[test]
    fn empty_viewport_view_box_and_blocked_descendants_suppress_the_subtree() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.source_kind = NodeKind::kSvgRoot;
        fragment.paint.svg_viewport = Some(SvgViewportGeometry::default());
        fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        let mut node = root(&fragment);
        node.paint_offset = Offset { x: 0.25, y: 0.25 };

        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
        assert!(SVGRootPainter::ShouldPaintSubtree(&node, &paint_info));

        node.fragment.as_mut().unwrap().paint.svg_view_box_is_empty = true;
        assert!(!SVGRootPainter::ShouldPaintSubtree(&node, &paint_info));
        node.fragment.as_mut().unwrap().paint.svg_view_box_is_empty = false;

        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .svg_viewport
            .as_mut()
            .unwrap()
            .disabled = true;
        assert!(!SVGRootPainter::ShouldPaintSubtree(&node, &paint_info));
        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .svg_viewport
            .as_mut()
            .unwrap()
            .disabled = false;

        let mut blocked = paint_info;
        blocked.SetDescendantPaintingBlocked();
        assert!(!SVGRootPainter::ShouldPaintSubtree(&node, &blocked));
    }

    #[test]
    fn pixel_snapped_empty_viewport_suppresses_the_subtree() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.source_kind = NodeKind::kSvgRoot;
        fragment.paint.svg_viewport = Some(SvgViewportGeometry::default());
        fragment.size = Size {
            width: 0.01,
            height: 10.0,
        };
        let node = root(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);

        assert!(!SVGRootPainter::ShouldPaintSubtree(&node, &paint_info));
    }
}
