// Translation status: complete
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/compositor_commit_data.h:1-143
// Source: cc/trees/compositor_commit_data.cc:1-35

use crate::{
    browser_controls_state::BrowserControlsState, scroll_snap_data::TargetSnapAreaElementIds,
    scroll_source_type::ScrollSourceType, snap_selection_strategy::SnapSelectionStrategy,
};
use compositor_tree::{
    commit_types::{
        kManipulationInfoNone, ManipulationInfo, SharedAnimatedImageFrameIndexMap, SwapPromise,
    },
    geometry::Vector2dF,
    ElementId,
};
use std::collections::BTreeSet;

// Source: cc/trees/compositor_commit_data.h:28-139;
//         cc/trees/compositor_commit_data.cc:11-35.
pub struct CompositorCommitData {
    // Source: cc/trees/compositor_commit_data.h:35-55;
    //         cc/trees/compositor_commit_data.cc:15-30.
    pub inner_viewport_scroll: ScrollUpdateInfo,
    // Source: cc/trees/compositor_commit_data.h:57-66.
    pub scrolls: Vec<ScrollUpdateInfo>,
    pub page_scale_delta: f32,
    pub is_pinch_gesture_active: bool,
    pub is_scroll_active: bool,
    // Source: cc/trees/compositor_commit_data.h:68-80.
    pub elastic_overscroll_delta: Vector2dF,
    pub overscroll_delta: Vector2dF,
    pub scroll_latched_element_id: ElementId,
    pub top_controls_delta: f32,
    pub bottom_controls_delta: f32,
    // Source: cc/trees/compositor_commit_data.h:83-102.
    pub scrollbars: Vec<ScrollbarsUpdateInfo>,
    pub swap_promises: Vec<Box<dyn SwapPromise>>,
    pub browser_controls_constraint: BrowserControlsState,
    pub browser_controls_constraint_changed: bool,
    // Source: cc/trees/compositor_commit_data.h:103-139.
    pub scroll_end_data: ScrollEndInfo,
    pub ongoing_scroll_animation: bool,
    pub is_auto_scrolling: bool,
    pub manipulation_info: ManipulationInfo,
    pub snap_strategy: Option<Box<dyn SnapSelectionStrategy>>,
    pub scroll_type: ScrollSourceType,
    pub advanced_image_animation_clients: BTreeSet<ElementId>,
    pub animated_image_frame_index_map: Option<SharedAnimatedImageFrameIndexMap>,
}

// Source: cc/trees/compositor_commit_data.h:35-55;
//         cc/trees/compositor_commit_data.cc:15-30.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScrollUpdateInfo {
    pub element_id: ElementId,
    pub scroll_delta: Vector2dF,
    pub snap_target_element_ids: Option<TargetSnapAreaElementIds>,
}

impl ScrollUpdateInfo {
    pub fn new(
        id: ElementId,
        delta: Vector2dF,
        snap_target_ids: Option<TargetSnapAreaElementIds>,
    ) -> Self {
        Self {
            element_id: id,
            scroll_delta: delta,
            snap_target_element_ids: snap_target_ids,
        }
    }
}

// Source: cc/trees/compositor_commit_data.h:88-95.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollbarsUpdateInfo {
    pub element_id: ElementId,
    pub hidden: bool,
}

impl Default for ScrollbarsUpdateInfo {
    fn default() -> Self {
        Self {
            element_id: ElementId::default(),
            hidden: true,
        }
    }
}

// Source: cc/trees/compositor_commit_data.h:103-111;
//         cc/trees/compositor_commit_data.cc:32-33.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScrollEndInfo {
    pub done_containers: BTreeSet<ElementId>,
}

// Source: cc/trees/compositor_commit_data.h:29-33;
//         cc/trees/compositor_commit_data.cc:11-13.
impl Default for CompositorCommitData {
    fn default() -> Self {
        Self {
            inner_viewport_scroll: ScrollUpdateInfo::default(),
            scrolls: Vec::new(),
            page_scale_delta: 1.0,
            is_pinch_gesture_active: false,
            is_scroll_active: false,
            elastic_overscroll_delta: Vector2dF::default(),
            overscroll_delta: Vector2dF::default(),
            scroll_latched_element_id: ElementId::default(),
            top_controls_delta: 0.0,
            bottom_controls_delta: 0.0,
            scrollbars: Vec::new(),
            swap_promises: Vec::new(),
            browser_controls_constraint: BrowserControlsState::kBoth,
            browser_controls_constraint_changed: false,
            scroll_end_data: ScrollEndInfo::default(),
            ongoing_scroll_animation: false,
            is_auto_scrolling: false,
            manipulation_info: kManipulationInfoNone,
            snap_strategy: None,
            scroll_type: ScrollSourceType::kNone,
            advanced_image_animation_clients: BTreeSet::new(),
            animated_image_frame_index_map: None,
        }
    }
}
