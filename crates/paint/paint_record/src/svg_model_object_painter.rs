#![allow(non_snake_case)]

use std::cell::RefCell;

use paint_model::PaintRect;
use prepaint::pre_paint_tree_walk::PaintTreeNode;

use crate::box_fragment_painter::BoxFragmentPainter;
use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::paint_context::PaintContext;
use crate::paint_engine::PaintHitTestOpaqueness;
use crate::paint_info::PaintInfo;
use crate::PaintPhase;

/// Shared SVG model-object paint operations.
///
/// Source: core/paint/svg_model_object_painter.h:16-42 and
/// core/paint/svg_model_object_painter.cc:15-27,55-71.
pub struct SVGModelObjectPainter;

impl SVGModelObjectPainter {
    /// Source: svg_model_object_painter.cc:15-27.
    pub fn CanUseCullRect(node: &PaintTreeNode<'_>) -> bool {
        let fragment = node
            .fragment
            .as_deref()
            .expect("SVG model object must own a fragment");
        let style = &fragment.paint.style;
        let has_transform = style.has_translate()
            || style.has_rotate()
            || style.has_scale()
            || style.offset_transform.is_some()
            || style.has_transform();
        if has_transform {
            return false;
        }
        !style.filter_moves_pixels()
    }

    /// Chromium VisualRectInLocalSVGCoordinates projected through the existing
    /// physical-fragment boundary. An absent source overflow remains unknown;
    /// callers must not substitute the frame rect for culling.
    pub fn VisualRect(node: &PaintTreeNode<'_>) -> Option<PaintRect> {
        let fragment = node.fragment.as_deref()?;
        if let Some(rect) = fragment.paint.svg_visual_rect {
            return Some(PaintRect {
                x: node.paint_offset.x + rect.offset.x,
                y: node.paint_offset.y + rect.offset.y,
                width: rect.size.width,
                height: rect.size.height,
            });
        }
        // LayoutSVGModelObject::VisualRectInLocalSVGCoordinates() returns the
        // decorated bounding box, not the anonymous NG box's (often empty)
        // ink overflow. Keep that source distinction for translated shapes
        // even when the Layout paint-geometry query was unavailable.
        // Chromium: layout_svg_model_object.h:48-52 and
        // layout_svg_shape.cc:353-400.
        if let Some(shape) = fragment.paint.svg_shape.as_deref() {
            let style = &fragment.paint.style;
            let has_stroke = style.svg_stroke_server.is_some()
                || style.svg_stroke_reference.is_some()
                || style.svg_stroke.is_some_and(|stroke| stroke.alpha > 0.0);
            let outset = if has_stroke && style.svg_stroke_width > 0.0 {
                // The exact stroked path remains in the raster record. This
                // conservative miter extent is only the DrawingRecorder/cull
                // bound and therefore cannot alter the rendered geometry.
                style.svg_stroke_width * 0.5 * style.svg_stroke_miter_limit.max(1.0)
            } else {
                0.0
            };
            return Some(PaintRect {
                x: node.paint_offset.x + shape.bounds_offset.x - outset,
                y: node.paint_offset.y + shape.bounds_offset.y - outset,
                width: shape.bounds_width + outset * 2.0,
                height: shape.bounds_height + outset * 2.0,
            });
        }
        let rect = fragment
            .paint
            .svg_visual_rect
            .or(fragment.paint.ink_overflow)
            .or(fragment.paint.contents_ink_overflow)
            .or(fragment.paint.self_ink_overflow)
            // Rechrom's source SVG shape object is a LayoutBlockFlow.  Its
            // LayoutBox::SelfVisualOverflowRect() is exported separately from
            // PhysicalFragment ink overflow and is the retained source value
            // corresponding to LayoutSVGModelObject::DecoratedBoundingBox().
            // Do not substitute the fragment frame when neither exists.
            .or(fragment.paint.box_self_visual_overflow)?;
        Some(PaintRect {
            x: node.paint_offset.x + rect.offset.x,
            y: node.paint_offset.y + rect.offset.y,
            width: rect.size.width,
            height: rect.size.height,
        })
    }

    /// Source: svg_model_object_painter.cc:29-40 and
    /// object_painter.cc:142-178. SVG children are always mixed when visible
    /// because fill/stroke hit rules cannot prove the whole visual rect opaque.
    pub fn RecordHitTestData(
        node: &PaintTreeNode<'_>,
        context: &RefCell<PaintContext<'_>>,
        paint_info: &PaintInfo<'_, '_>,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("SVG hit-test object must own a fragment");
        debug_assert!(fragment.paint.is_svg_child);
        debug_assert_eq!(paint_info.phase, PaintPhase::kForeground);
        if paint_info.ShouldOmitCompositingInfo() || !fragment.paint.style.visible() {
            return;
        }
        let Some(rect) = Self::VisualRect(node) else {
            return;
        };
        let opaqueness = if fragment.paint.visible_to_hit_testing {
            PaintHitTestOpaqueness::Mixed
        } else {
            PaintHitTestOpaqueness::Transparent
        };
        context
            .borrow_mut()
            .RecordHitTestData(node, node.EnclosingRect(rect), opaqueness);
    }

