#![allow(non_snake_case)]

use std::cell::RefCell;

use prepaint::pre_paint_tree_walk::PaintTreeNode;

use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, PaintPhase};

/// Source: core/paint/svg_mask_painter.h:24-44 and
/// core/paint/svg_mask_painter.cc:239-271.
pub struct SVGMaskPainter;

impl SVGMaskPainter {
    pub fn Paint<'n, 'f, 'c, 'o>(
        node: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
    ) {
        if !node.applies_mask || node.mask_layers.is_empty() {
            return;
        }
        if DrawingRecorder::UseCachedDrawingIfPossible(context, node, DisplayItemIdType::kSVGMask) {
            return;
        }

        // ObjectPaintProperties::MaskClip uses
        // ResourceBoundsForSVGChild: resolved SVG mask regions united with the
        // object's local SVG visual rect. CSS mask-border belongs to the
        // PaintLayer mask path.
        let visual_rect = node
            .ResolvedMaskClipRect()
            .expect("SVG mask must have resolved MaskClip bounds");
        let fragment = node
            .fragment
            .as_deref()
            .expect("SVG mask owner must have a fragment");
        let _recorder =
            DrawingRecorder::new(context, node, DisplayItemIdType::kSVGMask, visual_rect);
        context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawMask,
                phase: PaintPhase::kMask,
                node_id: fragment.node_id,
                rect: visual_rect,
                mask_layers: node.mask_layers.clone(),
                ..DisplayItem::default()
            },
            node,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PaintRecording;
    use layoutng_assembly::fragment_tree::FragmentPaintData;
    use paint_model::{DisplayMaskLayer, PaintRect};

    #[test]
    fn svg_mask_visual_rect_is_the_mask_clip_union() {
        let mut fragment = FragmentPaintData::default();
        fragment.node_id = 11;
        fragment.paint.has_source = true;
        fragment.paint.display_item_client_id = 19;
        fragment.paint.source_kind = layoutng_assembly::internal::layout_input::NodeKind::kSvgShape;
        fragment.paint.self_ink_overflow =
            Some(layoutng_assembly::fragment_tree::FragmentPaintRect {
                offset: layoutng_assembly::internal::layout_input::Offset { x: -5.0, y: 0.0 },
                size: layoutng_assembly::internal::layout_input::Size {
                    width: 2.0,
                    height: 2.0,
                },
            });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.applies_mask = true;
        node.mask_layers = vec![
            DisplayMaskLayer {
                clip_rect: PaintRect {
                    x: 2.0,
                    y: 3.0,
                    width: 4.0,
                    height: 5.0,
                },
                ..DisplayMaskLayer::default()
            },
            DisplayMaskLayer {
                clip_rect: PaintRect {
                    x: 10.0,
                    y: 1.0,
                    width: 2.0,
                    height: 4.0,
                },
                ..DisplayMaskLayer::default()
            },
        ];

        let mut output = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut output));
            SVGMaskPainter::Paint(&node, &context);
            context.borrow_mut().Finish();
        }
        let mask = output
            .items
            .iter()
            .find(|item| item.r#type == DisplayItemType::kDrawMask)
            .unwrap();
        assert_eq!(
            mask.rect,
            PaintRect {
                x: -5.0,
                y: 0.0,
                width: 17.0,
                height: 8.0,
            }
        );
        assert!(output
            .display_items
            .iter()
            .any(|item| item.id.r#type == DisplayItemIdType::kSVGMask));
    }

    #[test]
    fn svg_mask_visual_rect_excludes_css_mask_border() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.source_kind = layoutng_assembly::internal::layout_input::NodeKind::kSvgShape;
        fragment.paint.self_ink_overflow =
            Some(layoutng_assembly::fragment_tree::FragmentPaintRect {
                offset: layoutng_assembly::internal::layout_input::Offset { x: 1.0, y: 2.0 },
                size: layoutng_assembly::internal::layout_input::Size {
                    width: 3.0,
                    height: 4.0,
                },
            });
        fragment.paint.style.mask_box_image = Some(Default::default());
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.mask_layers.push(DisplayMaskLayer {
            clip_rect: PaintRect {
                x: 10.0,
                y: 20.0,
                width: 5.0,
                height: 6.0,
            },
            ..DisplayMaskLayer::default()
        });

        assert_eq!(
            node.ResolvedMaskClipRect(),
            Some(PaintRect {
                x: 1.0,
                y: 2.0,
                width: 14.0,
                height: 24.0,
            })
        );
    }
}
