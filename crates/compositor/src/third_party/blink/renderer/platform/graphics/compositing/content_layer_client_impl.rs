// Translation status: complete
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:1-35,65-68,80-84.
// Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:1-36,53-54,270-271.
// Includes, namespaces, noncopyable declaration, and Oilpan allocation map to
// Rust modules and the single resident Arc allocation. External Paint types
// remain their actual associated owners; no display-list/invalidator snapshot.
use super::{
    adjust_mask_layer_geometry::AdjustMaskLayerGeometry,
    paint_chunks_to_cc_layer::{ConvertInto, RasterUnderInvalidationCheckingParams},
    pending_layer::PendingLayer,
};
use crate::{
    adapter::{
        content_layer_client_impl::{
            CanvasChildPaintState, ContentDisplayItemList, ContentJson, ContentLayerClientServices,
            ContentLayerClientTypes, ContentPaintOpBuffer, RasterInvalidationCallback,
            RasterInvalidationTracking, RasterInvalidator,
        },
        geometry::{Rect, RectF},
        layer_tree_impl::Color4f,
        main_layer::LayerContext,
        paint_chunks_to_cc_layer::{PaintChunksServices, PaintChunksTypes},
        pending_layer::{
            PendingPaintArtifact, PendingPaintChunk, PendingPaintChunkSubset,
            PendingPropertyTreeState,
        },
        picture_layer::PictureDisplayItemList,
        property_types::BlendMode,
    },
    cc::layers::{
        content_layer_client::ContentLayerClient,
        layer::{Layer, PictureLayerRef},
        picture_layer::{PictureLayer, PictureLayerClientWeak},
    },
};
use std::sync::{Arc, RwLock, Weak};

// Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:69-79.
pub struct ContentLayerClientImpl<B: ContentLayerClientTypes> {
    // Source unique/scoped owners in reverse member destruction order.
    paint_chunk_debug_data_: Option<B::JsonArray>,
    debug_name_: String,
    has_empty_invalidations_: bool,
    canvas_child_paint_state_: Option<B::CanvasChildPaintState>,
    raster_invalidator_: B::RasterInvalidator,
    cc_display_item_list_: Option<Arc<B::ContentDisplayItemList>>,
    cc_picture_layer_: PictureLayerRef<B>,
    // Rust-only nonowning identity validates source CHECK_EQ(client(), this).
    identity_: PictureLayerClientWeak<B>,
    destroyed_: bool,
}

// Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:37-51.
fn DrawingShouldFillScrollingContentsLayer<B: ContentLayerClientTypes>(
    state: &B::PropertyTreeState,
    layer: &Layer<B>,
    c: &LayerContext<'_, B>,
    services: &dyn ContentLayerClientServices<B>,
) -> bool {
    if !layer.draws_content(c) {
        return false;
    }
    if let Some(scroll_rect) = services.ScrollContentsRect(state) {
        let bounds = layer.bounds(c);
        return bounds.width() >= scroll_rect.width && bounds.height() >= scroll_rect.height;
    }
    false
}

