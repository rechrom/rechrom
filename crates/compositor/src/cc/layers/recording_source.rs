// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/recording_source.h:1-89;
//         cc/layers/recording_source.cc:1-166.
// Rust modules replace includes, guards, exports and namespaces.

use super::content_layer_client::ContentLayerClient;
use crate::{
    adapter::{
        geometry::{Rect, Size},
        layer_tree_impl::Color4f,
        recording_source::{
            LayerRect, RecordingDisplayItemList, RecordingSourceServices, RecordingSourceTypes,
            ScaleToRoundedLayerRect,
        },
    },
    cc::base::{invalidation_region::InvalidationRegion, region::Region},
};
use std::sync::Arc;

// Source: cc/layers/recording_source.cc:18-24.
const kMaxOpsToAnalyzeForLayer: usize = 10;

// Source: cc/layers/recording_source.h:25-31,60-85.
// RecordingSource owns the display-list Arc and invalidation accumulator. Its
// RasterSource is intentionally an external typed product because playback,
// pixels and GPU/CPU resource implementations are outside compositor core.
pub struct RecordingSource<B: RecordingSourceTypes> {
    recorded_bounds_: Rect,
    size_: Size,
    slow_down_raster_scale_factor_for_debug_: i32,
    requires_clear_: bool,
    is_solid_color_: bool,
    can_use_recorded_bounds_: bool,
    force_update_: bool,
    solid_color_: Color4f,
    background_color_: Color4f,
    display_list_: Option<Arc<B::DisplayItemList>>,
    recording_scale_factor_: f32,
    directly_composited_image_info_: Option<B::DirectlyCompositedImageInfo>,
    invalidation_: InvalidationRegion<B::RegionBackend>,
}

/// Source: cc/raster/raster_source.cc:26-40. The RasterSource constructor is a
/// source friend of RecordingSource. Rust lends exactly those resident fields
/// to the external immutable metadata constructor, without a state snapshot.
pub struct RasterSourceFields<'a, B: RecordingSourceTypes> {
    pub display_list: &'a Option<Arc<B::DisplayItemList>>,
    pub background_color: &'a Color4f,
    pub requires_clear: &'a bool,
    pub is_solid_color: &'a bool,
    pub solid_color: &'a Color4f,
    pub recorded_bounds: &'a Rect,
    pub size: &'a Size,
    pub slow_down_raster_scale_factor_for_debug: &'a i32,
    pub recording_scale_factor: &'a f32,
    pub directly_composited_image_info: &'a Option<B::DirectlyCompositedImageInfo>,
}

// Source: cc/layers/recording_source.h:27-31;
//         cc/layers/recording_source.cc:28-29.
impl<B: RecordingSourceTypes> Default for RecordingSource<B> {
    fn default() -> Self {
        Self {
            recorded_bounds_: Rect::default(),
            size_: Size::default(),
            slow_down_raster_scale_factor_for_debug_: 0,
            requires_clear_: false,
            is_solid_color_: false,
            can_use_recorded_bounds_: false,
            force_update_: false,
            solid_color_: Color4f::default(),
            background_color_: Color4f::default(),
            display_list_: None,
            recording_scale_factor_: 1.0,
            directly_composited_image_info_: None,
            invalidation_: InvalidationRegion::new(),
        }
    }
}

