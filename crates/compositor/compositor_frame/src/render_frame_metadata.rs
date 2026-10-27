// Translation status: complete
// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/render_frame_metadata.h:1-159;
//         cc/trees/render_frame_metadata.cc:1-24.
use crate::render_metadata::{RenderFrameMetadataTypes, VerticalScrollDirection};
use compositor_draw::Color4f;
#[cfg(any(target_os = "android", target_os = "ios"))]
use compositor_tree::geometry::SizeF;
use compositor_tree::geometry::{PointF, Size};
// Source: cc/trees/render_frame_metadata.h:26-46.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DelegatedInkBrowserMetadata {
    pub delegated_ink_is_hovering: bool,
}
impl DelegatedInkBrowserMetadata {
    pub fn new(hovering: bool) -> Self {
        Self {
            delegated_ink_is_hovering: hovering,
        }
    }
}
// Source: cc/trees/render_frame_metadata.h:49-155.
pub struct RenderFrameMetadata<B: RenderFrameMetadataTypes> {
    // Source: cc/trees/render_frame_metadata.h:63.
    pub is_scroll_offset_at_top: bool,
    // Source: cc/trees/render_frame_metadata.h:68.
    pub root_background_color: Color4f,
    // Source: cc/trees/render_frame_metadata.h:71.
    pub root_scroll_offset: Option<PointF>,
    // Source: cc/trees/render_frame_metadata.h:12.
    pub selection: B::Selection,
    // Source: cc/trees/render_frame_metadata.h:82.
    pub is_mobile_optimized: bool,
    // Source: cc/trees/render_frame_metadata.h:89.
    pub delegated_ink_metadata: Option<DelegatedInkBrowserMetadata>,
    // Source: cc/trees/render_frame_metadata.h:92.
    pub device_scale_factor: f32,
    // Source: cc/trees/render_frame_metadata.h:96.
    pub viewport_size_in_pixels: Size,
    // Source: cc/trees/render_frame_metadata.h:13.
    pub local_surface_id: Option<B::LocalSurfaceId>,
    // Source: cc/trees/render_frame_metadata.h:102.
    pub page_scale_factor: f32,
    // Source: cc/trees/render_frame_metadata.h:104.
    pub external_page_scale_factor: f32,
    // Source: cc/trees/render_frame_metadata.h:108.
    pub top_controls_height: f32,
    // Source: cc/trees/render_frame_metadata.h:109.
    pub top_controls_shown_ratio: f32,
    // Source: cc/trees/render_frame_metadata.h:116.
    pub new_vertical_scroll_direction: VerticalScrollDirection,
    // Source: cc/trees/render_frame_metadata.h:123.
    pub primary_main_frame_item_sequence_number: i64,
    // Source: cc/trees/render_frame_metadata.h:14.
    pub tracked_element_rects: B::TrackedElementRects,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:132.
    pub bottom_controls_height: f32,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:133.
    pub bottom_controls_shown_ratio: f32,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:137.
    pub top_controls_min_height_offset: f32,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:138.
    pub bottom_controls_min_height_offset: f32,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:142.
    pub min_page_scale_factor: f32,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:143.
    pub max_page_scale_factor: f32,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:144.
    pub root_overflow_y_hidden: bool,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:146.
    pub scrollable_viewport_size: SizeF,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:147.
    pub root_layer_size: SizeF,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:151.
    pub has_transparent_background: bool,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    // Source: cc/trees/render_frame_metadata.h:153.
    pub has_offset_tag: bool,
}
impl<B: RenderFrameMetadataTypes> RenderFrameMetadata<B> {
    // Source: cc/trees/render_frame_metadata.h:122.
    pub const kInvalidItemSequenceNumber: i64 = -1;
}
// Source: cc/trees/render_frame_metadata.cc:11.
impl<B: RenderFrameMetadataTypes> Default for RenderFrameMetadata<B> {
    fn default() -> Self {
        Self {
            is_scroll_offset_at_top: true,
            root_background_color: Color4f {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
            root_scroll_offset: None,
            selection: B::Selection::default(),
            is_mobile_optimized: false,
            delegated_ink_metadata: None,
            device_scale_factor: 1.0,
            viewport_size_in_pixels: Size::default(),
            local_surface_id: None,
            page_scale_factor: 1.0,
            external_page_scale_factor: 1.0,
            top_controls_height: 0.0,
            top_controls_shown_ratio: 0.0,
            new_vertical_scroll_direction: VerticalScrollDirection::kNull,
            primary_main_frame_item_sequence_number: Self::kInvalidItemSequenceNumber,
            tracked_element_rects: B::TrackedElementRects::default(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            bottom_controls_height: 0.0,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            bottom_controls_shown_ratio: 0.0,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            top_controls_min_height_offset: 0.0,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            bottom_controls_min_height_offset: 0.0,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            min_page_scale_factor: 0.0,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            max_page_scale_factor: 0.0,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            root_overflow_y_hidden: false,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            scrollable_viewport_size: SizeF::default(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            root_layer_size: SizeF::default(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            has_transparent_background: false,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            has_offset_tag: false,
        }
    }
}
// Source: cc/trees/render_frame_metadata.cc:13-24 (memberwise copy/move/drop).
impl<B: RenderFrameMetadataTypes> Clone for RenderFrameMetadata<B> {
    fn clone(&self) -> Self {
        Self {
            is_scroll_offset_at_top: self.is_scroll_offset_at_top.clone(),
            root_background_color: self.root_background_color.clone(),
            root_scroll_offset: self.root_scroll_offset.clone(),
            selection: self.selection.clone(),
            is_mobile_optimized: self.is_mobile_optimized.clone(),
            delegated_ink_metadata: self.delegated_ink_metadata.clone(),
            device_scale_factor: self.device_scale_factor.clone(),
            viewport_size_in_pixels: self.viewport_size_in_pixels.clone(),
            local_surface_id: self.local_surface_id.clone(),
            page_scale_factor: self.page_scale_factor.clone(),
            external_page_scale_factor: self.external_page_scale_factor.clone(),
            top_controls_height: self.top_controls_height.clone(),
            top_controls_shown_ratio: self.top_controls_shown_ratio.clone(),
            new_vertical_scroll_direction: self.new_vertical_scroll_direction.clone(),
            primary_main_frame_item_sequence_number: self
                .primary_main_frame_item_sequence_number
                .clone(),
            tracked_element_rects: self.tracked_element_rects.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            bottom_controls_height: self.bottom_controls_height.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            bottom_controls_shown_ratio: self.bottom_controls_shown_ratio.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            top_controls_min_height_offset: self.top_controls_min_height_offset.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            bottom_controls_min_height_offset: self.bottom_controls_min_height_offset.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            min_page_scale_factor: self.min_page_scale_factor.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            max_page_scale_factor: self.max_page_scale_factor.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            root_overflow_y_hidden: self.root_overflow_y_hidden.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            scrollable_viewport_size: self.scrollable_viewport_size.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            root_layer_size: self.root_layer_size.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            has_transparent_background: self.has_transparent_background.clone(),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            has_offset_tag: self.has_offset_tag.clone(),
        }
    }
}
// Source: cc/trees/render_frame_metadata.cc:26-58.
impl<B: RenderFrameMetadataTypes> PartialEq for RenderFrameMetadata<B> {
    fn eq(&self, other: &Self) -> bool {
        let equal = true;
        let equal = equal && self.root_scroll_offset == other.root_scroll_offset;
        let equal = equal && self.root_background_color == other.root_background_color;
        let equal = equal && self.is_scroll_offset_at_top == other.is_scroll_offset_at_top;
        let equal = equal && self.selection == other.selection;
        let equal = equal && self.is_mobile_optimized == other.is_mobile_optimized;
        let equal = equal && self.delegated_ink_metadata == other.delegated_ink_metadata;
        let equal = equal && self.device_scale_factor == other.device_scale_factor;
        let equal = equal && self.viewport_size_in_pixels == other.viewport_size_in_pixels;
        let equal = equal && self.page_scale_factor == other.page_scale_factor;
        let equal = equal && self.external_page_scale_factor == other.external_page_scale_factor;
        let equal = equal && self.top_controls_height == other.top_controls_height;
        let equal = equal && self.top_controls_shown_ratio == other.top_controls_shown_ratio;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal = equal && self.bottom_controls_height == other.bottom_controls_height;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal = equal && self.bottom_controls_shown_ratio == other.bottom_controls_shown_ratio;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal =
            equal && self.top_controls_min_height_offset == other.top_controls_min_height_offset;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal = equal
            && self.bottom_controls_min_height_offset == other.bottom_controls_min_height_offset;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal = equal && self.min_page_scale_factor == other.min_page_scale_factor;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal = equal && self.max_page_scale_factor == other.max_page_scale_factor;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal = equal && self.root_overflow_y_hidden == other.root_overflow_y_hidden;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal = equal && self.scrollable_viewport_size == other.scrollable_viewport_size;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal = equal && self.root_layer_size == other.root_layer_size;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let equal = equal && self.has_transparent_background == other.has_transparent_background;
        let equal = equal && self.tracked_element_rects == other.tracked_element_rects;
        let equal = equal && self.local_surface_id == other.local_surface_id;
        let equal =
            equal && self.new_vertical_scroll_direction == other.new_vertical_scroll_direction;
        let equal = equal
            && self.primary_main_frame_item_sequence_number
                == other.primary_main_frame_item_sequence_number;
        // Source deliberately omits has_offset_tag from equality.
        equal
    }
    // Source: cc/trees/render_frame_metadata.cc:60-62.
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
