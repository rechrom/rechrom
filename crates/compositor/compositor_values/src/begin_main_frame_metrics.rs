// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/metrics/begin_main_frame_metrics.h:13-38. The main
// LayerTreeHost records it (LayerTreeHost::begin_main_frame_metrics), the
// commit carries it (CommitState::begin_main_frame_metrics) and the impl
// scheduler reports it (Scheduler::NotifyReadyToCommit).
use scroll_animation::TimeDelta;

#[derive(Clone, Debug, Default)]
pub struct BeginMainFrameMetrics {
    pub handle_input_events: TimeDelta,
    pub animate: TimeDelta,
    pub style_update: TimeDelta,
    pub layout_update: TimeDelta,
    pub accessibility: TimeDelta,
    pub prepaint: TimeDelta,
    pub compositing_inputs: TimeDelta,
    pub paint: TimeDelta,
    pub composite_commit: TimeDelta,
    pub update_layers: TimeDelta,
    pub should_measure_smoothness: bool,
}
