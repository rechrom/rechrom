// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::property_ids::{kInvalidPropertyNodeId, kRootPropertyNodeId};
use crate::{
    adapter::{
        geometry::{Point, Size},
        input_types::ElementId,
        property_types::{AddElementIdToTracedValue, PropertyTrace, RectFToString},
    },
    cc::{
        base::math_util::MathUtil,
        input::{
            main_thread_scrolling_reason::MainThreadRepaintReasons,
            overscroll_behavior::OverscrollBehavior, scroll_snap_data::SnapContainerData,
        },
    },
};

// Source: cc/trees/scroll_node.h:29-73
// Source: cc/trees/scroll_node.cc:16-22
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollNode {
    // Source: cc/trees/scroll_node.h:34
    pub id: i32,
    // Source: cc/trees/scroll_node.h:36
    pub parent_id: i32,
    // Source: cc/trees/scroll_node.h:38
    pub main_thread_repaint_reasons: MainThreadRepaintReasons,
    // Source: cc/trees/scroll_node.h:45
    pub container_bounds: Size,
    // Source: cc/trees/scroll_node.h:48
    pub bounds: Size,
    // Source: cc/trees/scroll_node.h:50
    pub max_scroll_offset_affected_by_page_scale: bool,
    // Source: cc/trees/scroll_node.h:51
    pub scrolls_inner_viewport: bool,
    // Source: cc/trees/scroll_node.h:52
    pub scrolls_outer_viewport: bool,
    // Source: cc/trees/scroll_node.h:53
    pub prevent_viewport_scrolling_from_inner: bool,
    // Source: cc/trees/scroll_node.h:54
    pub user_scrollable_horizontal: bool,
    // Source: cc/trees/scroll_node.h:55
    pub user_scrollable_vertical: bool,
    // Source: cc/trees/scroll_node.h:56
    pub is_composited: bool,
    // Source: cc/trees/scroll_node.h:57
    pub prevent_scroll_axis_locking: bool,
    // Source: cc/trees/scroll_node.h:59
    pub element_id: ElementId,
    // Source: cc/trees/scroll_node.h:62
    pub transform_id: i32,
    // Source: cc/trees/scroll_node.h:65
    pub container_origin: Point,
    pub overscroll_behavior: OverscrollBehavior,
    // Source: cc/trees/scroll_node.h:69
    pub snap_container_data: Option<SnapContainerData>,
}
impl Default for ScrollNode {
    // Source: cc/trees/scroll_node.h:35-68
    fn default() -> Self {
        Self {
            id: kInvalidPropertyNodeId,
            parent_id: kInvalidPropertyNodeId,
            main_thread_repaint_reasons: MainThreadRepaintReasons::default(),
            container_bounds: Size::default(),
            bounds: Size::default(),
            max_scroll_offset_affected_by_page_scale: false,
            scrolls_inner_viewport: false,
            scrolls_outer_viewport: false,
            prevent_viewport_scrolling_from_inner: false,
            user_scrollable_horizontal: false,
            user_scrollable_vertical: false,
            is_composited: false,
            prevent_scroll_axis_locking: false,
            element_id: ElementId::default(),
            transform_id: kRootPropertyNodeId,
            container_origin: Point::default(),
            overscroll_behavior: OverscrollBehavior::default(),
            snap_container_data: None,
        }
    }
}
impl ScrollNode {
    // Source: cc/trees/scroll_node.cc:16-18
    pub fn new() -> Self {
        Self::default()
    }
    // Source: cc/trees/scroll_node.cc:24-58
    pub fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        value.SetInteger("id", self.id);
        value.SetInteger("parent_id", self.parent_id);
        MathUtil::AddToTracedValue("container_bounds", &self.container_bounds, value);
        MathUtil::AddToTracedValue("bounds", &self.bounds, value);
        value.SetBoolean(
            "user_scrollable_horizontal",
            self.user_scrollable_horizontal,
        );
        value.SetBoolean("user_scrollable_vertical", self.user_scrollable_vertical);
        value.SetBoolean("scrolls_inner_viewport", self.scrolls_inner_viewport);
        value.SetBoolean("scrolls_outer_viewport", self.scrolls_outer_viewport);
        value.SetBoolean(
            "prevent_viewport_scrolling_from_inner",
            self.prevent_viewport_scrolling_from_inner,
        );
        AddElementIdToTracedValue(self.element_id, value);
        value.SetInteger("transform_id", self.transform_id);
        value.SetInteger("overscroll_behavior_x", self.overscroll_behavior.x as i32);
        value.SetInteger("overscroll_behavior_y", self.overscroll_behavior.y as i32);
        if let Some(snap_container_data) = &self.snap_container_data {
            value.SetString(
                "snap_container_rect",
                &RectFToString(snap_container_data.rect()),
            );
            if snap_container_data.size() != 0 {
                value.BeginArray("snap_area_rects");
                for i in 0..snap_container_data.size() {
                    value.AppendString(&RectFToString(snap_container_data.at(i).rect));
                }
                value.EndArray();
            }
        }
        value.SetBoolean(
            "prevent_scroll_axis_locking",
            self.prevent_scroll_axis_locking,
        );
        value.SetBoolean("is_composited", self.is_composited);
    }
}
