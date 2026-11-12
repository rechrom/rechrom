#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input::NodeKind;
use layoutng_assembly::internal::paint_input::PaintFilterType;
use paint_model::PaintRect;
use prepaint::pre_paint_tree_walk::PaintTreeNode;

use crate::paint_context::PaintContext;
use crate::svg_model_object_painter::SVGModelObjectPainter;

/// The container-specific tail of Blink's SVGContainerPainter.
///
/// Source: core/paint/svg_container_painter.h:15-28 and
/// core/paint/svg_container_painter.cc:148-152. Transform, effect and child
/// traversal scopes are shared with the translated retained paint walk, while
/// the container keeps ownership of its content-dependent outline decision.
pub struct SVGContainerPainter;

impl SVGContainerPainter {
    pub fn IsContainer(node: &PaintTreeNode<'_>) -> bool {
        node.fragment.as_deref().is_some_and(|fragment| {
            matches!(
                fragment.paint.source_kind,
                NodeKind::kSvgGroup
                    | NodeKind::kSvgResourceMarker
                    | NodeKind::kSvgViewportContainer
            )
        })
    }

    fn HasReferenceFilter(node: &PaintTreeNode<'_>) -> bool {
        node.fragment.as_deref().is_some_and(|fragment| {
            fragment
                .paint
                .style
                .filters
                .iter()
                .any(|operation| operation.r#type == PaintFilterType::kReference)
        })
    }

    /// Source: core/paint/svg_container_painter.cc:36-90 and
    /// scoped_svg_paint_state.cc:186-209.
    pub fn ShouldPaintSubtree(node: &PaintTreeNode<'_>) -> bool {
        debug_assert!(Self::IsContainer(node));
        let fragment = node
            .fragment
            .as_deref()
            .expect("SVG container must own a fragment");
        if fragment.paint.svg_view_box_is_empty {
            return false;
        }

        // SvgFilterPaintsForHiddenContent is stable in this Chromium revision:
        // an empty container paints only when its reference filter still needs
        // a property-bearing chunk.
        if node.children.is_empty() && !Self::HasReferenceFilter(node) {
            return false;
        }

        if fragment.paint.svg_hidden_container
            || fragment
                .paint
                .svg_descendant_may_have_transform_related_operations
            || !SVGModelObjectPainter::CanUseCullRect(node)
        {
            return true;
        }
        let (Some(cull_rect), Some(visual_rect)) =
            (node.self_cull_rect, SVGModelObjectPainter::VisualRect(node))
        else {
            return true;
        };
        // ComputeFragmentCullRect stores the inverse-mapped cull in the
        // container's LocalBorderBoxProperties space. VisualRect is already
        // expressed in that same local space, matching Chromium's
        // IntersectsTransformed(LocalToSVGParentTransform, local visual rect)
        // after the property-tree mapping has been consumed by PrePaint.
        Intersects(visual_rect, cull_rect)
    }

    /// Source: core/paint/svg_container_painter.cc:111-117.
    pub fn NeedsReferenceFilterChunk(node: &PaintTreeNode<'_>) -> bool {
        Self::IsContainer(node) && Self::HasReferenceFilter(node)
    }

    pub fn PaintOutlineAfterEffects<'n, 'f, 'c, 'o>(
        node: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
    ) {
        // SVGContainerPainter::Paint only paints an outline if the container
        // has content. This decision is outside ScopedSVGPaintState, after the
        // mask/effect lifetime and while the SVG transform remains active.
        if !node.children.is_empty() {
            SVGModelObjectPainter::PaintOutline(node, context);
        }
    }
}

fn Intersects(a: PaintRect, b: PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint_engine::DisplayItemType;
    use crate::PaintRecording;
    use layoutng_assembly::fragment_tree::{FragmentPaintData, FragmentPaintRect};
    use layoutng_assembly::internal::layout_input::{BorderLineStyle, Offset, Size};
    use layoutng_assembly::internal::layout_input_types::Color;
    use layoutng_assembly::internal::paint_input::PaintFilterOperation;
    use paint_model::PaintRect;

    fn container() -> PaintTreeNode<'static> {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.source_kind = NodeKind::kSvgGroup;
        fragment.paint.ink_overflow = Some(FragmentPaintRect {
            size: Size {
                width: 10.0,
                height: 10.0,
            },
            ..FragmentPaintRect::default()
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node
    }

    #[test]
    fn container_identity_does_not_capture_shapes() {
        let mut node = container();
        assert!(SVGContainerPainter::IsContainer(&node));
        node.fragment.as_mut().unwrap().paint.source_kind = NodeKind::kSvgShape;
        assert!(!SVGContainerPainter::IsContainer(&node));
    }

    #[test]
    fn container_outline_requires_a_child() {
        let mut node = container();
        let fragment = node.fragment.as_mut().unwrap();
        fragment.paint.style.outline_width = 1.0;
        fragment.paint.style.outline_style = BorderLineStyle::kSolid;
        fragment.paint.style.outline_color = Color {
            alpha: 1.0,
            ..Color::default()
        };
        node.outline_rects.push(PaintRect {
            width: 10.0,
            height: 10.0,
            ..PaintRect::default()
        });

        let mut empty_output = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut empty_output));
            SVGContainerPainter::PaintOutlineAfterEffects(&node, &context);
            context.borrow_mut().Finish();
        }
        assert!(!empty_output
            .items
            .iter()
            .any(|item| item.r#type == DisplayItemType::kStrokeRect));

        node.children.push(Box::new(PaintTreeNode::default()));
        let mut populated_output = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut populated_output));
            SVGContainerPainter::PaintOutlineAfterEffects(&node, &context);
            context.borrow_mut().Finish();
        }
        assert!(populated_output
            .items
            .iter()
            .any(|item| item.r#type == DisplayItemType::kStrokeRect));
    }

    #[test]
    fn empty_view_box_and_empty_content_skip_the_container() {
        let mut node = container();
        node.fragment.as_mut().unwrap().paint.svg_view_box_is_empty = true;
        node.children.push(Box::new(PaintTreeNode::default()));
        assert!(!SVGContainerPainter::ShouldPaintSubtree(&node));

        node.fragment.as_mut().unwrap().paint.svg_view_box_is_empty = false;
        node.children.clear();
        assert!(!SVGContainerPainter::ShouldPaintSubtree(&node));

        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .style
            .filters
            .push(PaintFilterOperation {
                r#type: PaintFilterType::kReference,
                ..PaintFilterOperation::default()
            });
        assert!(SVGContainerPainter::ShouldPaintSubtree(&node));
        assert!(SVGContainerPainter::NeedsReferenceFilterChunk(&node));
    }

    #[test]
    fn finite_cull_is_disabled_by_source_transform_aggregate() {
        let mut node = container();
        node.children.push(Box::new(PaintTreeNode::default()));
        node.self_cull_rect = Some(PaintRect {
            x: 100.0,
            y: 100.0,
            width: 20.0,
            height: 20.0,
        });
        node.fragment.as_mut().unwrap().paint.ink_overflow = Some(FragmentPaintRect {
            offset: Offset::default(),
            size: Size {
                width: 10.0,
                height: 10.0,
            },
        });
        assert!(!SVGContainerPainter::ShouldPaintSubtree(&node));

        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .svg_descendant_may_have_transform_related_operations = true;
        assert!(SVGContainerPainter::ShouldPaintSubtree(&node));
    }
}
