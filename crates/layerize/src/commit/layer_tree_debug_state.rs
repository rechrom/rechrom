// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/debug/layer_tree_debug_state.h:1-72; cc/debug/layer_tree_debug_state.cc:1-15.
// Source: cc/debug/layer_tree_debug_state.h:14-21.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum DebugBorderType {
    RENDERPASS = 0,
    SURFACE,
    LAYER,
}
pub const LAST_DEBUG_BORDER_TYPE: DebugBorderType = DebugBorderType::LAYER;
// Source dependency: std::bitset<LAST_DEBUG_BORDER_TYPE+1>, with fixed inline storage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DebugBorderTypes(u8);
impl DebugBorderTypes {
    pub fn test(&self, index: usize) -> bool {
        assert!(index < 3);
        self.0 & (1 << index) != 0
    }
    pub fn set(&mut self, index: usize, value: bool) {
        assert!(index < 3);
        if value {
            self.0 |= 1 << index;
        } else {
            self.0 &= !(1 << index);
        }
    }
    pub fn any(&self) -> bool {
        self.0 != 0
    }
    pub fn none(&self) -> bool {
        !self.any()
    }
    pub fn reset(&mut self) {
        self.0 = 0;
    }
    pub fn count(&self) -> usize {
        self.0.count_ones() as usize
    }
}
// Source: cc/debug/layer_tree_debug_state.h:23-67; cc/debug/layer_tree_debug_state.cc:10-15,54-55.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayerTreeDebugState {
    // Source: cc/debug/layer_tree_debug_state.h:28.
    pub debugger_paused: bool,
    // Source: cc/debug/layer_tree_debug_state.h:29.
    pub show_fps_counter: bool,
    // Source: cc/debug/layer_tree_debug_state.h:30.
    pub show_debug_borders: DebugBorderTypes,
    // Source: cc/debug/layer_tree_debug_state.h:32.
    pub show_contentful_paint_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:33.
    pub show_layout_shift_regions: bool,
    // Source: cc/debug/layer_tree_debug_state.h:34.
    pub show_paint_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:35.
    pub show_property_changed_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:36.
    pub show_surface_damage_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:37.
    pub show_screen_space_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:38.
    pub show_touch_event_handler_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:39.
    pub show_wheel_event_handler_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:40.
    pub show_scroll_event_handler_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:41.
    pub show_main_thread_scroll_hit_test_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:42.
    pub show_main_thread_scroll_repaint_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:43.
    pub show_raster_inducing_scroll_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:44.
    pub show_layer_animation_bounds_rects: bool,
    // Source: cc/debug/layer_tree_debug_state.h:46.
    pub slow_down_raster_scale_factor: i32,
    // Source: cc/debug/layer_tree_debug_state.h:47.
    pub rasterize_only_visible_content: bool,
    // Source: cc/debug/layer_tree_debug_state.h:67.
    record_rendering_stats_: bool,
}
impl LayerTreeDebugState {
    // Source: cc/debug/layer_tree_debug_state.cc:17-19.
    pub fn SetRecordRenderingStats(&mut self, enabled: bool) {
        self.record_rendering_stats_ = enabled;
    }
    // Source: cc/debug/layer_tree_debug_state.cc:21-23.
    pub fn RecordRenderingStats(&self) -> bool {
        self.record_rendering_stats_
    }
    // Source: cc/debug/layer_tree_debug_state.cc:25-27.
    pub fn ShouldCreateHudLayer(&self) -> bool {
        self.ShowDebugRects() || self.ShouldDrawHudInfo()
    }
    // Source: cc/debug/layer_tree_debug_state.cc:29-39.
    pub fn ShowDebugRects(&self) -> bool {
        self.show_paint_rects
            || self.show_property_changed_rects
            || self.show_surface_damage_rects
            || self.show_screen_space_rects
            || self.show_touch_event_handler_rects
            || self.show_wheel_event_handler_rects
            || self.show_scroll_event_handler_rects
            || self.show_main_thread_scroll_hit_test_rects
            || self.show_main_thread_scroll_repaint_rects
            || self.show_raster_inducing_scroll_rects
            || self.show_layer_animation_bounds_rects
            || self.show_layout_shift_regions
            || self.show_contentful_paint_rects
    }
    // Source: cc/debug/layer_tree_debug_state.cc:41-43.
    pub fn ShowMemoryStats(&self) -> bool {
        self.show_fps_counter
    }
    // Source: cc/debug/layer_tree_debug_state.cc:45-47.
    pub fn ShouldDrawHudInfo(&self) -> bool {
        self.show_fps_counter || self.debugger_paused
    }
    // Source: cc/debug/layer_tree_debug_state.cc:49-52.
    pub fn TurnOffHudInfoDisplay(&mut self) {
        self.show_fps_counter = false;
    }
}
