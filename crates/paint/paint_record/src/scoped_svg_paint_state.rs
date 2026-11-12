#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input::{NodeKind, SvgShapeGeometry};
use layoutng_assembly::internal::paint_input::PaintFilterType;
use prepaint::pre_paint_tree_walk::PaintTreeNode;

use crate::display_item_id::DisplayItemIdType;
use crate::paint_context::{PaintContext, ScopedPaintChunkProperties};
use crate::paint_info::PaintInfo;
use crate::svg_mask_painter::SVGMaskPainter;
use crate::PaintPhase;

/// The ordinary SVG-child effect lifetime of Blink's ScopedSVGPaintState.
///
/// Source: core/paint/scoped_svg_paint_state.cc:84-100,102-139. Ordinary SVG
/// children apply effects while their SVG painter walks content, then record
/// the mask source before leaving that effect scope. The concrete SVG painter
/// remains responsible for deciding whether content and outline exist. SVG
/// roots and foreignObject use PaintLayerPainter instead.
pub struct ScopedSVGPaintState;

impl ScopedSVGPaintState {
    pub fn OwnsPaintScope(node: &PaintTreeNode<'_>) -> bool {
        if node.is_paint_layer {
            return false;
        }
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        matches!(
            fragment.paint.source_kind,
            NodeKind::kSvgGroup
                | NodeKind::kSvgResourceMarker
                | NodeKind::kSvgText
                | NodeKind::kSvgInline
                | NodeKind::kSvgTSpan
                | NodeKind::kSvgTextPath
                | NodeKind::kSvgShape
                | NodeKind::kSvgViewportContainer
        )
    }

