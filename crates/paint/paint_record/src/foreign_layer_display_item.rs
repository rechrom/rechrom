//! Source-shaped `ForeignLayerDisplayItem` ownership and recording.
//!
//! Blink stores a `scoped_refptr<cc::Layer>` in the display item.  The Rust
//! package graph deliberately keeps `paint_record` below `compositor`, so the
//! equivalent strong owner is type-erased here and recovered by compositor at
//! its existing package boundary.  It is still the resident layer owner: no
//! numeric-id proxy or copied layer state is accepted.

use std::{any::Any, fmt, sync::Arc};

use layoutng_assembly::internal::layout_input::Offset;
use paint_chunk::DisplayItemClientInfo;
use paint_model::{
    display_item_id::DisplayItemId, paint_property_tree::PropertyTreeState, PaintRect,
};

use crate::{
    paint_context::PaintContext, RasterEffectOutset, RecordedDisplayItem, RecordedDisplayItemKind,
};

/// The source layer bounds used by the constructor's `gfx::Rect(origin,
/// layer->bounds())` expression.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ForeignLayerSize {
    pub width: f64,
    pub height: f64,
}

/// Strong type-erased equivalent of `scoped_refptr<cc::Layer>`.
///
/// Implementations must compare the actual resident allocation in
/// `EqualsLayer`, matching scoped-refptr pointer equality in Chromium.
pub trait ForeignLayer: Any + Send + Sync {
    fn LayerId(&self) -> i32;
    fn Bounds(&self) -> ForeignLayerSize;
    /// Source: video_painter.cc:85-87. VideoPainter sizes the resident layer
    /// to the pixel-snapped replaced-content rectangle immediately before it
    /// records the foreign-layer item. Non-video implementations may retain
    /// immutable bounds.
    fn SetBounds(&self, _bounds: ForeignLayerSize) {}
    /// Source: html_canvas_painter.cc:83-86. A resident canvas layer is made
    /// drawable and hit-testable immediately before its foreign display item
    /// is recorded. Other layer subtypes may keep their existing state.
    fn SetIsDrawable(&self, _drawable: bool) {}
    fn SetHitTestable(&self, _hit_testable: bool) {}
    /// Source: html_canvas_painter.cc:74-79. The color is already resolved by
    /// Layout/Paint; the resident canvas layer owns its compositing state.
    fn SetBackgroundColor(&self, _color: foundation::PaintColor) {}
    fn EqualsLayer(&self, other: &dyn ForeignLayer) -> bool;
    fn AsAny(&self) -> &dyn Any;
}

/// Source HTMLVideoElement::StyleDidChange filter/dynamic-range state. This
/// small value crosses with the same resident layer; PAC applies it when its
/// live LayerContext is available, like the existing video bounds handoff.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VideoPresentationStyle {
    pub nearest_neighbor: bool,
    pub dynamic_range_limit: foundation::DynamicRangeLimit,
}
#[derive(Clone)]
pub struct ForeignLayerDisplayItemData {
    layer_: Arc<dyn ForeignLayer>,
    video_style_: Option<VideoPresentationStyle>,
}

impl ForeignLayerDisplayItemData {
    pub fn new(layer: Arc<dyn ForeignLayer>) -> Self {
        Self {
            layer_: layer,
            video_style_: None,
        }
    }

    pub fn GetLayer(&self) -> &dyn ForeignLayer {
        self.layer_.as_ref()
    }
}

impl fmt::Debug for ForeignLayerDisplayItemData {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output
            .debug_struct("ForeignLayerDisplayItemData")
            .field("layer", &self.layer_.LayerId())
            .field("bounds", &self.layer_.Bounds())
            .finish()
    }
}

impl PartialEq for ForeignLayerDisplayItemData {
    fn eq(&self, other: &Self) -> bool {
        self.layer_.EqualsLayer(other.layer_.as_ref())
    }
}

impl RecordedDisplayItem {
    fn ForeignLayerData(&self) -> &ForeignLayerDisplayItemData {
        assert!(!self.IsTombstone());
        assert_eq!(self.kind, RecordedDisplayItemKind::ForeignLayer);
        self.foreign_layer
            .as_ref()
            .expect("ForeignLayerDisplayItem retains its resident layer owner")
    }

    // Source: foreign_layer_display_item.h:35-38.
    pub fn VideoPresentationStyle(&self) -> Option<VideoPresentationStyle> {
        self.ForeignLayerData().video_style_
    }

    pub fn GetForeignLayer(&self) -> &dyn ForeignLayer {
        self.ForeignLayerData().GetLayer()
    }