impl<B: RecordingSourceTypes> RecordingSource<B> {
    pub fn RasterSourceFields(&self) -> RasterSourceFields<'_, B> {
        RasterSourceFields {
            display_list: &self.display_list_,
            background_color: &self.background_color_,
            requires_clear: &self.requires_clear_,
            is_solid_color: &self.is_solid_color_,
            solid_color: &self.solid_color_,
            recorded_bounds: &self.recorded_bounds_,
            size: &self.size_,
            slow_down_raster_scale_factor_for_debug: &self.slow_down_raster_scale_factor_for_debug_,
            recording_scale_factor: &self.recording_scale_factor_,
            directly_composited_image_info: &self.directly_composited_image_info_,
        }
    }
    pub fn new() -> Self {
        Self::default()
    }

    // Source: cc/layers/recording_source.h:33-36;
    //         cc/layers/recording_source.cc:67-115.
    pub fn Update<C: crate::adapter::recording_source::RecordingSourceContext<B> + ?Sized>(
        &mut self,
        layer_size: Size,
        recording_scale_factor: f32,
        client: &mut dyn ContentLayerClient<B::DisplayItemList, C>,
        invalidation: &mut Region<B::RegionBackend>,
        context: &mut C,
    ) -> bool {
        self.invalidation_.Swap(invalidation);
        self.invalidation_.Clear();
        if self.size_ == layer_size && invalidation.IsEmpty() && !self.force_update_ {
            return false;
        }
        self.size_ = layer_size;
        self.recording_scale_factor_ = recording_scale_factor;
        let display_list = client.PaintContentsToDisplayList(context);
        let same_display_list = match (&self.display_list_, &display_list) {
            (None, None) => true,
            (Some(old), Some(new)) => Arc::ptr_eq(old, new),
            _ => false,
        };
        if same_display_list {
            assert!(!self.force_update_);
            return true;
        }
        self.force_update_ = false;
        let layer_rect = LayerRect(self.size_);
        let mut new_recorded_bounds = layer_rect;
        if self.can_use_recorded_bounds_ {
            if let Some(display_list_bounds) = display_list
                .as_ref()
                .expect("source display_list->bounds()")
                .bounds()
            {
                new_recorded_bounds.Intersect(display_list_bounds);
            }
        }
        if self.display_list_.as_ref().is_some_and(|old| {
            display_list
                .as_ref()
                .expect("source display_list->NeedsAdditionalInvalidationForLCDText")
                .NeedsAdditionalInvalidationForLCDText(old)
        }) {
            invalidation.AssignRect(layer_rect);
        } else if new_recorded_bounds != self.recorded_bounds_ {
            self.UpdateInvalidationForRecordedBounds(
                self.recorded_bounds_,
                new_recorded_bounds,
                invalidation,
            );
        }
        self.recorded_bounds_ = new_recorded_bounds;
        self.display_list_ = display_list;
        self.FinishDisplayItemListUpdate(context.RecordingServices());
        true
    }

    // Source: cc/layers/recording_source.h:37-43,45-50.
    pub fn size(&self) -> Size {
        self.size_
    }
    pub fn display_list(&self) -> Option<&B::DisplayItemList> {
        self.display_list_.as_deref()
    }
    pub fn SetEmptyBounds(&mut self) {
        self.size_ = Size::default();
        self.is_solid_color_ = false;
        self.recorded_bounds_ = Rect::default();
        self.display_list_ = None;
    }
    pub fn SetSlowdownRasterScaleFactor(&mut self, factor: i32) {
        self.slow_down_raster_scale_factor_for_debug_ = factor;
    }
    pub fn SetBackgroundColor(&mut self, background_color: Color4f) {
        self.background_color_ = background_color;
    }
    pub fn SetRequiresClear(&mut self, requires_clear: bool) {
        self.requires_clear_ = requires_clear;
    }
    pub fn SetCanUseRecordedBounds(&mut self, can_use_recorded_bounds: bool) {
        if self.can_use_recorded_bounds_ == can_use_recorded_bounds {
            return;
        }
        self.can_use_recorded_bounds_ = can_use_recorded_bounds;
        self.size_ = Size::default();
    }
    pub fn SetNeedsDisplayRect(&mut self, layer_rect: Rect) {
        if !layer_rect.IsEmpty() {
            let mut clamped = layer_rect;
            clamped.Intersect(LayerRect(self.size_));
            self.invalidation_.Union(clamped);
        }
    }
    pub fn set_force_update(&mut self) {
        self.force_update_ = true;
    }
    pub fn CreateRasterSource(
        &self,
        factory: &(impl RecordingSourceServices<B> + ?Sized),
    ) -> Arc<B::RasterSource> {
        factory.CreateRasterSource(self)
    }

    // Source: cc/layers/recording_source.h:52-58.
    pub fn recorded_bounds(&self) -> &Rect {
        &self.recorded_bounds_
    }
    pub fn is_solid_color(&self) -> bool {
        self.is_solid_color_
    }
    pub fn directly_composited_image_info(&self) -> Option<&B::DirectlyCompositedImageInfo> {
        self.directly_composited_image_info_.as_ref()
    }

    // Source: cc/layers/recording_source.h:75-78;
    //         cc/layers/recording_source.cc:31-58.
    fn UpdateInvalidationForRecordedBounds(
        &self,
        old_recorded_bounds: Rect,
        new_recorded_bounds: Rect,
        invalidation: &mut Region<B::RegionBackend>,
    ) {
        let mut newly_exposed_region = Region::new(new_recorded_bounds);
        newly_exposed_region.Subtract(&old_recorded_bounds);
        invalidation.Union(&newly_exposed_region);
        let mut no_longer_exposed_region = Region::new(old_recorded_bounds);
        no_longer_exposed_region.Subtract(&new_recorded_bounds);
        invalidation.Union(&no_longer_exposed_region);
    }
    fn FinishDisplayItemListUpdate(
        &mut self,
        services: &(impl RecordingSourceServices<B> + ?Sized),
    ) {
        services.TraceScope("cc", "RecordingSource::FinishDisplayItemListUpdate", None);
        self.DetermineIfSolidColor(services);
        let display_list = self.display_list_.as_ref().expect("display list");
        display_list.EmitTraceSnapshot();
        self.directly_composited_image_info_ = display_list.GetDirectlyCompositedImageInfo();
        if self.directly_composited_image_info_.is_some() {
            self.SetRequiresClear(true);
        }
    }

    // Source: cc/layers/recording_source.h:80-84;
    //         cc/layers/recording_source.cc:146-164.
    fn DetermineIfSolidColor(&mut self, services: &(impl RecordingSourceServices<B> + ?Sized)) {
        let display_list = self.display_list_.as_ref().expect("display list");
        self.is_solid_color_ = false;
        self.solid_color_ = Color4f::default();
        if display_list.TotalOpCount() > kMaxOpsToAnalyzeForLayer {
            return;
        }
        services.TraceScope(
            "cc",
            "RecordingSource::DetermineIfSolidColor",
            Some(display_list.TotalOpCount()),
        );
        self.is_solid_color_ = display_list.GetColorIfSolidInRect(
            ScaleToRoundedLayerRect(self.size_, self.recording_scale_factor_),
            &mut self.solid_color_,
            kMaxOpsToAnalyzeForLayer,
        );
    }
}

// Source: cc/layers/recording_source.cc:117-144.
// Setter bodies are grouped above with their declarations so their behavior is
// visible beside each resident field; every source branch is preserved.

// Source: cc/layers/recording_source.cc:166; cc/layers/recording_source.h:87-89.
// Namespace and include-guard closure map to this module boundary.
