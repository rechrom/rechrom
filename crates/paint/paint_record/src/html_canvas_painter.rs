//! Unaccelerated HTMLCanvasPainter snapshot recording.
//!
//! Source: third_party/blink/renderer/core/paint/html_canvas_painter.cc:20-105.

#![allow(non_snake_case)]

use std::cell::RefCell;

use crate::box_fragment_painter::IsVisible;
use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::foreign_layer_display_item::RecordForeignLayer;
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, RasterEffectOutset};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;
use foundation::InterpolationQuality;
use paint_chunk::DisplayItemClientInfo;
use paint_model::display_item_id::DisplayItemId;

pub struct HTMLCanvasPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> HTMLCanvasPainter<'n, 'f, 'c, 'o> {
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    /// Records Chromium's ordinary, unaccelerated canvas snapshot path.
    /// Foreign-layer canvases stay a later compositor resource boundary.
    pub fn PaintReplaced(&self, paint_info: &PaintInfo<'_, '_>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.ChildPaintBlockedByDisplayLock() {
            return;
        }
        let replaced = fragment
            .paint
            .replaced_content
            .as_ref()
            .expect("LayoutHTMLCanvas must publish replaced-content geometry");
        debug_assert!(replaced.is_canvas);
        let resource = fragment.paint.resources.as_deref().and_then(|resources| {
            resources
                .canvases
                .iter()
                .find(|resource| resource.dom_node_id == fragment.node_id)
        });
        // html_canvas_painter.cc:43-46.
        if paint_info.IsPrivacyPreserving()
            && resource.is_none_or(|resource| !resource.origin_clean)
        {
            return;
        }
        if resource.is_some_and(|resource| !resource.is_clear) {
            self.context.borrow_mut().SetFirstContentfulPaint();
        }
        // html_canvas_painter.cc:40-41 keeps the unaccelerated paint rect in
        // physical coordinates. Only the foreign-layer branch pixel-snaps.
        let destination = PaintRect {
            x: self.node.paint_offset.x + replaced.offset.x,
            y: self.node.paint_offset.y + replaced.offset.y,
            width: replaced.size.width,
            height: replaced.size.height,
        };
        if destination.width <= 0.0
            || destination.height <= 0.0
            || !IsVisible(self.node, &destination)
        {
            return;
        }
        // html_canvas_painter.cc:53-67. Canvas subtree display lists keep a
        // DOM-node placeholder rather than embedding a potentially accelerated
        // child snapshot. PaintArtifactCompositor resolves it recursively when
        // the parent canvas requests its child PaintRecord.
        if fragment.paint.is_in_canvas_subtree {
            let item_type = DisplayItemIdType::PaintPhaseToDrawingType(paint_info.phase);
            if DrawingRecorder::UseCachedDrawingIfPossible(self.context, self.node, item_type) {
                return;
            }
            let _drawing = DrawingRecorder::new(
                self.context,
                self.node,
                item_type,
                self.node.EnclosingRect(destination),
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kCustomData,
                    phase: paint_info.phase,
                    node_id: fragment.node_id,
                    custom_data_id: u32::try_from(fragment.node_id)
                        .expect("DOMNodeId must fit CustomDataOp's uint32 payload"),
                    ..Default::default()
                },
                self.node,
            );
            return;
        }
        // html_canvas_painter.cc:69-92. Printing/capture deliberately falls
        // through to the vector/software snapshot path. Nested canvases use
        // the separate CustomData substitution protocol and cannot publish a
        // foreign layer from inside that subtree.
        let layer = { self.context.borrow().ForeignLayerForNode(fragment.node_id) };
        if let Some(layer) = layer {
            // html_canvas_painter.cc:74-79. Color::Rgb() quantizes the
            // resolved CSS color to SkColor before SkColor4f::FromColor.
            if replaced.draws_background_onto_content_layer {
                let mut color = fragment.paint.style.background_color();
                let quantize = |component: f32| (component.clamp(0.0, 1.0) * 255.0).round() / 255.0;
                color.red = quantize(color.red);
                color.green = quantize(color.green);
                color.blue = quantize(color.blue);
                color.alpha = quantize(color.alpha);
                layer.SetBackgroundColor(color);
            }
            if !paint_info.ShouldOmitCompositingInfo() {
                let destination = self.node.PixelSnappedRect(&destination);
                layer.SetBounds(crate::ForeignLayerSize {
                    width: destination.width,
                    height: destination.height,
                });
                layer.SetIsDrawable(true);
                layer.SetHitTestable(true);
                let id = DisplayItemId {
                    client_id: fragment.paint.display_item_client_id,
                    r#type: DisplayItemIdType::kForeignLayerCanvas,
                    fragment: fragment.paint.display_item_fragment,
                };
                RecordForeignLayer(
                    &mut self.context.borrow_mut(),
                    id,
                    DisplayItemClientInfo::from_fragment(fragment),
                    layer,
                    layoutng_assembly::internal::layout_input::Offset {
                        x: destination.x,
                        y: destination.y,
                    },
                    RasterEffectOutset::kNone,
                    None,
                );
                return;
            }
        }
        let Some(resource) = resource else {
            return;
        };
        if resource.width == 0 || resource.height == 0 {
            return;
        }
        let item_type = DisplayItemIdType::PaintPhaseToDrawingType(paint_info.phase);
        if DrawingRecorder::UseCachedDrawingIfPossible(self.context, self.node, item_type) {
            return;
        }
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            item_type,
            self.node.EnclosingRect(destination),
        );
        // html_canvas_painter.cc:20-32. Canvas defaults to low interpolation;
        // pixelated/crisp-edges retain nearest-neighbour sampling.
        let image_interpolation_quality = match fragment.paint.style.image_interpolation_quality() {
            InterpolationQuality::kInterpolationNone => InterpolationQuality::kInterpolationNone,
            InterpolationQuality::kInterpolationLow
            | InterpolationQuality::kInterpolationMedium => InterpolationQuality::kInterpolationLow,
        };
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawImageRect,
                phase: paint_info.phase,
                node_id: fragment.node_id,
                rect: destination,
                source_rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: f64::from(resource.width),
                    height: f64::from(resource.height),
                },
                resource_id: resource.resource_id,
                image_interpolation_quality,
                dynamic_range_limit: fragment.paint.style.dynamic_range_limit(),
                ..Default::default()
            },
            self.node,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint_context::PaintContext;
    use crate::{PaintPhase, PaintRecording};
    use canvas::CanvasPaintResource;
    use layoutng_assembly::fragment_tree::{
        FragmentPaintData, PaintResources, ReplacedContentPaintData,
    };
    use layoutng_assembly::internal::layout_input::{Offset, Size};
    use paint_model::paint_flags::PaintFlag;
    use std::{
        any::Any,
        sync::{Arc, Mutex},
    };

    #[derive(Default)]
    struct CanvasLayerState {
        bounds: crate::ForeignLayerSize,
        drawable: bool,
        hit_testable: bool,
        background_color: Option<foundation::PaintColor>,
    }

    struct CanvasLayer {
        state: Mutex<CanvasLayerState>,
    }

    impl crate::ForeignLayer for CanvasLayer {
        fn LayerId(&self) -> i32 {
            55
        }

        fn Bounds(&self) -> crate::ForeignLayerSize {
            self.state.lock().unwrap().bounds
        }

        fn SetBounds(&self, bounds: crate::ForeignLayerSize) {
            self.state.lock().unwrap().bounds = bounds;
        }

        fn SetIsDrawable(&self, drawable: bool) {
            self.state.lock().unwrap().drawable = drawable;
        }

        fn SetHitTestable(&self, hit_testable: bool) {
            self.state.lock().unwrap().hit_testable = hit_testable;
        }

        fn SetBackgroundColor(&self, color: foundation::PaintColor) {
            self.state.lock().unwrap().background_color = Some(color);
        }

        fn EqualsLayer(&self, other: &dyn crate::ForeignLayer) -> bool {
            other
                .AsAny()
                .downcast_ref::<Self>()
                .is_some_and(|other| std::ptr::eq(self, other))
        }

        fn AsAny(&self) -> &dyn Any {
            self
        }
    }

    fn fragment(origin_clean: bool) -> FragmentPaintData {
        let mut fragment = FragmentPaintData::default();
        fragment.node_id = 42;
        fragment.paint.style.dynamic_range_limit =
            foundation::DynamicRangeLimit::from_mix(0.25, 0.5);
        fragment.paint.replaced_content = Some(ReplacedContentPaintData {
            offset: Offset { x: 3.25, y: 5.5 },
            size: Size {
                width: 20.5,
                height: 10.25,
            },
            is_canvas: true,
            ..Default::default()
        });
        fragment.paint.resources = Some(Arc::new(PaintResources {
            canvases: vec![CanvasPaintResource {
                dom_node_id: 42,
                resource_id: 1 << 63,
                revision: 7,
                width: 2,
                height: 1,
                origin_clean,
                is_clear: false,
                pixels: Arc::new(vec![1, 2, 3, 4, 5, 6, 7, 8]),
            }],
            ..Default::default()
        }));
        fragment
    }

    #[test]
    fn records_unaccelerated_canvas_snapshot_with_canvas_sampling() {
        let fragment = fragment(true);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 7.25, y: 11.125 };
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);

        HTMLCanvasPainter::new(&node, &context).PaintReplaced(&paint_info);

        drop(paint_info);
        drop(context);
        assert!(recording.first_contentful_painted);
        assert_eq!(recording.items.len(), 1);
        let item = &recording.items[0];
        assert_eq!(item.r#type, DisplayItemType::kDrawImageRect);
        assert_eq!(item.resource_id, 1 << 63);
        assert_eq!(item.source_rect.width, 2.0);
        assert_eq!(item.source_rect.height, 1.0);
        assert_eq!(item.rect.x, 10.5);
        assert_eq!(item.rect.y, 16.625);
        assert_eq!(item.rect.width, 20.5);
        assert_eq!(item.rect.height, 10.25);
        assert_eq!(
            item.image_interpolation_quality,
            InterpolationQuality::kInterpolationLow
        );
        assert_eq!(
            item.dynamic_range_limit,
            foundation::DynamicRangeLimit::from_mix(0.25, 0.5)
        );
    }

    #[test]
    fn privacy_preserving_paint_rejects_tainted_canvas_pixels() {
        let fragment = fragment(false);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new_with_flags(
            &context,
            PaintPhase::kForeground,
            false,
            PaintFlag::kPrivacyPreserving,
        );

        HTMLCanvasPainter::new(&node, &context).PaintReplaced(&paint_info);

        drop(paint_info);
        drop(context);
        assert!(recording.items.is_empty());
        assert!(!recording.first_contentful_painted);
    }

    #[test]
    fn ignored_paint_timing_scope_suppresses_canvas_contentful_signal() {
        let fragment = fragment(true);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        {
            let _scope = crate::ignore_paint_timing_scope::IgnorePaintTimingScope::new();
            crate::ignore_paint_timing_scope::IgnorePaintTimingScope::IncrementIgnoreDepth();
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
            HTMLCanvasPainter::new(&node, &context).PaintReplaced(&paint_info);
        }

        assert!(!recording.first_contentful_painted);
        assert_eq!(recording.items.len(), 1);
    }

    #[test]
    fn clear_canvas_does_not_mark_first_contentful_paint() {
        let mut fragment = fragment(true);
        Arc::make_mut(fragment.paint.resources.as_mut().unwrap()).canvases[0].is_clear = true;
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
            HTMLCanvasPainter::new(&node, &context).PaintReplaced(&paint_info);
        }

        assert!(!recording.first_contentful_painted);
        assert_eq!(recording.items.len(), 1);
    }

    #[test]
    fn nested_canvas_records_custom_data_placeholder() {
        let mut fragment = fragment(true);
        fragment.paint.is_in_canvas_subtree = true;
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
            HTMLCanvasPainter::new(&node, &context).PaintReplaced(&paint_info);
            context.borrow_mut().Finish();
        }

        assert!(recording.first_contentful_painted);
        assert_eq!(recording.items.len(), 1);
        assert_eq!(recording.items[0].r#type, DisplayItemType::kCustomData);
        assert_eq!(recording.items[0].custom_data_id, fragment.node_id as u32);
        assert_eq!(recording.display_items.len(), 1);
        assert_eq!(
            recording.display_items[0].id.r#type,
            DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kForeground)
        );
    }

    #[test]
    fn ordinary_canvas_records_resident_foreign_layer() {
        let mut fragment = fragment(true);
        fragment
            .paint
            .replaced_content
            .as_mut()
            .unwrap()
            .draws_background_onto_content_layer = true;
        fragment.paint.style.background_color = foundation::PaintColor {
            red: 0.1,
            green: 0.5,
            blue: 1.2,
            alpha: 0.6,
        };
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 7.25, y: 11.125 };
        let layer = Arc::new(CanvasLayer {
            state: Mutex::new(CanvasLayerState::default()),
        });
        let mut layers = foundation::TrustedIdentityHashMap::default();
        layers.insert(
            fragment.node_id,
            layer.clone() as Arc<dyn crate::ForeignLayer>,
        );
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        context.borrow_mut().BindForeignLayers(&layers);
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);

        HTMLCanvasPainter::new(&node, &context).PaintReplaced(&paint_info);
        context.borrow_mut().Finish();

        drop(paint_info);
        drop(context);
        assert!(recording.items.is_empty());
        assert_eq!(recording.display_items.len(), 1);
        let item = &recording.display_items[0];
        assert_eq!(item.id.r#type, DisplayItemIdType::kForeignLayerCanvas);
        assert_eq!(item.GetForeignLayer().LayerId(), 55);
        assert_eq!(
            item.visual_rect,
            PaintRect {
                x: 11.0,
                y: 17.0,
                width: 20.0,
                height: 10.0,
            }
        );
        let state = layer.state.lock().unwrap();
        assert_eq!(state.bounds.width, 20.0);
        assert_eq!(state.bounds.height, 10.0);
        assert!(state.drawable);
        assert!(state.hit_testable);
        assert_eq!(
            state.background_color,
            Some(foundation::PaintColor {
                red: 26.0 / 255.0,
                green: 128.0 / 255.0,
                blue: 1.0,
                alpha: 153.0 / 255.0,
            })
        );
    }
}