impl<B: ContentLayerClientTypes> ContentLayerClientImpl<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:31;
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:55-57.
    // Arc::new_cyclic supplies exactly the source weak client binding before
    // returning the sole resident object. No owner/service is captured by it.
    pub fn new(
        c: &mut LayerContext<'_, B>,
        services: &dyn ContentLayerClientServices<B>,
    ) -> Arc<RwLock<Self>> {
        Arc::new_cyclic(|weak: &Weak<RwLock<Self>>| {
            let client: PictureLayerClientWeak<B> = weak.clone();
            let picture = PictureLayer::<B>::Create(Some(client.clone()), c);
            let invalidator = services.CreateRasterInvalidator();
            RwLock::new(Self {
                cc_picture_layer_: picture,
                cc_display_item_list_: None,
                raster_invalidator_: invalidator,
                canvas_child_paint_state_: None,
                has_empty_invalidations_: false,
                debug_name_: String::new(),
                paint_chunk_debug_data_: None,
                identity_: client,
                destroyed_: false,
            })
        })
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:34;
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:59-61.
    // ClearClient needs the current canonical LayerContext. The owner calls
    // this at native finalization, before dropping its last GC/strong handle.
    pub fn Destroy(&mut self, c: &mut LayerContext<'_, B>) {
        assert!(!self.destroyed_);
        self.cc_picture_layer_
            .WithPictureLayer(c, |layer, c| layer.ClearClient(c));
        self.destroyed_ = true;
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:36.
    // Rust's owned invalidator is traced by ownership; there is no GC visitor.

    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:49.
    pub fn Layer(&self) -> &crate::cc::layers::layer::LayerRef<B> {
        self.cc_picture_layer_.LayerRef()
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:56.
    pub fn GetRasterInvalidator(&mut self) -> &mut B::RasterInvalidator {
        &mut self.raster_invalidator_
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:59-61.
    pub fn canvas_child_paint_state(&self) -> Option<&B::CanvasChildPaintState> {
        self.canvas_child_paint_state_.as_ref()
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:45-47;
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:63-96.
    pub fn AppendAdditionalInfoAsJSON(
        &self,
        flags: LayerTreeFlags,
        layer: &Layer<B>,
        json: &mut B::Json,
        c: &LayerContext<'_, B>,
        services: &dyn ContentLayerClientServices<B>,
    ) {
        if services.ExpensiveDChecksEnabled() && flags.includes_debug_info() {
            json.SetArray(
                "paintChunkContents",
                self.paint_chunk_debug_data_
                    .as_ref()
                    .expect("source debug data")
                    .clone(),
            );
        }
        if flags.includes_invalidations() || flags.includes_detailed_invalidations() {
            if let Some(tracking) = self.raster_invalidator_.GetTracking() {
                tracking.AsJSON(json, flags.includes_detailed_invalidations());
            }
        }
        #[cfg(debug_assertions)]
        if flags.includes_paint_records() {
            let mut canvas = services.CreateLoggingCanvas();
            let mut offsets = crate::adapter::prioritized_tile::ScrollOffsetMap::new();
            let list = self
                .cc_display_item_list_
                .as_ref()
                .expect("source display list");
            for element in list.raster_inducing_scrolls() {
                offsets.insert(element, services.CurrentScrollOffset(layer, element, c));
            }
            list.RasterForLogging(&mut canvas, &offsets);
            json.SetPaintRecordLog("paintRecord", canvas);
        }
        #[cfg(not(debug_assertions))]
        let _ = (layer, c);
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:51-52;
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:98-114.
    pub fn UpdateCcPictureLayer(
        &mut self,
        pending: &PendingLayer<B>,
        property_state_for_paint: B::PropertyTreeState,
        c: &mut LayerContext<'_, B>,
        services: &dyn PaintChunksServices<B>,
    ) where
        B: PaintChunksTypes,
    {
        let chunks = pending.Chunks();
        self.cc_picture_layer_.WithPictureLayer(c, |layer, _| {
            assert!(layer
                .client()
                .as_ref()
                .is_some_and(|client| client.ptr_eq(&self.identity_)));
        });
        if services.ExpensiveDChecksEnabled() {
            let mut data = services.NewJsonArray();
            for index in 0..chunks.len() {
                services.AppendChunkDebugData(&mut data, chunks, index);
            }
            self.paint_chunk_debug_data_ = Some(data);
        }
        // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:116-129.
        let layer_state = pending.GetPropertyTreeState();
        let (mut offset, mut bounds) = pending.Bounds();
        let old_bounds = self.raster_invalidator_.LayerBounds();
        let is_mask = layer_state.EffectBlendMode() == BlendMode::kDstIn;
        if is_mask {
            AdjustMaskLayerGeometry(
                services,
                &pending.GetPropertyTreeState().TransformNode(),
                &mut offset,
                &mut bounds,
            );
        }
        debug_assert_eq!(old_bounds, *self.Layer().bounds(c));
        self.has_empty_invalidations_ = false;
        {
            // Borrow split for RasterInvalidator::AddRasterInvalidation ->
            // Callback::InvalidateRect. The same synchronous source body below
            // receives only the three resident fields it actually mutates.
            let mut callback = InvalidationCallback::<B> {
                list: &mut self.cc_display_item_list_,
                has_empty: &mut self.has_empty_invalidations_,
                picture: &self.cc_picture_layer_,
                c,
                services,
            };
            self.raster_invalidator_
                .Generate(chunks, offset, bounds, &layer_state, &mut callback);
        }
        // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:131-144.
        let checking = if services.PaintUnderInvalidationCheckingEnabled() {
            Some(RasterUnderInvalidationCheckingParams {
                tracking: self
                    .raster_invalidator_
                    .GetTrackingMutable()
                    .expect("source tracking enabled"),
                interest_rect: Rect::new(0, 0, bounds.width(), bounds.height()),
                debug_name: chunks
                    .artifact()
                    .ClientDebugName(chunks.chunk(0).client_id()),
            })
        } else {
            None
        };
        self.canvas_child_paint_state_ = services.CanvasChildPaintState(&layer_state);
        // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:146-172.
        self.cc_picture_layer_
            .LayerRef()
            .SetOffsetToTransformParent(offset, c);
        self.cc_picture_layer_.LayerRef().SetBounds(bounds, c);
        self.cc_picture_layer_
            .LayerRef()
            .SetHitTestOpaqueness(pending.GetHitTestOpaqueness(), c);
        let may_be_unchanged = self.cc_display_item_list_.is_some()
            && bounds == old_bounds
            && self.cc_picture_layer_.LayerRef().draws_content(c) == pending.DrawsContent()
            && checking.is_none();
        let only_empty_invalidations =
            self.has_empty_invalidations_ && self.cc_display_item_list_.is_some();
        if may_be_unchanged {
            debug_assert_eq!(*self.cc_picture_layer_.LayerRef().bounds(c), bounds);
            if !services.RasterInducingScrollEnabled() || !only_empty_invalidations {
                return;
            }
        }
        // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:174-192.
        let had_raster_inducing_scroll =
            Self::ListHasRasterInducingScroll(&self.cc_display_item_list_);
        let previous = self.cc_display_item_list_.take();
        self.cc_display_item_list_ = Some(Arc::new(services.CreateDisplayItemList()));
        ConvertInto::<B>(
            chunks,
            &property_state_for_paint,
            offset,
            checking,
            Arc::get_mut(self.cc_display_item_list_.as_mut().unwrap())
                .expect("new unshared display list"),
            services,
        );
        if services.RasterInducingScrollEnabled() {
            if (had_raster_inducing_scroll || self.HasRasterInducingScroll())
                && only_empty_invalidations
            {
                self.cc_picture_layer_
                    .WithPictureLayer(c, |layer, c| layer.SetForceUpdateRecordingSource(c));
            } else if may_be_unchanged {
                self.cc_display_item_list_ = previous;
                return;
            }
        }
        // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:194-221.
        self.cc_picture_layer_
            .LayerRef()
            .SetIsDrawable(pending.DrawsContent(), c);
        if is_mask
            || DrawingShouldFillScrollingContentsLayer(&layer_state, self.Layer(), c, services)
        {
            let list = Arc::get_mut(self.cc_display_item_list_.as_mut().unwrap())
                .expect("unfinalized display list");
            list.StartPaint();
            list.PushNoop();
            list.EndPaintOfUnpaired(Rect::new(0, 0, bounds.width(), bounds.height()));
        }
        Arc::get_mut(self.cc_display_item_list_.as_mut().unwrap())
            .expect("unfinalized display list")
            .Finalize();
        self.cc_picture_layer_
            .LayerRef()
            .SetBackgroundColor(pending.ComputeBackgroundColor(services), c);
        let contents_opaque = self.cc_picture_layer_.LayerRef().background_color(c)
            != Color4f::default()
            && services.RectContains(
                *pending.RectKnownToBeOpaque(),
                RectF::new(
                    offset.x(),
                    offset.y(),
                    bounds.width() as f32,
                    bounds.height() as f32,
                ),
            );
        self.cc_picture_layer_
            .LayerRef()
            .SetContentsOpaque(contents_opaque, c);
        if !contents_opaque {
            self.cc_picture_layer_.LayerRef().SetContentsOpaqueForText(
                self.cc_display_item_list_
                    .as_ref()
                    .unwrap()
                    .has_draw_text_ops()
                    && pending.TextKnownToBeOnOpaqueBackground(),
                c,
            );
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:54;
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:223-226.
    pub fn HasRasterInducingScroll(&self) -> bool {
        Self::ListHasRasterInducingScroll(&self.cc_display_item_list_)
    }
    fn ListHasRasterInducingScroll(list: &Option<Arc<B::ContentDisplayItemList>>) -> bool {
        list.as_ref()
            .is_some_and(|list| list.raster_inducing_scrolls().next().is_some())
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:58;
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:247-264.
    pub fn GetCanvasChildPaintRecord(
        &self,
        c: &LayerContext<'_, B>,
        services: &dyn ContentLayerClientServices<B>,
    ) -> Option<B::CanvasChildPaintRecord> {
        let state = self.canvas_child_paint_state_.as_ref()?;
        let offset = self.Layer().offset_to_transform_parent(c) - state.reference_box_offset();
        let list = self
            .cc_display_item_list_
            .as_ref()
            .expect("source canvas child display list");
        let record = if offset.x() == 0.0 && offset.y() == 0.0 {
            list.DeepCopyAsRecord()
        } else {
            let mut result = services.CreatePaintOpBuffer();
            result.PushTranslate(offset.x(), offset.y());
            result.AppendDisplayListBuffer(list);
            result.ReleaseAsRecord()
        };
        Some(services.CreateCanvasChildPaintRecord(state.clone(), record))
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:63;
    // Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:266-269.
    pub fn ApproximateUnsharedMemoryUsage(&self) -> usize {
        std::mem::size_of::<Self>() + self.raster_invalidator_.ApproximateUnsharedMemoryUsage()
            - std::mem::size_of::<B::RasterInvalidator>()
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.h:38-42.
impl<'c, B: ContentLayerClientTypes>
    ContentLayerClient<B::ContentDisplayItemList, LayerContext<'c, B>>
    for ContentLayerClientImpl<B>
{
    fn PaintContentsToDisplayList(
        &mut self,
        _context: &mut LayerContext<'c, B>,
    ) -> Option<Arc<B::ContentDisplayItemList>> {
        self.cc_display_item_list_.clone()
    }
    fn FillsBoundsCompletely(&self) -> bool {
        false
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.cc:228-245.
// Single source callback body, split from its resident owner only to express
// Generate's synchronous borrow of the invalidator and remaining client fields.
struct InvalidationCallback<'a, 'c, B: ContentLayerClientTypes> {
    list: &'a mut Option<Arc<B::ContentDisplayItemList>>,
    has_empty: &'a mut bool,
    picture: &'a PictureLayerRef<B>,
    c: &'a mut LayerContext<'c, B>,
    services: &'a dyn ContentLayerClientServices<B>,
}
impl<B: ContentLayerClientTypes> RasterInvalidationCallback for InvalidationCallback<'_, '_, B> {
    fn InvalidateRect(&mut self, rect: Rect) {
        if rect.IsEmpty() {
            if self.services.RasterInducingScrollEnabled() {
                *self.has_empty = true;
            }
            return;
        }
        *self.list = None;
        self.picture.LayerRef().SetNeedsDisplayRect(&rect, self.c);
    }
}

// Native unsigned LayersAsJSON flag value. Unknown/native helper bits survive
// unchanged; only the four source ContentLayerClient flags are inspected here.
#[derive(Clone, Copy, Default)]
pub struct LayerTreeFlags(pub u32);
impl LayerTreeFlags {
    pub fn includes_debug_info(self) -> bool {
        self.0 & (1 << 0) != 0
    }
    pub fn includes_invalidations(self) -> bool {
        self.0 & (1 << 1) != 0
    }
    pub fn includes_detailed_invalidations(self) -> bool {
        self.0 & (1 << 2) != 0
    }
    pub fn includes_paint_records(self) -> bool {
        self.0 & (1 << 6) != 0
    }
}

impl<B: ContentLayerClientTypes> Drop for ContentLayerClientImpl<B> {
    fn drop(&mut self) {
        assert!(
            self.destroyed_,
            "source ClearClient must run with live LayerContext before finalization"
        );
    }
}

// Rust ownership mapping of Oilpan Member<ContentLayerClientImpl> and the
// cc::PictureLayer's nonowning virtual client pointer. Every handle refers to
// the same allocation; all guards below are call-scoped live field borrows.
pub struct ContentLayerClientRef<B: ContentLayerClientTypes>(
    Arc<RwLock<ContentLayerClientImpl<B>>>,
);
pub type CanvasStateRead<'a, B> = crate::adapter::main_layer::LiveFieldRead<
    std::sync::RwLockReadGuard<'a, ContentLayerClientImpl<B>>,
    <B as crate::adapter::pending_layer::PendingLayerTypes>::CanvasChildPaintState,
>;
impl<B: ContentLayerClientTypes> Clone for ContentLayerClientRef<B> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<B: ContentLayerClientTypes> ContentLayerClientRef<B> {
    pub fn AppendAdditionalInfoAsJSON(&self,flags:LayerTreeFlags,layer:&Layer<B>,json:&mut B::Json,c:&LayerContext<'_,B>,services:&dyn ContentLayerClientServices<B>){self.0.read().unwrap().AppendAdditionalInfoAsJSON(flags,layer,json,c,services)}
    pub fn new(c: &mut LayerContext<'_, B>, services: &dyn ContentLayerClientServices<B>) -> Self {
        Self(ContentLayerClientImpl::new(c, services))
    }
    pub fn LayerRef(&self) -> crate::cc::layers::layer::LayerRef<B> {
        self.0.read().unwrap().Layer().clone()
    }
    pub fn SetTracksRasterInvalidations(&self, value: bool) {
        self.0
            .write()
            .unwrap()
            .raster_invalidator_
            .SetTracksRasterInvalidations(value);
    }
    pub fn SetOldPaintArtifact(&self, artifact: Arc<B::PaintArtifact>) {
        self.0
            .write()
            .unwrap()
            .raster_invalidator_
            .SetOldPaintArtifact(artifact);
    }
    pub fn HasRasterInducingScroll(&self) -> bool {
        self.0.read().unwrap().HasRasterInducingScroll()
    }
    pub fn UpdateForRasterInducingScroll(&self, chunks: &B::PaintChunkSubset) {
        self.0
            .write()
            .unwrap()
            .raster_invalidator_
            .UpdateForRasterInducingScroll(chunks);
    }
    pub fn UpdateCcPictureLayer(
        &self,
        pending: &PendingLayer<B>,
        state: B::PropertyTreeState,
        c: &mut LayerContext<'_, B>,
        services: &dyn PaintChunksServices<B>,
    ) where
        B: PaintChunksTypes,
    {
        // Generate receives only the split invalidation callback fields. It
        // cannot acquire this client lock or dispatch to the whole client.
        self.0
            .write()
            .unwrap()
            .UpdateCcPictureLayer(pending, state, c, services);
    }
    pub fn GetCanvasChildPaintRecord(
        &self,
        c: &LayerContext<'_, B>,
        services: &dyn ContentLayerClientServices<B>,
    ) -> Option<B::CanvasChildPaintRecord> {
        self.0
            .read()
            .unwrap()
            .GetCanvasChildPaintRecord(c, services)
    }
    pub fn canvas_child_paint_state(&self) -> Option<CanvasStateRead<'_, B>> {
        let owner = self.0.read().unwrap();
        owner.canvas_child_paint_state_.as_ref()?;
        Some(crate::adapter::main_layer::LiveFieldRead::new(
            owner,
            |client| client.canvas_child_paint_state_.as_ref().unwrap(),
        ))
    }
    pub fn GetRasterInvalidator(
        &self,
    ) -> crate::adapter::main_layer::LiveFieldWrite<
        std::sync::RwLockWriteGuard<'_, ContentLayerClientImpl<B>>,
        B::RasterInvalidator,
    > {
        crate::adapter::main_layer::LiveFieldWrite::new(
            self.0.write().unwrap(),
            |c| &c.raster_invalidator_,
            |c| &mut c.raster_invalidator_,
        )
    }
    pub fn ApproximateUnsharedMemoryUsage(&self) -> usize {
        self.0.read().unwrap().ApproximateUnsharedMemoryUsage()
    }
    pub fn Release(self, c: &mut LayerContext<'_, B>) {
        if Arc::strong_count(&self.0) == 1 {
            self.0.write().unwrap().Destroy(c);
        }
        drop(self);
    }
}
