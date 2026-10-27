// Translation status: complete
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/trees/proxy_common.h:1-52
// Source: cc/trees/proxy_common.cc:1-61
use crate::adapter::{
    begin_frame::BeginFrameArgs,
    commit_types::{
        ActiveFrameSequenceTrackers, BeginMainFrameReasons, BeginMainFrameTraceId,
        BeginMainFrameTraceIdProvider, MutatorEvents,
    },
};
use crate::cc::trees::compositor_commit_data::CompositorCommitData;

// Source: cc/trees/proxy_common.h:26-43;
//         cc/trees/proxy_common.cc:14-17.
pub struct BeginMainFrameAndCommitState {
    pub begin_frame_args: BeginFrameArgs,
    pub commit_data: Option<Box<CompositorCommitData>>,
    pub completed_image_decode_requests: Vec<(i32, bool)>,
    pub mutator_events: Option<Box<dyn MutatorEvents>>,
    pub active_sequence_trackers: ActiveFrameSequenceTrackers,
    pub evicted_ui_resources: bool,
    pub trace_id: BeginMainFrameTraceId,
    pub reason: BeginMainFrameReasons,
    pub consecutive_no_damage_main_frames: i32,
}

impl BeginMainFrameAndCommitState {
    /// The source constructor obtains this id from base tracing. Passing its
    /// provider keeps that process-global service outside the compositor core.
    pub fn new(trace_ids: &mut dyn BeginMainFrameTraceIdProvider) -> Self {
        Self {
            begin_frame_args: BeginFrameArgs::default(),
            commit_data: None,
            completed_image_decode_requests: Vec::new(),
            mutator_events: None,
            active_sequence_trackers: 0,
            evicted_ui_resources: false,
            trace_id: trace_ids.NextTraceId(),
            reason: 0,
            consecutive_no_damage_main_frames: 0,
        }
    }
}

// Source: cc/trees/proxy_common.h:45-48
// Source: cc/trees/proxy_common.cc:19-59
pub use compositor_scheduler::GetThrottlingFactor;
