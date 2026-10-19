// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::property_ids::kInvalidPropertyNodeId;
use crate::{
    adapter::{
        geometry::RectF,
        property_types::{InlinedVector, PropertyTrace, TracedValueJSON},
    },
    cc::base::math_util::MathUtil,
};
use std::cell::RefCell;

// Source: cc/trees/clip_node.h:23-26
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConditionalClip {
    // Source: cc/trees/clip_node.h:24
    pub is_clipped: bool,
    // Source: cc/trees/clip_node.h:25
    pub clip_rect: RectF,
}
// Source: cc/trees/clip_node.h:28-31
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipRectData {
    // Source: cc/trees/clip_node.h:29
    pub target_id: i32,
    // Source: cc/trees/clip_node.h:30
    pub clip: ConditionalClip,
}

// Source: cc/trees/clip_node.h:33-82
// Source: cc/trees/clip_node.cc:14-17
#[derive(Clone, Debug)]
pub struct ClipNode {
    // Source: cc/trees/clip_node.h:46
    pub id: i32,
    // Source: cc/trees/clip_node.h:48
    pub parent_id: i32,
    // Source: cc/trees/clip_node.h:30
    pub clip: RectF,
    // Source: cc/trees/clip_node.h:61
    pub cached_clip_rects: RefCell<InlinedVector<ClipRectData, 3>>,
    // Source: cc/trees/clip_node.h:65
    pub cached_accumulated_rect_in_screen_space: RectF,
    // Source: cc/trees/clip_node.h:71
    pub pixel_moving_filter_id: i32,
    // Source: cc/trees/clip_node.h:74
    pub transform_id: i32,
}
impl Default for ClipNode {
    // Source: cc/trees/clip_node.h:46-74
    fn default() -> Self {
        Self {
            id: kInvalidPropertyNodeId,
            parent_id: kInvalidPropertyNodeId,
            clip: RectF::default(),
            cached_clip_rects: RefCell::default(),
            cached_accumulated_rect_in_screen_space: RectF::default(),
            pixel_moving_filter_id: kInvalidPropertyNodeId,
            transform_id: kInvalidPropertyNodeId,
        }
    }
}
// Source: cc/trees/clip_node.cc:24-28
impl PartialEq for ClipNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent_id == other.parent_id
            && self.clip == other.clip
            && self.pixel_moving_filter_id == other.pixel_moving_filter_id
            && self.transform_id == other.transform_id
    }
}
impl ClipNode {
    // Source: cc/trees/clip_node.cc:14-17
    pub fn new() -> Self {
        Self::default()
    }
    // Source: cc/trees/clip_node.cc:19-21
    pub fn AppliesLocalClip(&self) -> bool {
        self.pixel_moving_filter_id == kInvalidPropertyNodeId
    }
    // Source: cc/trees/clip_node.cc:30-34
    pub fn ToString(&self) -> String {
        let mut value = TracedValueJSON::default();
        self.AsValueInto(&mut value);
        value.ToFormattedJSON()
    }
    // Source: cc/trees/clip_node.cc:37-43
    pub fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        value.SetInteger("id", self.id);
        value.SetInteger("parent_id", self.parent_id);
        MathUtil::AddToTracedValue("clip", &self.clip, value);
        value.SetInteger("pixel_moving_filter_id", self.pixel_moving_filter_id);
        value.SetInteger("transform_id", self.transform_id);
    }
}
