//! Software-frame path for `VideoPainter`.
//!
//! Source: third_party/blink/renderer/core/paint/video_painter.cc:20-114.
//! Accelerated frames remain owned by `kForeignLayerVideo`; this module
//! records the source fallback used when compositing information is omitted.

#![allow(non_snake_case)]

use std::cell::RefCell;

use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::foreign_layer_display_item::{RecordVideoForeignLayer, VideoPresentationStyle};
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, PaintPhase};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;
use foundation::InterpolationQuality;
use paint_chunk::DisplayItemClientInfo;
use paint_model::display_item_id::DisplayItemId;

pub struct VideoPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> VideoPainter<'n, 'f, 'c, 'o> {
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    /// Records the immutable current frame published by the media owner.
    /// Chromium normally records a foreign layer and takes this path for
    /// printing/capture. Rechrom's CPU compositor uses the same software path
    /// until a resident video layer subtype is available.
    pub fn PaintReplaced(&self, paint_info: &PaintInfo<'_, '_>) {
        if !matches!(
            paint_info.phase,
            PaintPhase::kForeground | PaintPhase::kSelectionDragImage
        ) {
            return;
        }
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
            .expect("LayoutVideo must publish replaced-content geometry");
        debug_assert!(replaced.is_media && replaced.is_video);
        // Source: video_painter.cc:38-47. Protected/unreadable video surfaces
        // need this player-origin fact even when there is no software bitmap.
        if paint_info.IsPrivacyPreserving() && replaced.video_would_taint_origin {
            return;
        }
        let replaced_rect = PaintRect {
            x: self.node.paint_offset.x + replaced.offset.x,
            y: self.node.paint_offset.y + replaced.offset.y,
            width: replaced.size.width,
            height: replaced.size.height,
        };
        let destination = self.node.PixelSnappedRect(&replaced_rect);
        if destination.width <= 0.0 || destination.height <= 0.0 {
            return;
        }
        let item_type = DisplayItemIdType::PaintPhaseToDrawingType(paint_info.phase);
        if DrawingRecorder::UseCachedDrawingIfPossible(self.context, self.node, item_type) {
            return;
        }
        // video_painter.cc:58-62. The compositor owns accelerated frame
        // submission after this point, so Paint records the image signal now.
        self.context.borrow_mut().SetImagePainted();
        // Source: core/paint/video_painter.cc:80-94. Ordinary foreground
        // paint records the resident media layer. Software frame drawing is
        // reserved for printing/capture (kOmitCompositingInfo) and the
        // selection-drag path.
        if paint_info.phase == PaintPhase::kForeground && !paint_info.ShouldOmitCompositingInfo() {
            let layer = self.context.borrow().ForeignLayerForNode(fragment.node_id);
            if let Some(layer) = layer {
                layer.SetBounds(crate::ForeignLayerSize {
                    width: destination.width,
                    height: destination.height,
                });
                layer.SetIsDrawable(true);
                layer.SetHitTestable(true);
                let id = DisplayItemId {
                    client_id: fragment.paint.display_item_client_id,
                    r#type: DisplayItemIdType::kForeignLayerVideo,
                    fragment: fragment.paint.display_item_fragment,
                };
                RecordVideoForeignLayer(
                    &mut self.context.borrow_mut(),
                    id,
                    DisplayItemClientInfo::from_fragment(fragment),
                    layer,
                    layoutng_assembly::internal::layout_input::Offset {
                        x: destination.x,
                        y: destination.y,
                    },
                    VideoPresentationStyle {
                        nearest_neighbor: fragment.paint.style.image_interpolation_quality()
                            == InterpolationQuality::kInterpolationNone,
                        dynamic_range_limit: fragment.paint.style.dynamic_range_limit(),
                    },
                );
            }
            return;
        }
        if !paint_info.ShouldOmitCompositingInfo() {
            return;
        }
        let Some(resource_id) = fragment.paint.video_frame_resource_id else {
            return;
        };
        let Some(frame) = fragment
            .paint
            .resources
            .as_deref()
            .and_then(|resources| {
                resources
                    .images
                    .iter()
                    .find(|image| image.id == resource_id)
            })
            .filter(|image| {
                image.width != 0
                    && image.height != 0
                    && image.BitmapPixels().is_some_and(|pixels| {
                        (image.width as usize)
                            .checked_mul(image.height as usize)
                            .and_then(|pixels| pixels.checked_mul(4))
                            == Some(pixels.len())
                    })
            })
        else {
            return;
        };
        if paint_info.IsPrivacyPreserving() && !frame.is_cors_same_origin {
            return;
        }
        // Source: core/paint/video_painter.cc:95-99. The frame destination is
        // the replaced-content rect, while the recorder bounds follow the
        // content box when LayoutReplaced clips there.
        let content_box_rect = PaintRect {
            x: self.node.paint_offset.x + fragment.paint.border.left + fragment.paint.padding.left,
            y: self.node.paint_offset.y + fragment.paint.border.top + fragment.paint.padding.top,
            width: fragment.content_size.width,
            height: fragment.content_size.height,
        };
        let visual_rect = if replaced.clips_to_content_box {
            content_box_rect
        } else {
            replaced_rect
        };
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            item_type,
            self.node.EnclosingRect(visual_rect),
        );
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawImageRect,
                phase: paint_info.phase,
                node_id: fragment.node_id,
                rect: destination,
                source_rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: f64::from(frame.width),
                    height: f64::from(frame.height),
                },
                resource_id,
                image_interpolation_quality: if fragment.paint.style.image_interpolation_quality()
                    == InterpolationQuality::kInterpolationNone
                {
                    InterpolationQuality::kInterpolationNone
                } else {
                    InterpolationQuality::kInterpolationLow
                },
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
    use crate::ForeignLayer as _;
    use crate::{PaintPhase, PaintRecording};
    use image_resource::{PaintImage, PaintImageContent};
    use layoutng_assembly::fragment_tree::{
        FragmentPaintData, PaintResources, ReplacedContentPaintData,
    };
    use layoutng_assembly::internal::layout_input::{Offset, Size};
    use paint_model::paint_flags::PaintFlag;
    use std::{
        any::Any,
        sync::{Arc, Mutex},
    };

    struct ResizableVideoLayer {
        bounds: Mutex<crate::ForeignLayerSize>,
        drawable: Mutex<bool>,
        hit_testable: Mutex<bool>,
    }

    impl crate::ForeignLayer for ResizableVideoLayer {
        fn LayerId(&self) -> i32 {
            91
        }

        fn Bounds(&self) -> crate::ForeignLayerSize {
            *self.bounds.lock().unwrap()
        }

        fn SetBounds(&self, bounds: crate::ForeignLayerSize) {
            *self.bounds.lock().unwrap() = bounds;
        }

        fn SetIsDrawable(&self, drawable: bool) {
            *self.drawable.lock().unwrap() = drawable;
        }

        fn SetHitTestable(&self, hit_testable: bool) {
            *self.hit_testable.lock().unwrap() = hit_testable;
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

    fn fragment(is_cors_same_origin: bool) -> FragmentPaintData {
        let mut fragment = FragmentPaintData::default();
        fragment.node_id = 42;
        fragment.paint.video_frame_resource_id = Some(73);
        fragment.paint.replaced_content = Some(ReplacedContentPaintData {
            offset: Offset { x: 3.25, y: 5.5 },
            size: Size {
                width: 20.5,
                height: 10.25,
            },
            is_media: true,
            is_video: true,
            ..Default::default()
        });
        fragment.paint.resources = Some(Arc::new(PaintResources {
            images: vec![PaintImage {
                id: 73,
                width: 2,
                height: 1,
                is_cors_same_origin,
                content: PaintImageContent::Bitmap(Arc::new(vec![1, 2, 3, 4, 5, 6, 7, 8])),
                ..Default::default()
            }],
            ..Default::default()
        }));
        fragment
    }

    #[test]
    fn records_current_software_frame_at_pixel_snapped_replaced_rect() {
        let fragment = fragment(true);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 7.25, y: 11.125 };
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new_with_flags(
            &context,
            PaintPhase::kForeground,
            false,
            PaintFlag::kOmitCompositingInfo,
        );

        VideoPainter::new(&node, &context).PaintReplaced(&paint_info);

        drop(paint_info);
        drop(context);
        assert!(recording.image_painted);
        assert_eq!(recording.items.len(), 1);
        let item = &recording.items[0];
        assert_eq!(item.r#type, DisplayItemType::kDrawImageRect);
        assert_eq!(item.resource_id, 73);
        assert_eq!(item.source_rect.width, 2.0);
        assert_eq!(item.source_rect.height, 1.0);
        assert_eq!(item.rect.x, 11.0);
        assert_eq!(item.rect.y, 17.0);
        assert_eq!(item.rect.width, 20.0);
        assert_eq!(item.rect.height, 10.0);
    }

    #[test]
    fn privacy_preserving_paint_rejects_cross_origin_video_frame() {
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

        VideoPainter::new(&node, &context).PaintReplaced(&paint_info);

        drop(paint_info);
        drop(context);
        assert!(recording.items.is_empty());
        assert!(!recording.image_painted);
    }

    #[test]
    fn ordinary_foreground_records_resident_video_foreign_layer() {
        let fragment = fragment(true);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 7.25, y: 11.125 };
        let layer = Arc::new(ResizableVideoLayer {
            bounds: Mutex::new(crate::ForeignLayerSize::default()),
            drawable: Mutex::new(false),
            hit_testable: Mutex::new(false),
        });
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let mut layers = foundation::TrustedIdentityHashMap::default();
        layers.insert(
            fragment.node_id,
            layer.clone() as Arc<dyn crate::ForeignLayer>,
        );
        context.borrow_mut().BindForeignLayers(&layers);
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);

        VideoPainter::new(&node, &context).PaintReplaced(&paint_info);
        context.borrow_mut().Finish();

        drop(paint_info);
        drop(context);
        assert!(recording.image_painted);
        assert!(recording.items.is_empty());
        assert_eq!(recording.display_items.len(), 1);
        assert_eq!(
            recording.display_items[0].GetType(),
            DisplayItemIdType::kForeignLayerVideo
        );
        assert_eq!(
            layer.Bounds(),
            crate::ForeignLayerSize {
                width: 20.0,
                height: 10.0,
            }
        );
        assert!(*layer.drawable.lock().unwrap());
        assert!(*layer.hit_testable.lock().unwrap());
    }
}
