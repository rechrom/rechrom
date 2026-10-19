// Translation status: complete
// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:1-74;
//         third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.cc:1-131.
// Rust modules replace include guards, includes, exports and namespaces.

use crate::adapter::{
    geometry::{Rect, RectF, ToEnclosingRect, Vector2dF},
    paint_geometry::{ChunkGeometryMapper, PaintChunkView, RasterEffectOutset},
};

// Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:17-25,52-70.
// The mapper is stack-scoped like STACK_ALLOCATED in the source. PropertyTreeState
// values retain canonical Paint node handles, while all mapping is delegated to
// the live Paint GeometryMapper implementation.
pub struct ChunkToLayerMapper<'a, M: ChunkGeometryMapper + ?Sized> {
    mapper_: &'a M,
    layer_state_: M::PropertyTreeState,
    layer_offset_: Vector2dF,
    chunk_state_: M::PropertyTreeState,
    transform_: M::Transform,
    clip_rect_: M::FloatClipRect,
    raster_effect_outset_: RasterEffectOutset,
    has_filter_that_moves_pixels_: bool,
}

impl<'a, M: ChunkGeometryMapper + ?Sized> ChunkToLayerMapper<'a, M> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:24-25;
    //         third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.cc:14-19.
    pub fn new(mapper: &'a M, layer_state: &M::PropertyTreeState, layer_offset: Vector2dF) -> Self {
        Self {
            mapper_: mapper,
            layer_state_: layer_state.clone(),
            layer_offset_: layer_offset,
            chunk_state_: layer_state.clone(),
            transform_: mapper
                .MakeTranslation(Vector2dF::new(-layer_offset.x(), -layer_offset.y())),
            clip_rect_: mapper.MakeDefaultClipRect(),
            raster_effect_outset_: RasterEffectOutset::kNone,
            has_filter_that_moves_pixels_: false,
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:27-28.
    pub fn LayerState(&self) -> &M::PropertyTreeState {
        &self.layer_state_
    }
    pub fn LayerOffset(&self) -> Vector2dF {
        self.layer_offset_
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:30-35;
    //         third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.cc:21-68.
    pub fn SwitchToChunk<C: PaintChunkView<M>>(&mut self, chunk: &C) {
        self.SwitchToChunkWithState(chunk, &chunk.properties_unalias());
    }
    pub fn SwitchToChunkWithState<C: PaintChunkView<M>>(
        &mut self,
        chunk: &C,
        new_chunk_state: &M::PropertyTreeState,
    ) {
        self.raster_effect_outset_ = chunk.raster_effect_outset();
        debug_assert!(*new_chunk_state == chunk.properties_unalias());
        if new_chunk_state == &self.chunk_state_ {
            return;
        }

        if new_chunk_state == &self.layer_state_ {
            self.has_filter_that_moves_pixels_ = false;
            self.transform_ = self.mapper_.MakeTranslation(Vector2dF::new(
                -self.layer_offset_.x(),
                -self.layer_offset_.y(),
            ));
            self.clip_rect_ = self.mapper_.MakeDefaultClipRect();
            self.chunk_state_ = new_chunk_state.clone();
            return;
        }

        if !self
            .mapper_
            .SameTransformNode(new_chunk_state, &self.chunk_state_)
        {
            let success = self.mapper_.SourceToDestinationProjection(
                new_chunk_state,
                &self.layer_state_,
                &mut self.transform_,
            );
            if success {
                self.mapper_.PostTranslate(
                    &mut self.transform_,
                    Vector2dF::new(-self.layer_offset_.x(), -self.layer_offset_.y()),
                );
            } else {
                self.mapper_.MakeIdentity(&mut self.transform_);
                self.mapper_.ScaleToZero(&mut self.transform_);
            }
        }

        self.has_filter_that_moves_pixels_ = !self
            .mapper_
            .SameNearestPixelMovingFilterClip(new_chunk_state, &self.layer_state_);
        if self.has_filter_that_moves_pixels_ {
            self.clip_rect_ = self.mapper_.InfiniteLooseFloatClipRect();
        } else if !self
            .mapper_
            .SameClipNode(new_chunk_state, &self.chunk_state_)
        {
            self.clip_rect_ = self
                .mapper_
                .LocalToAncestorClipRect(new_chunk_state, &self.layer_state_);
            if !self.mapper_.ClipIsInfinite(&self.clip_rect_) {
                self.mapper_.MoveClip(
                    &mut self.clip_rect_,
                    Vector2dF::new(-self.layer_offset_.x(), -self.layer_offset_.y()),
                );
            }
        }
        self.chunk_state_ = new_chunk_state.clone();
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:35.
    pub fn ChunkState(&self) -> &M::PropertyTreeState {
        &self.chunk_state_
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:37-38;
    //         third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.cc:70-99.
    pub fn MapVisualRect(&self, rect: Rect) -> Rect {
        if self.has_filter_that_moves_pixels_ {
            return self.MapUsingGeometryMapper(rect);
        }
        if rect.IsEmpty() {
            return Rect::default();
        }
        let mut mapped_rect = self.mapper_.MapRect(&self.transform_, rect.into());
        if !rectf_is_empty(mapped_rect) && !self.mapper_.ClipIsInfinite(&self.clip_rect_) {
            intersect_rectf(&mut mapped_rect, self.mapper_.ClipRect(&self.clip_rect_));
        }
        let mut result = Rect::default();
        if !rectf_is_empty(mapped_rect) {
            self.InflateForRasterEffectOutset(&mut mapped_rect);
            result = ToEnclosingRect(mapped_rect);
        }
        #[cfg(debug_assertions)]
        {
            let slow_result = self.MapUsingGeometryMapper(rect);
            if result != slow_result {
                eprintln!(
                    "ChunkToLayerMapper::MapVisualRect: different fast/slow results: {:?} {:?}",
                    result, slow_result
                );
            }
        }
        result
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:40-42,55;
    //         third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.cc:101-122.
    fn MapUsingGeometryMapper(&self, rect: Rect) -> Rect {
        self.MapVisualRectFromState(rect, &self.chunk_state_)
    }
    pub fn MapVisualRectFromState(&self, rect: Rect, state: &M::PropertyTreeState) -> Rect {
        let mut visual_rect = self.mapper_.MakeFloatClipRect(rect.into());
        self.mapper_
            .LocalToAncestorVisualRect(state, &self.layer_state_, &mut visual_rect);
        if rectf_is_empty(self.mapper_.ClipRect(&visual_rect)) {
            return Rect::default();
        }
        let mut result = self.mapper_.ClipRect(&visual_rect);
        result.Offset(-self.layer_offset_.x(), -self.layer_offset_.y());
        self.InflateForRasterEffectOutset(&mut result);
        ToEnclosingRect(result)
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:44-50.
    pub fn Transform(&self) -> &M::Transform {
        &self.transform_
    }
    pub fn ClipRect(&self) -> &M::FloatClipRect {
        &self.clip_rect_
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.h:56;
    //         third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.cc:124-129.
    fn InflateForRasterEffectOutset(&self, rect: &mut RectF) {
        let outset = match self.raster_effect_outset_ {
            RasterEffectOutset::kNone => 0.0,
            RasterEffectOutset::kHalfPixel => 0.5,
            RasterEffectOutset::kWholePixel => 1.0,
        };
        if outset != 0.0 {
            *rect = RectF::new(
                rect.x() - outset,
                rect.y() - outset,
                rect.width() + 2.0 * outset,
                rect.height() + 2.0 * outset,
            );
        }
    }
}

fn rectf_is_empty(rect: RectF) -> bool {
    rect.width() <= 0.0 || rect.height() <= 0.0
}

fn intersect_rectf(rect: &mut RectF, other: RectF) {
    let left = rect.x().max(other.x());
    let top = rect.y().max(other.y());
    let right = rect.right().min(other.right());
    let bottom = rect.bottom().min(other.bottom());
    *rect = if left >= right || top >= bottom {
        RectF::default()
    } else {
        RectF::new(left, top, right - left, bottom - top)
    };
}