    pub fn PaintsPhase(node: &PaintTreeNode<'_>, phase: PaintPhase) -> bool {
        !Self::OwnsPaintScope(node) || phase == PaintPhase::kForeground
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

    /// Source: scoped_svg_paint_state.cc:186-210 and
    /// svg_shape_painter.cc:43-57. Shape emptiness is disabled rendering and
    /// is rejected before paint-behavior selection; visibility-hidden shapes
    /// retain only their reference-filter component.
    pub fn ShouldPaint(node: &PaintTreeNode<'_>, paint_info: &PaintInfo<'_, '_>) -> bool {
        if !Self::OwnsPaintScope(node) {
            return true;
        }
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if fragment.paint.source_kind != NodeKind::kSvgShape {
            return true;
        }
        let Some(shape) = fragment.paint.svg_shape.as_deref() else {
            return false;
        };
        let empty = if shape.geometry == SvgShapeGeometry::kPath {
            shape.path.is_empty()
        } else {
            shape.bounds_width <= 0.0 || shape.bounds_height <= 0.0
        };
        if empty {
            return false;
        }
        fragment.paint.style.visible() || Self::NeedsReferenceFilterChunk(node, paint_info)
    }

    pub fn PaintsContent(node: &PaintTreeNode<'_>) -> bool {
        node.fragment.as_deref().is_none_or(|fragment| {
            fragment.paint.source_kind != NodeKind::kSvgShape || fragment.paint.style.visible()
        })
    }

    pub fn NeedsReferenceFilterChunk(
        node: &PaintTreeNode<'_>,
        paint_info: &PaintInfo<'_, '_>,
    ) -> bool {
        paint_info.phase == PaintPhase::kForeground
            && !paint_info.IsRenderingClipPathAsMaskImage()
            && node.fragment.as_deref().is_some_and(|fragment| {
                fragment.paint.source_kind == NodeKind::kSvgShape
                    && !fragment.paint.style.visible()
                    && Self::HasReferenceFilter(node)
            })
    }

    pub fn FinishEffectsAfterContents<'n, 'f, 'c, 'o>(
        node: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
    ) {
        // The mask source is a sibling of the content effect's filter and
        // MaskClip scopes. Close those flat recording scopes before selecting
        // the retained Mask property state, exactly as the PaintLayer path
        // does for CSS boxes.
        {
            let mut context = context.borrow_mut();
            context.EndFilter(node);
            context.EndCssClip(node);
            context.EndMaskClip(node);
        }
        if node.applies_mask {
            let _mask_properties = ScopedPaintChunkProperties::mask_with_type(
                context,
                node,
                DisplayItemIdType::kSVGMask,
            );
            SVGMaskPainter::Paint(node, context);
        }
        context.borrow_mut().EndEffectsAfterFilter(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint_engine::DisplayItemType;
    use crate::PaintRecording;
    use layoutng_assembly::fragment_tree::{FragmentPaintData, FragmentPaintRect};
    use layoutng_assembly::internal::layout_input::{BorderLineStyle, Size};
    use layoutng_assembly::internal::layout_input_types::Color;
    use paint_model::{DisplayMaskLayer, PaintRect};

    #[test]
    fn no_layer_svg_mask_uses_svg_mask_identity_after_content_effects() {
        let mut fragment = FragmentPaintData::default();
        fragment.node_id = 7;
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.source_kind = NodeKind::kSvgShape;
        fragment.paint.display_item_client_id = 19;
        fragment.paint.style.outline_width = 1.0;
        fragment.paint.style.outline_color = Color {
            alpha: 1.0,
            ..Color::default()
        };
        fragment.paint.style.outline_style = BorderLineStyle::kSolid;
        fragment.paint.ink_overflow = Some(FragmentPaintRect {
            size: Size {
                width: 10.0,
                height: 10.0,
            },
            ..FragmentPaintRect::default()
        });

        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.applies_mask = true;
        node.mask_layers.push(DisplayMaskLayer {
            clip_rect: PaintRect {
                width: 20.0,
                height: 20.0,
                ..PaintRect::default()
            },
            ..DisplayMaskLayer::default()
        });
        node.mask_properties = Some(Default::default());
        node.outline_rects.push(PaintRect {
            width: 10.0,
            height: 10.0,
            ..PaintRect::default()
        });

        assert!(ScopedSVGPaintState::OwnsPaintScope(&node));
        assert!(ScopedSVGPaintState::PaintsPhase(
            &node,
            PaintPhase::kForeground
        ));
        assert!(!ScopedSVGPaintState::PaintsPhase(
            &node,
            PaintPhase::kOutline
        ));
        assert!(!node.is_paint_layer);

        let mut output = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut output));
            context.borrow_mut().BeginNode(&node);
            context.borrow_mut().EndEscapableGeometry(&node);
            ScopedSVGPaintState::FinishEffectsAfterContents(&node, &context);
            crate::svg_model_object_painter::SVGModelObjectPainter::PaintOutline(&node, &context);
            context.borrow_mut().EndGeometry(&node);
            context.borrow_mut().Finish();
        }

        assert!(output
            .display_items
            .iter()
            .any(|item| item.id.r#type == DisplayItemIdType::kSVGMask));
        assert!(output.items.iter().any(|item| {
            item.node_id == fragment.node_id && item.r#type == DisplayItemType::kDrawMask
        }));
        let mask = output
            .items
            .iter()
            .position(|item| item.r#type == DisplayItemType::kDrawMask)
            .unwrap();
        let outline = output
            .items
            .iter()
            .enumerate()
            .skip(mask + 1)
            .find(|(_, item)| {
                item.node_id == fragment.node_id
                    && item.phase == PaintPhase::kSelfOutlineOnly
                    && (item.r#type as u8) >= (DisplayItemType::kDrawRect as u8)
            })
            .map(|(index, _)| index)
            .expect("SVG outline follows the mask source");
        assert!(mask < outline);
    }

    #[test]
    fn shape_disabled_rendering_precedes_reference_filter_behavior() {
        use layoutng_assembly::internal::layout_input::SvgShapeData;
        use layoutng_assembly::internal::paint_input::PaintFilterOperation;
        use paint_model::paint_flags::PaintFlag;

        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.source_kind = NodeKind::kSvgShape;
        fragment.paint.svg_shape = Some(std::sync::Arc::new(SvgShapeData::default()));
        fragment.paint.style.SetVisibleForTesting(false);
        fragment.paint.style.filters.push(PaintFilterOperation {
            r#type: PaintFilterType::kReference,
            ..PaintFilterOperation::default()
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        let paint_info = crate::paint_info::PaintInfo::new(&context, PaintPhase::kForeground);
        assert!(!ScopedSVGPaintState::ShouldPaint(&node, &paint_info));

        let shape = std::sync::Arc::make_mut(
            node.fragment
                .as_mut()
                .unwrap()
                .paint
                .svg_shape
                .as_mut()
                .unwrap(),
        );
        shape.geometry = SvgShapeGeometry::kRectangle;
        shape.bounds_width = 10.0;
        shape.bounds_height = 10.0;
        assert!(ScopedSVGPaintState::ShouldPaint(&node, &paint_info));
        assert!(!ScopedSVGPaintState::PaintsContent(&node));
        assert!(ScopedSVGPaintState::NeedsReferenceFilterChunk(
            &node,
            &paint_info
        ));

        let clip_info = crate::paint_info::PaintInfo::new_with_flags(
            &context,
            PaintPhase::kForeground,
            false,
            PaintFlag::kPaintingClipPathAsMask,
        );
        assert!(!ScopedSVGPaintState::ShouldPaint(&node, &clip_info));
        assert!(!ScopedSVGPaintState::NeedsReferenceFilterChunk(
            &node, &clip_info
        ));

        node.fragment.as_mut().unwrap().paint.style.filters.clear();
        assert!(!ScopedSVGPaintState::ShouldPaint(&node, &paint_info));
    }
}
