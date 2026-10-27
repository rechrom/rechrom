// Translation status: complete
// Source: cc/trees/frame_data.h:1-89
// Source: cc/trees/frame_data.cc:1-63
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use crate::frame_data_boundary::*;
use compositor_draw::damage_reason::DamageReasonSet;
use compositor_scheduler::{begin_frame::BeginFrameArgs, TimeDelta};
use compositor_tree::LayerTreeId;
use std::collections::{BTreeMap, HashMap};

// Source: cc/trees/frame_data.h:23-33
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrackedElementStatus {
    #[default]
    kNone,
    kHasTrackedElements,
    kHasTrackedElementsNeedingOcclusion,
}

// Source: cc/trees/frame_data.h:35-85
// Source render-pass lists own unique_ptrs. A surface-list pointer is represented
// by its canonical tree identity and resolved live; no list is copied or owned.
pub struct FrameData<B: FrameDataTypes> {
    pub frame_token: u32,
    pub checkerboarded_needs_raster: bool,
    pub checkerboarded_needs_record: bool,
    pub activation_dependencies: Vec<B::SurfaceId>,
    pub deadline_in_frames: Option<u32>,
    pub use_default_lower_bound_deadline: bool,
    pub render_passes: Vec<Box<B::RenderPass>>,
    pub unbounded_render_passes: Vec<Box<B::RenderPass>>,
    pub render_surface_list: Option<LayerTreeId>,
    pub will_draw_layers: Vec<i32>,
    pub has_no_damage: bool,
    pub begin_frame_ack: B::BeginFrameAck,
    pub origin_begin_main_frame_args: BeginFrameArgs,
    pub damage_reasons: DamageReasonSet,
    pub video_layer_preferred_intervals: BTreeMap<TimeDelta, u32>,
    pub has_shared_element_resources: bool,
    pub has_view_transition_save_directive: bool,
    pub has_copy_requests: bool,
    pub trees_in_viz_timing_details: Option<B::TreesInVizTiming>,
    pub tracked_element_status: TrackedElementStatus,
}

impl<B: FrameDataTypes> Default for FrameData<B> {
    // Source: cc/trees/frame_data.cc:18-19; cc/trees/frame_data.h:45-84
    // Rust move-only ownership preserves deleted copy operations and defaulted
    // destruction of all source members.
    fn default() -> Self {
        Self {
            frame_token: 0,
            checkerboarded_needs_raster: false,
            checkerboarded_needs_record: false,
            activation_dependencies: Vec::new(),
            deadline_in_frames: None,
            use_default_lower_bound_deadline: false,
            render_passes: Vec::new(),
            unbounded_render_passes: Vec::new(),
            render_surface_list: None,
            will_draw_layers: Vec::new(),
            has_no_damage: false,
            begin_frame_ack: B::BeginFrameAck::default(),
            origin_begin_main_frame_args: BeginFrameArgs::default(),
            damage_reasons: DamageReasonSet::default(),
            video_layer_preferred_intervals: BTreeMap::new(),
            has_shared_element_resources: false,
            has_view_transition_save_directive: false,
            has_copy_requests: false,
            trees_in_viz_timing_details: None,
            tracked_element_status: TrackedElementStatus::kNone,
        }
    }
}

impl<B: FrameDataTypes> FrameData<B> {
    // Source: cc/trees/frame_data.h:57-58
    // Safe dereference of the source's non-owning pointer. Only the current
    // active allocation may resolve it; recycled/other-tree IDs are rejected.
    pub fn RenderSurfaceList<'a>(
        &self,
        resident: &'a impl FrameDataSurfaceList,
    ) -> Option<&'a [i32]> {
        let owner = self.render_surface_list?;
        assert_eq!(
            owner,
            resident.ActiveTreeId(),
            "FrameData surface list no longer belongs to the active resident tree"
        );
        Some(resident.GetActiveRenderSurfaceList())
    }

    // Source: cc/trees/frame_data.cc:21-50
    pub fn AsValueInto<V: FrameDataTraceValue>(
        &self,
        value: &mut V,
        services: &impl FrameDataTraceServices,
    ) {
        value.SetBoolean("has_no_damage", self.has_no_damage);
        let client_name = services.GetClientNameForMetrics();
        let is_renderer = client_name.is_some_and(|name| name == "Renderer");
        let quads_enabled = services.TraceCategoryEnabled("disabled-by-default-viz.quads");
        if is_renderer && quads_enabled {
            let mut resource_id_to_index_map = HashMap::<B::ResourceId, usize>::new();
            value.BeginArray("render_passes");
            for render_pass in &self.render_passes {
                value.BeginDictionary();
                render_pass.AsValueInto(value, &mut resource_id_to_index_map);
                value.EndDictionary();
            }
            value.EndArray();
            value.BeginArray("unbounded_render_passes");
            for render_pass in &self.unbounded_render_passes {
                value.BeginDictionary();
                render_pass.AsValueInto(value, &mut resource_id_to_index_map);
                value.EndDictionary();
            }
            value.EndArray();
        }
    }

    // Source: cc/trees/frame_data.cc:52-56
    pub fn ToString(&self, services: &impl FrameDataTraceServices) -> String {
        let mut value = services.NewTracedValueJSON();
        self.AsValueInto(&mut value, services);
        value.ToFormattedJSON()
    }

    // Source: cc/trees/frame_data.cc:58-61
    pub fn set_trees_in_viz_timestamps(&mut self, timing_details: &B::TreesInVizTiming) {
        self.trees_in_viz_timing_details = Some(timing_details.clone());
    }
}
