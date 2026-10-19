// Translation status: complete
// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/commit_state.h:1-226; cc/trees/commit_state.cc:1-9.
use super::{
    browser_controls_params::BrowserControlsParams, viewport_property_ids::ViewportPropertyIds,
};
use crate::adapter::{
    commit_state::*,
    commit_types::BeginMainFrameTraceId,
    geometry::{Rect, Size, Vector2dF},
    input_types::ElementId,
    layer_tree_impl::{
        Color4f, OverlayTransform, PropertyChangeForcesCommitCriteria, PropertyTreesFor,
        UIResourceId, UnguessableToken,
    },
    property_tree::PropertyTreesOwnerId,
    time::{TimeDelta, TimeTicks},
};
use crate::cc::{
    debug::layer_tree_debug_state::LayerTreeDebugState,
    input::{
        event_listener_properties::{
            EventListenerClass, EventListenerProperties, kEventListenerClassCount,
        },
        layer_selection_bound::LayerSelection,
        overscroll_behavior::OverscrollBehavior,
        page_scale_animation::PendingPageScaleAnimation,
    },
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
// Source: cc/trees/commit_state.h:51.
pub const kInvalidSourceFrameNumber: i32 = -1;
// Source: cc/trees/commit_state.h:62-190.
// Rust move/drop supplies ownership transfer and cleanup of take/reset members.
// Source deletes copy assignment; CopyPersistent is the selective copy constructor.
pub struct CommitState<B: CommitStateTypes> {
    // Source: cc/trees/commit_state.h:77.
    pub surface_ranges: BTreeMap<B::SurfaceRange, i32>,
    // Source: cc/trees/commit_state.h:78.
    pub visual_properties_update_duration: TimeDelta,
    // Source: cc/trees/commit_state.h:79.
    pub needs_gpu_rasterization_histogram: bool,
    // Source: cc/trees/commit_state.h:80.
    pub have_scroll_event_handlers: bool,
    // Source: cc/trees/commit_state.h:81.
    pub is_external_pinch_gesture_active: bool,
    // Source: cc/trees/commit_state.h:86.
    pub is_viewport_mobile_optimized: bool,
    // Source: cc/trees/commit_state.h:87.
    pub may_throttle_if_undrawn_frames: bool,
    // Source: cc/trees/commit_state.h:88.
    pub prefers_reduced_motion: bool,
    // Source: cc/trees/commit_state.h:89.
    pub browser_controls_params: BrowserControlsParams,
    // Source: cc/trees/commit_state.h:91.
    pub event_listener_properties: [EventListenerProperties; kEventListenerClassCount],
    // Source: cc/trees/commit_state.h:92.
    pub bottom_controls_shown_ratio: f32,
    // Source: cc/trees/commit_state.h:93.
    pub device_scale_factor: f32,
    // Source: cc/trees/commit_state.h:94.
    pub external_page_scale_factor: f32,
    // Source: cc/trees/commit_state.h:95.
    pub page_scale_factor_limits_set: bool,
    // Source: cc/trees/commit_state.h:96.
    pub max_page_scale_factor: f32,
    // Source: cc/trees/commit_state.h:97.
    pub min_page_scale_factor: f32,
    // Source: cc/trees/commit_state.h:98.
    pub page_scale_factor: f32,
    // Source: cc/trees/commit_state.h:99.
    pub painted_device_scale_factor: f32,
    // Source: cc/trees/commit_state.h:100.
    pub top_controls_shown_ratio: f32,
    // Source: cc/trees/commit_state.h:101.
    pub display_color_spaces: B::DisplayColorSpaces,
    // Source: cc/trees/commit_state.h:103.
    pub display_transform_hint: OverlayTransform,
    // Source: cc/trees/commit_state.h:104.
    pub device_viewport_rect: Rect,
    // Source: cc/trees/commit_state.h:105.
    pub visual_device_viewport_size: Size,
    // Source: cc/trees/commit_state.h:106.
    pub elastic_overscroll: Vector2dF,
    // Source: cc/trees/commit_state.h:108.
    pub max_safe_area_insets: InsetsF,
    // Source: cc/trees/commit_state.h:109.
    pub hud_layer_id: i32,
    // Source: cc/trees/commit_state.h:110.
    pub source_frame_number: i32,
    // Source: cc/trees/commit_state.h:111.
    pub selection: LayerSelection,
    // Source: cc/trees/commit_state.h:112.
    pub debug_state: LayerTreeDebugState,
    // Source: cc/trees/commit_state.h:113.
    pub overscroll_behavior: OverscrollBehavior,
    // Source: cc/trees/commit_state.h:114.
    pub background_color: Color4f,
    // Source: cc/trees/commit_state.h:115.
    pub viewport_property_ids: ViewportPropertyIds,
    // Source: cc/trees/commit_state.h:116.
    pub local_surface_id_from_parent: B::LocalSurfaceId,
    // Source: cc/trees/commit_state.h:123.
    pub clear_caches_on_next_commit: bool,
    // Source: cc/trees/commit_state.h:126.
    pub force_send_metadata_request: bool,
    // Source: cc/trees/commit_state.h:127.
    pub commit_waits_for_activation: bool,
    // Source: cc/trees/commit_state.h:128.
    pub needs_full_tree_sync: bool,
    // Source: cc/trees/commit_state.h:129.
    pub needs_surface_ranges_sync: bool,
    // Source: cc/trees/commit_state.h:130.
    pub new_local_surface_id_request: bool,
    // Source: cc/trees/commit_state.h:131.
    pub next_commit_forces_recalculate_raster_scales: bool,
    // Source: cc/trees/commit_state.h:132.
    pub next_commit_forces_redraw: bool,
    // Source: cc/trees/commit_state.h:133.
    pub property_change_forces_commit_criteria: PropertyChangeForcesCommitCriteria,
    // Source: cc/trees/commit_state.h:135.
    pub trace_id: BeginMainFrameTraceId,
    // Source: cc/trees/commit_state.h:136.
    pub event_metrics: Vec<B::EventMetrics>,
    // Source: cc/trees/commit_state.h:138.
    pub property_trees: PropertyTreesFor<B>,
    // Source: cc/trees/commit_state.h:144.
    pub begin_main_frame_metrics: Option<Box<B::BeginMainFrameMetrics>>,
    // Source: cc/trees/commit_state.h:150.
    pub delegated_ink_metadata: Option<Box<B::DelegatedInkMetadata>>,
    // Source: cc/trees/commit_state.h:152.
    pub pending_page_scale_animation: Option<Box<PendingPageScaleAnimation>>,
    // Source: cc/trees/commit_state.h:154.
    pub queued_image_decodes: Vec<(i32, Box<B::DrawImage>, bool)>,
    // Rust fields drop in declaration order, while C++ members drop in reverse.
    // Keep callback capture cancellation in the source destructor order.
    // Source: cc/trees/commit_state.h:161.
    pub pending_successful_presentation_callbacks: Vec<B::SuccessfulPresentationCallback>,
    // Source: cc/trees/commit_state.h:159.
    pub pending_presentation_callbacks: Vec<B::PresentationCallback>,
    // Source: cc/trees/commit_state.h:163.
    // Moved-from source unique_ptr slots remain present after FinishCommit.
    pub benchmarks: Vec<Option<Box<B::MicroBenchmarkImpl>>>,
    // Source: cc/trees/commit_state.h:167.
    // std::move of each source unique_ptr retains null entries in this vector.
    pub view_transition_requests: Vec<Option<Box<B::ViewTransitionRequest>>>,
    // Source: cc/trees/commit_state.h:169.
    pub swap_promises: Vec<Box<B::SwapPromise>>,
    // Source: cc/trees/commit_state.h:170.
    pub ui_resource_request_queue: Vec<B::UIResourceRequest>,
    // Source: cc/trees/commit_state.h:171.
    pub ui_resource_sizes: BTreeMap<UIResourceId, Size>,
    // Source: cc/trees/commit_state.h:172.
    pub layer_ids_that_should_push_properties: BTreeSet<i32>,
    // Source: cc/trees/commit_state.h:173.
    pub picture_layer_ids_with_new_raster_source: BTreeSet<i32>,
    // Source: cc/trees/commit_state.h:174.
    pub layer_update_rects: BTreeMap<i32, Rect>,
    // Source: cc/trees/commit_state.h:179.
    pub scrollers_clobbering_active_value: BTreeSet<ElementId>,
    // Source: cc/trees/commit_state.h:183.
    pub screenshot_destination_token: UnguessableToken,
    // Source: cc/trees/commit_state.h:188.
    pub primary_main_frame_item_sequence_number: i64,
}
impl<B: CommitStateTypes> CommitState<B> {
    // Source: cc/trees/commit_state.cc:11-12; cc/trees/commit_state.h:77-189.
    // The embedder supplies the new canonical PropertyTrees identity.
    pub fn new(owner: PropertyTreesOwnerId) -> Self {
        Self {
            surface_ranges: BTreeMap::new(),
            visual_properties_update_duration: TimeDelta::default(),
            needs_gpu_rasterization_histogram: false,
            have_scroll_event_handlers: false,
            is_external_pinch_gesture_active: false,
            is_viewport_mobile_optimized: false,
            may_throttle_if_undrawn_frames: true,
            prefers_reduced_motion: false,
            browser_controls_params: BrowserControlsParams::default(),
            event_listener_properties: [EventListenerProperties::kNone; kEventListenerClassCount],
            bottom_controls_shown_ratio: 0.0,
            device_scale_factor: 1.0,
            external_page_scale_factor: 1.0,
            page_scale_factor_limits_set: false,
            max_page_scale_factor: 1.0,
            min_page_scale_factor: 1.0,
            page_scale_factor: 1.0,
            painted_device_scale_factor: 1.0,
            top_controls_shown_ratio: 0.0,
            display_color_spaces: B::DisplayColorSpaces::default(),
            display_transform_hint: OverlayTransform::None,
            device_viewport_rect: Rect::default(),
            visual_device_viewport_size: Size::default(),
            elastic_overscroll: Vector2dF::default(),
            max_safe_area_insets: InsetsF::default(),
            hud_layer_id: -1,
            source_frame_number: 0,
            selection: LayerSelection::default(),
            debug_state: LayerTreeDebugState::default(),
            overscroll_behavior: OverscrollBehavior::default(),
            background_color: Color4f {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
            viewport_property_ids: ViewportPropertyIds::default(),
            local_surface_id_from_parent: B::LocalSurfaceId::default(),
            clear_caches_on_next_commit: false,
            force_send_metadata_request: false,
            commit_waits_for_activation: false,
            needs_full_tree_sync: false,
            needs_surface_ranges_sync: false,
            new_local_surface_id_request: false,
            next_commit_forces_recalculate_raster_scales: false,
            next_commit_forces_redraw: false,
            property_change_forces_commit_criteria: PropertyChangeForcesCommitCriteria::kNone,
            trace_id: BeginMainFrameTraceId(0),
            event_metrics: Vec::new(),
            property_trees: PropertyTreesFor::<B>::new(owner),
            begin_main_frame_metrics: None,
            delegated_ink_metadata: None,
            pending_page_scale_animation: None,
            queued_image_decodes: Vec::new(),
            pending_presentation_callbacks: Vec::new(),
            pending_successful_presentation_callbacks: Vec::new(),
            benchmarks: Vec::new(),
            view_transition_requests: Vec::new(),
            swap_promises: Vec::new(),
            ui_resource_request_queue: Vec::new(),
            ui_resource_sizes: BTreeMap::new(),
            layer_ids_that_should_push_properties: BTreeSet::new(),
            picture_layer_ids_with_new_raster_source: BTreeSet::new(),
            layer_update_rects: BTreeMap::new(),
            scrollers_clobbering_active_value: BTreeSet::new(),
            screenshot_destination_token: UnguessableToken::default(),
            primary_main_frame_item_sequence_number: -1,
        }
    }
    // Source: cc/trees/commit_state.cc:13-47.
    pub fn CopyPersistent(&self, owner: PropertyTreesOwnerId) -> Self {
        let mut copy = Self::new(owner);
        copy.surface_ranges = self.surface_ranges.clone();
        copy.visual_properties_update_duration = self.visual_properties_update_duration.clone();
        copy.have_scroll_event_handlers = self.have_scroll_event_handlers.clone();
        copy.is_external_pinch_gesture_active = self.is_external_pinch_gesture_active.clone();
        copy.is_viewport_mobile_optimized = self.is_viewport_mobile_optimized.clone();
        copy.may_throttle_if_undrawn_frames = self.may_throttle_if_undrawn_frames.clone();
        copy.prefers_reduced_motion = self.prefers_reduced_motion.clone();
        copy.browser_controls_params = self.browser_controls_params.clone();
        copy.event_listener_properties = self.event_listener_properties.clone();
        copy.bottom_controls_shown_ratio = self.bottom_controls_shown_ratio.clone();
        copy.device_scale_factor = self.device_scale_factor.clone();
        copy.external_page_scale_factor = self.external_page_scale_factor.clone();
        copy.page_scale_factor_limits_set = self.page_scale_factor_limits_set.clone();
        copy.max_page_scale_factor = self.max_page_scale_factor.clone();
        copy.min_page_scale_factor = self.min_page_scale_factor.clone();
        copy.page_scale_factor = self.page_scale_factor.clone();
        copy.painted_device_scale_factor = self.painted_device_scale_factor.clone();
        copy.top_controls_shown_ratio = self.top_controls_shown_ratio.clone();
        copy.display_color_spaces = self.display_color_spaces.clone();
        copy.display_transform_hint = self.display_transform_hint.clone();
        copy.device_viewport_rect = self.device_viewport_rect.clone();
        copy.visual_device_viewport_size = self.visual_device_viewport_size.clone();
        copy.elastic_overscroll = self.elastic_overscroll.clone();
        copy.max_safe_area_insets = self.max_safe_area_insets.clone();
        copy.hud_layer_id = self.hud_layer_id.clone();
        copy.source_frame_number = self.source_frame_number.clone();
        copy.selection = self.selection.clone();
        copy.debug_state = self.debug_state.clone();
        copy.overscroll_behavior = self.overscroll_behavior.clone();
        copy.background_color = self.background_color.clone();
        copy.viewport_property_ids = self.viewport_property_ids.clone();
        copy.local_surface_id_from_parent = self.local_surface_id_from_parent.clone();
        copy.primary_main_frame_item_sequence_number =
            self.primary_main_frame_item_sequence_number.clone();
        // The source copy constructor omits needs_gpu_rasterization_histogram,
        // despite its position in the persistent section; it remains false here too.
        copy
    }
    // Source: cc/trees/commit_state.cc:49-54.
    pub fn SurfaceRanges(&self) -> BTreeSet<B::SurfaceRange> {
        let mut ranges = BTreeSet::new();
        for (range, _) in &self.surface_ranges {
            ranges.insert(range.clone());
        }
        ranges
    }
    // Source: cc/trees/commit_state.cc:56-61.
    pub fn GetEventListenerProperties(&self, class: EventListenerClass) -> EventListenerProperties {
        debug_assert!(class as i32 >= EventListenerClass::kPointerRawUpdate as i32);
        debug_assert!(class as i32 <= EventListenerClass::kTouchEndOrCancel as i32);
        self.event_listener_properties[class as usize]
    }
}
// Source: cc/trees/commit_state.h:192-213.
pub struct ThreadUnsafeCommitState<B: CommitStateTypes> {
    pub mutator_host: B::MutatorHostHandle,
    pub root_layer: Option<B::MainRootLayer>,
    pub layer_id_map: HashMap<i32, B::MainLayerRef>,
}
// Source: cc/trees/commit_state.h:192-214 (implicit memberwise copy).
impl<B: CommitStateTypes> Clone for ThreadUnsafeCommitState<B> {
    fn clone(&self) -> Self {
        Self {
            mutator_host: self.mutator_host,
            root_layer: self.root_layer.clone(),
            layer_id_map: self.layer_id_map.clone(),
        }
    }
}
impl<B: CommitStateTypes> ThreadUnsafeCommitState<B> {
    // Source: cc/trees/commit_state.cc:63-66.
    pub fn new(mutator_host: B::MutatorHostHandle) -> Self {
        Self {
            mutator_host,
            root_layer: None,
            layer_id_map: HashMap::new(),
        }
    }
    // Source: cc/trees/commit_state.h:199-202.
    pub fn begin<'a, C: MainLayerListContext<B>>(&'a self, context: &'a C) -> C::Iterator<'a> {
        context.LayerListConstIterator(self.root_layer.as_ref())
    }
    pub fn end<'a, C: MainLayerListContext<B>>(&'a self, context: &'a C) -> C::Iterator<'a> {
        context.LayerListConstIterator(None)
    }
    // Source: cc/trees/commit_state.h:204.
    pub fn num_layers(&self) -> usize {
        self.layer_id_map.len()
    }
    // Source: cc/trees/commit_state.h:205-208.
    pub fn LayerById(&self, id: i32) -> Option<B::MainLayerRef> {
        self.layer_id_map.get(&id).cloned()
    }
}
// Source: cc/trees/commit_state.h:216-222.
#[derive(Clone, Copy, Debug, Default)]
pub struct CommitTimestamps {
    pub start: TimeTicks,
    pub finish: TimeTicks,
}
