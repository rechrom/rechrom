// Translation status: complete
// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.h:1-26.
// Source: third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.cc:1-43.

use crate::adapter::{
    geometry::{RectF, Size, SizeF, Vector2dF},
    paint_geometry::MaskGeometryMapper,
};

// Source: third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.h:13-22.
// Source: third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.cc:16-20.
pub fn AdjustMaskLayerGeometry<M: MaskGeometryMapper + ?Sized>(
    mapper: &M,
    transform: &M::TransformPaintPropertyNode,
    layer_offset: &mut Vector2dF,
    layer_bounds: &mut Size,
) {
    // Source: third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.cc:21-33.
    let mut pixel_rect = RectF::FromSize(SizeF::new(1.0, 1.0));
    mapper.SourceToDestinationRectFromRoot(transform, &mut pixel_rect);

    // Source: third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.cc:34-40.
    const K_MAX_OUTSET: i32 = 1000;
    let outset = (pixel_rect.width().max(pixel_rect.height()) * 2.0)
        .ceil()
        .clamp(0.0, K_MAX_OUTSET as f32) as i32;
    *layer_offset -= Vector2dF::new(outset as f32, outset as f32);
    *layer_bounds += Size::new(2 * outset, 2 * outset);
}

// Source: third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.h:24-26.
// Source: third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.cc:41-43.
// Namespace and include-guard closure map to the Rust module boundary.