    /// Source: svg_model_object_painter.cc:40-53. The DOM Element relation is
    /// resolved by Layout; Paint receives only the optional non-null crop ID.
    pub fn RecordRegionCaptureData(node: &PaintTreeNode<'_>, context: &RefCell<PaintContext<'_>>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("SVG region-capture object must own a fragment");
        let Some(crop_id) = fragment.paint.region_capture_crop_id else {
            return;
        };
        let Some(rect) = Self::VisualRect(node) else {
            return;
        };
        context
            .borrow_mut()
            .RecordRegionCaptureData(node, crop_id, node.EnclosingRect(rect));
    }

    /// Source: svg_model_object_painter.cc:55-71.
    pub fn PaintOutline<'n, 'f, 'c, 'o>(
        node: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
    ) {
        BoxFragmentPainter::PaintSVGOutline(node, context);
    }
}

/// Source: svg_model_object_painter.h:44-75.
///
/// SVG children use Layout's source visual overflow as the DrawingRecorder
/// bounds. Containers are excluded because they record effects around child
/// painters and compute their bounds at that scope instead.
pub(crate) struct SVGDrawingRecorder<'c, 'o> {
    _recorder: DrawingRecorder<'c, 'o>,
}

impl<'c, 'o> SVGDrawingRecorder<'c, 'o> {
    pub(crate) fn new(
        context: &'c RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
    ) -> Self {
        let fragment = node
            .fragment
            .as_deref()
            .expect("SVG drawing owner must have a fragment");
        if !matches!(
            fragment.paint.source_kind,
            layoutng_assembly::internal::layout_input::NodeKind::kSvgShape
                | layoutng_assembly::internal::layout_input::NodeKind::kSvgText
                | layoutng_assembly::internal::layout_input::NodeKind::kSvgInline
                | layoutng_assembly::internal::layout_input::NodeKind::kSvgTSpan
                | layoutng_assembly::internal::layout_input::NodeKind::kSvgTextPath
        ) {
            panic!("SVGDrawingRecorder requires a non-container SVG child");
        }
        let visual_rect = SVGModelObjectPainter::VisualRect(node).unwrap_or_else(|| {
            std::panic::panic_any(foundation::UnsupportedLayout::new(
                "source SVG visual rect required by SVGDrawingRecorder",
            ))
        });
        Self {
            _recorder: DrawingRecorder::new(
                context,
                node,
                item_type,
                node.EnclosingRect(visual_rect),
            ),
        }
    }

    pub(crate) fn for_phase(
        context: &'c RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        phase: PaintPhase,
    ) -> Self {
        Self::new(
            context,
            node,
            DisplayItemIdType::PaintPhaseToDrawingType(phase),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::fragment_tree::FragmentPaintData;
    use layoutng_assembly::internal::paint_input::{PaintFilterOperation, PaintFilterType};

    fn node() -> PaintTreeNode<'static> {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node
    }

    #[test]
    fn transforms_and_pixel_moving_filters_disable_svg_culling() {
        let mut node = node();
        assert!(SVGModelObjectPainter::CanUseCullRect(&node));

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
        assert!(!SVGModelObjectPainter::CanUseCullRect(&node));
    }

    #[test]
    fn svg_drawing_recorder_uses_source_visual_overflow() {
        use crate::PaintRecording;
        use layoutng_assembly::fragment_tree::FragmentPaintRect;
        use layoutng_assembly::internal::layout_input::{NodeKind, Offset, Size};

        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.source_kind = NodeKind::kSvgShape;
        fragment.paint.self_ink_overflow = Some(FragmentPaintRect {
            offset: Offset { x: -1.25, y: 2.5 },
            size: Size {
                width: 10.5,
                height: 20.25,
            },
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut output = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut output));
            let _recorder = SVGDrawingRecorder::for_phase(&context, &node, PaintPhase::kForeground);
        }
        assert_eq!(output.display_items.len(), 1);
        assert_eq!(
            output.display_items[0].visual_rect,
            PaintRect {
                x: -2.0,
                y: 2.0,
                width: 12.0,
                height: 21.0,
            }
        );
    }

    #[test]
    fn svg_visual_rect_uses_source_layout_box_visual_overflow_when_ink_is_unset() {
        use layoutng_assembly::fragment_tree::FragmentPaintRect;
        use layoutng_assembly::internal::layout_input::{Offset, Size};

        let mut fragment = FragmentPaintData::default();
        fragment.paint.box_self_visual_overflow = Some(FragmentPaintRect {
            offset: Offset { x: -2.0, y: 3.0 },
            size: Size {
                width: 14.0,
                height: 9.0,
            },
        });
        let mut node = PaintTreeNode::default();
        node.paint_offset = Offset { x: 5.0, y: 7.0 };
        node.SetOwnedFragmentForTesting(&fragment);

        assert_eq!(
            SVGModelObjectPainter::VisualRect(&node),
            Some(PaintRect {
                x: 3.0,
                y: 10.0,
                width: 14.0,
                height: 9.0,
            })
        );
    }
}
