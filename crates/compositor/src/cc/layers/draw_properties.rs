// Translation status: complete
// Copyright 2012 The Chromium Authors
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/draw_properties.h:1-18,65-69;
//         cc/layers/draw_properties.cc:1-8,12-13.
// Includes, export guards and namespace are represented by this Rust module.
use crate::{
    adapter::{geometry::Rect, occlusion::OcclusionTransform, property_types::MaskFilterInfo},
    cc::trees::occlusion::Occlusion,
};

// Source: cc/layers/draw_properties.h:19-64.
// T is the same native gfx::Transform capability used by canonical Occlusion;
// the layer owns all these values. There is no copied host/tree state here.
#[derive(Clone)]
pub struct DrawProperties<T: OcclusionTransform> {
    // Source: cc/layers/draw_properties.h:23-25.
    // Content space to target surface space (the layer's draw transform).
    pub target_space_transform: T,
    // Source: cc/layers/draw_properties.h:27-28.
    // Content space to screen/viewport space.
    pub screen_space_transform: T,
    // Source: cc/layers/draw_properties.h:30-31.
    // Known occlusion above the layer, mapped into its content space.
    pub occlusion_in_content_space: Occlusion<T>,
    // Source: cc/layers/draw_properties.h:33-36.
    // Surface reparenting and compounded ancestor opacity can change this
    // from the LayerImpl's own opacity.
    pub opacity: f32,
    // Source: cc/layers/draw_properties.h:38-41.
    // Cached potentially-animating status of the screen transform chain.
    pub screen_space_transform_is_animating: bool,
    // Source: cc/layers/draw_properties.h:43-44.
    pub is_clipped: bool,
    // Source: cc/layers/draw_properties.h:46-48.
    pub is_fast_rounded_corner: bool,
    // Source: cc/layers/draw_properties.h:50-52.
    pub visible_layer_rect: Rect,
    // Source: cc/layers/draw_properties.h:54-56.
    pub visible_drawable_content_rect: Rect,
    // Source: cc/layers/draw_properties.h:58-60.
    pub clip_rect: Rect,
    // Source: cc/layers/draw_properties.h:62-64.
    pub mask_filter_info: MaskFilterInfo,
}
impl<T: OcclusionTransform> DrawProperties<T> {
    // Source: cc/layers/draw_properties.h:20;
    //         cc/layers/draw_properties.cc:9.
    pub fn new() -> Self {
        Self::default()
    }
}
impl<T: OcclusionTransform> Default for DrawProperties<T> {
    // Source: cc/layers/draw_properties.cc:9, plus in-class initializers at
    // cc/layers/draw_properties.h:36,41,44,48. Native member constructors supply
    // identity transforms and empty occlusion/rect/mask values.
    fn default() -> Self {
        Self {
            target_space_transform: T::default(),
            screen_space_transform: T::default(),
            occlusion_in_content_space: Occlusion::default(),
            opacity: 0.0,
            screen_space_transform_is_animating: false,
            is_clipped: false,
            is_fast_rounded_corner: false,
            visible_layer_rect: Rect::default(),
            visible_drawable_content_rect: Rect::default(),
            clip_rect: Rect::default(),
            mask_filter_info: MaskFilterInfo::default(),
        }
    }
}
// Source: cc/layers/draw_properties.h:21;
//         cc/layers/draw_properties.cc:11.
// Rust member destruction is the source's defaulted destructor. Clone above
// represents C++'s implicit value-copy operations, including owned Occlusion.