    // Source: foreign_layer_display_item.cc:34-39.
    pub fn EqualsForeignLayerForUnderInvalidation(&self, other: &Self) -> bool {
        if self.kind != RecordedDisplayItemKind::ForeignLayer
            || other.kind != RecordedDisplayItemKind::ForeignLayer
            || self.IsTombstone()
            || other.IsTombstone()
        {
            return false;
        }
        self.GetForeignLayer().EqualsLayer(other.GetForeignLayer())
    }
}

/// Source: foreign_layer_display_item.cc:17-32.
pub fn NewForeignLayerDisplayItem(
    id: DisplayItemId,
    client: DisplayItemClientInfo,
    layer: Arc<dyn ForeignLayer>,
    origin: Offset,
    outset: RasterEffectOutset,
) -> RecordedDisplayItem {
    assert!(id.r#type.IsForeignLayerType());
    let bounds = layer.Bounds();
    RecordedDisplayItem {
        is_cacheable: client.is_cacheable,
        is_tombstone: false,
        paint_invalidation_reason: client.invalidation_reason,
        scrollbar: None,
        foreign_layer: Some(ForeignLayerDisplayItemData::new(layer)),
        kind: RecordedDisplayItemKind::ForeignLayer,
        id,
        visual_rect: PaintRect {
            x: origin.x,
            y: origin.y,
            width: bounds.width,
            height: bounds.height,
        },
        visual_rect_is_accurate: true,
        draws_content: true,
        raster_effect_outset: outset,
        record_rect_known_to_be_opaque: PaintRect::default(),
        record_background: Default::default(),
        record_has_text: false,
        record_begin: 0,
        record_end: 0,
        scroll_translation: None,
    }
}

/// Source: foreign_layer_display_item.cc:48-68.
///
/// `raster_effect_outset` is supplied explicitly because Rechrom's compact
/// `DisplayItemClientInfo` intentionally contains only chunk/cache state; it
/// is the value returned by Blink's `VisualRectOutsetForRasterEffects()`.
pub fn RecordForeignLayer(
    context: &mut PaintContext<'_>,
    id: DisplayItemId,
    client: DisplayItemClientInfo,
    layer: Arc<dyn ForeignLayer>,
    origin: Offset,
    raster_effect_outset: RasterEffectOutset,
    properties: Option<&PropertyTreeState>,
) {
    RecordLayer(
        context,
        id,
        client,
        layer,
        origin,
        raster_effect_outset,
        properties,
        None,
    );
}

pub fn RecordVideoForeignLayer(
    context: &mut PaintContext<'_>,
    id: DisplayItemId,
    client: DisplayItemClientInfo,
    layer: Arc<dyn ForeignLayer>,
    origin: Offset,
    style: VideoPresentationStyle,
) {
    assert_eq!(
        id.r#type,
        crate::display_item_id::DisplayItemIdType::kForeignLayerVideo
    );
    RecordLayer(
        context,
        id,
        client,
        layer,
        origin,
        RasterEffectOutset::kNone,
        None,
        Some(style),
    );
}
fn RecordLayer(
    context: &mut PaintContext<'_>,
    id: DisplayItemId,
    client: DisplayItemClientInfo,
    layer: Arc<dyn ForeignLayer>,
    origin: Offset,
    raster_effect_outset: RasterEffectOutset,
    properties: Option<&PropertyTreeState>,
    video_style: Option<VideoPresentationStyle>,
) {
    let previous_properties = properties.map(|_| context.chunk_properties());
    if let Some(properties) = properties {
        context.update_chunk_properties(properties.clone());
    }
    let mut item = NewForeignLayerDisplayItem(id, client, layer, origin, raster_effect_outset);
    item.foreign_layer
        .as_mut()
        .expect("foreign layer record")
        .video_style_ = video_style;
    context.AppendForeignLayerDisplayItem(item, client);
    if let Some(previous_properties) = previous_properties {
        context.update_chunk_properties(previous_properties);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paint_model::display_item_id::DisplayItemIdType;

    #[derive(Debug)]
    struct TestLayer {
        id: i32,
        bounds: ForeignLayerSize,
    }

    impl ForeignLayer for TestLayer {
        fn LayerId(&self) -> i32 {
            self.id
        }
        fn Bounds(&self) -> ForeignLayerSize {
            self.bounds
        }
        fn EqualsLayer(&self, other: &dyn ForeignLayer) -> bool {
            other
                .AsAny()
                .downcast_ref::<Self>()
                .is_some_and(|other| std::ptr::eq(self, other))
        }
        fn AsAny(&self) -> &dyn Any {
            self
        }
    }

    fn client() -> DisplayItemClientInfo {
        DisplayItemClientInfo {
            invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            is_cacheable: true,
            is_just_created: false,
        }
    }

    #[test]
    fn constructor_uses_origin_and_resident_layer_bounds() {
        let layer: Arc<dyn ForeignLayer> = Arc::new(TestLayer {
            id: 17,
            bounds: ForeignLayerSize {
                width: 320.0,
                height: 180.0,
            },
        });
        let item = NewForeignLayerDisplayItem(
            DisplayItemId {
                client_id: 9,
                r#type: DisplayItemIdType::kForeignLayerVideo,
                fragment: 2,
            },
            client(),
            layer,
            Offset { x: 4.0, y: 6.0 },
            RasterEffectOutset::kWholePixel,
        );
        assert_eq!(item.kind, RecordedDisplayItemKind::ForeignLayer);
        assert_eq!(
            item.visual_rect,
            PaintRect {
                x: 4.0,
                y: 6.0,
                width: 320.0,
                height: 180.0
            }
        );
        assert_eq!(item.GetForeignLayer().LayerId(), 17);
        assert!(item.draws_content);
        assert_eq!(item.record_begin, item.record_end);
    }

    #[test]
    fn equality_uses_layer_allocation_identity() {
        let layer: Arc<dyn ForeignLayer> = Arc::new(TestLayer {
            id: 17,
            bounds: ForeignLayerSize::default(),
        });
        let id = DisplayItemId {
            client_id: 9,
            r#type: DisplayItemIdType::kForeignLayerCanvas,
            fragment: 0,
        };
        let first = NewForeignLayerDisplayItem(
            id,
            client(),
            Arc::clone(&layer),
            Offset::default(),
            RasterEffectOutset::kNone,
        );
        let same = NewForeignLayerDisplayItem(
            id,
            client(),
            layer,
            Offset::default(),
            RasterEffectOutset::kNone,
        );
        let other = NewForeignLayerDisplayItem(
            id,
            client(),
            Arc::new(TestLayer {
                id: 17,
                bounds: ForeignLayerSize::default(),
            }),
            Offset::default(),
            RasterEffectOutset::kNone,
        );
        assert!(first.EqualsForeignLayerForUnderInvalidation(&same));
        assert!(!first.EqualsForeignLayerForUnderInvalidation(&other));
    }

    #[test]
    fn json_and_move_keep_the_resident_owner_not_an_id_proxy() {
        let concrete = Arc::new(TestLayer {
            id: 23,
            bounds: ForeignLayerSize {
                width: 10.0,
                height: 20.0,
            },
        });
        let erased: Arc<dyn ForeignLayer> = concrete.clone();
        let mut source = NewForeignLayerDisplayItem(
            DisplayItemId {
                client_id: 3,
                r#type: DisplayItemIdType::kForeignLayerPlugin,
                fragment: 0,
            },
            client(),
            erased,
            Offset::default(),
            RasterEffectOutset::kNone,
        );
        assert_eq!(Arc::strong_count(&concrete), 2);
        let mut list = crate::DisplayItemList::default();
        list.AppendByMoving(&mut source);
        assert!(source.IsTombstone());
        assert!(source.foreign_layer.is_none());
        assert_eq!(Arc::strong_count(&concrete), 2);
        assert!(list[0]
            .GetForeignLayer()
            .AsAny()
            .downcast_ref::<TestLayer>()
            .is_some_and(|layer| std::ptr::eq(layer, concrete.as_ref())));
        assert_eq!(
            list[0].PropertiesAsJSON(&crate::PaintRecording::default())["layer"],
            23
        );
    }

    #[test]
    fn record_foreign_layer_appends_one_source_item_and_no_paint_record() {
        let mut output = crate::PaintRecording::default();
        {
            let mut context = crate::paint_context::PaintContext::new(&mut output);
            RecordForeignLayer(
                &mut context,
                DisplayItemId {
                    client_id: 5,
                    r#type: DisplayItemIdType::kForeignLayerRemoteFrame,
                    fragment: 1,
                },
                client(),
                Arc::new(TestLayer {
                    id: 29,
                    bounds: ForeignLayerSize {
                        width: 64.0,
                        height: 48.0,
                    },
                }),
                Offset { x: 7.0, y: 9.0 },
                RasterEffectOutset::kHalfPixel,
                None,
            );
            context.Finish();
        }
        assert!(output.items.is_empty());
        assert_eq!(output.display_items.len(), 1);
        assert_eq!(output.chunks.len(), 1);
        assert_eq!(output.chunks[0].begin_index, 0);
        assert_eq!(output.chunks[0].end_index, 1);
        assert_eq!(output.display_items[0].record_begin, 0);
        assert_eq!(output.display_items[0].record_end, 0);
        assert_eq!(output.display_items[0].GetForeignLayer().LayerId(), 29);
    }
}
